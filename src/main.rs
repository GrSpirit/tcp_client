use std::io::prelude::*;
use std::env;
use std::net::{TcpStream};
use std::io;
use hex;
use std::error::Error;

fn read_line() -> String {
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

struct FixNumber {
    precision: usize,
    matrix: String,
}

impl FixNumber {
    fn from_str(data: &str) -> Result<FixNumber, Box<dyn Error>> {
        let parts: Vec<_> = data.split(".").collect();
        if parts.iter().all(|v| v.chars().all(|c| c.is_ascii_digit())) {
            match parts.len() {
                1 => Ok(FixNumber{
                    precision: 0,
                    matrix: String::from(parts[0])
                }),
                2 => Ok(FixNumber{
                    precision: parts[1].len(),
                    matrix: String::from(parts[0]) + parts[1]
                }),
                _ => Err("Cannot parse fix number".into())
            }
        }
        else {
            Err("FixNumber is not a digit string".into())
        }
    }
}

enum DataValue {
    ShortValue(u16),
    LongValue(u32),
    StringValue(String),
    HexValue(Vec<u8>),
    FixValue(FixNumber)
}

impl DataValue {
    fn from_str(data_type: &str, data_value: &str) -> Result<DataValue, Box<dyn Error>> {
        use DataValue::*;
        match data_type {
            "N" => Ok(ShortValue(data_value.parse::<u16>()?)),
            "U" => Ok(LongValue(data_value.parse::<u32>()?)),
            "S" => Ok(StringValue(String::from(data_value))),
            "H" => Ok(HexValue(hex::decode(data_value)?)),
            "D" => Ok(FixValue(FixNumber::from_str(data_value)?)),
            _ => Err("Unknown DataValue".into())
        }
    }
}

struct Field {
    number: u32,
    value: DataValue,
}

impl Field {
    fn from_str(data: &str) -> Result<Field, Box<dyn Error>> {
        let row: Vec<_> = data.split_whitespace().collect();
        if row.len() < 3 {
            Err("Wrong field format".into())
        }
        else {
            Ok(Field {
                number: row[0].parse::<u32>().unwrap(),
                value: DataValue::from_str(row[1], row[2])?,
            })
        }
    }
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

fn build_bitmap(message: &[Field]) -> u32 {
    message.iter().fold(0, |r, f| r | f.number)
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: tcp_client [ip_address:port]");
        return Ok(());
    }
    let addr = args[1].clone();
    let mut stream = TcpStream::connect(addr)
        .expect("Couldn't connect to the server...");


    let message = read_message();
    let mut buf: Vec<u8> = Vec::new();
    // Bitmap
    buf.extend_from_slice(&build_bitmap(&message).to_le_bytes());
    
    for f in &message {
        let mut row = Vec::new();
        match &f.value {
            DataValue::ShortValue(val) => row.extend_from_slice(&val.to_le_bytes()),
            DataValue::LongValue(val) =>  row.extend_from_slice(&val.to_le_bytes()),
            DataValue::StringValue(val) => {
                row.extend_from_slice(format!("{:03}", val.len()).as_bytes());
                row.extend_from_slice(val.as_bytes())
            },
            DataValue::HexValue(val) => {
                row.extend_from_slice(format!("{:03}", val.len()).as_bytes());
                row.extend_from_slice(&val)
            },
            DataValue::FixValue(val) => {
                let len = format!("{}", val.matrix.len()% 10);
                row.extend_from_slice(len.as_bytes());
                row.push(val.precision as u8);
                row.extend_from_slice(val.matrix.as_bytes())
            }
        }
        buf.append(&mut row);
    }

    match stream.write(&buf[..]) {
        Ok(x) => println!("Sent {} bytes", x),
        Err(e) => println!("Error: {}", e)
    }
    println!("Quit");
    Ok(())
}
