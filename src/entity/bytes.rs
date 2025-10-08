use std::fmt::Display;

use base64::{engine::general_purpose, Engine as _};
use serde::Serialize;

#[derive(Debug, PartialEq, Clone)]
pub struct ByteWrapper(Vec<u8>);
impl ByteWrapper {
    pub(crate) fn deserialize(value: &str) -> ByteWrapper {
        ByteWrapper(general_purpose::STANDARD.decode(&value[2..]).unwrap())
    }
    
    pub(crate) fn from_vec(vec: Vec<u8>) -> ByteWrapper {
        ByteWrapper(vec)
    }
    
    pub(crate) fn can_deserialize(value: &str) -> bool {
        value.starts_with("~b")
    }
    
    pub(crate) fn from_array(arg: &[u8; 5]) -> ByteWrapper {
        ByteWrapper(arg.to_vec())
    }
}

impl Display for ByteWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "~b{}", general_purpose::STANDARD.encode(&self.0))
    }
}

impl Serialize for ByteWrapper {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("~b{}", general_purpose::STANDARD.encode(&self.0)))
    }
}