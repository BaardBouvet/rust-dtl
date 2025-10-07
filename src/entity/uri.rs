use serde::Serialize;


#[derive(Debug, PartialEq, Clone)]
pub struct URI(String);
impl URI {
    pub(crate) fn deserialize(value: &str) -> URI {
        Self::parse(&value[2..])
    }
    
    pub(crate) fn can_deserialize(value: &str) -> bool {
        value.starts_with("~r")
    }
    
    pub(crate) fn parse(arg: &str) -> URI {
        URI(arg.to_owned())
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