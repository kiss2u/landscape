use std::{num::ParseIntError, str::FromStr};

#[derive(Debug, PartialEq, Clone)]
pub struct NumberRange {
    pub start: u16,
    pub end: u16,
}

impl FromStr for NumberRange {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('-').collect();
        match parts.as_slice() {
            [single] => {
                let num = single.parse().map_err(|e: ParseIntError| e.to_string())?;
                Ok(NumberRange { start: num, end: num })
            }
            [start, end] => {
                let start = start.parse().map_err(|e: ParseIntError| e.to_string())?;
                let end = end.parse().map_err(|e: ParseIntError| e.to_string())?;
                Ok(NumberRange { start, end })
            }
            _ => Err("Invalid range format. Use '10-20' or '50'.".to_string()),
        }
    }
}
