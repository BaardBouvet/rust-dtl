use serde::Serialize;


#[derive(Debug, PartialEq, Clone)]
pub struct URI(String);
impl URI {
    pub(crate) fn deserialize(value: &str) -> URI {
        URI(value[2..].to_owned())
    }
    
    pub(crate) fn can_deserialize(value: &str) -> bool {
        value.starts_with("~r")
    }
}

impl Serialize for URI {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("~r{}", self.0))
    }
}