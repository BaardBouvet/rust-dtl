use std::collections::HashMap;

use serde_json::{Value, Map};


#[derive(Debug, Clone)]
struct DtlContext {
    source: Value,
    target: Map<String, Value>,
    created: Vec<Value>,
    rules: HashMap<String, fn(DtlContext) -> DtlContext>,
    parent: Option<Box<DtlContext>>,
    filter_called: bool,
}

impl DtlContext {
    // internal functions

    fn new(source: &Value) -> Self {
        DtlContext {
            source: source.clone(),
            target: Map::new(),
            created: Vec::new(),
            rules: HashMap::new(),
            parent: None,
            filter_called: false,
        }
    }

    fn get_output(self) -> Vec<Value> {
        let mut output = self.created;
        if (!self.filter_called) {
            output.push(Value::Object(self.target));
        }
        output
    }

    fn add_rule(&mut self, arg: &str, source: fn(DtlContext) -> DtlContext)  {
        self.rules.insert(arg.to_string(), source);
    }

    
    fn sub_context(&self, source: &Value) -> DtlContext {
        DtlContext {
            source: source.clone(),
            target: Map::new(),
            created: Vec::new(),
            rules: self.rules.clone(),
            parent: Some(Box::new(self.clone()  )),
            filter_called: false,
        }
    }  


    // DTL functions

    fn add(&mut self, key: &str, value: Value) {
        self.target.insert(key.to_string(), value);
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

    fn lower(source: &Value) -> Value {
        if let Value::String(ref s) = source {
            Value::String(s.to_lowercase())
        } else {
            Value::Null
        }
    }

    fn null_literal() -> Value {
        Value::Null
    }

    fn number_literal(n: usize) -> Value {
        Value::Number(serde_json::Number::from(n))
    }

    fn eval_path(arg: &[&str], value: &Value) -> Value {
        if arg.is_empty() {
            return value.clone();
        }
        let (first, rest) = arg.split_first().unwrap();
        if let Value::Object(map) = value {
            if let Some(v) = map.get(*first) {
                return DtlContext::eval_path(rest, v);
            }
        }
        Value::Null
    }
    
    fn source(&self) -> &Value {
        &self.source
    }

    fn filter(&mut self) {
        self.filter_called = true;
    }
    
    fn create(&mut self, _source: Vec<Value>) {
        self.created.extend(_source);
    }
    
    fn apply(&self, rule_name: &str, source: Value) -> Vec<Value> {
        let rule = self.rules.get(rule_name);   
        match rule {
            Some(f) => match source {
                Value::Object(ref _obj) => f(self.sub_context(&source)).get_output(),
                Value::Array(ref arr) => arr.iter().flat_map(|v| f(self.sub_context(v)).get_output() ).collect(),
                _ => vec![],
            },
            None => vec![],
        }
    }

}


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

fn hello_world(source: &Value) -> Vec<Value> {
    let mut ctx = DtlContext::new(source);
    ctx.add("hello", 
    DtlContext::concat(&[
        DtlContext::string_literal("wor"), 
        DtlContext::number_literal(1), 
        DtlContext::concat(&
            [
            DtlContext::string_literal("l"), 
            DtlContext::lower(
                &DtlContext::eval_path(&["x", "y"], ctx.source())
            ), 
            DtlContext::null_literal()
            ]
        )])
    );
    ctx.get_output()
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
fn create_foo(source: &Value) -> Vec<Value> {
    let mut ctx = DtlContext::new(source);
    ctx.add_rule("foo", |mut ctx| {
        ctx.add("bar", ctx.source().clone());
        ctx
    });
    ctx.create(
        ctx.apply("foo", DtlContext::eval_path(&["foo"], ctx.source()))
    );
    ctx.filter();
    ctx.get_output()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use pretty_assertions::assert_eq;  // optional, nicer diffs

    #[test]
    fn test_hello_world() {
        let source = json!({
            "x": { "y": "D" }
        });
        let result = hello_world(&source);
        let expected = json!({
            "hello": "world"
        });
        assert_eq!(1, result.len());
        assert_eq!(expected, result[0]);
    }

        #[test]
    fn test_create_foo() {
        let source = json!({
            "foo": ["bar", "baz"],
        });
        let result = create_foo(&source);
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
}