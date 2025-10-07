use serde_json::Value;

use crate::dtl::{*};

mod dtl;
mod entity;

// TODO think about extension types, datetime, etc, should we make a separate DtlValue enum?

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
        //TODO think about list handling and when to use references
        concat(&list_literal(&[
            string_literal("wor"),
            number_literal(1),
            concat(&list_literal(&[
                string_literal("l"),
                lower(path(
                    list_literal(&[string_literal("x"), string_literal("y")]),
                    source,
                )),
                null_literal(),
            ])),
        ]),
    ));
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
fn map_upper2(_: &Value) -> Vec<Value> {
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
