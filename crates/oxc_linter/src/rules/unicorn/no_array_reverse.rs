use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;

use oxc_ast::{
    AstKind,
    ast::{ArrayExpressionElement, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
    utils::is_temporary_array,
};

fn no_array_reverse_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Use `Array#toReversed()` instead of `Array#reverse()`.")
        .with_help("`Array#reverse()` mutates the original array. Use `Array#toReversed()` to return a new reversed array without modifying the original.")
        .with_label(span)
}

#[derive(Debug, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct NoArrayReverse {
    /// This rule allows `array.reverse()` as an expression statement by default.
    /// Set to `false` to forbid `Array#reverse()` even if it's an expression statement.
    ///
    /// Examples of **incorrect** code for this rule with this option set to `false`:
    /// ```js
    /// array.reverse();
    /// ```
    allow_expression_statement: bool,

    /// When set to `true`, allows reversing fresh arrays created by array literals, spreads,
    /// `arr.map()`, `arr.filter()`, `new Array()`, `Array.from()`, `Object.keys()`, etc. without
    /// making another copy. This avoids the double allocation of `toReversed()` when it is not
    /// needed. Instance methods are recognized by name, assuming built-in array behavior.
    ///
    /// Examples of **correct** code with `allowOnTemporaries` set to `true`:
    /// ```js
    /// const reversed = array.map(transform).reverse();
    /// const values = Object.values(object).reverse();
    /// ```
    allow_on_temporaries: bool,
}

impl Default for NoArrayReverse {
    fn default() -> Self {
        Self { allow_expression_statement: true, allow_on_temporaries: false }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefer using `Array#toReversed()` over `Array#reverse()`.
    ///
    /// ### Why is this bad?
    ///
    /// `Array#reverse()` modifies the original array in place, which can lead to unintended side effects—especially
    /// when the original array is used elsewhere in the code.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// const reversed = [...array].reverse();
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const reversed = [...array].toReversed();
    /// ```
    NoArrayReverse,
    unicorn,
    suspicious,
    recommended,
    suggestion,
    config = NoArrayReverse,
    version = "1.15.0",
    short_description = "Prefer using `Array#toReversed()` over `Array#reverse()`.",
);

impl Rule for NoArrayReverse {
    fn from_configuration(value: Value) -> Result<Self, serde_json::error::Error> {
        DefaultRuleConfig::<Self>::from_value(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };
        if !call_expr.arguments.is_empty() || call_expr.optional {
            return;
        }
        let Some(member_expr) = call_expr.callee.get_member_expr() else {
            return;
        };
        let Some((span, static_property_name)) = member_expr.static_property_info() else {
            return;
        };
        if static_property_name != "reverse" {
            return;
        }
        if self.allow_on_temporaries && is_temporary_array(member_expr.object(), ctx) {
            return;
        }
        let is_spread = match member_expr.object() {
            Expression::ArrayExpression(array) => {
                array.elements.len() == 1
                    && matches!(array.elements[0], ArrayExpressionElement::SpreadElement(_))
            }
            _ => false,
        };
        if self.allow_expression_statement && !is_spread {
            let parent = ctx.nodes().parent_node(node.id());
            let parent_is_expression_statement = match parent.kind() {
                AstKind::ExpressionStatement(_) => true,
                AstKind::ChainExpression(_) => {
                    let grand_parent = ctx.nodes().parent_node(parent.id());
                    matches!(grand_parent.kind(), AstKind::ExpressionStatement(_))
                }
                _ => false,
            };
            if parent_is_expression_statement {
                return;
            }
        }
        // The replacement is a suggestion, not an auto-fix, matching upstream
        // eslint-plugin-unicorn (`hasSuggestions: true`, not `fixable`): the
        // receiver is not statically known to be an array, and non-array
        // fluent APIs with a `reverse()` method (e.g. Dexie's
        // `Collection#reverse()`) have no `toReversed()`, so applying the
        // replacement blindly under `--fix` turns working code into a
        // runtime `TypeError`.
        ctx.diagnostic_with_suggestion(no_array_reverse_diagnostic(span), |fixer| {
            fixer.replace(span, "toReversed")
        });
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "reversed = [...array].reverse()",
            Some(serde_json::json!([{ "allowOnTemporaries": true }])),
        ),
        (
            "[...items].reverse();",
            Some(serde_json::json!([{
                "allowOnTemporaries": true,
                "allowExpressionStatement": false
            }])),
        ),
        (
            "const result = items.map(fn).reverse();",
            Some(serde_json::json!([{ "allowOnTemporaries": true }])),
        ),
        (
            "const result = Object.values(object).reverse();",
            Some(serde_json::json!([{ "allowOnTemporaries": true }])),
        ),
        ("reversed =[...array].toReversed()", None),
        ("reversed =array.toReversed()", None),
        ("reversed =[...array].reverse", None),
        ("reversed =[...array].reverse?.()", None),
        ("array.reverse()", None),
        ("array.reverse?.()", None),
        ("array?.reverse()", None),
        ("if (true) array.reverse()", None),
        ("reversed = array.reverse(extraArgument)", None),
    ];

    let fail = vec![
        (
            "const result = getArray().reverse();",
            Some(serde_json::json!([{ "allowOnTemporaries": true }])),
        ),
        (
            "const result = items.map(fn).reverse();",
            Some(serde_json::json!([{ "allowOnTemporaries": false }])),
        ),
        ("reversed = [...array].reverse()", None),
        ("reversed = [...array]?.reverse()", None),
        ("reversed = array.reverse()", None),
        ("reversed = array?.reverse()", None),
        ("array.reverse()", Some(serde_json::json!([{"allowExpressionStatement": false}]))),
        ("array?.reverse()", Some(serde_json::json!([{"allowExpressionStatement": false}]))),
        ("[...array].reverse()", Some(serde_json::json!([{"allowExpressionStatement": true}]))),
        ("reversed = [...(0, array)].reverse()", None),
    ];

    let fix = vec![
        (
            "const result = getArray().reverse();",
            "const result = getArray().toReversed();",
            Some(serde_json::json!([{ "allowOnTemporaries": true }])),
        ),
        ("reversed = [...array].reverse()", "reversed = [...array].toReversed()", None),
        ("reversed = [...array]?.reverse()", "reversed = [...array]?.toReversed()", None),
        (
            "a.reverse()",
            "a.toReversed()",
            Some(serde_json::json!([{"allowExpressionStatement": false}])),
        ),
        ("reversed = array?.reverse()", "reversed = array?.toReversed()", None),
    ];

    Tester::new(NoArrayReverse::NAME, NoArrayReverse::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
