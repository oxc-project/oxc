use std::ops::Mul;

use oxc_ast::{
    ast::{
        Argument, AssignmentTarget, Expression, MemberExpression, UnaryExpression,
        VariableDeclarationKind,
    },
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::operator::UnaryOperator;

use crate::{context::LintContext, rule::Rule, AstNode};

enum NoMagicNumberReportReason<'a> {
    MustUseConst(&'a Span),
    NoMagicNumber(&'a Span, &'a str),
}

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

impl InternConfig<'_> {
    fn try_node<'a>(
        node: &'a AstNode<'a>,
        parent_node: &'a AstNode<'a>,
    ) -> Result<InternConfig<'a>, OxcDiagnostic> {
        let is_negative = matches!(parent_node.kind(), AstKind::UnaryExpression(unary) if unary.operator == UnaryOperator::UnaryNegation);

        if let AstKind::NumericLiteral(numeric) = node.kind() {
            if is_negative {
                return Ok(InternConfig {
                    node: parent_node,
                    value: 0.0 - numeric.value,
                    raw: format!("-{}", numeric.raw),
                });
            } else {
                return Ok(InternConfig { node, value: numeric.value, raw: numeric.raw.into() });
            }
        } else {
            return Err(OxcDiagnostic::warn(format!(
                "expected AstKind BingIntLiteral or NumericLiteral, got {:?}",
                node.kind().debug_name()
            )));
        }
    }

    pub fn from<'a>(node: &'a AstNode<'a>, parent: &'a AstNode<'a>) -> InternConfig<'a> {
        return InternConfig::try_node(node, parent).unwrap();
    }
}
impl Rule for NoMagicNumbers {
    fn from_configuration(value: serde_json::Value) -> Self {
        return Self(Box::new(NoMagicNumbersConfig::try_from(&value).unwrap()));
    }
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::NumericLiteral(literal) => {
                let config = InternConfig::from(node, ctx.nodes().parent_node(node.id()).unwrap());

                let parent = ctx.nodes().parent_node(config.node.id()).unwrap();
                let parent_parent = ctx.nodes().parent_node(parent.id()).unwrap();

                if self.is_skipable(&config, parent, parent_parent) {
                    return;
                }

                if let Some(reason) = self.get_report_reason(&parent, &config, &literal.span) {
                    ctx.diagnostic(match reason {
                        NoMagicNumberReportReason::MustUseConst(span) => {
                            must_use_const_diagnostic(span)
                        }
                        NoMagicNumberReportReason::NoMagicNumber(span, raw) => {
                            no_magic_number_diagnostic(span, raw)
                        }
                    });
                }
            }
            _ => {}
        }
    }
}

impl NoMagicNumbers {
    fn is_numeric_value<'a>(expression: &Expression<'a>, number: &f64) -> bool {
        // check for Expression::NumericLiteral
        if expression.is_number(*number) {
            return true;
        }

        if let Expression::UnaryExpression(unary) = expression {
            if let Expression::NumericLiteral(_) = &unary.argument {
                if unary.operator == UnaryOperator::UnaryPlus {
                    return unary.argument.is_number(*number);
                }

                if unary.operator == UnaryOperator::UnaryNegation {
                    return unary.argument.is_number(number * -1.0);
                }
            }
        }

        false
    }
    fn is_ignore_value(&self, number: &f64) -> bool {
        self.ignore.contains(number)
    }

    fn is_default_value<'a>(&self, number: &f64, parent_node: &AstNode<'a>) -> bool {
        if let AstKind::AssignmentTargetWithDefault(assignment) = parent_node.kind() {
            return NoMagicNumbers::is_numeric_value(&assignment.init, number);
        }

        if let AstKind::AssignmentPattern(pattern) = parent_node.kind() {
            return NoMagicNumbers::is_numeric_value(&pattern.right, number);
        }

        false
    }

    fn is_class_field_initial_value<'a>(&self, number: &f64, parent_node: &AstNode<'a>) -> bool {
        if let AstKind::PropertyDefinition(property) = parent_node.kind() {
            return NoMagicNumbers::is_numeric_value(property.value.as_ref().unwrap(), number);
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

    ///  Returns whether the given node is used as an array index.
    ///  Value must coerce to a valid array index name: "0", "1", "2" ... "4294967294".
    ///
    ///  All other values, like "-1", "2.5", or "4294967295", are just "normal" object properties,
    ///  which can be created and accessed on an array in addition to the array index properties,
    ///  but they don't affect array's length and are not considered by methods such as .map(), .forEach() etc.
    ///
    ///  The maximum array length by the specification is 2 ** 32 - 1 = 4294967295,
    ///  thus the maximum valid index is 2 ** 32 - 2 = 4294967294.
    ///
    ///  All notations are allowed, as long as the value coerces to one of "0", "1", "2" ... "4294967294".
    ///
    ///  Valid examples:
    ///  a[0], a[1], a[1.2e1], a[0xAB], a[0n], a[1n]
    ///  a[-0] (same as a[0] because -0 coerces to "0")
    ///  a[-0n] (-0n evaluates to 0n)
    ///
    ///  Invalid examples:
    ///  a[-1], a[-0xAB], a[-1n], a[2.5], a[1.23e1], a[12e-1]
    ///  a[4294967295] (above the max index, it's an access to a regular property a["4294967295"])
    ///  a[999999999999999999999] (even if it wasn't above the max index, it would be a["1e+21"])
    ///  a[1e310] (same as a["Infinity"])
    fn is_array_index<'a>(&self, node: &AstNode<'a>, parent_node: &AstNode<'a>) -> bool {
        println!(
            "
        
        node: {node:?}

        "
        );

        fn is_unanary_index(unary: &UnaryExpression) -> bool {
            match &unary.argument {
                Expression::NumericLiteral(numeric) => {
                    if unary.operator == UnaryOperator::UnaryNegation {
                        return numeric.value == 0.0;
                    }
                    // ToDo: check why ("foo[+0]", ignore_array_indexes.clone()), should fail
                    return false;

                    // return numeric.value >= 0.0
                    //     && numeric.value.fract() == 0.0
                    //     && numeric.value < u32::MAX as f64;
                }
                _ => false,
            }
        }
        match node.kind() {
            AstKind::UnaryExpression(unary) => {
                return is_unanary_index(&unary);
            }
            AstKind::NumericLiteral(numeric) => {
                match parent_node.kind() {
                    AstKind::MemberExpression(expression) => {
                        if let MemberExpression::ComputedMemberExpression(computed_expression) =
                            expression
                        {
                            return computed_expression.expression.is_number(numeric.value)
                                && numeric.value >= 0.0
                                && numeric.value.fract() == 0.0
                                && numeric.value < u32::MAX as f64;
                        }

                        false
                    }
                    AstKind::UnaryExpression(unary) => {
                        return is_unanary_index(&unary);
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    fn is_skipable<'a>(
        &self,
        config: &InternConfig<'a>,
        parent: &AstNode<'a>,
        parent_parent: &AstNode<'a>,
    ) -> bool {
        if self.is_ignore_value(&config.value)
            || self.is_parse_int_radix(&config.value, &parent_parent)
            || (self.ignore_default_values && self.is_default_value(&config.value, &parent))
            || (self.ignore_class_field_initial_values
                && self.is_class_field_initial_value(&config.value, &parent))
            || (self.ignore_array_indexes && self.is_array_index(&config.node, &parent))
        {
            return true;
        }

        if !self.detect_objects
            && (matches!(parent.kind(), AstKind::ObjectExpression(_))
                || matches!(parent.kind(), AstKind::ObjectProperty(_)))
        {
            return true;
        }

        false
    }

    fn get_report_reason<'a>(
        &self,
        parent: &'a AstNode<'a>,
        config: &'a InternConfig<'a>,
        span: &'a Span,
    ) -> Option<NoMagicNumberReportReason<'a>> {
        match parent.kind() {
            AstKind::VariableDeclarator(declarator) => {
                if self.enforce_const && declarator.kind != VariableDeclarationKind::Const {
                    return Some(NoMagicNumberReportReason::MustUseConst(span));
                }

                None
            }
            AstKind::AssignmentExpression(expression) => {
                if let AssignmentTarget::AssignmentTargetIdentifier(_) = expression.left {
                    return Some(NoMagicNumberReportReason::NoMagicNumber(span, &config.raw));
                }

                None
            }
            AstKind::JSXExpressionContainer(_) => None,
            _ => return Some(NoMagicNumberReportReason::NoMagicNumber(span, &config.raw)),
        }
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
        // ("foo[0n]", ignore_array_indexes.clone()), // { "ecmaVersion": 2020 },
        // ("foo[-0n]", ignore_array_indexes.clone()), // { "ecmaVersion": 2020 },
        // ("foo[1n]", ignore_array_indexes.clone()), // { "ecmaVersion": 2020 },
        // ("foo[100n]", ignore_array_indexes.clone()), // { "ecmaVersion": 2020 },
        // ("foo[0xABn]", ignore_array_indexes.clone()), // { "ecmaVersion": 2020 },
        // ("foo[4294967294n]", ignore_array_indexes.clone()), // { "ecmaVersion": 2020 },
        ("var a = <input maxLength={10} />;", None), // {                "parserOptions": {                    "ecmaFeatures": {                        "jsx": true                    }                }            },
        ("var a = <div objectProp={{ test: 1}}></div>;", None), // {                "parserOptions": {                    "ecmaFeatures": {                        "jsx": true                    }                }            },
        // ("f(100n)", Some(serde_json::json!([{ "ignore": ["100n"] }]))), // { "ecmaVersion": 2020 },
        // ("f(-100n)", Some(serde_json::json!([{ "ignore": ["-100n"] }]))), // { "ecmaVersion": 2020 },
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
        (
            "var data = ['foo', 'bar', 'baz']; var third = data[3];",
            Some(serde_json::json!([{"ignoreArrayIndexes": false}])),
        ),
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
        // ("foo[-1n]", ignore_array_indexes.clone()), // { "ecmaVersion": 2020 },
        // ("foo[-100n]", ignore_array_indexes.clone()), // { "ecmaVersion": 2020 },
        // ("foo[-0x12n]", ignore_array_indexes.clone()), // { "ecmaVersion": 2020 },
        // ("foo[4294967295n]", ignore_array_indexes.clone()), // { "ecmaVersion": 2020 },
        ("foo[+0]", ignore_array_indexes.clone()),
        ("foo[+1]", ignore_array_indexes.clone()),
        ("foo[-(-1)]", ignore_array_indexes.clone()),
        // ("foo[+0n]", ignore_array_indexes.clone()), // { "ecmaVersion": 2020 },
        // ("foo[+1n]", ignore_array_indexes.clone()), // { "ecmaVersion": 2020 },
        // ("foo[- -1n]", ignore_array_indexes.clone()), // { "ecmaVersion": 2020 },
        ("100 .toString()", ignore_array_indexes.clone()),
        ("200[100]", ignore_array_indexes.clone()),
        // ("var a = <div arrayProp={[1,2,3]}></div>;", None), // {                "parserOptions": {                    "ecmaFeatures": {                        "jsx": true                    }                }            },
        ("var min, max, mean; min = 1; max = 10; mean = 4;", Some(serde_json::json!([{}]))),
        // ("f(100n)", Some(serde_json::json!([{ "ignore": [100] }]))), // { "ecmaVersion": 2020 },
        // ("f(-100n)", Some(serde_json::json!([{ "ignore": ["100n"] }]))), // { "ecmaVersion": 2020 },
        // ("f(100n)", Some(serde_json::json!([{ "ignore": ["-100n"] }]))), // { "ecmaVersion": 2020 },
        // ("f(100)", Some(serde_json::json!([{ "ignore": ["100n"] }]))),
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
