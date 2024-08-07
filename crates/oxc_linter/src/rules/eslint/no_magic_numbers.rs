use std::ops::Mul;

use oxc_ast::{
    ast::{Argument, AssignmentTarget, Expression, VariableDeclarationKind},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::operator::UnaryOperator;

use crate::{context::LintContext, rule::Rule, AstNode};

fn must_use_const_diagnostic(span: &Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Number constants declarations must use 'const'.").with_label(*span)
}

fn no_magic_number_diagnostic(span: &Span, raw: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("No magic number: {raw}")).with_label(*span)
}

#[derive(Debug, Default, Clone)]

pub struct NoMagicNumbers(Box<NoMagicNumbersConfig>);

impl std::ops::Deref for NoMagicNumbers {
    type Target = NoMagicNumbersConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct NoMagicNumbersConfig {
    ignore: Vec<f64>,
    ignore_array_indexes: bool,
    ignore_default_values: bool,
    ignore_class_field_initial_values: bool,
    enforce_const: bool,
    detect_objects: bool,
}

impl Default for NoMagicNumbersConfig {
    fn default() -> Self {
        Self {
            ignore: vec![],
            ignore_array_indexes: false,
            ignore_default_values: false,
            ignore_class_field_initial_values: false,
            enforce_const: false,
            detect_objects: false,
        }
    }
}

impl TryFrom<&serde_json::Value> for NoMagicNumbersConfig {
    type Error = OxcDiagnostic;

    fn try_from(raw: &serde_json::Value) -> Result<Self, Self::Error> {
        println!("raw: {raw:?}");

        if raw.is_null() {
            return Ok(NoMagicNumbersConfig::default());
        }

        raw.as_array().unwrap().get(0).map_or_else(
            || {
                Err(OxcDiagnostic::warn(
                    "Expecting object for eslint/no-magic-numbers configuration",
                ))
            },
            |object| {
                println!("config: {object:?}");
                Ok(Self {
                    ignore: object
                        .get("ignore")
                        .and_then(serde_json::Value::as_array)
                        .map(|v| {
                            v.iter()
                                .filter_map(serde_json::Value::as_f64)
                                .map(|v| f64::try_from(v).unwrap())
                                .collect()
                        })
                        .unwrap_or_default(),
                    ignore_array_indexes: object
                        .get("ignoreArrayIndexes")
                        .unwrap_or_else(|| &serde_json::Value::Bool(false))
                        .as_bool()
                        .unwrap(),
                    ignore_default_values: object
                        .get("ignoreDefaultValues")
                        .unwrap_or_else(|| &serde_json::Value::Bool(false))
                        .as_bool()
                        .unwrap(),
                    ignore_class_field_initial_values: object
                        .get("ignoreClassFieldInitialValues")
                        .unwrap_or_else(|| &serde_json::Value::Bool(false))
                        .as_bool()
                        .unwrap(),
                    enforce_const: object
                        .get("enforceConst")
                        .unwrap_or_else(|| &serde_json::Value::Bool(false))
                        .as_bool()
                        .unwrap(),
                    detect_objects: object
                        .get("detectObjects")
                        .unwrap_or_else(|| &serde_json::Value::Bool(false))
                        .as_bool()
                        .unwrap(),
                })
            },
        )
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

#[derive(Debug)]
struct InternConfig<'a> {
    node: &'a AstNode<'a>,
    value: f64,
    raw: String,
}

impl Rule for NoMagicNumbers {
    fn from_configuration(value: serde_json::Value) -> Self {
        return Self(Box::new(NoMagicNumbersConfig::try_from(&value).unwrap()));
    }
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {

        let ok_ast_kinds: Vec<AstKind> = if self.detect_objects {
            vec![]
        } else {
            vec![
            AstKind::ObjectExpression,
            AstKind::PropertyDefinition,
            AstKind::AssignmentExpression
            ]
        };

        match node.kind() {
            AstKind::NumericLiteral(literal) => {
                let parent_node = ctx.nodes().parent_node(node.id());

                let config: InternConfig = match parent_node.unwrap().kind() {
                    AstKind::UnaryExpression(unary)
                        if unary.operator == UnaryOperator::UnaryNegation =>
                    {
                        InternConfig {
                            node: parent_node.unwrap(),
                            value: 0.0 - literal.value,
                            raw: format!("-{}", literal.raw),
                        }
                    }
                    _ => InternConfig { node, value: literal.value, raw: literal.raw.to_string() },
                };

                let parent = ctx.nodes().parent_node(config.node.id()).unwrap();
                let parent_parent = ctx.nodes().parent_node(parent.id()).unwrap();

                println!(
                    "
                
                config {config:?}
                parent: {parent:?}
                parent_parent: {parent_parent:?}

                "
                );

                if self.is_ignore_value(&config.value)
                    || (self.ignore_default_values && self.is_default_value(&config.value, &parent))
                    || (self.ignore_class_field_initial_values
                        && self.is_class_field_initial_value(&config.value, &parent))
                    || self.is_parse_int_radix(&config.value, &parent_parent)
                {
                    return;
                }

                match parent.kind() {
                    AstKind::VariableDeclarator(declarator)
                        if self.enforce_const
                            && declarator.kind != VariableDeclarationKind::Const =>
                    {
                        println!("declarator: {declarator:?}");
                        ctx.diagnostic(must_use_const_diagnostic(&literal.span));
                    }
                    AstKind::BinaryExpression(expression) => {
                        if expression.left.is_number(literal.value) {
                            ctx.diagnostic(no_magic_number_diagnostic(&literal.span, &config.raw))
                        } else if expression.right.is_number(literal.value) {
                            ctx.diagnostic(no_magic_number_diagnostic(&literal.span, &config.raw))
                        }
                    }
                    AstKind::AssignmentExpression(expression) if self.detect_objects => {
                        if let AssignmentTarget::AssignmentTargetIdentifier(_) = expression.left {
                            ctx.diagnostic(no_magic_number_diagnostic(&literal.span, &config.raw))
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

impl NoMagicNumbers {
    fn is_ignore_value(&self, number: &f64) -> bool {
        self.ignore.contains(number)
    }

    fn is_default_value<'a>(&self, number: &f64, parent_node: &AstNode<'a>) -> bool {
        if let AstKind::AssignmentPattern(pattern) = parent_node.kind() {
            return pattern.right.is_number(*number);
        }

        false
    }

    fn is_class_field_initial_value<'a>(&self, number: &f64, parent_node: &AstNode<'a>) -> bool {
        if let AstKind::PropertyDefinition(property) = parent_node.kind() {
            return property.value.as_ref().unwrap().is_number(*number);
        }

        false
    }

    fn is_parse_int_radix<'a>(&self, number: &f64, parent_parent_node: &AstNode<'a>) -> bool {
        if let AstKind::CallExpression(expression) = parent_parent_node.kind() {
            if expression.arguments.get(1).is_none() {
                return false;
            }

            let argument = expression.arguments.get(1).unwrap();
            return match argument {
                Argument::NumericLiteral(numeric) => numeric.value == *number,
                Argument::UnaryExpression(unary)
                    if unary.operator == UnaryOperator::UnaryNegation =>
                {
                    if let Expression::NumericLiteral(numeric) = &unary.argument {
                        return numeric.value == number.mul(-1.0);
                    }

                    return false;
                }
                _ => false,
            };
        }

        false
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let ignore_array_indexes = Some(serde_json::json!([{"ignoreArrayIndexes": true}]));
    let ignore_default_values = Some(serde_json::json!([{"ignoreDefaultValues": true}]));
    let enforce_const = Some(serde_json::json!([{ "enforceConst": true}]));
    let ignore_class_field_initial_values =
        Some(serde_json::json!([{ "ignoreClassFieldInitialValues": true }]));

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
        ("var data = ['foo', 'bar', 'baz']; var third = data[3];", ignore_array_indexes.clone()),
        ("foo[0]", ignore_array_indexes.clone()),
        ("foo[-0]", ignore_array_indexes.clone()),
        ("foo[1]", ignore_array_indexes.clone()),
        ("foo[100]", ignore_array_indexes.clone()),
        ("foo[200.00]", ignore_array_indexes.clone()),
        ("foo[3e4]", ignore_array_indexes.clone()),
        ("foo[1.23e2]", ignore_array_indexes.clone()),
        ("foo[230e-1]", ignore_array_indexes.clone()),
        ("foo[0b110]", ignore_array_indexes.clone()), // { "ecmaVersion": 2015 },
        ("foo[0o71]", ignore_array_indexes.clone()),  // { "ecmaVersion": 2015 },
        ("foo[0xABC]", ignore_array_indexes.clone()),
        ("foo[0123]", ignore_array_indexes.clone()), // {                "sourceType": "script"            },
        ("foo[5.0000000000000001]", ignore_array_indexes.clone()),
        ("foo[4294967294]", ignore_array_indexes.clone()),
        ("foo[0n]", ignore_array_indexes.clone()), // { "ecmaVersion": 2020 },
        ("foo[-0n]", ignore_array_indexes.clone()), // { "ecmaVersion": 2020 },
        ("foo[1n]", ignore_array_indexes.clone()), // { "ecmaVersion": 2020 },
        ("foo[100n]", ignore_array_indexes.clone()), // { "ecmaVersion": 2020 },
        ("foo[0xABn]", ignore_array_indexes.clone()), // { "ecmaVersion": 2020 },
        ("foo[4294967294n]", ignore_array_indexes.clone()), // { "ecmaVersion": 2020 },
        ("var a = <input maxLength={10} />;", None), // {                "parserOptions": {                    "ecmaFeatures": {                        "jsx": true                    }                }            },
        ("var a = <div objectProp={{ test: 1}}></div>;", None), // {                "parserOptions": {                    "ecmaFeatures": {                        "jsx": true                    }                }            },
        ("f(100n)", Some(serde_json::json!([{ "ignore": ["100n"] }]))), // { "ecmaVersion": 2020 },
        ("f(-100n)", Some(serde_json::json!([{ "ignore": ["-100n"] }]))), // { "ecmaVersion": 2020 },
        ("const { param = 123 } = sourceObject;", ignore_default_values.clone()), // { "ecmaVersion": 6 },
        ("const func = (param = 123) => {}", ignore_default_values.clone()), // { "ecmaVersion": 6 },
        ("const func = ({ param = 123 }) => {}", ignore_default_values.clone()), // { "ecmaVersion": 6 },
        ("const [one = 1, two = 2] = []", ignore_default_values.clone()), // { "ecmaVersion": 6 },
        ("var one, two; [one = 1, two = 2] = []", ignore_default_values.clone()), // { "ecmaVersion": 6 },
        ("var x = parseInt?.(y, 10);", None), // { "ecmaVersion": 2020 },
        ("var x = Number?.parseInt(y, 10);", None), // { "ecmaVersion": 2020 },
        ("var x = (Number?.parseInt)(y, 10);", None), // { "ecmaVersion": 2020 },
        ("foo?.[777]", ignore_array_indexes.clone()), // { "ecmaVersion": 2020 },
        ("class C { foo = 2; }", ignore_class_field_initial_values.clone()), // { "ecmaVersion": 2022 },
        ("class C { foo = -2; }", ignore_class_field_initial_values.clone()), // { "ecmaVersion": 2022 },
        ("class C { static foo = 2; }", ignore_class_field_initial_values.clone()), // { "ecmaVersion": 2022 },
        ("class C { #foo = 2; }", ignore_class_field_initial_values.clone()), // { "ecmaVersion": 2022 },
        ("class C { static #foo = 2; }", ignore_class_field_initial_values.clone()), // { "ecmaVersion": 2022 }
    ];

    let fail = vec![
        ("var foo = 42", enforce_const.clone()), // { "ecmaVersion": 6 },
        ("var foo = 0 + 1;", None),
        // ("var foo = 42n", enforce_const.clone()), // {                "ecmaVersion": 2020            },
        // ("var foo = 0n + 1n;", None), // {                "ecmaVersion": 2020            },
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
        ("var data = ['foo', 'bar', 'baz']; var third = data[3];", ignore_array_indexes.clone()),
        ("foo[-100]", ignore_array_indexes.clone()),
        ("foo[-1.5]", ignore_array_indexes.clone()),
        ("foo[-1]", ignore_array_indexes.clone()),
        ("foo[-0.1]", ignore_array_indexes.clone()),
        ("foo[-0b110]", ignore_array_indexes.clone()), // { "ecmaVersion": 2015 },
        ("foo[-0o71]", ignore_array_indexes.clone()),  // { "ecmaVersion": 2015 },
        ("foo[-0x12]", ignore_array_indexes.clone()),
        ("foo[-012]", ignore_array_indexes.clone()), // { "sourceType": "script" },
        ("foo[0.1]", ignore_array_indexes.clone()),
        ("foo[0.12e1]", ignore_array_indexes.clone()),
        ("foo[1.5]", ignore_array_indexes.clone()),
        ("foo[1.678e2]", ignore_array_indexes.clone()),
        ("foo[56e-1]", ignore_array_indexes.clone()),
        ("foo[5.000000000000001]", ignore_array_indexes.clone()),
        ("foo[100.9]", ignore_array_indexes.clone()),
        ("foo[4294967295]", ignore_array_indexes.clone()),
        ("foo[1e300]", ignore_array_indexes.clone()),
        ("foo[1e310]", ignore_array_indexes.clone()),
        ("foo[-1e310]", ignore_array_indexes.clone()),
        ("foo[-1n]", ignore_array_indexes.clone()), // { "ecmaVersion": 2020 },
        ("foo[-100n]", ignore_array_indexes.clone()), // { "ecmaVersion": 2020 },
        ("foo[-0x12n]", ignore_array_indexes.clone()), // { "ecmaVersion": 2020 },
        ("foo[4294967295n]", ignore_array_indexes.clone()), // { "ecmaVersion": 2020 },
        ("foo[+0]", ignore_array_indexes.clone()),
        ("foo[+1]", ignore_array_indexes.clone()),
        ("foo[-(-1)]", ignore_array_indexes.clone()),
        ("foo[+0n]", ignore_array_indexes.clone()), // { "ecmaVersion": 2020 },
        ("foo[+1n]", ignore_array_indexes.clone()), // { "ecmaVersion": 2020 },
        ("foo[- -1n]", ignore_array_indexes.clone()), // { "ecmaVersion": 2020 },
        ("100 .toString()", ignore_array_indexes.clone()),
        ("200[100]", ignore_array_indexes.clone()),
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
        ("class C { foo = 2 + 3; }", ignore_class_field_initial_values.clone()), // { "ecmaVersion": 2022 },
        ("class C { 2; }", ignore_class_field_initial_values.clone()), // { "ecmaVersion": 2022 },
        ("class C { [2]; }", ignore_class_field_initial_values.clone()), // { "ecmaVersion": 2022 }
    ];

    Tester::new(NoMagicNumbers::NAME, pass, fail).test_and_snapshot();
}
