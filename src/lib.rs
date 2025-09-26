use serde_json::{Value, Map};


/*

            [
              ["add", "hello", 
                ["concat", "wor", 1, 
                  ["concat", "l", ["lower", "_S.x.y"], null]
                ]
              ]
            ]

            // needs to be optimized

            [
              ["add", "hello", 
                ["concat", ["string_literal", "wor"], ["number_literal", 1], 
                  ["concat", ["string_literal", "l"], ["lower", ["path", "x", "y", ["source"]]], ["null_literal"]]
                ]
              ]
            ]

 */
fn generated_udf(source: &Value) -> Value {
    let mut ctx = DtlContext::new(source);
    ctx.add("hello", 
    ctx.concat(&[
        ctx.string_literal("wor"), 
        ctx.number_literal(1), 
        ctx.concat(&
            [
            ctx.string_literal("l"), 
            ctx.lower(
                &ctx.eval_path(&["x", "y"], ctx.source())
            ), 
            ctx.null_literal()
            ]
        )])
    );
    Value::Object(ctx.result.clone())
}

struct DtlContext {
    source: Value,
    result: Map<String, Value>,
}

impl DtlContext {
    fn new(source: &Value) -> Self {
        DtlContext {
            source: source.clone(),
            result: Map::new(),
        }
    }

    fn add(&mut self, key: &str, value: Value) {
        self.result.insert(key.to_string(), value);
    }

    fn string_literal(&self, s: &str) -> Value {
        Value::String(s.to_string())
    }

    fn concat(&self, parts: &[Value]) -> Value {
        let mut s = String::new();
        for part in parts {
            if let Value::String(ref ss) = part {
                s.push_str(ss);
            }
        }
        Value::String(s)
    }

    fn lower(&self, source: &Value) -> Value {
        if let Value::String(ref s) = source {
            Value::String(s.to_lowercase())
        } else {
            Value::Null
        }
    }

    fn null_literal(&self) -> Value {
        Value::Null
    }

    fn number_literal(&self, n: usize) -> Value {
        Value::Number(serde_json::Number::from(n))
    }

    fn eval_path(&self, arg: &[&str], value: &Value) -> Value {
        if arg.is_empty() {
            return value.clone();
        }
        let (first, rest) = arg.split_first().unwrap();
        if let Value::Object(map) = value {
            if let Some(v) = map.get(*first) {
                return self.eval_path(rest, v);
            }
        }
        Value::Null
    }
    
    fn source(&self) -> &Value {
        &self.source
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use pretty_assertions::assert_eq;  // optional, nicer diffs

    #[test]
    fn test_generated_code() {
        // This function is just to ensure the generated code compiles.
        let source = json!({
            "x": { "y": "D" }
        });
        let result = generated_udf(&source);
        let expected = json!({
            "hello": "world"
        });
        assert_eq!(expected, result);
    }
}