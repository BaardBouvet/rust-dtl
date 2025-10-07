use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;

#[derive(Debug, PartialEq, Clone)]
pub struct Date(NaiveDate);
impl Date {
    pub(crate) fn deserialize(value: &str) -> Date {
        Self::parse(&value[2..])
    }
    
    pub(crate) fn can_deserialize(value: &str) -> bool {
        value.starts_with("~t") && !value.contains("T")
    }
    
    pub(crate) fn parse(arg: &str) -> Date {
        Date(NaiveDate::parse_from_str(&arg, DATE_FMT).unwrap())
    }
}

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
#[derive(Debug, PartialEq, Clone)]
pub struct DateTimeWrapper(DateTime<Utc>);
impl DateTimeWrapper {
    pub(crate) fn deserialize(value: &str) -> DateTimeWrapper {
        Self::parse(&value[2..])
    }
    
    pub(crate) fn can_deserialize(value: &str) -> bool {
        value.starts_with("~t") && value.contains("T")
    }
    
    pub(crate) fn parse(arg: &str) -> DateTimeWrapper {
        DateTimeWrapper(DateTime::parse_from_str(&arg, DATE_TIME_FMT).unwrap().to_utc())
    }
}

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
