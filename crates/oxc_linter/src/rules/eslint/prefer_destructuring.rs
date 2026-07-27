use oxc_ast::AstKind;
use oxc_macros::declare_oxc_lint;

use crate::{
    AstNode,
    context::LintContext,
    rule::Rule,
    rules::shared::prefer_destructuring::{
        DOCUMENTATION, PreferDestructuring as PreferDestructuringInner, PreferDestructuringConfig,
    },
};

#[derive(Debug, Default, Clone)]
pub struct PreferDestructuring(PreferDestructuringInner);

declare_oxc_lint!(
    PreferDestructuring,
    eslint,
    style,
    conditional_fix,
    config = PreferDestructuringConfig,
    docs = DOCUMENTATION,
    version = "1.10.0",
    short_description = "Require destructuring from arrays and/or objects.",
);

impl Rule for PreferDestructuring {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        PreferDestructuringInner::from_configuration(value).map(Self)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::AssignmentExpression(assign_expr) if assign_expr.operator.is_assign() => {
                self.0.run_on_assignment_expression(assign_expr, ctx);
            }
            AstKind::VariableDeclarator(declarator) => {
                self.0.run_on_variable_declarator(declarator, ctx);
            }
            _ => {}
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("var [foo] = array;", None),
        ("var { foo } = object;", None),
        ("const foo: string = object.foo;", None),
        ("const foo: string = object['foo'];", None),
        ("const foo: string = array[0];", None),
        (
            "const object = { foo: 'value' as const }; const foo: string = object.foo;",
            Some(serde_json::json!([
                {
                    "VariableDeclarator": { "array": false, "object": true },
                    "AssignmentExpression": { "array": false, "object": false }
                },
                {
                    "enforceForDeclarationWithTypeAnnotation": false,
                    "enforceForRenamedProperties": false
                }
            ])),
        ),
        (
            "const foo: string = object.foo;",
            Some(serde_json::json!([{ "object": true }, { "enforceForRenamedProperties": true }])),
        ),
        (
            "a = b.c",
            Some(
                serde_json::json!([{ "AssignmentExpression": { "object": false } }, { "enforceForRenamedProperties": true }]),
            ),
        ),
        (
            "let a = arr[0];",
            Some(
                serde_json::json!([{ "AssignmentExpression": { "object": true, "array": true } }]),
            ),
        ),
        (
            "var a = arr[0];",
            Some(serde_json::json!([{ "VariableDeclarator": { "object": true } }])),
        ),
        ("a = arr[0];", Some(serde_json::json!([{ "AssignmentExpression": { "object": true } }]))),
        ("let a = arr[0];", Some(serde_json::json!([{ "object": true }]))),
        ("var foo;", None),
        (
            "var foo = object.bar;",
            Some(serde_json::json!([{ "VariableDeclarator": { "object": true } }])),
        ),
        ("var foo = object.bar;", Some(serde_json::json!([{ "object": true }]))),
        (
            "var foo = object.bar;",
            Some(
                serde_json::json!([ { "VariableDeclarator": { "object": true } }, { "enforceForRenamedProperties": false }, ]),
            ),
        ),
        (
            "var foo = object.bar;",
            Some(serde_json::json!([{ "object": true }, { "enforceForRenamedProperties": false }])),
        ),
        (
            "var foo = object['bar'];",
            Some(
                serde_json::json!([ { "VariableDeclarator": { "object": true } }, { "enforceForRenamedProperties": false }, ]),
            ),
        ),
        (
            "var foo = object[bar];",
            Some(serde_json::json!([{ "object": true }, { "enforceForRenamedProperties": false }])),
        ),
        (
            "var { bar: foo } = object;",
            Some(
                serde_json::json!([ { "VariableDeclarator": { "object": true } }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "var { bar: foo } = object;",
            Some(serde_json::json!([{ "object": true }, { "enforceForRenamedProperties": true }])),
        ),
        (
            "var { [bar]: foo } = object;",
            Some(
                serde_json::json!([ { "VariableDeclarator": { "object": true } }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "var { [bar]: foo } = object;",
            Some(serde_json::json!([{ "object": true }, { "enforceForRenamedProperties": true }])),
        ),
        (
            "var foo = array[0];",
            Some(serde_json::json!([{ "VariableDeclarator": { "array": false } }])),
        ),
        ("var foo = array[0];", Some(serde_json::json!([{ "array": false }]))),
        (
            "var foo = object.foo;",
            Some(serde_json::json!([{ "VariableDeclarator": { "object": false } }])),
        ),
        (
            "var foo = object['foo'];",
            Some(serde_json::json!([{ "VariableDeclarator": { "object": false } }])),
        ),
        ("({ foo } = object);", None),
        (
            "var foo = array[0];",
            Some(
                serde_json::json!([ { "VariableDeclarator": { "array": false } }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "var foo = array[0];",
            Some(serde_json::json!([{ "array": false }, { "enforceForRenamedProperties": true }])),
        ),
        ("[foo] = array;", None),
        ("foo += array[0]", None),
        ("foo &&= array[0]", None), // { "ecmaVersion": 2021 },
        ("foo += bar.foo", None),
        ("foo ||= bar.foo", None),    // { "ecmaVersion": 2021 },
        ("foo ??= bar['foo']", None), // { "ecmaVersion": 2021 },
        (
            "foo = object.foo;",
            Some(
                serde_json::json!([ { "AssignmentExpression": { "object": false } }, { "enforceForRenamedProperties": true } ]),
            ),
        ),
        (
            "foo = object.foo;",
            Some(
                serde_json::json!([ { "AssignmentExpression": { "object": false } }, { "enforceForRenamedProperties": false } ]),
            ),
        ),
        (
            "foo = array[0];",
            Some(
                serde_json::json!([ { "AssignmentExpression": { "array": false } }, { "enforceForRenamedProperties": true } ]),
            ),
        ),
        (
            "foo = array[0];",
            Some(serde_json::json!([ { "AssignmentExpression": { "array": false } } ])),
        ),
        (
            "foo = array[0];",
            Some(
                serde_json::json!([ { "VariableDeclarator": { "array": true }, "AssignmentExpression": { "array": false } } ]),
            ),
        ),
        (
            "var foo = array[0];",
            Some(
                serde_json::json!([ { "VariableDeclarator": { "array": false }, "AssignmentExpression": { "array": true } } ]),
            ),
        ),
        (
            "foo = object.foo;",
            Some(
                serde_json::json!([ { "VariableDeclarator": { "object": true }, "AssignmentExpression": { "object": false } } ]),
            ),
        ),
        (
            "var foo = object.foo;",
            Some(
                serde_json::json!([ { "VariableDeclarator": { "object": false }, "AssignmentExpression": { "object": true } } ]),
            ),
        ),
        ("class Foo extends Bar { static foo() {var foo = super.foo} }", None),
        ("foo = bar[foo];", None),
        ("var foo = bar[foo];", None),
        ("var {foo: {bar}} = object;", Some(serde_json::json!([{ "object": true }]))),
        ("var {bar} = object.foo;", Some(serde_json::json!([{ "object": true }]))),
        ("var foo = array?.[0];", None),
        ("var foo = object?.foo;", None),
        ("class C { #x; foo() { const x = this.#x; } }", None),
        ("class C { #x; foo() { x = this.#x; } }", None),
        ("class C { #x; foo(a) { x = a.#x; } }", None),
        (
            "class C { #x; foo() { const x = this.#x; } }",
            Some(
                serde_json::json!([ { "array": true, "object": true }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "class C { #x; foo() { const y = this.#x; } }",
            Some(
                serde_json::json!([ { "array": true, "object": true }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "class C { #x; foo() { x = this.#x; } }",
            Some(
                serde_json::json!([ { "array": true, "object": true }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "class C { #x; foo() { y = this.#x; } }",
            Some(
                serde_json::json!([ { "array": true, "object": true }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "class C { #x; foo(a) { x = a.#x; } }",
            Some(
                serde_json::json!([ { "array": true, "object": true }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "class C { #x; foo(a) { y = a.#x; } }",
            Some(
                serde_json::json!([ { "array": true, "object": true }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "class C { #x; foo() { x = this.a.#x; } }",
            Some(
                serde_json::json!([ { "array": true, "object": true }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        ("using foo = array[0];", None), // { "sourceType": "module", "ecmaVersion": 2026, }
        ("using foo = object.foo;", None), // { "sourceType": "module", "ecmaVersion": 2026, }
        ("await using foo = array[0];", None), // { "sourceType": "module", "ecmaVersion": 2026, }
        ("await using foo = object.foo;", None), // { "sourceType": "module", "ecmaVersion": 2026, }
    ];

    let fail = vec![
        ("var foo = array[0];", None),
        ("foo = array[0];", None),
        ("var foo = object.foo;", None),
        (
            "var foo: string = object.foo;",
            Some(
                serde_json::json!([{ "object": true }, { "enforceForDeclarationWithTypeAnnotation": true }]),
            ),
        ),
        (
            "var foo: string = object['foo'];",
            Some(
                serde_json::json!([{ "object": true }, { "enforceForDeclarationWithTypeAnnotation": true }]),
            ),
        ),
        (
            "var foo: string = array[0];",
            Some(
                serde_json::json!([{ "array": true }, { "enforceForDeclarationWithTypeAnnotation": true }]),
            ),
        ),
        (
            "var foo: string = object.foo;",
            Some(serde_json::json!([
                { "object": true },
                {
                    "enforceForDeclarationWithTypeAnnotation": true,
                    "enforceForRenamedProperties": true
                }
            ])),
        ),
        (
            "var foo: string = object['foo'];",
            Some(serde_json::json!([
                { "object": true },
                {
                    "enforceForDeclarationWithTypeAnnotation": true,
                    "enforceForRenamedProperties": true
                }
            ])),
        ),
        ("var foo = (a, b).foo;", None),
        ("var length = (() => {}).length;", None),
        ("var foo = (a = b).foo;", None),
        ("var foo = (a || b).foo;", None),
        ("var foo = (f()).foo;", None),
        ("var foo = object.bar.foo;", None),
        (
            "var foobar = object.bar;",
            Some(
                serde_json::json!([ { "VariableDeclarator": { "object": true } }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "var foobar = object.bar;",
            Some(serde_json::json!([{ "object": true }, { "enforceForRenamedProperties": true }])),
        ),
        (
            "var foo = object[bar];",
            Some(
                serde_json::json!([ { "VariableDeclarator": { "object": true } }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "var foo = object[bar];",
            Some(serde_json::json!([{ "object": true }, { "enforceForRenamedProperties": true }])),
        ),
        (
            "var foo = object[foo];",
            Some(serde_json::json!([{ "object": true }, { "enforceForRenamedProperties": true }])),
        ),
        ("var foo = object['foo'];", None),
        ("foo = object.foo;", None),
        ("foo = object['foo'];", None),
        (
            "var foo = array[0];",
            Some(
                serde_json::json!([ { "VariableDeclarator": { "array": true } }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "foo = array[0];",
            Some(serde_json::json!([{ "AssignmentExpression": { "array": true } }])),
        ),
        (
            "var foo = array[0];",
            Some(
                serde_json::json!([ { "VariableDeclarator": { "array": true }, "AssignmentExpression": { "array": false }, }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "var foo = array[0];",
            Some(
                serde_json::json!([ { "VariableDeclarator": { "array": true }, "AssignmentExpression": { "array": false }, }, ]),
            ),
        ),
        (
            "foo = array[0];",
            Some(
                serde_json::json!([ { "VariableDeclarator": { "array": false }, "AssignmentExpression": { "array": true }, }, ]),
            ),
        ),
        (
            "foo = object.foo;",
            Some(
                serde_json::json!([ { "VariableDeclarator": { "array": true, "object": false }, "AssignmentExpression": { "object": true }, }, ]),
            ),
        ),
        ("class Foo extends Bar { static foo() {var bar = super.foo.bar} }", None),
        ("var /* comment */ foo = object.foo;", None),
        ("var a, /* comment */foo = object.foo;", None),
        ("var foo /* comment */ = object.foo;", None),
        ("var a, foo /* comment */ = object.foo;", None),
        ("var foo /* comment */ = object.foo, a;", None),
        (
            "var foo // comment
             = object.foo;",
            None,
        ),
        ("var foo = /* comment */ object.foo;", None),
        (
            "var foo = // comment
             object.foo;",
            None,
        ),
        ("var foo = (/* comment */ object).foo;", None),
        ("var foo = (object /* comment */).foo;", None),
        ("var foo = bar(/* comment */).foo;", None),
        ("var foo = bar/* comment */.baz.foo;", None),
        (
            "var foo = bar[// comment
            baz].foo;",
            None,
        ),
        (
            "var foo // comment
             = bar(/* comment */).foo;",
            None,
        ),
        ("var foo = bar/* comment */.baz/* comment */.foo;", None),
        (
            "var foo = object// comment
            .foo;",
            None,
        ),
        ("var foo = object./* comment */foo;", None),
        ("var foo = (/* comment */ object.foo);", None),
        ("var foo = (object.foo /* comment */);", None),
        ("var foo = object.foo/* comment */;", None),
        ("var foo = object.foo// comment", None),
        ("var foo = object.foo/* comment */, a;", None),
        (
            "var foo = object.foo// comment
            , a;",
            None,
        ),
        ("var foo = object.foo, /* comment */ a;", None),
    ];

    let fix: Vec<(&str, &str, Option<serde_json::Value>)> = vec![
        ("var foo = object.foo;", "var {foo} = object;", None),
        ("var foo = (a, b).foo;", "var {foo} = (a, b);", None),
        // ("var length = (() => {}).length;", "var {length} = () => {};", None),
        // ("var foo = (a = b).foo;", "var {foo} = a = b;", None),
        // ("var foo = (a || b).foo;", "var {foo} = a || b;", None),
        ("var foo = (f()).foo;", "var {foo} = f();", None),
        ("var foo = object.bar.foo;", "var {foo} = object.bar;", None),
        (
            "class Foo extends Bar { static foo() {var bar = super.foo.bar} }",
            "class Foo extends Bar { static foo() {var {bar} = super.foo} }",
            None,
        ),
        ("var /* comment */ foo = object.foo;", "var /* comment */ {foo} = object;", None),
        ("var a, /* comment */foo = object.foo;", "var a, /* comment */{foo} = object;", None),
        ("var foo = bar(/* comment */).foo;", "var {foo} = bar(/* comment */);", None),
        ("var foo = bar/* comment */.baz.foo;", "var {foo} = bar/* comment */.baz;", None),
        (
            "var foo = bar[// comment
                baz].foo;",
            "var {foo} = bar[// comment
                baz];",
            None,
        ),
        ("var foo = (bar[baz]).foo;", "var {foo} = bar[baz];", None),
        ("var foo = object.foo/* comment */;", "var {foo} = object/* comment */;", None),
        ("var foo = object.foo// comment", "var {foo} = object// comment", None),
        ("var foo = object.foo/* comment */, a;", "var {foo} = object/* comment */, a;", None),
        (
            "var foo = object.foo// comment
                , a;",
            "var {foo} = object// comment
                , a;",
            None,
        ),
        ("var foo = object.foo, /* comment */ a;", "var {foo} = object, /* comment */ a;", None),
        ("var foo = object['foo'];", "var {foo} = object;", None),
        (
            "var foo: string = object.foo;",
            "var foo: string = object.foo;",
            Some(
                serde_json::json!([{ "object": true }, { "enforceForDeclarationWithTypeAnnotation": true }]),
            ),
        ),
        (
            "var foo: string = object['foo'];",
            "var foo: string = object['foo'];",
            Some(
                serde_json::json!([{ "object": true }, { "enforceForDeclarationWithTypeAnnotation": true }]),
            ),
        ),
        ("foo = object.foo;", "foo = object.foo;", None),
        ("foo = object['foo'];", "foo = object['foo'];", None),
    ];

    Tester::new(PreferDestructuring::NAME, PreferDestructuring::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
