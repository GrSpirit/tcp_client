mod message;
use message::*;

use structopt::StructOpt;
use std::io::prelude::*;
use std::net::{TcpStream};
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

enum OutStream {
    TcpStream(TcpStream),
    FileStream(File)
}

impl Write for OutStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            OutStream::TcpStream(strm) => strm.write(buf),
            OutStream::FileStream(strm) => strm.write(buf)
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            OutStream::TcpStream(strm) => strm.flush(),
            OutStream::FileStream(strm) => strm.flush()
        }

    }
}

fn read_line() -> std::io::Result<String> {
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
    let mode = ProgramMode::from_args();

    let mut out_stream = match mode {
        ProgramMode::Tcp{ addr } => 
            OutStream::TcpStream(TcpStream::connect(addr)
                .expect("Couldn't connect to the server...")),
        ProgramMode::File{ file_name } =>
            OutStream::FileStream(File::create(&file_name)
                .expect("Couldn't create a file...")),
    };

    println!("Enter message");

    let message = read_message()?;
    let mut buf: Vec<u8> = Vec::new();
    // Bitmap
    buf.extend_from_slice(&build_bitmap(&message).to_le_bytes());
    message.iter().for_each(|f|
        buf.append(&mut f.value.to_bytes())
    );

    let n = out_stream.write(&buf)?;
    println!("Written {} bytes", n);
    
    println!("Quit");
    Ok(())
}
