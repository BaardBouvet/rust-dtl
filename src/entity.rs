use core::fmt;
use std::collections::HashMap;

use serde::{de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor}, Deserialize, Serialize};
use serde_json::Number;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Entity {
    #[serde(rename = "_id")]
    id: String,
    #[serde(rename = "_filtered")]
    filtered: bool,
    #[serde(flatten)]
    content: HashMap<String, EntityValue>,
}

#[derive(Debug, PartialEq)]
struct Uri {
    uri: String,
}

impl Uri {
    fn new(arg: &str) -> Self {
        Self {
            uri: arg.to_owned(),
        }
    }
}

#[derive(Debug, PartialEq)]
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
        S: serde::Serializer,
    {
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

struct KeyClassifier;

enum KeyClass {
    Map(String),
}

impl<'de> DeserializeSeed<'de> for KeyClassifier {
    type Value = KeyClass;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(self)
    }
}

impl<'de> Visitor<'de> for KeyClassifier {
    type Value = KeyClass;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string key")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match s {
            _ => Ok(KeyClass::Map(s.to_owned())),
        }
    }

    fn visit_string<E>(self, s: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match s.as_str() {
            _ => Ok(KeyClass::Map(s)),
        }
    }
}

impl<'de> Deserialize<'de> for EntityValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct EntityValueVisitor;

        impl<'de> Visitor<'de> for EntityValueVisitor {
            type Value = EntityValue;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("any valid JSON value")
            }

            #[inline]
            fn visit_bool<E>(self, value: bool) -> Result<EntityValue, E> {
                Ok(EntityValue::Bool(value))
            }
            #[inline]

            fn visit_i64<E>(self, value: i64) -> Result<EntityValue, E> {
                Ok(EntityValue::Number(value.into()))
            }

            fn visit_i128<E>(self, value: i128) -> Result<EntityValue, E>
            where
                E: serde::de::Error,
            {
                let de = serde::de::value::I128Deserializer::new(value);
                Number::deserialize(de).map(EntityValue::Number)
            }

            #[inline]
            fn visit_u64<E>(self, value: u64) -> Result<EntityValue, E> {
                Ok(EntityValue::Number(value.into()))
            }

            fn visit_u128<E>(self, value: u128) -> Result<EntityValue, E>
            where
                E: serde::de::Error,
            {
                let de = serde::de::value::U128Deserializer::new(value);
                Number::deserialize(de).map(EntityValue::Number)
            }

            #[inline]
            fn visit_f64<E>(self, value: f64) -> Result<EntityValue, E> {
                Ok(Number::from_f64(value).map_or(EntityValue::Null, EntityValue::Number))
            }

            #[inline]
            fn visit_str<E>(self, value: &str) -> Result<EntityValue, E>
            where
                E: serde::de::Error,
            {
                self.visit_string(String::from(value))
            }

            #[inline]
            fn visit_string<E>(self, value: String) -> Result<EntityValue, E> {
                if value.starts_with("~u") {
                    Ok(EntityValue::Uri(Uri{uri: value[2..].to_owned()}))
                } else {
                    Ok(EntityValue::String(value))
                }
            }

            #[inline]
            fn visit_none<E>(self) -> Result<EntityValue, E> {
                Ok(EntityValue::Null)
            }

            #[inline]
            fn visit_some<D>(self, deserializer: D) -> Result<EntityValue, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                Deserialize::deserialize(deserializer)
            }

            #[inline]
            fn visit_unit<E>(self) -> Result<EntityValue, E> {
                Ok(EntityValue::Null)
            }

            #[inline]
            fn visit_seq<V>(self, mut visitor: V) -> Result<EntityValue, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let mut vec = Vec::new();

                while let Some(elem) = (visitor.next_element())? {
                    vec.push(elem);
                }

                Ok(EntityValue::Array(vec))
            }

            fn visit_map<V>(self, mut visitor: V) -> Result<EntityValue, V::Error>
            where
                V: MapAccess<'de>,
            {
                match (visitor.next_key_seed(KeyClassifier))? {
                    #[cfg(feature = "arbitrary_precision")]
                    Some(KeyClass::Number) => {
                        let number: NumberFromString = tri!(visitor.next_value());
                        Ok(Value::Number(number.value))
                    }
                    #[cfg(feature = "raw_value")]
                    Some(KeyClass::RawValue) => {
                        let value = tri!(visitor.next_value_seed(crate::raw::BoxedFromString));
                        crate::from_str(value.get()).map_err(de::Error::custom)
                    }
                    Some(KeyClass::Map(first_key)) => {
                        let mut values = HashMap::new();

                        values.insert(first_key, (visitor.next_value())?);
                        while let Some((key, value)) = (visitor.next_entry())? {
                            values.insert(key, value);
                        }

                        Ok(EntityValue::Object(values))
                    }
                    None => Ok(EntityValue::Object(HashMap::new())),
                }
            }
        }
        deserializer.deserialize_any(EntityValueVisitor)
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
                ("string".to_owned(), EntityValue::String("value".to_owned())),
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
        assert_eq!(entity, deserialized);
    }
}
