use oxc_ast::{
    AstKind,
    ast::{BinaryExpression, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::{BinaryOperator, UnaryOperator};
use schemars::JsonSchema;

use crate::fixer::{RuleFix, RuleFixer};
use crate::{AstNode, context::LintContext, rule::Rule};

fn eqeqeq_diagnostic(actual: &str, expected: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Expected {expected} and instead saw {actual}"))
        .with_help(format!("Prefer {expected} operator"))
        .with_label(span)
}

#[derive(Debug, Default, Clone, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct Eqeqeq {
    compare_type: CompareType,
    null_type: NullType,
}

#[derive(Debug, Default, Clone, JsonSchema)]
#[serde(rename_all = "lowercase")]
enum CompareType {
    #[default]
    Always,
    Smart,
}

impl CompareType {
    pub fn from(raw: &str) -> Self {
        match raw {
            "smart" => Self::Smart,
            _ => Self::Always,
        }
    }
}

#[derive(Debug, Default, Clone, JsonSchema)]
#[serde(rename_all = "lowercase")]
enum NullType {
    #[default]
    Always,
    Never,
    Ignore,
}

impl NullType {
    pub fn from(raw: &str) -> Self {
        match raw {
            "always" => Self::Always,
            "never" => Self::Never,
            _ => Self::Ignore,
        }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Requires the use of the `===` and `!==` operators, disallowing the use of `==` and `!=`.
    ///
    /// ### Why is this bad?
    ///
    /// Using non-strict equality operators leads to unexpected behavior due to type coercion, which can cause hard-to-find bugs.
    ///
    /// ### Options
    ///
    /// First option:
    /// - Type: `string`
    /// - Default: `"always"`
    ///
    /// Possible values:
    /// * `"always"` - always require `===`/`!==`
    /// * `"smart"` - allow safe comparisons (`typeof`, literals, nullish)
    ///
    /// Second option (only used with `"always"`):
    /// - Type: `object`
    /// - Properties:
    ///   - `null`: `string` (default: `"always"`) - `"ignore"` allows `== null` and `!= null`.
    ///
    /// Possible values for `null`:
    /// * `"always"` - always require `=== null`/`!== null`
    /// * `"never"` - always require `== null`/`!= null`
    /// * `"ignore"` - allow both `== null`/`!= null` and `=== null`/`!== null`
    ///
    /// Example JSON configuration:
    /// ```json
    /// {
    ///   "eqeqeq": ["error", "always", { "null": "ignore" }]
    /// }
    /// ```
    ///
    /// ### Examples
    ///
    /// #### `"always"` (default)
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// /* eslint eqeqeq: "error" */
    ///
    /// if (x == 42) {}
    /// if ("" == text) {}
    /// if (obj.getStuff() != undefined) {}
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// /* eslint eqeqeq: "error" */
    ///
    /// if (x === 42) {}
    /// if ("" === text) {}
    /// if (obj.getStuff() !== undefined) {}
    /// ```
    ///
    /// #### `"smart"`
    ///
    /// Examples of **incorrect** code for this rule with the `"smart"` option:
    /// ```js
    /// /* eslint eqeqeq: ["error", "smart"] */
    ///
    /// if (x == 42) {}
    /// if ("" == text) {}
    /// ```
    ///
    /// Examples of **correct** code for this rule with the `"smart"` option:
    /// ```js
    /// /* eslint eqeqeq: ["error", "smart"] */
    ///
    /// if (typeof foo == "undefined") {}
    /// if (foo == null) {}
    /// if (foo != null) {}
    /// ```
    ///
    /// #### `{"null": "ignore"}` (with `"always"` first option)
    ///
    /// Examples of **incorrect** code for this rule with the `{ "null": "ignore" }` option:
    /// ```js
    /// /* eslint eqeqeq: ["error", "always", { "null": "ignore" }] */
    /// if (x == 42) {}
    /// if ("" == text) {}
    /// ```
    ///
    /// Examples of **correct** code for this rule with the `{ "null": "ignore" }` option:
    /// ```js
    /// /* eslint eqeqeq: ["error", "always", { "null": "ignore" }] */
    /// if (foo == null) {}
    /// if (foo != null) {}
    /// ```
    ///
    /// #### `{"null": "always"}` (default - with `"always"` first option)
    ///
    /// Examples of **incorrect** code for this rule with the `{ "null": "always" }` option:
    /// ```js
    /// /* eslint eqeqeq: ["error", "always", { "null": "always" }] */
    ///
    /// if (foo == null) {}
    /// if (foo != null) {}
    /// ```
    ///
    /// Examples of **correct** code for this rule with the `{ "null": "always" }` option:
    /// ```js
    /// /* eslint eqeqeq: ["error", "always", { "null": "always" }] */
    ///
    /// if (foo === null) {}
    /// if (foo !== null) {}
    /// ```
    ///
    /// #### `{"null": "never"}` (with `"always"` first option)
    ///
    /// Examples of **incorrect** code for this rule with the `{ "null": "never" }` option:
    /// ```js
    /// /* eslint eqeqeq: ["error", "always", { "null": "never" }] */
    ///
    /// if (x == 42) {}
    /// if ("" == text) {}
    /// if (foo === null) {}
    /// if (foo !== null) {}
    /// ```
    ///
    /// Examples of **correct** code for this rule with the `{ "null": "never" }` option:
    /// ```js
    /// /* eslint eqeqeq: ["error", "always", { "null": "never" }] */
    ///
    /// if (x === 42) {}
    /// if ("" === text) {}
    /// if (foo == null) {}
    /// if (foo != null) {}
    /// ```
    ///
    Eqeqeq,
    eslint,
    pedantic,
    fix = conditional_fix_dangerous,
    config = Eqeqeq,
);

impl Eqeqeq {
    fn report_inverse_null_comparison(&self, binary_expr: &BinaryExpression, ctx: &LintContext) {
        if !matches!(self.null_type, NullType::Never) {
            return;
        }
        let operator = binary_expr.operator.as_str();
        let truncated_operator = &operator[..operator.len() - 1];
        // There are some uncontrolled cases to auto fix.
        // In ESLint, `null >= null` will be auto fixed to `null > null` which is also wrong.
        // So I just report it.
        ctx.diagnostic(eqeqeq_diagnostic(operator, truncated_operator, binary_expr.span));
    }
}

impl Rule for Eqeqeq {
    fn from_configuration(value: serde_json::Value) -> Self {
        let first_arg = value.get(0).and_then(serde_json::Value::as_str);

        let null_type = value
            .get(usize::from(first_arg.is_some()))
            .and_then(|v| v.get("null"))
            .and_then(serde_json::Value::as_str)
            .map(NullType::from)
            .unwrap_or_default();

        let compare_type = first_arg.map(CompareType::from).unwrap_or_default();

        Self { compare_type, null_type }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::BinaryExpression(binary_expr) = node.kind() else {
            return;
        };
        let is_null_comparison = is_null_check(binary_expr);
        let must_enforce_null = matches!(self.null_type, NullType::Always);

        if !matches!(binary_expr.operator, BinaryOperator::Equality | BinaryOperator::Inequality) {
            if is_null_comparison {
                self.report_inverse_null_comparison(binary_expr, ctx);
            }
            return;
        }

        let is_typeof_check = is_type_of_binary(binary_expr);
        let is_same_type_literals =
            are_literals_and_same_type(&binary_expr.left, &binary_expr.right);
        // The "smart" option enforces the use of `===` and `!==` except for these cases:
        //  - Comparing two literal values
        //  - Evaluating the value of typeof
        //  - Comparing against null
        if matches!(self.compare_type, CompareType::Smart)
            && (is_typeof_check || is_same_type_literals || is_null_comparison)
        {
            return;
        }

        if !must_enforce_null && is_null_comparison {
            return;
        }

        let operator = binary_expr.operator.as_str();
        let (preferred_operator, preferred_operator_with_padding) =
            to_strict_eq_operator_str(binary_expr.operator);

        let operator_span = get_operator_span(binary_expr, operator, ctx);

        let fix_kind = if is_typeof_check || is_same_type_literals {
            FixKind::SafeFix
        } else {
            FixKind::DangerousFix
        };

        ctx.diagnostic_with_fix_of_kind(
            eqeqeq_diagnostic(operator, preferred_operator, operator_span),
            fix_kind,
            |fixer| apply_rule_fix(&fixer, binary_expr, preferred_operator_with_padding),
        );
    }
}

#[expect(clippy::cast_possible_truncation)]
fn get_operator_span(binary_expr: &BinaryExpression, operator: &str, ctx: &LintContext) -> Span {
    let left_end = binary_expr.left.span().end;
    let right_start = binary_expr.right.span().start;
    let between_text = Span::new(left_end, right_start).source_text(ctx.source_text());
    let offset = between_text.find(operator).unwrap_or(0) as u32;

    let operator_start = left_end + offset;
    let operator_end = operator_start + operator.len() as u32;

    Span::new(operator_start, operator_end)
}

fn apply_rule_fix<'a>(
    fixer: &RuleFixer<'_, 'a>,
    binary_expr: &'a BinaryExpression,
    preferred_operator_with_padding: &'static str,
) -> RuleFix {
    let span = Span::new(binary_expr.left.span().end, binary_expr.right.span().start);

    fixer.replace(span, preferred_operator_with_padding)
}

fn to_strict_eq_operator_str(operator: BinaryOperator) -> (&'static str, &'static str) {
    match operator {
        BinaryOperator::Equality => ("===", " === "),
        BinaryOperator::Inequality => ("!==", " !== "),
        _ => unreachable!(
            "Only Equality and Inequality operators are supported in to_strict_eq_operator_str"
        ),
    }
}

fn is_type_of(expr: &Expression) -> bool {
    matches!(
        expr,
        Expression::UnaryExpression(unary_expr) if matches!(unary_expr.operator, UnaryOperator::Typeof)
    )
}

/// Checks if either operand of a binary expression is a `typeof` operation.
fn is_type_of_binary(binary_expr: &BinaryExpression) -> bool {
    is_type_of(&binary_expr.left) || is_type_of(&binary_expr.right)
}

/// Checks if operands are literals of the same type
fn are_literals_and_same_type(left: &Expression, right: &Expression) -> bool {
    matches!(
        (left, right),
        (Expression::BooleanLiteral(_), Expression::BooleanLiteral(_))
            | (Expression::NullLiteral(_), Expression::NullLiteral(_))
            | (Expression::StringLiteral(_), Expression::StringLiteral(_))
            | (Expression::NumericLiteral(_), Expression::NumericLiteral(_))
            | (Expression::BigIntLiteral(_), Expression::BigIntLiteral(_))
            | (Expression::RegExpLiteral(_), Expression::RegExpLiteral(_))
            | (Expression::TemplateLiteral(_), Expression::TemplateLiteral(_))
    )
}

fn is_null_check(binary_expr: &BinaryExpression) -> bool {
    matches!(
        (&binary_expr.left, &binary_expr.right),
        (_, Expression::NullLiteral(_)) | (Expression::NullLiteral(_), _)
    )
}

#[test]
fn test() {
    use serde_json::json;

    use crate::tester::Tester;

    let pass = vec![
        ("typeof foo == 'undefined'", Some(json!(["smart"]))),
        ("'hello' != 'world'", Some(json!(["smart"]))),
        ("0 == 0", Some(json!(["smart"]))),
        ("true == true", Some(json!(["smart"]))),
        ("foo == null", Some(json!(["smart"]))),
        ("foo === null", None),
        // Always use === or !== with `null`
        ("null === null", Some(json!(["always", {"null": "always"}]))),
        // Never use === or !== with `null`
        ("null == null", Some(json!(["always", {"null": "never"}]))),
        // Do not apply this rule to `null`.
        ("null == null", Some(json!(["smart", {"null": "ignore"}]))),
        // Issue: <https://github.com/oxc-project/oxc/issues/8773>
        ("href != null", Some(json!([{"null": "ignore"}]))),
    ];

    let fail = vec![
        // ESLint will perform like below case
        ("null >= 1", Some(json!(["always", {"null": "never"}]))),
        ("typeof foo == 'undefined'", None),
        ("'hello' != 'world'", None),
        ("0 == 0", None),
        ("true == true", None),
        ("foo == null", None),
        ("a == b", None),
        ("foo == true", None),
        ("bananas != 1", None),
        ("value == undefined", None),
        ("null == null", Some(json!(["always", {"null": "always"}]))),
    ];

    let fix = vec![
        ("null==null", "null === null", None),
        ("'foo'=='foo'", "'foo' === 'foo'", None),
        ("typeof a == b", "typeof a === b", None),
        ("1000  !=  1000", "1000 !== 1000", None),
        ("(1000 + 1) != 1000", "(1000 + 1) !== 1000", None),
        ("a == b", "a === b", None),
    ];

    Tester::new(Eqeqeq::NAME, Eqeqeq::PLUGIN, pass, fail).expect_fix(fix).test_and_snapshot();
}
