use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Number;

#[derive(Serialize, Deserialize, Debug)]
struct Entity {
    #[serde(rename = "_id")]
    id: String,
    #[serde(rename = "_filtered")]
    filtered: bool,
    #[serde(flatten)]
    content: HashMap<String, EntityValue>,
}

#[derive(Deserialize, Debug)]
struct Uri {
    uri: String,
}

impl Uri {
    fn new(arg: &str) -> Self {
        Self { uri: arg.to_owned() }
    }
}

#[derive(Deserialize, Debug)]
enum EntityValue {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Uri(Uri),
    Array(Vec<EntityValue>),
    Object(HashMap<String, EntityValue>),
}

impl Serialize for Uri {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        serializer.serialize_str(&format!("~u{}", self.uri))
    }
}

impl Serialize for EntityValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            EntityValue::Null => serializer.serialize_none(),
            EntityValue::Bool(b) => serializer.serialize_bool(*b),
            EntityValue::Number(number) => number.serialize(serializer),
            EntityValue::String(s) => serializer.serialize_str(s),
            EntityValue::Uri(u) => u.serialize(serializer),
            EntityValue::Array(v) => v.serialize(serializer),
            EntityValue::Object(m) => m.serialize(serializer),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::json; // optional, nicer diffs

    #[test]
    fn main() {
        let entity = Entity {
            id: "1".to_owned(),
            filtered: false,
            content: HashMap::from([
                (
                    "string".to_owned(),
                    EntityValue::String("value".to_owned()),
                ),
                ("uri".to_owned(), EntityValue::Uri(Uri::new("db.no"))),
                (
                    "num".to_owned(),
                    EntityValue::Number(Number::from_f64(1.0).unwrap()),
                ),
                ("null".to_owned(), EntityValue::Null),
                ("boolean".to_owned(), EntityValue::Bool(true)),
                ("empty_array".to_owned(), EntityValue::Array(vec![])),
                (
                    "empty_object".to_owned(),
                    EntityValue::Object(HashMap::new()),
                ),
                (
                    "object_with_uri".to_owned(),
                    EntityValue::Object(HashMap::from([(
                        "uri".to_owned(),
                        EntityValue::Uri(Uri::new("vg.no")),
                    )])),
                ),
            ]),
        };

        // Convert the Entity to a JSON string.
        let serialized = serde_json::to_string(&entity).unwrap();

        // Prints serialized = { .. }
        println!("serialized = {}", serialized);

        // Convert the JSON string back to an Entity.
        let deserialized: Entity = serde_json::from_str(&serialized).unwrap();

        // Prints deserialized = Entity { .. }
        println!("deserialized = {:?}", deserialized);
    }
}
