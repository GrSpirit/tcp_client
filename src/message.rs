use std::error::Error;
use std::collections::BTreeMap;
use hex;

pub type Message = BTreeMap<u32, DataValue>;

#[derive(Debug)]
pub struct FixNumber {
    pub precision: usize,
    pub matrix: String,
}

impl FixNumber {
    pub fn from_str(data: &str) -> Result<FixNumber, Box<dyn Error>> {
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

#[derive(Debug)]
pub enum DataValue {
    ShortValue(u16),
    LongValue(u32),
    StringValue(String),
    HexValue(Vec<u8>),
    FixValue(FixNumber)
}

impl DataValue {
    pub fn from_str(data_type: &str, data_value: &str) -> Result<DataValue, Box<dyn Error>> {
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
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut row = Vec::new();
        match &self {
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
        row
    }
}

#[derive(Debug)]
pub enum AddStrError {
    ParseStr(Box<dyn Error>),
    FormatStr(),
    MaxNumber(u32),
    DuplicateNumber(u32)
}

impl std::fmt::Display for AddStrError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use AddStrError::*;
        match *self {
            ParseStr(ref err) => err.fmt(f),
            FormatStr() => write!(f, "Wrong field format"),
            MaxNumber(x) => write!(f, "The number {} exceeded maximum", x),
            DuplicateNumber(x) => write!(f, "The number {} is duplicated", x)
        }
    }
}

impl Error for AddStrError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl From<std::num::ParseIntError> for AddStrError {
    fn from(err: std::num::ParseIntError) -> AddStrError {
        AddStrError::ParseStr(Box::new(err))
    }
}

impl From<Box<dyn Error>> for AddStrError {
    fn from(err: Box<dyn Error>) -> AddStrError {
        AddStrError::ParseStr(err)
    }
}


pub trait AddStr {
    fn add_str(&mut self, data: &str) -> Result<(), AddStrError>;
}

impl AddStr for Message {
    fn add_str(&mut self, data: &str) -> Result<(), AddStrError> {
        let row: Vec<_> = data.split_whitespace().collect();
        if row.len() < 3 {
            Err(AddStrError::FormatStr())
        }
        else {
            let number: u32 = row[0].parse()?;
            if number >= 0u32.count_zeros() {
                //return Err("Maximum field number exceeded".into());
                return Err(AddStrError::MaxNumber(number));
            }
            let data = row.iter().cloned().skip(2).collect::<Vec<_>>().join(" ");
            match self.get(&number) {
                None => self.insert(number, DataValue::from_str(row[1], &data)?),
                _ => return Err(AddStrError::DuplicateNumber(number))
                //_ => return Err("Duplicate field number".into())
            };

            Ok(())
        }
    }
}

pub fn build_bitmap(message: &Message) -> u32 {
    message.keys().fold(0, |r, f| r | (1u32 << f))
}

