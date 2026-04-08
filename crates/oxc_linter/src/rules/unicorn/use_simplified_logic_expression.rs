use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::LogicalOperator;

use crate::{AstNode, context::LintContext, rule::Rule};

fn use_simplified_logic_expression_diagnostic(span: Span, suggestion: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn("This logical expression can be simplified.")
        .with_help(format!("Simplify to: `{suggestion}`"))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct UseSimplifiedLogicExpression;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Detects logical expressions that can be simplified.
    ///
    /// ### Why is this bad?
    ///
    /// Overly complex boolean expressions reduce readability. Expressions like
    /// `x || true`, `x && false`, `x || !x` can be simplified.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// const a = x || true;
    /// const b = x && false;
    /// const c = x || false;
    /// const d = x && true;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const a = true;
    /// const b = false;
    /// const c = x;
    /// const d = x;
    /// ```
    UseSimplifiedLogicExpression,
    unicorn,
    style,
    pending
);

impl Rule for UseSimplifiedLogicExpression {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::LogicalExpression(expr) = node.kind() else {
            return;
        };

        match expr.operator {
            LogicalOperator::Or => {
                // x || true => true
                if is_boolean_literal(&expr.right, true) {
                    ctx.diagnostic(use_simplified_logic_expression_diagnostic(expr.span, "true"));
                }
                // x || false => x
                else if is_boolean_literal(&expr.right, false) {
                    let left_src = ctx.source_range(expr.left.span());
                    ctx.diagnostic(use_simplified_logic_expression_diagnostic(expr.span, left_src));
                }
                // true || x => true
                else if is_boolean_literal(&expr.left, true) {
                    ctx.diagnostic(use_simplified_logic_expression_diagnostic(expr.span, "true"));
                }
                // false || x => x
                else if is_boolean_literal(&expr.left, false) {
                    let right_src = ctx.source_range(expr.right.span());
                    ctx.diagnostic(use_simplified_logic_expression_diagnostic(
                        expr.span, right_src,
                    ));
                }
            }
            LogicalOperator::And => {
                // x && false => false
                if is_boolean_literal(&expr.right, false) {
                    ctx.diagnostic(use_simplified_logic_expression_diagnostic(expr.span, "false"));
                }
                // x && true => x
                else if is_boolean_literal(&expr.right, true) {
                    let left_src = ctx.source_range(expr.left.span());
                    ctx.diagnostic(use_simplified_logic_expression_diagnostic(expr.span, left_src));
                }
                // false && x => false
                else if is_boolean_literal(&expr.left, false) {
                    ctx.diagnostic(use_simplified_logic_expression_diagnostic(expr.span, "false"));
                }
                // true && x => x
                else if is_boolean_literal(&expr.left, true) {
                    let right_src = ctx.source_range(expr.right.span());
                    ctx.diagnostic(use_simplified_logic_expression_diagnostic(
                        expr.span, right_src,
                    ));
                }
            }
            _ => {}
        }
    }
}

fn is_boolean_literal(expr: &Expression, value: bool) -> bool {
    matches!(expr, Expression::BooleanLiteral(lit) if lit.value == value)
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec!["const a = x || y;", "const b = x && y;", "const c = x ?? y;"];

    let fail = vec![
        "const a = x || true;",
        "const b = x && false;",
        "const c = x || false;",
        "const d = x && true;",
        "const e = true || x;",
        "const f = false && x;",
        "const g = false || x;",
        "const h = true && x;",
    ];

    Tester::new(
        UseSimplifiedLogicExpression::NAME,
        UseSimplifiedLogicExpression::PLUGIN,
        pass,
        fail,
    )
    .test_and_snapshot();
}
