use serde_json::Value;

/*

           [
             ["add", "hello",
               ["concat", "wor", 1,
                 ["concat", "l", ["lower", "_S.x.y"], null]
               ]
             ]
           ]

           // needs to be rewritten to

           [
             ["add", "hello",
               ["concat", ["string_literal", "wor"], ["number_literal", 1],
                 ["concat", ["string_literal", "l"], ["lower", ["path", "x", "y", ["source"]]], ["null_literal"]]
               ]
             ]
           ]

*/

fn hello_world2(source: &Value) -> Vec<Value> {
    let mut target = Target::new();
    target.add(
        "hello",
        concat(&[
            string_literal("wor"),
            number_literal(1),
            concat(&[
                string_literal("l"),
                lower(path(
                    list_literal(&[string_literal("x"), string_literal("y")]),
                    source,
                )),
                null_literal(),
            ]),
        ]),
    );
    target.output()
}

/*
           [
             ["create",
               ["apply", "foo", ["path", "foo", ["source"]]]
             ],
             ["filter"]
           ]

           foo:
           [
             ["add", "bar", ["source"]]
           ]
*/

fn create_foo2(source: &Value) -> Vec<Value> {
    let foo = |source: &Value| {
        let mut target = Target::new();
        target.add("bar", source.clone());
        target.output()
    };
    let mut target = Target::new();
    target.create(apply(foo, path(string_literal("foo"), source)));
    target.filter();
    target.output()
}

/*
            [
              ["add", "bar",
                ["map",
                  ["upper", "_."],
                  ["list", "a, "B", "c"]
                ]
              ]
            ]
*/
fn map_upper2(source: &Value) -> Vec<Value> {
    let mut target = Target::new();
    target.add(
        "bar",
        map(
            |s| upper(s),
            &Value::Array(vec![
                Value::String("a".into()),
                Value::String("B".into()),
                Value::String("c".into()),
            ]),
        ),
    );
    target.output()
}

struct Target {
    target: Value,
    filtered: bool,
    created_targets: Vec<Value>,
}

impl Target {
    fn new() -> Self {
        Target {
            target: Value::Object(serde_json::Map::new()),
            filtered: false,
            created_targets: Vec::new(),
        }
    }

    fn add(&mut self, property_name: &'static str, value: Value) {
        match self.target {
            Value::Object(ref mut map) => {
                map.insert(property_name.into(), value);
            }
            _ => {}
        }
    }

    fn output(&self) -> Vec<Value> {
        let mut output = self.created_targets.clone();
        if !self.filtered {
            output.push(self.target.clone());
        }
        output
    }

    fn filter(&mut self) {
        self.filtered = true;
    }

    fn create(&mut self, value: Value) {
        match value {
            Value::Array(arr) => self.created_targets.extend(arr),
            v => self.created_targets.push(v),
        }
    }
}

fn lower(source: &Value) -> Value {
    match source {
        Value::Array(arr) => Value::Array(
            arr.into_iter()
                .map(|s| match s {
                    Value::String(s) => Value::String(s.to_lowercase()),
                    _ => Value::Null,
                })
                .collect(),
        ),
        Value::String(s) => Value::String(s.to_lowercase()),
        _ => return Value::Array(vec![]),
    }
}

// TODO refactor with above
fn upper(s: &Value) -> Value {
    match s {
        Value::String(s) => Value::String(s.to_uppercase()),
        _ => Value::Null,
    }
}

fn list_literal(content: &[Value]) -> Value {
    Value::Array(content.to_vec())
}

fn null_literal() -> Value {
    Value::Null
}

fn number_literal(n: i32) -> Value {
    Value::Number(n.into())
}

fn string_literal(s: &str) -> Value {
    Value::String(s.to_string())
}

fn concat(parts: &[Value]) -> Value {
    let mut s = String::new();
    for part in parts {
        if let Value::String(ref ss) = part {
            s.push_str(ss);
        }
    }
    Value::String(s)
}

fn apply(function: impl Fn(&Value) -> Vec<Value>, items: &Value) -> Value {
    match items {
        Value::Array(arr) => Value::Array(arr.iter().flat_map(|v| function(v)).collect()),
        _ => Value::Array(vec![]),
    }
}

fn map(function: impl Fn(&Value) -> Value, items: &Value) -> Value {
    match items {
        Value::Array(arr) => Value::Array(arr.iter().map(|item| function(item)).collect()),
        _ => Value::Null,
    }
}

fn path<'a>(arg: Value, value: &'a Value) -> &'a Value {
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
    use serde_json::json; // optional, nicer diffs

    #[test]
    fn test_hello_world2() {
        let source = json!({
            "x": { "y": "D" }
        });
        let result = hello_world2(&source);
        let expected = json!({
            "hello": "world"
        });
        assert_eq!(1, result.len());
        assert_eq!(expected, result[0]);
    }

    #[test]
    fn test_create_foo2() {
        let source = json!({
            "foo": ["bar", "baz"],
        });
        let result = create_foo2(&source);
        let expected1: Value = json!(
            {"bar": "bar"}
        );
        let expected2: Value = json!(
            {"bar": "baz"}
        );
        assert_eq!(2, result.len());
        assert_eq!(expected1, result[0]);
        assert_eq!(expected2, result[1]);
    }

    #[test]
    fn test_map_upper2() {
        let source = json!({});
        let result = map_upper2(&source);
        let expected1: Value = json!(
            {"bar": ["A", "B", "C"]}
        );
        assert_eq!(1, result.len());
        assert_eq!(expected1, result[0]);
    }
}
