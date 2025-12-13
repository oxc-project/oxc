use oxc_ast::{
    AstKind,
    ast::{ArrayExpressionElement, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

fn no_array_reverse_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Use `Array#toReversed()` instead of `Array#reverse()`.")
        .with_help("`Array#reverse()` mutates the original array. Use `Array#toReversed()` to return a new reversed array without modifying the original.")
        .with_label(span)
}

#[derive(Debug, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct NoArrayReverse {
    /// This rule allows `array.reverse()` as an expression statement by default.
    /// Set to `false` to forbid `Array#reverse()` even if it's an expression statement.
    ///
    /// Examples of **incorrect** code for this rule with this option set to `false`:
    /// ```js
    /// array.reverse();
    /// ```
    allow_expression_statement: bool,
}

impl Default for NoArrayReverse {
    fn default() -> Self {
        Self { allow_expression_statement: true }
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
    fix,
    config = NoArrayReverse,
);

impl Rule for NoArrayReverse {
    fn from_configuration(value: Value) -> Self {
        serde_json::from_value::<DefaultRuleConfig<NoArrayReverse>>(value)
            .unwrap_or_default()
            .into_inner()
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
        ctx.diagnostic_with_fix(no_array_reverse_diagnostic(span), |fixer| {
            fixer.replace(span, "toReversed")
        });
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
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
