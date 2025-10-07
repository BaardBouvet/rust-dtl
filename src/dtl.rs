use serde_json::Value;

#[derive(Debug)]
pub struct Target {
    target: Value,
    filtered: bool,
    created_targets: Vec<Value>,
}

impl Target {
    pub fn new() -> Self {
        Target {
            target: Value::Object(serde_json::Map::new()),
            filtered: false,
            created_targets: Vec::new(),
        }
    }

    pub fn add(&mut self, property_name: &'static str, value: Value) {
        match self.target {
            Value::Object(ref mut map) => {
                map.insert(property_name.into(), value);
            }
            _ => {}
        }
    }

    pub fn output(&self) -> Vec<Value> {
        let mut output = self.created_targets.clone();
        if !self.filtered {
            output.push(self.target.clone());
        }
        output
    }

    pub fn filter(&mut self) {
        self.filtered = true;
    }

    pub fn create(&mut self, value: Value) {
        match value {
            Value::Array(arr) => self.created_targets.extend(arr),
            v => self.created_targets.push(v),
        }
    }
}

fn string_helper(source: &Value, function: impl Fn(&String) -> String) -> Value {
    match source {
        Value::Array(arr) => Value::Array(
            arr.into_iter()
                .filter_map(|s| match s {
                    Value::String(s) => Some(Value::String(function(s))),
                    _ => None,
                })
                .collect(),
        ),
        Value::String(s) => Value::String(function(s)),
        _ => return Value::Array(vec![]),
    }
}

pub fn lower(source: &Value) -> Value {
    string_helper(source, |s| s.to_lowercase())
}

pub fn upper(source: &Value) -> Value {
    string_helper(source, |s| s.to_uppercase())
}

pub fn list_literal(content: &[Value]) -> Value {
    Value::Array(content.to_vec())
}

pub fn null_literal() -> Value {
    Value::Null
}

pub fn number_literal(n: i32) -> Value {
    Value::Number(n.into())
}

pub fn string_literal(s: &str) -> Value {
    Value::String(s.to_string())
}

pub fn concat(parts: &Value) -> Value {
    match parts {
        Value::String(s) => Value::String(s.clone()),
        Value::Array(arr) => {
            let mut s = String::new();
            for part in arr {
                if let Value::String(ref ss) = part {
                    s.push_str(ss);
                }
            }
            Value::String(s)
        }
        _ => Value::String(String::new()),
    }
}

pub fn apply(function: impl Fn(&Value) -> Vec<Value>, items: &Value) -> Value {
    match items {
        Value::Array(arr) => Value::Array(arr.iter().flat_map(|v| function(v)).collect()),
        _ => Value::Array(vec![]),
    }
}

pub fn map(function: impl Fn(&Value) -> Value, items: &Value) -> Value {
    match items {
        Value::Array(arr) => Value::Array(arr.iter().map(|item| function(item)).collect()),
        _ => Value::Null,
    }
}

pub fn path<'a>(arg: Value, value: &'a Value) -> &'a Value {
    fn eval_path<'a>(arg: &[&str], value: &'a Value) -> &'a Value {
        if arg.is_empty() {
            return value;
        }
        let (first, rest) = arg.split_first().unwrap();
        if let Value::Object(map) = value {
            if let Some(v) = map.get(*first) {
                return eval_path(rest, v);
            }
        }
        &Value::Null
    }

    match arg {
        Value::String(s) => eval_path(&[&s], value),
        Value::Array(arr) => {
            let paths: Vec<&str> = arr
                .iter()
                .filter_map(|v| {
                    if let Value::String(s) = v {
                        Some(s.as_str())
                    } else {
                        None
                    }
                })
                .collect();
            eval_path(&paths, value)
        }
        _ => &Value::Null,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::{json}; // optional, nicer diffs

    #[test]
    fn test_lower() {
        assert_eq!(json!(["a", "b"]), lower(&json!(["a", "B", 1, null, []])));
    }

    #[test]
    fn test_concat() {
        assert_eq!(json!("aB"), concat(&json!(["a", "B", 1, null, []])));
        assert_eq!(json!("a"), concat(&json!("a")));
    }

    #[test]
    fn test_non_transit() {
        assert_eq!("http://google.com", json!("http://google.com").as_str().unwrap());
    }
}
