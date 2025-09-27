use serde_json::{Value};

use crate::dtl::DtlContext;

mod dtl;


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

fn hello_world(mut ctx: DtlContext) -> DtlContext {
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
    ctx
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
fn create_foo(mut ctx: DtlContext) -> DtlContext {
    ctx.add_rule("foo", |mut ctx: DtlContext| {
        ctx.add("bar", ctx.source().clone());
        ctx
    });
    ctx.create(
        ctx.apply("foo", DtlContext::eval_path(&["foo"], ctx.source()))
    );
    ctx.filter();
    ctx
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
        let result = hello_world(DtlContext::new(&source)).get_output();
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
        let result = create_foo(DtlContext::new(&source)).get_output();
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