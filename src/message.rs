use std::error::Error;
use hex;

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
pub struct Field {
    pub number: u32,
    pub value: DataValue,
}

impl Field {
    pub fn from_str(data: &str) -> Result<Field, Box<dyn Error>> {
        let row: Vec<_> = data.split_whitespace().collect();
        if row.len() < 3 {
            Err("Wrong field format".into())
        }
        else {
            let data = row.iter().cloned().skip(2).collect::<Vec<_>>().join(" "); 

            Ok(Field {
                number: row[0].parse::<u32>()?,
                value: DataValue::from_str(row[1], &data)?,
            })
        }
    }
}

pub fn build_bitmap(message: &[Field]) -> u32 {
    message.iter().fold(0, |r, f| r | f.number)
}

