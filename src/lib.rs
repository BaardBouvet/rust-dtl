use serde_json::{Value, Map};
use std::collections::HashMap;

#[derive(Clone)]
pub struct DtlContext<'a> {
    variables: HashMap<String, Value>,
    rules: &'a HashMap<String, Vec<Value>>,
}

fn eval_rule<'a>(
    source: &Value,
    default_rule: &Vec<Value>,
    rules: &'a HashMap<String, Vec<Value>>,
) -> Vec<Value> {
    // Mutable result map
    let mut result: Map<String, Value> = Map::new();

    // This is the target object we'll mutate
    let mut results: Vec<Value> = vec![];

    // Context with reference to mutable result
    let mut variables: HashMap<String, Value> = HashMap::new();
    variables.insert("_S".to_string(), source.clone());
    // _T will be assigned after initializing result

    let mut context = DtlContext {
        variables,
        rules,
    };

    // set _T to point to our target object
    context.variables.insert("_T".to_string(), Value::Object(result.clone()));

    let mut filter_invoked = false;

    for rule in default_rule {
        match rule {
            Value::Array(arr) => {
                if arr.is_empty() {
                    continue;
                }

                let transform_type = arr[0].as_str().unwrap_or("");

                match transform_type {
                    "add" => {
                        let target_property = arr.get(1).and_then(Value::as_str).unwrap();
                        let expression = &arr[2];
                        let value = eval_expression(expression, &context);
                        result.insert(target_property.to_string(), value);

                        // Update _T
                        context.variables.insert("_T".to_string(), Value::Object(result.clone()));
                    }
                    "copy" => {
                        let include_pattern = arr.get(1).and_then(Value::as_str).unwrap();
                        let include_regex = glob_to_regex(include_pattern);
                        if let Value::Object(source_obj) = source {
                            let mut content: Map<String, Value> = source_obj.iter()
                                .filter(|(k, _)| include_regex.is_match(k))
                                .map(|(k, v)| (k.clone(), v.clone()))
                                .collect();

                            if arr.len() == 3 {
                                let exclude_expr = &arr[2];
                                let exclude_values = eval_expression(exclude_expr, &DtlContext {
                                    variables: HashMap::new(),
                                    rules: &HashMap::new(),
                                });

                                match exclude_values {
                                    Value::String(ref s) => {
                                        let exclude_regex = glob_to_regex(s);
                                        content.retain(|k, _| !exclude_regex.is_match(k));
                                    }
                                    Value::Array(ref a) => {
                                        for val in a {
                                            if let Value::String(ref s) = val {
                                                let exclude_regex = glob_to_regex(s);
                                                content.retain(|k, _| !exclude_regex.is_match(k));
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }

                            result.extend(content);
                            context.variables.insert("_T".to_string(), Value::Object(result.clone()));
                        }
                    }
                    "create" => {
                        let expr = &arr[1];
                        let create_result = eval_expression(expr, &context);
                        match create_result {
                            Value::Array(ref arr) => {
                                for e in arr {
                                    if let Value::Object(ref obj) = e {
                                        results.push(Value::Object(obj.clone()));
                                    }
                                }
                            }
                            Value::Object(ref obj) => {
                                results.push(Value::Object(obj.clone()));
                            }
                            _ => {}
                        }
                    }
                    "filter" => {
                        // discard main result
                        filter_invoked = true;
                    }
                    "comment" => {
                        // noop
                    }
                    _ => panic!("No such transform function: {}", transform_type),
                }
            }
            _ => panic!("Transform must be a JSON array: {:?}", rule),
        }
    }

    // Push main result only if it wasn't filtered out
    if !filter_invoked {
        results.push(Value::Object(result));
    }

    results
}

fn glob_to_regex(pattern: &str) -> regex::Regex {
    // very simple: replace '*' with ".*", escape other regex chars
    // (this is simplistic; in production you'd want more careful escaping)
    let mut regex_str = String::from("^");
    for c in pattern.chars() {
        match c {
            '*' => regex_str.push_str(".*"),
            '.' | '\\' | '+' | '?' | '(' | ')' | '[' | ']' | '{' | '}' | '^' | '$' | '|' => {
                regex_str.push('\\');
                regex_str.push(c);
            }
            other => regex_str.push(other),
        }
    }
    regex_str.push('$');
    regex::Regex::new(&regex_str).unwrap()
}

fn dtl_path<'a>(target: &'a Map<String, Value>, path: &[String]) -> Option<&'a Value> {
    if path.is_empty() {
        return None;
    }
    let first = &path[0];
    let mut current = target.get(first)?;
    for p in &path[1..] {
        if let Value::Object(map) = current {
            current = map.get(p)?;
        } else {
            return None;
        }
    }
    Some(current)
}

fn eval_expression(expression: &Value, context: &DtlContext) -> Value {
    match expression {
        Value::String(_) | Value::Number(_) | Value::Bool(_) | Value::Null => {
            eval_primitive(expression, context)
        }
        Value::Array(arr) => {
            eval_function(arr, context)
        }
        Value::Object(_) => {
            expression.clone()
        }
    }
}

fn eval_primitive(expression: &Value, context: &DtlContext) -> Value {
    if let Value::String(s) = expression {
        // splitting on '.'
        let parts: Vec<&str> = s.split('.').collect();
        if parts.is_empty() {
            return expression.clone();
        }
        let var_name = parts[0];
        if let Some(variable) = context.variables.get(var_name) {
            let path = &parts[1..];
            if path.is_empty() || (path.len() == 1 && path[0].is_empty()) {
                return variable.clone();
            }
            if let Value::Object(map) = variable {
                let path_strings: Vec<String> = path.iter().map(|&p| p.to_string()).collect();
                if let Some(v) = dtl_path(map, &path_strings) {
                    return v.clone();
                } else {
                    return Value::Null;
                }
            } else {
                return expression.clone();
            }
        } else {
            return expression.clone();
        }
    }
    expression.clone()
}

fn eval_function(arr: &Vec<Value>, context: &DtlContext) -> Value {
    if arr.is_empty() {
        return Value::Null;
    }
    let func = arr[0].as_str().expect("Function should be a string");
    let args: Vec<&Value> = arr.iter().skip(1).collect();
    match func {
        "concat" => eval_concat(&args, context),
        "list" => {
            let vals: Vec<Value> = args.iter()
                .map(|a| eval_expression(a, context))
                .collect();
            Value::Array(vals)
        }
        "map" => eval_map(&args, context),
        "lower" => eval_string_function(&args, context, |s| s.to_lowercase()),
        "upper" => eval_string_function(&args, context, |s| s.to_uppercase()),
        "apply" => eval_apply(&args, context),
        other => panic!("Invalid transform function: {}", other),
    }
}

fn eval_apply(args: &[&Value], context: &DtlContext) -> Value {
    // args[0] = rule name, args[1] = value
    let rule_name = args.get(0)
        .and_then(|v| v.as_str())
        .expect("apply first arg must be rule name string");
    let value_expr = args.get(1)
        .expect("apply second arg must be value");
    let value = eval_expression(value_expr, context);
    let rule_content = context.rules.get(rule_name)
        .unwrap_or_else(|| panic!("Invalid rule in context: {}", rule_name));
    match value {
        Value::Object(ref obj) => {
            let result = eval_rule(&Value::Object(obj.clone()), rule_content, context.rules);
            if result.len() == 1 {
                result.into_iter().next().unwrap()
            } else {
                Value::Array(result)
            }
        }
        Value::Array(ref arr) => {
            let mut results: Vec<Value> = Vec::new();
            for e in arr {
                let res = eval_rule(e, rule_content, context.rules);
                if res.len() == 1 {
                    results.push(res.into_iter().next().unwrap());
                } else {
                    results.push(Value::Array(res));
                }
            }
            Value::Array(results)
        }
        _ => Value::Array(Vec::new()),
    }
}

fn eval_string_function<F>(
    args: &[&Value],
    context: &DtlContext,
    func: F,
) -> Value
where
    F: Fn(&str) -> String,
{
    let arg0 = args.get(0).expect("string function needs one argument");
    let evaluated = eval_expression(arg0, context);
    match evaluated {
        Value::Array(ref arr) => {
            let mut out: Vec<Value> = Vec::new();
            for e in arr {
                if let Value::String(s) = e {
                    out.push(Value::String(func(&s)));
                }
            }
            Value::Array(out)
        }
        Value::String(ref s) => {
            Value::String(func(&s))
        }
        _ => Value::Null,
    }
}

fn eval_map(args: &[&Value], context: &DtlContext) -> Value {
    // args[0] must be a function (array), args[1] is list
    let func_expr = args.get(0).expect("map first arg must be function");
    let list_expr = args.get(1).expect("map second arg must be list");
    let list_val = eval_expression(list_expr, context);
    if let Value::Array(ref arr) = list_val {
        if let Value::Array(func_arr) = func_expr {
            let mut out: Vec<Value> = Vec::new();
            for e in arr {
                // build a temporary context with variable "_" -> e
                let mut temp_vars = HashMap::new();
                temp_vars.insert("_".to_string(), e.clone());
                let temp_context = DtlContext {
                    variables: temp_vars,
                    rules: context.rules,
                };
                let res = eval_function(func_arr, &temp_context);
                out.push(res);
            }
            Value::Array(out)
        } else {
            Value::Array(Vec::new())
        }
    } else {
        Value::Array(Vec::new())
    }
}

fn eval_concat(args: &[&Value], context: &DtlContext) -> Value {
    let mut s = String::new();
    for arg in args {
        let ev = eval_expression(arg, context);
        if let Value::String(ref ss) = ev {
            s.push_str(ss);
        }
    }
    Value::String(s)
}

pub fn eval_transform(
    source: &Map<String, Value>,
    rules: &HashMap<String, Vec<Value>>,
) -> Vec<Value> {
    if let Some(default_rule) = rules.get("default") {
        eval_rule(&Value::Object(source.clone()), default_rule, rules)
    } else {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use pretty_assertions::assert_eq;  // optional, nicer diffs

    #[test]
    fn test_concat() {
        let source = json!({
            "x": { "y": "D" }
        });
        let rule = serde_json::from_str::<Vec<Value>>(r#"
            [
              ["add", "hello", 
                ["concat", "wor", 1, 
                  ["concat", "l", ["lower", "_S.x.y"], null]
                ]
              ]
            ]
        "#).unwrap();
        println!("Rule: {}", serde_json::to_string_pretty(&rule).unwrap());

        let result = eval_rule(&source, &rule, &HashMap::new());

        // result should be an array of one object
        let expected = json!({
            "hello": "world"
        });
        let arr = result;
        assert_eq!(expected, arr[0]);
    }

    #[test]
    fn test_target() {
        let source = json!({});
        let rule = serde_json::from_str::<Vec<Value>>(r#"
            [
              ["add", "hello", {"foo": "world"}],
              ["add", "bar", "_T.hello.foo"]
            ]
        "#).unwrap();
        let result = eval_rule(&source, &rule, &HashMap::new());
        let expected = json!({
            "hello": { "foo": "world" },
            "bar": "world"
        });
        let arr = result;
        assert_eq!(expected, arr[0]);
    }

    #[test]
    fn test_list() {
        let source = json!({});
        let rule = serde_json::from_str::<Vec<Value>>(r#"
            [
              ["add", "bar", 
                ["list", "hello", ["concat", "worl", "d"]]
              ]
            ]
        "#).unwrap();
        let result = eval_rule(&source, &rule, &HashMap::new());
        let expected = json!({
            "bar": ["hello", "world"]
        });
        let arr = result;
        assert_eq!(expected, arr[0]);
    }

    #[test]
    fn test_lower() {
        let source = json!({});
        let rule = serde_json::from_str::<Vec<Value>>(r#"
            [
              ["add", "bar", 
                ["lower", ["list", "A", "B", "C"]]
              ]
            ]
        "#).unwrap();
        let result = eval_rule(&source, &rule, &HashMap::new());
        let expected = json!({
            "bar": ["a", "b", "c"]
        });
        let arr = result;
        assert_eq!(expected, arr[0]);
    }

    #[test]
    fn test_map() {
        let source = json!({});
        let rule = serde_json::from_str::<Vec<Value>>(r#"
            [
              ["add", "bar", 
                ["map", 
                  ["upper", "_."], 
                  ["list", "A", "B", "C"]
                ]
              ]
            ]
        "#).unwrap();
        let result = eval_rule(&source, &rule, &HashMap::new());
        let expected = json!({
            "bar": ["A", "B", "C"]
        });
        let arr = result;
        assert_eq!(expected, arr[0]);
    }

    #[test]
    fn test_copy() {
        let source = json!({
            "foo": "bar",
            "bar": "baz"
        });
        let rule = serde_json::from_str::<Vec<Value>>(r#"
            [
              ["copy", "*", "bar"]
            ]
        "#).unwrap();
        let result = eval_rule(&source, &rule, &HashMap::new());
        let expected = json!({
            "foo": "bar"
        });
        let arr = result;
        assert_eq!(expected, arr[0]);
    }

    #[test]
    fn test_comment_noop() {
        let source = json!({
            "foo": "bar"
        });
        let rule = serde_json::from_str::<Vec<Value>>(r#"
            [
              ["add", "foo", "_S.foo"],
              ["comment", "this is a comment"]
            ]
        "#).unwrap();
        let result = eval_rule(&source, &rule, &HashMap::new());
        let expected = json!({
            "foo": "bar"
        });
        let arr = result;
        assert_eq!(expected, arr[0]);
    }

    #[test]
    fn test_add_and_copy_combined() {
        let source = json!({
            "foo": "bar",
            "baz": "qux"
        });
        let rule = serde_json::from_str::<Vec<Value>>(r#"
            [
              ["add", "hello", "world"],
              ["copy", "ba*"]
            ]
        "#).unwrap();
        let result = eval_rule(&source, &rule, &HashMap::new());
        let expected = json!({
            "hello": "world",
            "baz": "qux"
        });
        let arr = result;
        assert_eq!(expected, arr[0]);
    }

    #[test]
    fn test_create_with_object() {
        let source = json!({});
        let rule = serde_json::from_str::<Vec<Value>>(r#"
            [
              ["create", {"foo": "bar"}],
              ["filter"]
            ]
        "#).unwrap();
        let result = eval_rule(&source, &rule, &HashMap::new());
        let expected = json!({
            "foo": "bar"
        });
        let arr = result;
        assert_eq!(expected, arr[0]);
        assert_eq!(1, arr.len());
    }


    #[test]
    fn test_filter_removes_result() {
        let source = json!({
            "foo": "bar"
        });
        let rule = serde_json::from_str::<Vec<Value>>(r#"
            [
              ["add", "foo", "_S.foo"],
              ["filter"]
            ]
        "#).unwrap();
        let result = eval_rule(&source, &rule, &HashMap::new());
        // After filter, results should be empty (no objects)
        assert_eq!(0, result.len());
    }

    #[test]
    fn test_create_apply() {
        let source = json!({
            "foo": ["bar", "baz"],
        });
        let rule = serde_json::from_str::<Vec<Value>>(r#"
            [
              ["create",
                ["apply", "foo", "_S.foo"]
              ],
              ["filter"]
            ]
        "#).unwrap();
        let foo_rule = serde_json::from_str::<Vec<Value>>(r#"
            [
              ["add", "bar", "_S."]
            ]
        "#).unwrap();
        let mut rules = HashMap::new();
        rules.insert("foo".to_string(), foo_rule);
        let result = eval_rule(&source, &rule, &rules);
        let expected1: Value = json!(
            {"bar": "bar"}
        );
        let expected2: Value = json!(
            {"bar": "baz"}
        );
        let arr = result;
        assert_eq!(expected1, arr[0]);
        assert_eq!(expected2, arr[1]);
        assert_eq!(2, arr.len());
    }

    #[test]
    fn test_create_apply_no_filter() {
        let source = json!({
            "foo": ["bar", "baz"],
        });
        let rule = serde_json::from_str::<Vec<Value>>(r#"
            [
              ["copy", "*"],
              ["create",
                ["apply", "foo", "_S.foo"]
              ]
            ]
        "#).unwrap();
        let foo_rule = serde_json::from_str::<Vec<Value>>(r#"
            [
              ["add", "bar", "_S."]
            ]
        "#).unwrap();
        let mut rules = HashMap::new();
        rules.insert("foo".to_string(), foo_rule);
        let result = eval_rule(&source, &rule, &rules);
        let expected1: Value = json!(
            {"bar": "bar"}
        );
        let expected2: Value = json!(
            {"bar": "baz"}
        );
        let expected3: Value = json!(
            {"foo": ["bar", "baz"]}
        );
        let arr = result;
        assert_eq!(expected1, arr[0]);
        assert_eq!(expected2, arr[1]);
        assert_eq!(expected3, arr[2]);
        assert_eq!(3, arr.len());
    }

}