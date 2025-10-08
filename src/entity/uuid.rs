use core::fmt;
use std::fmt::Display;

use serde::Serialize;


#[derive(Debug, PartialEq, Clone)]
pub struct UUID(String);
impl UUID {
    pub(crate) fn can_deserialize(value: &str) -> bool {
        value.starts_with("~u")
    }
    
    pub(crate) fn deserialize(value: &str) -> UUID {
        UUID(value[2..].to_owned())
    }
    
    pub(crate) fn parse(arg: &str) -> UUID {
        UUID(arg.to_owned())
    }
}

impl Display for UUID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "~u{}", self.0)
    }
}

impl Serialize for UUID {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("~u{}", self.0))
    }
}