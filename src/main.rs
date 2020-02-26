mod message;
use message::*;

use structopt::StructOpt;
use std::io::prelude::*;
use std::net::{TcpStream};
use std::fs::File;
use std::error::Error;

#[derive(StructOpt)]
#[structopt(name = "tcp_client", about = "Example of tcp message client")]
enum Modes {
    Tcp {
        #[structopt(short = "a", long)]
        addr: String
    },
    File {
        #[structopt(short = "f", long)]
        file_name: String
    },
}

fn read_line() -> String {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

fn read_message() -> Vec<Field> {
    let mut result = Vec::new();
    let mut line = read_line();
    while line.len() > 1 {
        result.push(Field::from_str(&line).unwrap());
        line = read_line();
    }
    result
}

fn main() -> Result<(), Box<dyn Error>> {
    let mode = Modes::from_args();

    let message = read_message();
    let mut buf: Vec<u8> = Vec::new();
    // Bitmap
    buf.extend_from_slice(&build_bitmap(&message).to_le_bytes());
    message.iter().for_each(|f|
        buf.append(&mut f.value.to_bytes())
    );

    match mode {
        Modes::Tcp{ addr } => {
            let mut stream = TcpStream::connect(addr)
                .expect("Couldn't connect to the server...");
            match stream.write(&buf[..]) {
                Ok(x) => println!("Sent {} bytes", x),
                Err(e) => println!("Error: {}", e)
            }
        },
        Modes::File{ file_name } => {
            let mut file = File::create(&file_name)?;
            file.write_all(&buf)?;
        },
    }
    
    println!("Quit");
    Ok(())
}
