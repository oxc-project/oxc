use oxc_ast::{
    AstKind,
    ast::{Argument, ArrayExpressionElement, Expression},
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

fn no_array_sort_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Use `Array#toSorted()` instead of `Array#sort()`.")
        .with_help("`Array#sort()` mutates the original array. Use `Array#toSorted()` to return a new sorted array without modifying the original.")
        .with_label(span)
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct NoArraySort {
    /// When set to `true` (default), allows `array.sort()` as an expression statement.
    /// Set to `false` to forbid `Array#sort()` even if it's an expression statement.
    ///
    /// Example of **incorrect** code for this rule with `allowExpressionStatement` set to `false`:
    /// ```js
    /// array.sort();
    /// ```
    allow_expression_statement: bool,
}

impl Default for NoArraySort {
    fn default() -> Self {
        Self { allow_expression_statement: true }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefer using `Array#toSorted()` over `Array#sort()`.
    ///
    /// ### Why is this bad?
    ///
    /// `Array#sort()` modifies the original array in place, which can lead to unintended side effects—especially
    /// when the original array is used elsewhere in the code.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// const sorted = [...array].sort();
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const sorted = [...array].toSorted();
    /// ```
    NoArraySort,
    unicorn,
    suspicious,
    fix,
    config = NoArraySort,
);

impl Rule for NoArraySort {
    fn from_configuration(value: Value) -> Self {
        serde_json::from_value::<DefaultRuleConfig<NoArraySort>>(value)
            .unwrap_or_default()
            .into_inner()
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };
        if call_expr.optional {
            return;
        }
        if call_expr.arguments.len() > 1 {
            return;
        }
        if call_expr.arguments.len() == 1
            && matches!(call_expr.arguments[0], Argument::SpreadElement(_))
        {
            return;
        }
        let Some(member_expr) = call_expr.callee.get_member_expr() else {
            return;
        };
        let Some((span, static_property_name)) = member_expr.static_property_info() else {
            return;
        };
        if static_property_name != "sort" {
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

        ctx.diagnostic_with_fix(no_array_sort_diagnostic(span), |fixer| {
            fixer.replace(span, "toSorted")
        });
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("sorted = [...array].toSorted()", None),
        ("sorted = array.toSorted()", None),
        ("sorted = [...array].sort", None),
        ("sorted = [...array].sort?.()", None),
        ("array.sort()", None),
        ("array.sort?.()", None),
        ("array?.sort()", None),
        ("if (true) array.sort()", None),
        ("sorted = array.sort(...[])", None),
        ("sorted = array.sort(...[compareFn])", None),
        ("sorted = array.sort(compareFn, extraArgument)", None),
    ];

    let fail = vec![
        ("sorted = [...array].sort()", None),
        ("sorted = [...array]?.sort()", None),
        ("sorted = array.sort()", None),
        ("sorted = array?.sort()", None),
        ("sorted = [...array].sort(compareFn)", None),
        ("sorted = [...array]?.sort(compareFn)", None),
        ("sorted = array.sort(compareFn)", None),
        ("sorted = array?.sort(compareFn)", None),
        ("array.sort()", Some(serde_json::json!([{"allowExpressionStatement": false}]))),
        ("array?.sort()", Some(serde_json::json!([{"allowExpressionStatement": false}]))),
        ("[...array].sort()", Some(serde_json::json!([{"allowExpressionStatement": false}]))),
        ("sorted = [...(0, array)].sort()", None),
    ];

    let fix = vec![
        ("sorted = [...array].sort()", "sorted = [...array].toSorted()", None),
        ("sorted = [...array]?.sort()", "sorted = [...array]?.toSorted()", None),
        (
            "a.sort()",
            "a.toSorted()",
            Some(serde_json::json!([{"allowExpressionStatement": false}])),
        ),
        ("sorted = array?.sort()", "sorted = array?.toSorted()", None),
    ];

    Tester::new(NoArraySort::NAME, NoArraySort::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
