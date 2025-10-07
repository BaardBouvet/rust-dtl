use crate::{dtl::*, entity::EntityValue};

mod dtl;
mod entity;

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

fn hello_world2(source: &EntityValue) -> Vec<EntityValue> {
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

fn create_foo2(source: &EntityValue) -> Vec<EntityValue> {
    let foo = |source: &EntityValue| {
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
fn map_upper2(_: &EntityValue) -> Vec<EntityValue> {
    let mut target = Target::new();
    target.add(
        "bar",
        map(
            |s| upper(s),
            &EntityValue::Array(vec![
                EntityValue::String("a".into()),
                EntityValue::String("B".into()),
                EntityValue::String("c".into()),
            ]),
        ),
    );
    target.output()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_hello_world2() {
        let source = json(r#"{
            "x": { "y": "D" }
        }"#);
        let result = hello_world2(&source);
        let expected = json(r#"{
            "hello": "world"
        }"#);
        assert_eq!(1, result.len());
        assert_eq!(expected, result[0]);
    }

    #[test]
    fn test_create_foo2() {
        let source = json(r#"{
            "foo": ["bar", "baz"]
        }"#);
        let result = create_foo2(&source);
        let expected1: EntityValue = json(r#"
            {"bar": "bar"}
        "#);
        let expected2: EntityValue = json(r#"
            {"bar": "baz"}
        "#);
        assert_eq!(2, result.len());
        assert_eq!(expected1, result[0]);
        assert_eq!(expected2, result[1]);
    }

    #[test]
    fn test_map_upper2() {
        let source = json(r#"{}"#);
        let result = map_upper2(&source);
        let expected1: EntityValue = json(r#"
            {"bar": ["A", "B", "C"]}
        "#);
        assert_eq!(1, result.len());
        assert_eq!(expected1, result[0]);
    }
}
