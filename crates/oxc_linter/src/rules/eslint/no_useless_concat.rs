use oxc_ast::{
    ast::{BinaryExpression, Expression},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::{identifier::is_line_terminator, operator::BinaryOperator};

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Default, Clone)]
pub struct NoUselessConcat;

fn no_useless_concat_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected string concatenation of literals.")
        .with_help("Rewrite into one string literal")
        .with_label(span)
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow unnecessary concatenation of literals or template literals
    ///
    /// ### Why is this bad?
    ///
    /// It’s unnecessary to concatenate two strings together.
    ///
    /// ### Example
    /// ```javascript
    /// var foo = "a" + "b";
    /// ```
    NoUselessConcat,
    eslint,
    suspicious
);

impl Rule for NoUselessConcat {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::BinaryExpression(binary_expr) = node.kind() else {
            return;
        };

        if binary_expr.operator != BinaryOperator::Addition {
            return;
        }

        let left = get_left(binary_expr);
        let right = get_right(binary_expr);

        if left.is_string_literal() && right.is_string_literal() {
            let left_span = left.span();
            let right_span = right.span();
            let span = Span::new(left_span.end, right_span.start);
            let source_text = span.source_text(ctx.source_text());
            if source_text.chars().any(is_line_terminator) {
                return;
            }
            let span = Span::new(left_span.start, right_span.end);
            ctx.diagnostic(no_useless_concat_diagnostic(span));
        }
    }
}

fn get_left<'a>(expr: &'a BinaryExpression<'a>) -> &'a Expression<'a> {
    let mut left = &expr.left;
    loop {
        if let Expression::BinaryExpression(binary_expr) = left {
            if binary_expr.operator == BinaryOperator::Addition {
                left = &binary_expr.right;
                continue;
            }
        }
        break;
    }
    left
}

fn get_right<'a>(expr: &'a BinaryExpression<'a>) -> &'a Expression<'a> {
    let mut right = &expr.right;
    loop {
        if let Expression::BinaryExpression(binary_expr) = right {
            if binary_expr.operator == BinaryOperator::Addition {
                right = &binary_expr.left;
                continue;
            }
        }
        break;
    }
    right
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "var a = 1 + 1;",
        "var a = 1 * '2';",
        "var a = 1 - 2;",
        "var a = foo + bar;",
        "var a = 'foo' + bar;",
        "var foo = 'foo' +
        'bar';",
        "var string = (number + 1) + 'px';",
        "'a' + 1",
        "1 + '1'",
        "1 + `1`",
        "`1` + 1",
        "(1 + +2) + `b`",
        "
          'a'
          + 'b'
          + 'c'
        ",
    ];

    let fail = vec![
        "'a' + 'b'",
        "'a' +
        'b' + 'c'",
        "foo + 'a' + 'b'",
        "'a' + 'b' + 'c'",
        "(foo + 'a') + ('b' + 'c')",
        "`a` + 'b'",
        "`a` + `b`",
        "foo + `a` + `b`",
        "foo + 'a' + 'b'",
        "'a' +
        'b' + 'c'
        + 'd'
        ",
        "'a' + 'b' + 'c' + 'd' + 'e' + foo",
    ];

    Tester::new(NoUselessConcat::NAME, NoUselessConcat::PLUGIN, pass, fail).test_and_snapshot();
}
