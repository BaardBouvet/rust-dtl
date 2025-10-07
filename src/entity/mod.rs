use std::collections::HashMap;

use serde::{
    de::{MapAccess, SeqAccess, Visitor},
    Deserialize, Serialize,
};
use serde_json::Number;

use crate::entity::{
    bytes::ByteWrapper,
    datetime::{Date, DateTimeWrapper},
    decimal::BigDecimalWrapper,
    ni::NI,
    uri::URI,
    uuid::UUID,
};

mod bytes;
mod datetime;
mod decimal;
mod ni;
mod uri;
mod uuid;

// TODO not sure if we need this wrapper in this library
#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Entity {
    #[serde(rename = "_id")]
    id: String,
    #[serde(rename = "_deleted")]
    deleted: bool,
    #[serde(rename = "_ts")]
    timestamp: u128,
    #[serde(rename = "_filtered")]
    filtered: bool,
    #[serde(rename = "_updated")]
    updated: u64,
    #[serde(rename = "_hash")]
    hash: String,
    #[serde(rename = "_previous")]
    previous: Option<u64>,
    #[serde(flatten)]
    content: HashMap<String, EntityValue>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum EntityValue {
    Null,
    Bool(bool),
    // not sure how we can use bigint with serde
    Number(Number),
    String(String),
    URI(URI),
    Date(Date),
    DateTime(DateTimeWrapper),
    UUID(UUID),
    Bytes(ByteWrapper),
    NI(NI),
    Decimal(BigDecimalWrapper),
    Array(Vec<EntityValue>),
    Object(HashMap<String, EntityValue>),
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
            EntityValue::URI(u) => u.serialize(serializer),
            EntityValue::Array(v) => v.serialize(serializer),
            EntityValue::Object(m) => m.serialize(serializer),
            EntityValue::Date(d) => d.serialize(serializer),
            EntityValue::DateTime(d) => d.serialize(serializer),
            EntityValue::Bytes(b) => b.serialize(serializer),
            EntityValue::NI(n) => n.serialize(serializer),
            EntityValue::Decimal(d) => d.serialize(serializer),
            EntityValue::UUID(u) => u.serialize(serializer),
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
                if !value.starts_with("~") {
                    // optimization to avoid all the checks for non-transit strings
                    Ok(EntityValue::String(value))
                } else {
                    if URI::can_deserialize(&value) {
                        Ok(EntityValue::URI(URI::deserialize(&value)))
                    } else if DateTimeWrapper::can_deserialize(&value) {
                        Ok(EntityValue::DateTime(DateTimeWrapper::deserialize(&value)))
                    } else if Date::can_deserialize(&value) {
                        Ok(EntityValue::Date(Date::deserialize(&value)))
                    } else if ByteWrapper::can_deserialize(&value) {
                        Ok(EntityValue::Bytes(ByteWrapper::deserialize(&value)))
                    } else if NI::can_deserialize(&value) {
                        Ok(EntityValue::NI(NI::deserialize(&value)))
                    } else if BigDecimalWrapper::can_deserialize(&value) {
                        Ok(EntityValue::Decimal(BigDecimalWrapper::deserialize(&value)))
                    } else if UUID::can_deserialize(&value) {
                        Ok(EntityValue::UUID(UUID::deserialize(&value)))
                    } else {
                        Ok(EntityValue::String(value))
                    }
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
                let mut values = HashMap::new();

                while let Some((key, value)) = (visitor.next_entry())? {
                    values.insert(key, value);
                }

                Ok(EntityValue::Object(values))
            }
        }
        deserializer.deserialize_any(EntityValueVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn uuid() {
        let entity = EntityValue::UUID(UUID::parse("123"));
        let serialized = serde_json::to_string(&entity).unwrap();
        assert_eq!(serialized, "\"~u123\"");
        let deserialized: EntityValue = serde_json::from_str(&serialized).unwrap();
        assert_eq!(entity, deserialized);
    }
    #[test]
    fn decimal() {
        let entity = EntityValue::Decimal(BigDecimalWrapper::parse("123.456"));
        let serialized = serde_json::to_string(&entity).unwrap();
        assert_eq!(serialized, "\"~f123.456\"");
        let deserialized: EntityValue = serde_json::from_str(&serialized).unwrap();
        assert_eq!(entity, deserialized);
    }

    #[test]
    fn ni() {
        let entity = EntityValue::NI(NI::new("foo", "bar"));
        let serialized = serde_json::to_string(&entity).unwrap();
        assert_eq!(serialized, "\"~:foo:bar\"");
        let deserialized: EntityValue = serde_json::from_str(&serialized).unwrap();
        assert_eq!(entity, deserialized);
    }

    #[test]
    fn bytes() {
        let entity = EntityValue::Bytes(ByteWrapper::from_vec(vec![255]));
        let serialized = serde_json::to_string(&entity).unwrap();
        assert_eq!(serialized, "\"~b/w==\"");
        let deserialized: EntityValue = serde_json::from_str(&serialized).unwrap();
        assert_eq!(entity, deserialized);
    }

    #[test]
    fn date() {
        let entity = EntityValue::Date(Date::parse("2020-01-01"));
        let serialized = serde_json::to_string(&entity).unwrap();
        assert_eq!(serialized, "\"~t2020-01-01\"");
        let deserialized: EntityValue = serde_json::from_str(&serialized).unwrap();
        assert_eq!(entity, deserialized);
    }

    #[test]
    fn datetime() {
        let entity = EntityValue::DateTime(DateTimeWrapper::parse("2014-07-08T09:10:11.0+0000"));
        let serialized = serde_json::to_string(&entity).unwrap();
        assert_eq!(serialized, "\"~t2014-07-08T09:10:11.000000000+0000\"");
        let deserialized: EntityValue = serde_json::from_str(&serialized).unwrap();
        assert_eq!(entity, deserialized);
    }

    #[test]
    fn main() {
        fn current_time_in_millis() -> u128 {
            use std::time::{SystemTime, UNIX_EPOCH};
            let start = SystemTime::now();
            let since_the_epoch = start
                .duration_since(UNIX_EPOCH)
                .expect("time should go forward");
            since_the_epoch.as_millis()
        }
        let entity = Entity {
            id: "1".to_owned(),
            filtered: false,
            updated: 0,
            hash: "a".to_owned(),
            previous: None,
            deleted: false,
            timestamp: current_time_in_millis(),
            content: HashMap::from([
                ("string".to_owned(), EntityValue::String("value".to_owned())),
                (
                    "uri".to_owned(),
                    EntityValue::URI(URI::parse("http://db.no")),
                ),
                (
                    "float".to_owned(),
                    EntityValue::Number(Number::from_f64(1.0).unwrap()),
                ),
                (
                    "integer".to_owned(),
                    EntityValue::Number(Number::from_i128(1).unwrap()),
                ),
                ("null".to_owned(), EntityValue::Null),
                ("boolean".to_owned(), EntityValue::Bool(true)),
                ("bytes".to_owned(), EntityValue::Bytes(ByteWrapper::from_array(b"hello"))),
                ("ni".to_owned(), EntityValue::NI(NI::new(&"foo", &"bar"))),
                ("uuid".to_owned(), EntityValue::UUID(UUID::parse(&"1"))),
                ("empty_array".to_owned(), EntityValue::Array(vec![])),
                (
                    "empty_object".to_owned(),
                    EntityValue::Object(HashMap::new()),
                ),
                (
                    "object_with_uri".to_owned(),
                    EntityValue::Object(HashMap::from([(
                        "uri".to_owned(),
                        EntityValue::URI(URI::parse("http://vg.no")),
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
