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
        #[structopt(short, long)]
        addr: String
    },
    File {
        #[structopt(short, long)]
        file_name: String
    },
}

fn read_line() -> Result<String, Box<dyn Error>> {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn read_message() -> Result<Vec<Field>, Box<dyn Error>> {
    let mut result: Vec<Field> = Vec::new();
    let mut line = read_line()?;
    while line.len() > 1 {
        let field = Field::from_str(&line)?;
        if result.iter().any(|f| f.number == field.number) {
            return Err("Duplicate field number".into());
        }
        result.push(field);
        line = read_line()?;
    }
    Ok(result)
}

fn main() -> Result<(), Box<dyn Error>> {
    let mode = Modes::from_args();
    println!("Enter message");

    let message = read_message()?;
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
            let n = stream.write(&buf[..])?; 
            println!("Sent {} bytes", n);
        },
        Modes::File{ file_name } => {
            let mut file = File::create(&file_name)?;
            let n = file.write(&buf)?;
            println!("Written {} bytes", n);
        },
    }
    
    println!("Quit");
    Ok(())
}
