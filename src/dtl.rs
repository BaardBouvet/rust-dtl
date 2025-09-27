use std::collections::HashMap;

use serde_json::{Map, Value};

#[derive(Debug, Clone)]
pub(crate) struct DtlContext {
    pub(crate) source: Value,
    pub(crate) target: Map<String, Value>,
    pub(crate) created: Vec<Value>,
    pub(crate) rules: HashMap<String, fn(DtlContext) -> DtlContext>,
    pub(crate) parent: Option<Box<DtlContext>>,
    pub(crate) filter_called: bool,
}

impl DtlContext {
    // internal functions

    pub(crate) fn new(source: &Value) -> Self {
        DtlContext {
            source: source.clone(),
            target: Map::new(),
            created: Vec::new(),
            rules: HashMap::new(),
            parent: None,
            filter_called: false,
        }
    }

    pub(crate) fn get_output(self) -> Vec<Value> {
        let mut output = self.created;
        if (!self.filter_called) {
            output.push(Value::Object(self.target));
        }
        output
    }

    pub(crate) fn add_rule(&mut self, arg: &str, source: fn(DtlContext) -> DtlContext)  {
        self.rules.insert(arg.to_string(), source);
    }


    pub(crate) fn sub_context(&self, source: &Value) -> DtlContext {
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

    pub(crate) fn add(&mut self, key: &str, value: Value) {
        self.target.insert(key.to_string(), value);
    }

    pub(crate) fn string_literal(s: &str) -> Value {
        Value::String(s.to_string())
    }

    pub(crate) fn concat(parts: &[Value]) -> Value {
        let mut s = String::new();
        for part in parts {
            if let Value::String(ref ss) = part {
                s.push_str(ss);
            }
        }
        Value::String(s)
    }

    pub(crate) fn lower(source: &Value) -> Value {
        if let Value::String(ref s) = source {
            Value::String(s.to_lowercase())
        } else {
            Value::Null
        }
    }

    pub(crate) fn null_literal() -> Value {
        Value::Null
    }

    pub(crate) fn number_literal(n: usize) -> Value {
        Value::Number(serde_json::Number::from(n))
    }

    pub(crate) fn eval_path(arg: &[&str], value: &Value) -> Value {
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

    pub(crate) fn source(&self) -> &Value {
        &self.source
    }

    pub(crate) fn filter(&mut self) {
        self.filter_called = true;
    }

    pub(crate) fn create(&mut self, _source: Vec<Value>) {
        self.created.extend(_source);
    }

    pub(crate) fn apply(&self, rule_name: &str, source: Value) -> Vec<Value> {
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
