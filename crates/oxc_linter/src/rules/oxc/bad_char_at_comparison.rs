use oxc_ast::{ast::Expression, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::BinaryOperator;

use crate::{ast_util::is_method_call, context::LintContext, rule::Rule, AstNode};

fn bad_char_at_comparison_diagnostic(
    char_at: Span,
    compared_string: Span,
    len: usize,
) -> OxcDiagnostic {
    OxcDiagnostic::warn("Invalid comparison with `charAt` method")
        .with_help("`String.prototype.charAt` returns a string of length 1. If the return value is compared with a string of length greater than 1, the comparison will always be false.")
        .with_labels([
            char_at.label("`charAt` called here"),
            compared_string.label(format!("And compared with a string of length {len} here")),
        ])
}

#[derive(Debug, Default, Clone)]
pub struct BadCharAtComparison;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule warns when the return value of the `charAt` method is used to compare a string of length greater than 1.
    ///
    /// ### Why is this bad?
    ///
    /// The `charAt` method returns a string of length 1. If the return value is compared with a string of length greater than 1, the comparison will always be false.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// a.charAt(4) === 'a2';
    /// a.charAt(4) === '/n';
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// a.charAt(4) === 'a'
    /// a.charAt(4) === '\n';
    /// ```
    BadCharAtComparison,
    oxc,
    correctness
);

impl Rule for BadCharAtComparison {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        if !is_method_call(call_expr, None, Some(&["charAt"]), Some(1), Some(1)) {
            return;
        }

        let Some(parent) = ctx.nodes().parent_node(node.id()) else {
            return;
        };

        let AstKind::BinaryExpression(binary_expr) = parent.kind() else {
            return;
        };
        if !matches!(
            binary_expr.operator,
            BinaryOperator::Equality
                | BinaryOperator::Inequality
                | BinaryOperator::StrictEquality
                | BinaryOperator::StrictInequality
        ) {
            return;
        };

        let comparison_with = if binary_expr.left.span() == call_expr.span {
            &binary_expr.right
        } else {
            &binary_expr.left
        };

        if let Expression::StringLiteral(string_lit) = comparison_with {
            if !is_string_valid(string_lit.value.as_str()) {
                ctx.diagnostic(bad_char_at_comparison_diagnostic(
                    call_expr.span,
                    string_lit.span,
                    string_lit.value.len(),
                ));
            }
        }
    }
}

fn is_string_valid(str: &str) -> bool {
    if str.len() < 2 || str.chars().count() == 1 {
        return true;
    }

    false
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"a.charAt(4) === 'a'",
        "a.charAt(4) === '\\n'",
        "a.charAt(4) === '\t'",
        r"a.charAt(4) === 'a'",
        r"a.charAt(4) === '\ufeff'",
        r"a.charAt(4) !== '\ufeff'",
        "chatAt(4) === 'a2'",
        "new chatAt(4) === 'a'",
    ];

    let fail = vec![
        r"a.charAt(4) === 'aa'",
        "a.charAt(4) === '/n'",
        "a.charAt(3) === '/t'",
        r"a.charAt(4) === 'ac'",
        r"a.charAt(822) !== 'foo'",
        r"a.charAt(4) === '\\ukeff'",
    ];

    Tester::new(BadCharAtComparison::NAME, BadCharAtComparison::PLUGIN, pass, fail)
        .test_and_snapshot();
}
