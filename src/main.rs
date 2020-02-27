mod message;
use message::*;

use structopt::StructOpt;
use std::io::prelude::*;
use std::net::TcpStream;
use std::fs::File;
use std::error::Error;

#[derive(StructOpt)]
#[structopt(name = "tcp_client", about = "Example of tcp message client")]
enum ProgramMode {
    Tcp {
        #[structopt(short, long)]
        addr: String
    },
    File {
        #[structopt(short, long)]
        file_name: String
    },
}

fn read_line() -> std::io::Result<String> {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn read_message() -> std::io::Result<Message> {
    let mut result: Message = Message::new();
    let mut line = read_line()?;
    while line.len() > 1 {
        match result.add_str(&line) {
            Err(err) => println!("{}", err),
            _ => ()
        };
        line = read_line()?;
    }
    Ok(result)
}

fn main() -> Result<(), Box<dyn Error>> {
    let mode = ProgramMode::from_args();

    let mut out_stream: Box<dyn Write> = match mode {
        ProgramMode::Tcp{ addr } => 
            Box::new(TcpStream::connect(addr)
                .expect("Couldn't connect to the server...")),
        ProgramMode::File{ file_name } =>
            Box::new(File::create(&file_name)
                .expect("Couldn't create a file...")),
    };

    println!("Enter message");

    let message = read_message()?;
    let mut buf: Vec<u8> = Vec::new();
    // Build bitmap
    buf.extend_from_slice(&build_bitmap(&message).to_le_bytes());

    // Build message
    message.values().for_each(|f|
        buf.append(&mut f.to_bytes())
    );

    let n = out_stream.write(&buf)?;
    println!("Written {} bytes", n);
    
    println!("Quit");
    Ok(())
}
