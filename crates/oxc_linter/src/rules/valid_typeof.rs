use oxc_ast::{
    ast::{Expression, UnaryOperator},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use phf::{phf_map, Map};

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

fn is_typeof_expr(expr: &Expression) -> bool {
    if let Expression::UnaryExpression(unary) = expr && unary.operator == UnaryOperator::Typeof {
        true
    } else {
        false
    }
}
const VALID_TYPE: Map<&'static str, bool> = phf_map! {
    "symbol" => false,
    "undefined" => false,
    "object" => false,
    "boolean" => false,
    "number" => false,
    "string" => false,
    "function" => false,
    "bigint" => false,
};
impl Rule for ValidTypeof {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::UnaryExpression(unary) = node.get().kind()
            && unary.operator == UnaryOperator::Typeof
            && let AstKind::BinaryExpression(binary) = ctx.parent_kind(node)
            && binary.operator.is_equality()
        {
            let (sibling,sibling_id) = if let Expression::UnaryExpression(left) = &binary.left && **left== *unary{
                (&binary.right, node.next_sibling().unwrap())
            } else {
                (&binary.left, node.previous_sibling().unwrap())
            };

            if let Expression::StringLiteral(lit) = sibling {
                if !VALID_TYPE.contains_key(lit.value.as_str()) {
                    ctx.diagnostic(ValidTypeofDiagnostic::InvalidValue(None, sibling.span()));
                }
                return;
            }
            if let Expression::TemplateLiteral(template) = sibling && template.expressions.is_empty() {
                if let Some(value) = template.quasi() && !VALID_TYPE.contains_key(value.as_str()) {
                    ctx.diagnostic(ValidTypeofDiagnostic::InvalidValue(None, sibling.span()));
                }
                return;
            }

            if sibling.is_undefined()
                && ctx.semantic().is_unresolved_reference(sibling_id.into())
            {
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
            if self.require_string_literals && !is_typeof_expr(sibling) {
                ctx.diagnostic(ValidTypeofDiagnostic::NotString(None, sibling.span()));
            }
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
