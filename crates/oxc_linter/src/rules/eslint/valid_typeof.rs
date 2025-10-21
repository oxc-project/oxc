use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::UnaryOperator;
use schemars::JsonSchema;

use crate::{AstNode, context::LintContext, rule::Rule};

fn not_string(help: Option<&'static str>, span: Span) -> OxcDiagnostic {
    let mut d =
        OxcDiagnostic::warn("Typeof comparisons should be to string literals.").with_label(span);
    if let Some(x) = help {
        d = d.with_help(x);
    }
    d
}

fn invalid_value(help: Option<&'static str>, span: Span) -> OxcDiagnostic {
    let mut d = OxcDiagnostic::warn("Invalid typeof comparison value.").with_label(span);
    if let Some(x) = help {
        d = d.with_help(x);
    }
    d
}

#[derive(Debug, Clone, Default, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct ValidTypeof {
    /// The `requireStringLiterals` option when set to `true`, allows the comparison of `typeof`
    /// expressions with only string literals or other `typeof` expressions, and disallows
    /// comparisons to any other value. Default is `false`.
    ///
    /// With `requireStringLiterals` set to `true`, the following are examples of **incorrect** code:
    /// ```js
    /// typeof foo === undefined
    /// typeof bar == Object
    /// typeof baz === "strnig"
    /// typeof qux === "some invalid type"
    /// typeof baz === anotherVariable
    /// typeof foo == 5
    /// ```
    ///
    /// With `requireStringLiterals` set to `true`, the following are examples of **correct** code:
    /// ```js
    /// typeof foo === "undefined"
    /// typeof bar == "object"
    /// typeof baz === "string"
    /// typeof bar === typeof qux
    /// ```
    require_string_literals: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce comparing `typeof` expressions against valid strings.
    ///
    /// ### Why is this bad?
    ///
    /// For a vast majority of use cases, the result of the `typeof` operator is one of the
    /// following string literals: `"undefined"`, `"object"`, `"boolean"`, `"number"`, `"string"`,
    /// `"function"`, `"symbol"`, and `"bigint"`. It is usually a typing mistake to compare the
    /// result of a `typeof` operator to other string literals.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// typeof foo === "strnig"
    /// typeof foo == "undefimed"
    /// typeof bar != "nunber"     // spellchecker:disable-line
    /// typeof bar !== "fucntion"     // spellchecker:disable-line
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// typeof foo === "string"
    /// typeof bar == "undefined"
    /// typeof foo === baz
    /// typeof bar === typeof qux
    /// ```
    ValidTypeof,
    eslint,
    correctness,
    conditional_fix,
    config = ValidTypeof,
);

impl Rule for ValidTypeof {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        // match on `typeof` unary expression for better performance
        let _unary_expr = match node.kind() {
            AstKind::UnaryExpression(unary_expr)
                if unary_expr.operator == UnaryOperator::Typeof =>
            {
                unary_expr
            }
            _ => return,
        };

        let binary_expr = match ctx.nodes().parent_kind(node.id()) {
            AstKind::BinaryExpression(binary_expr) if binary_expr.operator.is_equality() => {
                binary_expr
            }
            _ => return,
        };

        let ((Expression::UnaryExpression(_), sibling) | (sibling, Expression::UnaryExpression(_))) =
            (&binary_expr.left, &binary_expr.right)
        else {
            return;
        };

        if let Expression::StringLiteral(lit) = sibling {
            if !VALID_TYPES.contains(&lit.value.as_str()) {
                ctx.diagnostic(invalid_value(None, sibling.span()));
            }
            return;
        }

        if let Expression::TemplateLiteral(template) = sibling
            && let Some(quasi) = template.single_quasi()
        {
            if !VALID_TYPES.contains(&quasi.as_str()) {
                ctx.diagnostic(invalid_value(None, sibling.span()));
            }
            return;
        }

        if let Expression::Identifier(ident) = sibling
            && ident.name == "undefined"
            && ctx.scoping().root_unresolved_references().contains_key(ident.name.as_str())
        {
            ctx.diagnostic_with_fix(
                if self.require_string_literals {
                    not_string(Some("Use `\"undefined\"` instead of `undefined`."), sibling.span())
                } else {
                    invalid_value(
                        Some("Use `\"undefined\"` instead of `undefined`."),
                        sibling.span(),
                    )
                },
                |fixer| fixer.replace(sibling.span(), "\"undefined\""),
            );
            return;
        }

        if self.require_string_literals
            && !matches!(sibling, Expression::UnaryExpression(unary) if unary.operator == UnaryOperator::Typeof)
        {
            ctx.diagnostic(not_string(None, sibling.span()));
        }
    }

    fn from_configuration(value: serde_json::Value) -> Self {
        let require_string_literals = value.get(0).is_some_and(|config| {
            config
                .get("requireStringLiterals")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false)
        });

        Self { require_string_literals }
    }
}

const VALID_TYPES: [&str; 8] =
    ["bigint", "boolean", "function", "number", "object", "string", "symbol", "undefined"];

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("typeof foo === 'string'", None),
        ("typeof foo === 'object'", None),
        ("typeof foo === 'function'", None),
        ("typeof foo === 'undefined'", None),
        ("typeof foo === 'boolean'", None),
        ("typeof foo === 'number'", None),
        ("typeof foo === 'bigint'", None),
        ("'string' === typeof foo", None),
        ("'object' === typeof foo", None),
        ("'function' === typeof foo", None),
        ("'undefined' === typeof foo", None),
        ("'boolean' === typeof foo", None),
        ("'number' === typeof foo", None),
        ("typeof foo === typeof bar", None),
        ("typeof foo === baz", None),
        ("typeof foo !== someType", None),
        ("typeof bar != someType", None),
        ("someType === typeof bar", None),
        ("someType == typeof bar", None),
        ("typeof foo == 'string'", None),
        ("typeof(foo) === 'string'", None),
        ("typeof(foo) !== 'string'", None),
        ("typeof(foo) == 'string'", None),
        ("typeof(foo) != 'string'", None),
        ("var oddUse = typeof foo + 'thing'", None),
        ("function f(undefined) { typeof x === undefined }", None),
        ("typeof foo === `str${somethingElse}`", None),
        ("typeof foo === 'number'", Some(serde_json::json!([{ "requireStringLiterals": true }]))),
        ("typeof foo === \"number\"", Some(serde_json::json!([{ "requireStringLiterals": true }]))),
        (
            "var baz = typeof foo + 'thing'",
            Some(serde_json::json!([{ "requireStringLiterals": true }])),
        ),
        ("typeof foo === typeof bar", Some(serde_json::json!([{ "requireStringLiterals": true }]))),
        ("typeof foo === `string`", Some(serde_json::json!([{ "requireStringLiterals": true }]))),
        ("`object` === typeof foo", Some(serde_json::json!([{ "requireStringLiterals": true }]))),
    ];

    let fail = vec![
        ("typeof foo === 'strnig'", None),
        ("'strnig' === typeof foo", None),
        ("if (typeof bar === 'umdefined') {}", None),
        ("typeof foo !== 'strnig'", None),
        ("'strnig' !== typeof foo", None),
        ("if (typeof bar !== 'umdefined') {}", None),
        ("typeof foo != 'strnig'", None),
        ("'strnig' != typeof foo", None),
        ("if (typeof bar != 'umdefined') {}", None),
        ("typeof foo == 'strnig'", None),
        ("'strnig' == typeof foo", None),
        ("if (typeof bar == 'umdefined') {}", None),
        ("if (typeof bar === `umdefined`) {}", None),
        (
            "typeof foo == 'invalid string'",
            Some(serde_json::json!([{ "requireStringLiterals": true }])),
        ),
        ("if (typeof bar !== undefined) {}", None),
        ("typeof foo == Object", Some(serde_json::json!([{ "requireStringLiterals": true }]))),
        ("typeof foo === undefined", Some(serde_json::json!([{ "requireStringLiterals": true }]))),
        ("undefined === typeof foo", Some(serde_json::json!([{ "requireStringLiterals": true }]))),
        ("undefined == typeof foo", Some(serde_json::json!([{ "requireStringLiterals": true }]))),
        (
            "typeof foo === `undefined${foo}`",
            Some(serde_json::json!([{ "requireStringLiterals": true }])),
        ),
        (
            "typeof foo === `${string}`",
            Some(serde_json::json!([{ "requireStringLiterals": true }])),
        ),
    ];

    let fix = vec![("typeof foo === undefined", r#"typeof foo === "undefined""#)];

    Tester::new(ValidTypeof::NAME, ValidTypeof::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
