use serde::Serialize;


#[derive(Debug, PartialEq, Clone)]
pub struct NI {
    namespace: String,
    identifier: String,
}
impl NI {
    pub(crate) fn can_deserialize(value: &str) -> bool {
        value.starts_with("~:")
    }
    
    pub(crate) fn deserialize(value: &str) -> Self {
        let rest = &value[2..];
        if let Some(last_colon_index) = rest.rfind(':') {
            Self {
                namespace: rest[0..last_colon_index].to_owned(),
                identifier: rest[last_colon_index + 1..].to_owned(),
            }
        } else {
            // TODO how to return sensible error
            todo!()
        }
    }
    
    pub(crate) fn new(namespace: &str, identifier: &str) -> Self {
        Self { namespace: namespace.to_owned(), identifier: identifier.to_owned() }
    }
}

impl Serialize for NI {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("~:{}:{}", self.namespace, self.identifier))
    }
}