use oxc_ast::{ast::Expression, AstKind};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::UnaryOperator;
use phf::{phf_set, Set};

use crate::{context::LintContext, fixer::Fix, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
enum ValidTypeofDiagnostic {
    #[error("eslint(valid-typeof): Typeof comparisons should be to string literals.")]
    #[diagnostic(severity(warning))]
    NotString(#[help] Option<&'static str>, #[label] Span),
    #[error("eslint(valid-typeof): Invalid typeof comparison value.")]
    #[diagnostic(severity(warning))]
    InvalidValue(#[help] Option<&'static str>, #[label] Span),
}

#[derive(Debug, Clone, Default)]
pub struct ValidTypeof {
    /// true requires typeof expressions to only be compared to string literals or other typeof expressions, and disallows comparisons to any other value.
    require_string_literals: bool,
}
declare_oxc_lint!(
    /// ### What it does
    /// Enforce comparing `typeof` expressions against valid strings
    ///
    /// ### Why is this bad?
    /// It is usually a typing mistake to compare the result of a typeof operator to other string literals.
    /// ### Example
    /// ```javascript
    /// requireStringLiterals: false
    /// incorrect:
    /// typeof foo === "strnig"
    /// correct:
    /// typeof foo === "string"
    /// typeof foo === baz
    ///
    /// requireStringLiterals: true
    /// incorrect:
    /// typeof foo === baz
    /// ```
    ValidTypeof,
    nursery,
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
            Some(AstKind::BinaryExpression(binary_expr)) if binary_expr.operator.is_equality() => {
                binary_expr
            }
            _ => return,
        };

        let ((Expression::UnaryExpression(_), sibling) | (sibling, Expression::UnaryExpression(_))) =
            (&binary_expr.left, &binary_expr.right) else { return  };

        if let Expression::StringLiteral(lit) = sibling {
            if !VALID_TYPES.contains(lit.value.as_str()) {
                ctx.diagnostic(ValidTypeofDiagnostic::InvalidValue(None, sibling.span()));
            }
            return;
        }

        if let Expression::TemplateLiteral(template) = sibling && template.expressions.is_empty() {
                if let Some(value) = template.quasi() && !VALID_TYPES.contains(value.as_str()) {
                    ctx.diagnostic(ValidTypeofDiagnostic::InvalidValue(None, sibling.span()));
                }
                return;
            }

        if let Expression::Identifier(ident) = sibling
            && ident.name == "undefined"
            && ctx.semantic().is_reference_to_global_variable(ident) {
            ctx.diagnostic_with_fix(
                if self.require_string_literals {
                    ValidTypeofDiagnostic::NotString(
                        Some("Use `\"undefined\"` instead of `undefined`."),
                        sibling.span(),
                    )
                } else {
                    ValidTypeofDiagnostic::InvalidValue(
                        Some("Use `\"undefined\"` instead of `undefined`."),
                        sibling.span(),
                    )
                },
                || Fix::new("\"undefined\"", sibling.span()),
            );
            return;
        }

        if self.require_string_literals
            && !matches!(sibling, Expression::UnaryExpression(unary) if unary.operator == UnaryOperator::Typeof)
        {
            ctx.diagnostic(ValidTypeofDiagnostic::NotString(None, sibling.span()));
        }
    }

    fn from_configuration(value: serde_json::Value) -> Self {
        let require_string_literals = value.get(0).map_or(false, |config| {
            config
                .get("requireStringLiterals")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false)
        });

        Self { require_string_literals }
    }
}

const VALID_TYPES: Set<&'static str> = phf_set! {
    "symbol",
    "undefined",
    "object",
    "boolean",
    "number",
    "string",
    "function",
    "bigint",
};

#[test]
#[allow(clippy::too_many_lines)]
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

    Tester::new(ValidTypeof::NAME, pass, fail).test_and_snapshot();
}
