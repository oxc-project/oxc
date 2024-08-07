use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::operator::UnaryOperator;

use crate::{
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::Rule,
    AstNode,
};

#[derive(Debug, Clone)]
pub struct NoMagicNumbers {
    ignore: Vec<f64>,
    ignore_array_indexes: bool,
    ignore_default_values: bool,
    ignore_class_field_initial_values: bool,
    enforce_const: bool,
    detect_objects: bool,
}

impl Default for NoMagicNumbers {
    fn default() -> Self {
        Self {
            ignore: vec![],
            ignore_array_indexes: false,
            ignore_default_values: false,
            ignore_class_field_initial_values: false,
            enforce_const: false,
            detect_objects: false
        }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    ///
    /// ### Why is this bad?
    ///
    ///
    /// ### Example
    /// ```javascript
    /// ```
    NoMagicNumbers,
    nursery, // TODO: change category to `correctness`, `suspicious`, `pedantic`, `perf`, `restriction`, or `style`
             // See <https://oxc.rs/docs/contribute/linter.html#rule-category> for details

    pending  // TODO: describe fix capabilities. Remove if no fix can be done,
             // keep at 'pending' if you think one could be added but don't know how.
             // Options are 'fix', 'fix-dangerous', 'suggestion', and 'suggestion-dangerous'
);

struct InternConfig<'a> {
    node: &'a AstNode<'a>,
    value: f64,
    raw: String
}

impl Rule for NoMagicNumbers {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {

        match node.kind() {
            AstKind::NumericLiteral(literal) => {
                let parent_node = ctx.nodes().parent_node(node.id());

                let intern_config: InternConfig = match parent_node.unwrap().kind() {
                    AstKind::UnaryExpression(unary) if unary.operator == UnaryOperator::UnaryNegation => {
                        InternConfig {
                            node: parent_node.unwrap(),
                            value: 0.0 - literal.value,
                            raw: format!("-{}", literal.raw)
                        }
                    },
                    _ => {
                        InternConfig {
                            node,
                            value: literal.value,
                            raw: literal.raw.to_string()
                        }
                    }
                };

                if self.is_ignore_value(&intern_config.value) {
                    return;
                }
            },
            _ => {}
        }
    }
}

impl NoMagicNumbers {

    fn is_ignore_value(&self, number: &f64) -> bool {
        self.ignore.contains(number)
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let ignore_array_indexes = Some(serde_json::json!([{"ignoreArrayIndexes": true}]));
    let ignore_default_values = Some(serde_json::json!([{ "ignoreDefaultValues": true }]));
    
    let pass = vec![
        ("var x = parseInt(y, 10);", None),
        ("var x = parseInt(y, -10);", None),
        ("var x = Number.parseInt(y, 10);", None),
        ("const foo = 42;", None), // { "ecmaVersion": 6 },
        (
            "var foo = 42;",
            Some(serde_json::json!([{                "enforceConst": false            }])),
        ), // { "ecmaVersion": 6 },
        ("var foo = -42;", None),
        (
            "var foo = 0 + 1 - 2 + -2;",
            Some(serde_json::json!([{                "ignore": [0, 1, 2, -2]            }])),
        ),
        (
            "var foo = 0 + 1 + 2 + 3 + 4;",
            Some(serde_json::json!([{                "ignore": [0, 1, 2, 3, 4]            }])),
        ),
        ("var foo = { bar:10 }", None),
        (
            "setTimeout(function() {return 1;}, 0);",
            Some(serde_json::json!([{                "ignore": [0, 1]            }])),
        ),
        (
            "var data = ['foo', 'bar', 'baz']; var third = data[3];",
            ignore_array_indexes.clone(),
        ),
        (
            "foo[0]",
            ignore_array_indexes.clone(),
        ),
        (
            "foo[-0]",
            ignore_array_indexes.clone(),
        ),
        (
            "foo[1]",
            ignore_array_indexes.clone(),
        ),
        (
            "foo[100]",
            ignore_array_indexes.clone(),
        ),
        (
            "foo[200.00]",
            ignore_array_indexes.clone(),
        ),
        (
            "foo[3e4]",
            ignore_array_indexes.clone(),
        ),
        (
            "foo[1.23e2]",
            ignore_array_indexes.clone(),
        ),
        (
            "foo[230e-1]",
            ignore_array_indexes.clone(),
        ),
        (
            "foo[0b110]",
            ignore_array_indexes.clone(),
        ), // { "ecmaVersion": 2015 },
        (
            "foo[0o71]",
            ignore_array_indexes.clone(),
        ), // { "ecmaVersion": 2015 },
        (
            "foo[0xABC]",
            ignore_array_indexes.clone(),
        ),
        (
            "foo[0123]",
            ignore_array_indexes.clone(),
        ), // {                "sourceType": "script"            },
        (
            "foo[5.0000000000000001]",
            ignore_array_indexes.clone(),
        ),
        (
            "foo[4294967294]",
            ignore_array_indexes.clone(),
        ),
        (
            "foo[0n]",
            ignore_array_indexes.clone(),
        ), // { "ecmaVersion": 2020 },
        (
            "foo[-0n]",
            ignore_array_indexes.clone(),
        ), // { "ecmaVersion": 2020 },
        (
            "foo[1n]",
            ignore_array_indexes.clone(),
        ), // { "ecmaVersion": 2020 },
        (
            "foo[100n]",
            ignore_array_indexes.clone(),
        ), // { "ecmaVersion": 2020 },
        (
            "foo[0xABn]",
            ignore_array_indexes.clone(),
        ), // { "ecmaVersion": 2020 },
        (
            "foo[4294967294n]",
            ignore_array_indexes.clone(),
        ), // { "ecmaVersion": 2020 },
        ("var a = <input maxLength={10} />;", None), // {                "parserOptions": {                    "ecmaFeatures": {                        "jsx": true                    }                }            },
        ("var a = <div objectProp={{ test: 1}}></div>;", None), // {                "parserOptions": {                    "ecmaFeatures": {                        "jsx": true                    }                }            },
        ("f(100n)", Some(serde_json::json!([{ "ignore": ["100n"] }]))), // { "ecmaVersion": 2020 },
        ("f(-100n)", Some(serde_json::json!([{ "ignore": ["-100n"] }]))), // { "ecmaVersion": 2020 },
        (
            "const { param = 123 } = sourceObject;",
            ignore_default_values.clone(),
        ), // { "ecmaVersion": 6 },
        (
            "const func = (param = 123) => {}",
            ignore_default_values.clone(),
        ), // { "ecmaVersion": 6 },
        (
            "const func = ({ param = 123 }) => {}",
            ignore_default_values.clone(),
        ), // { "ecmaVersion": 6 },
        (
            "const [one = 1, two = 2] = []",
            ignore_default_values.clone(),
        ), // { "ecmaVersion": 6 },
        (
            "var one, two; [one = 1, two = 2] = []",
            ignore_default_values.clone(),
        ), // { "ecmaVersion": 6 },
        ("var x = parseInt?.(y, 10);", None), // { "ecmaVersion": 2020 },
        ("var x = Number?.parseInt(y, 10);", None), // { "ecmaVersion": 2020 },
        ("var x = (Number?.parseInt)(y, 10);", None), // { "ecmaVersion": 2020 },
        ("foo?.[777]", Some(serde_json::json!([{ "ignoreArrayIndexes": true }]))), // { "ecmaVersion": 2020 },
        (
            "class C { foo = 2; }",
            Some(serde_json::json!([{ "ignoreClassFieldInitialValues": true }])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { foo = -2; }",
            Some(serde_json::json!([{ "ignoreClassFieldInitialValues": true }])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { static foo = 2; }",
            Some(serde_json::json!([{ "ignoreClassFieldInitialValues": true }])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { #foo = 2; }",
            Some(serde_json::json!([{ "ignoreClassFieldInitialValues": true }])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { static #foo = 2; }",
            Some(serde_json::json!([{ "ignoreClassFieldInitialValues": true }])),
        ), // { "ecmaVersion": 2022 }
    ];

    let fail = vec![
        (
            "var foo = 42",
            Some(serde_json::json!([{                "enforceConst": true            }])),
        ), // { "ecmaVersion": 6 },
        ("var foo = 0 + 1;", None),
        (
            "var foo = 42n",
            Some(serde_json::json!([{                "enforceConst": true            }])),
        ), // {                "ecmaVersion": 2020            },
        ("var foo = 0n + 1n;", None), // {                "ecmaVersion": 2020            },
        ("a = a + 5;", None),
        ("a += 5;", None),
        ("var foo = 0 + 1 + -2 + 2;", None),
        (
            "var foo = 0 + 1 + 2;",
            Some(serde_json::json!([{                "ignore": [0, 1]            }])),
        ),
        (
            "var foo = { bar:10 }",
            Some(serde_json::json!([{                "detectObjects": true            }])),
        ),
        ("console.log(0x1A + 0x02); console.log(071);", None), // {                "sourceType": "script"            },
        (
            "var stats = {avg: 42};",
            Some(serde_json::json!([{                "detectObjects": true            }])),
        ),
        ("var colors = {}; colors.RED = 2; colors.YELLOW = 3; colors.BLUE = 4 + 5;", None),
        ("function getSecondsInMinute() {return 60;}", None),
        ("function getNegativeSecondsInMinute() {return -60;}", None),
        ("var data = ['foo', 'bar', 'baz']; var third = data[3];", None),
        ("var data = ['foo', 'bar', 'baz']; var third = data[3];", Some(serde_json::json!([{}]))),
        (
            "var data = ['foo', 'bar', 'baz']; var third = data[3];",
            Some(serde_json::json!([{                "ignoreArrayIndexes": false            }])),
        ),
        (
            "foo[-100]",
            Some(serde_json::json!([{                "ignoreArrayIndexes": true            }])),
        ),
        (
            "foo[-1.5]",
            Some(serde_json::json!([{                "ignoreArrayIndexes": true            }])),
        ),
        (
            "foo[-1]",
            Some(serde_json::json!([{                "ignoreArrayIndexes": true            }])),
        ),
        (
            "foo[-0.1]",
            Some(serde_json::json!([{                "ignoreArrayIndexes": true            }])),
        ),
        (
            "foo[-0b110]",
            Some(serde_json::json!([{                "ignoreArrayIndexes": true            }])),
        ), // { "ecmaVersion": 2015 },
        (
            "foo[-0o71]",
            Some(serde_json::json!([{                "ignoreArrayIndexes": true            }])),
        ), // { "ecmaVersion": 2015 },
        (
            "foo[-0x12]",
            Some(serde_json::json!([{                "ignoreArrayIndexes": true            }])),
        ),
        (
            "foo[-012]",
            Some(serde_json::json!([{                "ignoreArrayIndexes": true            }])),
        ), // { "sourceType": "script" },
        (
            "foo[0.1]",
            Some(serde_json::json!([{                "ignoreArrayIndexes": true            }])),
        ),
        (
            "foo[0.12e1]",
            Some(serde_json::json!([{                "ignoreArrayIndexes": true            }])),
        ),
        (
            "foo[1.5]",
            Some(serde_json::json!([{                "ignoreArrayIndexes": true            }])),
        ),
        (
            "foo[1.678e2]",
            Some(serde_json::json!([{                "ignoreArrayIndexes": true            }])),
        ),
        (
            "foo[56e-1]",
            Some(serde_json::json!([{                "ignoreArrayIndexes": true            }])),
        ),
        (
            "foo[5.000000000000001]",
            Some(serde_json::json!([{                "ignoreArrayIndexes": true            }])),
        ),
        (
            "foo[100.9]",
            Some(serde_json::json!([{                "ignoreArrayIndexes": true            }])),
        ),
        (
            "foo[4294967295]",
            Some(serde_json::json!([{                "ignoreArrayIndexes": true            }])),
        ),
        (
            "foo[1e300]",
            Some(serde_json::json!([{                "ignoreArrayIndexes": true            }])),
        ),
        (
            "foo[1e310]",
            Some(serde_json::json!([{                "ignoreArrayIndexes": true            }])),
        ),
        (
            "foo[-1e310]",
            Some(serde_json::json!([{                "ignoreArrayIndexes": true            }])),
        ),
        (
            "foo[-1n]",
            Some(serde_json::json!([{                "ignoreArrayIndexes": true            }])),
        ), // { "ecmaVersion": 2020 },
        (
            "foo[-100n]",
            Some(serde_json::json!([{                "ignoreArrayIndexes": true            }])),
        ), // { "ecmaVersion": 2020 },
        (
            "foo[-0x12n]",
            Some(serde_json::json!([{                "ignoreArrayIndexes": true            }])),
        ), // { "ecmaVersion": 2020 },
        (
            "foo[4294967295n]",
            Some(serde_json::json!([{                "ignoreArrayIndexes": true            }])),
        ), // { "ecmaVersion": 2020 },
        (
            "foo[+0]",
            Some(serde_json::json!([{                "ignoreArrayIndexes": true            }])),
        ),
        (
            "foo[+1]",
            Some(serde_json::json!([{                "ignoreArrayIndexes": true            }])),
        ),
        (
            "foo[-(-1)]",
            Some(serde_json::json!([{                "ignoreArrayIndexes": true            }])),
        ),
        (
            "foo[+0n]",
            Some(serde_json::json!([{                "ignoreArrayIndexes": true            }])),
        ), // { "ecmaVersion": 2020 },
        (
            "foo[+1n]",
            Some(serde_json::json!([{                "ignoreArrayIndexes": true            }])),
        ), // { "ecmaVersion": 2020 },
        (
            "foo[- -1n]",
            Some(serde_json::json!([{                "ignoreArrayIndexes": true            }])),
        ), // { "ecmaVersion": 2020 },
        (
            "100 .toString()",
            Some(serde_json::json!([{                "ignoreArrayIndexes": true            }])),
        ),
        (
            "200[100]",
            Some(serde_json::json!([{                "ignoreArrayIndexes": true            }])),
        ),
        ("var a = <div arrayProp={[1,2,3]}></div>;", None), // {                "parserOptions": {                    "ecmaFeatures": {                        "jsx": true                    }                }            },
        ("var min, max, mean; min = 1; max = 10; mean = 4;", Some(serde_json::json!([{}]))),
        ("f(100n)", Some(serde_json::json!([{ "ignore": [100] }]))), // { "ecmaVersion": 2020 },
        ("f(-100n)", Some(serde_json::json!([{ "ignore": ["100n"] }]))), // { "ecmaVersion": 2020 },
        ("f(100n)", Some(serde_json::json!([{ "ignore": ["-100n"] }]))), // { "ecmaVersion": 2020 },
        ("f(100)", Some(serde_json::json!([{ "ignore": ["100n"] }]))),
        (
            "const func = (param = 123) => {}",
            Some(serde_json::json!([{ "ignoreDefaultValues": false }])),
        ), // { "ecmaVersion": 6 },
        ("const { param = 123 } = sourceObject;", Some(serde_json::json!([{}]))), // { "ecmaVersion": 6 },
        ("const { param = 123 } = sourceObject;", None), // { "ecmaVersion": 6 },
        (
            "const { param = 123 } = sourceObject;",
            Some(serde_json::json!([{ "ignoreDefaultValues": false }])),
        ), // { "ecmaVersion": 6 },
        (
            "const [one = 1, two = 2] = []",
            Some(serde_json::json!([{ "ignoreDefaultValues": false }])),
        ), // { "ecmaVersion": 6 },
        (
            "var one, two; [one = 1, two = 2] = []",
            Some(serde_json::json!([{ "ignoreDefaultValues": false }])),
        ), // { "ecmaVersion": 6 },
        ("class C { foo = 2; }", None),                  // { "ecmaVersion": 2022 },
        ("class C { foo = 2; }", Some(serde_json::json!([{}]))), // { "ecmaVersion": 2022 },
        (
            "class C { foo = 2; }",
            Some(serde_json::json!([{ "ignoreClassFieldInitialValues": false }])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { foo = -2; }",
            Some(serde_json::json!([{ "ignoreClassFieldInitialValues": false }])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { static foo = 2; }",
            Some(serde_json::json!([{ "ignoreClassFieldInitialValues": false }])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { #foo = 2; }",
            Some(serde_json::json!([{ "ignoreClassFieldInitialValues": false }])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { static #foo = 2; }",
            Some(serde_json::json!([{ "ignoreClassFieldInitialValues": false }])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { foo = 2 + 3; }",
            Some(serde_json::json!([{ "ignoreClassFieldInitialValues": true }])),
        ), // { "ecmaVersion": 2022 },
        ("class C { 2; }", Some(serde_json::json!([{ "ignoreClassFieldInitialValues": true }]))), // { "ecmaVersion": 2022 },
        ("class C { [2]; }", Some(serde_json::json!([{ "ignoreClassFieldInitialValues": true }]))), // { "ecmaVersion": 2022 }
    ];

    Tester::new(NoMagicNumbers::NAME, pass, fail).test_and_snapshot();
}
