use std::str::FromStr;

use bigdecimal::BigDecimal;
use serde::Serialize;

#[derive(Debug, PartialEq, Clone)]
pub struct BigDecimalWrapper(BigDecimal);
impl BigDecimalWrapper {
    pub(crate) fn deserialize(value: &str) -> BigDecimalWrapper {
        BigDecimalWrapper(BigDecimal::from_str(&value[2..]).unwrap())
    }
    
    pub(crate) fn can_deserialize(value: &str) -> bool {
        value.starts_with("~f")
    }
    
    pub(crate) fn parse(arg: &str) -> BigDecimalWrapper {
        BigDecimalWrapper(BigDecimal::from_str(arg).unwrap())
    }
}

impl Serialize for BigDecimalWrapper {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("~f{}", self.0.to_plain_string()))
    }
}
