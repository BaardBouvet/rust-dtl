use std::collections::HashMap;

use crate::entity::EntityValue;

#[derive(Debug)]
pub struct Target {
    target: EntityValue,
    filtered: bool,
    created_targets: Vec<EntityValue>,
}

impl Target {
    pub fn new() -> Self {
        Target {
            target: EntityValue::Object(HashMap::new()),
            filtered: false,
            created_targets: Vec::new(),
        }
    }

    pub fn add(&mut self, property_name: &'static str, value: EntityValue) {
        match self.target {
            EntityValue::Object(ref mut map) => {
                map.insert(property_name.into(), value);
            }
            _ => {}
        }
    }

    pub fn output(&self) -> Vec<EntityValue> {
        let mut output = self.created_targets.clone();
        if !self.filtered {
            output.push(self.target.clone());
        }
        output
    }

    pub fn filter(&mut self) {
        self.filtered = true;
    }

    pub fn create(&mut self, value: EntityValue) {
        match value {
            EntityValue::Array(arr) => self.created_targets.extend(arr),
            v => self.created_targets.push(v),
        }
    }
}

fn string_helper(source: &EntityValue, function: impl Fn(&String) -> String) -> EntityValue {
    match source {
        EntityValue::Array(arr) => EntityValue::Array(
            arr.into_iter()
                .filter_map(|s| match s {
                    EntityValue::String(s) => Some(EntityValue::String(function(s))),
                    _ => None,
                })
                .collect(),
        ),
        EntityValue::String(s) => EntityValue::String(function(s)),
        _ => return EntityValue::Array(vec![]),
    }
}

pub fn lower(source: &EntityValue) -> EntityValue {
    string_helper(source, |s| s.to_lowercase())
}

pub fn upper(source: &EntityValue) -> EntityValue {
    string_helper(source, |s| s.to_uppercase())
}

pub fn list_literal(content: &[EntityValue]) -> EntityValue {
    EntityValue::Array(content.to_vec())
}

pub fn null_literal() -> EntityValue {
    EntityValue::Null
}

pub fn number_literal(n: i32) -> EntityValue {
    EntityValue::Number(n.into())
}

pub fn string_literal(s: &str) -> EntityValue {
    EntityValue::String(s.to_string())
}

pub fn concat(parts: &EntityValue) -> EntityValue {
    match parts {
        EntityValue::String(s) => EntityValue::String(s.clone()),
        EntityValue::Array(arr) => {
            let mut s = String::new();
            for part in arr {
                if let EntityValue::String(ref ss) = part {
                    s.push_str(ss);
                }
            }
            EntityValue::String(s)
        }
        _ => EntityValue::String(String::new()),
    }
}

pub fn apply(function: impl Fn(&EntityValue) -> Vec<EntityValue>, items: &EntityValue) -> EntityValue {
    match items {
        EntityValue::Array(arr) => EntityValue::Array(arr.iter().flat_map(|v| function(v)).collect()),
        _ => EntityValue::Array(vec![]),
    }
}

pub fn map(function: impl Fn(&EntityValue) -> EntityValue, items: &EntityValue) -> EntityValue {
    match items {
        EntityValue::Array(arr) => EntityValue::Array(arr.iter().map(|item| function(item)).collect()),
        _ => EntityValue::Null,
    }
}

pub fn path<'a>(arg: EntityValue, value: &'a EntityValue) -> &'a EntityValue {
    fn eval_path<'a>(arg: &[&str], value: &'a EntityValue) -> &'a EntityValue {
        if arg.is_empty() {
            return value;
        }
        let (first, rest) = arg.split_first().unwrap();
        if let EntityValue::Object(map) = value {
            if let Some(v) = map.get(*first) {
                return eval_path(rest, v);
            }
        }
        &EntityValue::Null
    }

    match arg {
        EntityValue::String(s) => eval_path(&[&s], value),
        EntityValue::Array(arr) => {
            let paths: Vec<&str> = arr
                .iter()
                .filter_map(|v| {
                    if let EntityValue::String(s) = v {
                        Some(s.as_str())
                    } else {
                        None
                    }
                })
                .collect();
            eval_path(&paths, value)
        }
        _ => &EntityValue::Null,
    }
}

pub fn json(json: &str) -> EntityValue {
    serde_json::from_str(json).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_lower() {
        assert_eq!(json(r#"["a", "b"]"#), lower(&json(r#"["a", "B", 1, null, []]"#)));
    }

    #[test]
    fn test_concat() {
        assert_eq!(json(r#""aB""#), concat(&json(r#"["a", "B", 1, null, []]"#)));
        assert_eq!(json(r#""a""#), concat(&json(r#""a""#)));
    }
}
