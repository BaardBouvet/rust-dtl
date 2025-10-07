use std::{collections::HashMap, str::FromStr};

use base64::{engine::general_purpose, Engine as _};
use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{
    de::{MapAccess, SeqAccess, Visitor},
    Deserialize, Serialize,
};
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
struct Uri(String);

impl Serialize for Uri {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("~r{}", self.0))
    }
}

#[derive(Debug, PartialEq)]
struct Date(NaiveDate);

const DATE_FMT: &str = "%Y-%m-%d";

impl Serialize for Date {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("~t{}", self.0.format(DATE_FMT)))
    }
}

// TODO consider using a long to store the nanos since epoch
#[derive(Debug, PartialEq)]
struct DateTimeWrapper(DateTime<Utc>);

const DATE_TIME_FMT: &str = "%Y-%m-%dT%H:%M:%S.%f%z";

impl Serialize for DateTimeWrapper {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        //TODO check if trailing zeros is a problem
        serializer.serialize_str(&format!("~t{}", self.0.format(DATE_TIME_FMT)))
    }
}

#[derive(Debug, PartialEq)]
struct ByteWrapper(Vec<u8>);

impl Serialize for ByteWrapper {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("~b{}", general_purpose::STANDARD.encode(&self.0)))
    }
}

#[derive(Debug, PartialEq)]
struct NI{
    namespace: String,
    identifier: String,
}

impl Serialize for NI {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("~:{}:{}", self.namespace, self.identifier))
    }
}

#[derive(Debug, PartialEq)]
struct BigDecimalWrapper(BigDecimal);

impl Serialize for BigDecimalWrapper {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("~f{}", self.0.to_plain_string()))
    }
}

#[derive(Debug, PartialEq)]
struct UUID(String);

impl Serialize for UUID {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("~u{}", self.0))
    }
}

#[derive(Debug, PartialEq)]
enum EntityValue {
    Null,
    Bool(bool),
    // not sure how we can use bigint with serde
    Number(Number),
    String(String),
    Uri(Uri),
    Date(Date),
    DateTime(DateTimeWrapper),
    // TODO uuid
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
            EntityValue::Uri(u) => u.serialize(serializer),
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
                if value.starts_with("~r") {
                    Ok(EntityValue::Uri(Uri(value[2..].to_owned())))
                } else if value.starts_with("~t") {
                    if value.contains("T") {
                        Ok(EntityValue::DateTime(DateTimeWrapper(
                            DateTime::parse_from_str(&value[2..], DATE_TIME_FMT).unwrap().to_utc(),
                        )))
                    } else {
                        Ok(EntityValue::Date(Date(
                            NaiveDate::parse_from_str(&value[2..], DATE_FMT).unwrap(),
                        )))
                    }
                } else if value.starts_with("~b") {
                    Ok(EntityValue::Bytes(ByteWrapper(general_purpose::STANDARD.decode(&value[2..]).unwrap())))
                } else if value.starts_with("~:") {
                    let rest = &value[2..];
                    if let Some(last_colon_index) = rest.rfind(':') {
                        Ok(EntityValue::NI(NI{namespace: rest[0..last_colon_index].to_owned(), identifier: rest[last_colon_index + 1..].to_owned()}))
                    } else {
                        // TODO how to return sensible error
                        todo!()
                    }
                } else if value.starts_with("~f") {
                    Ok(EntityValue::Decimal(BigDecimalWrapper(BigDecimal::from_str(&value[2..]).unwrap())))
                } else if value.starts_with("~u") {
                    Ok(EntityValue::UUID(UUID(value[2..].to_owned())))
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
    use std::str::FromStr;

    use super::*;
    use chrono::TimeZone;
    use pretty_assertions::assert_eq;

    #[test]
    fn uuid() {
        let entity = EntityValue::UUID(UUID("123".to_owned()));
        let serialized = serde_json::to_string(&entity).unwrap();
        assert_eq!(serialized, "\"~u123\"");
        let deserialized: EntityValue = serde_json::from_str(&serialized).unwrap();
        assert_eq!(entity, deserialized);
    }
    #[test]
    fn decimal() {
        let entity = EntityValue::Decimal(BigDecimalWrapper(BigDecimal::from_str("123.456").unwrap()));
        let serialized = serde_json::to_string(&entity).unwrap();
        assert_eq!(serialized, "\"~f123.456\"");
        let deserialized: EntityValue = serde_json::from_str(&serialized).unwrap();
        assert_eq!(entity, deserialized);
    }

    #[test]
    fn ni() {
        let entity = EntityValue::NI(NI{namespace: "foo".to_owned(), identifier: "bar".to_owned()});
        let serialized = serde_json::to_string(&entity).unwrap();
        assert_eq!(serialized, "\"~:foo:bar\"");
        let deserialized: EntityValue = serde_json::from_str(&serialized).unwrap();
        assert_eq!(entity, deserialized);
    }

    #[test]
    fn bytes() {
        let entity = EntityValue::Bytes(ByteWrapper(vec![255]));
        let serialized = serde_json::to_string(&entity).unwrap();
        assert_eq!(serialized, "\"~b/w==\"");
        let deserialized: EntityValue = serde_json::from_str(&serialized).unwrap();
        assert_eq!(entity, deserialized);
    }

    #[test]
    fn date() {
        let entity = EntityValue::Date(Date(NaiveDate::from_str("2020-01-01").unwrap()));
        let serialized = serde_json::to_string(&entity).unwrap();
        assert_eq!(serialized, "\"~t2020-01-01\"");
        let deserialized: EntityValue = serde_json::from_str(&serialized).unwrap();
        assert_eq!(entity, deserialized);
    }

    #[test]
    fn datetime() {
        let entity = EntityValue::DateTime(DateTimeWrapper(Utc.with_ymd_and_hms(2014, 7, 8, 9, 10, 11).unwrap()));
        let serialized = serde_json::to_string(&entity).unwrap();
        assert_eq!(serialized, "\"~t2014-07-08T09:10:11.000000000+0000\"");
        let deserialized: EntityValue = serde_json::from_str(&serialized).unwrap();
        assert_eq!(entity, deserialized);
    }

    #[test]
    fn main() {
        let entity = Entity {
            id: "1".to_owned(),
            filtered: false,
            content: HashMap::from([
                ("string".to_owned(), EntityValue::String("value".to_owned())),
                ("uri".to_owned(), EntityValue::Uri(Uri("db.no".to_owned()))),
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
                ("empty_array".to_owned(), EntityValue::Array(vec![])),
                (
                    "empty_object".to_owned(),
                    EntityValue::Object(HashMap::new()),
                ),
                (
                    "object_with_uri".to_owned(),
                    EntityValue::Object(HashMap::from([(
                        "uri".to_owned(),
                        EntityValue::Uri(Uri("vg.no".to_owned())),
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
