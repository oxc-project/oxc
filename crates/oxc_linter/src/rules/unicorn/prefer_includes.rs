use oxc_ast::{ast::Expression, AstKind};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::operator::{BinaryOperator, UnaryOperator};

use crate::{
    ast_util::{call_expr_method_callee_info, is_method_call},
    context::LintContext,
    rule::Rule,
    AstNode,
};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-unicorn(prefer-includes): Prefer `includes()` over `indexOf()` when checking for existence or non-existence.")]
#[diagnostic(severity(warning))]
struct PreferIncludesDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct PreferIncludes;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefer `includes()` over `indexOf()` when checking for existence or non-existence.
    ///
    /// All built-ins have `.includes()` in addition to `.indexOf()`.
    ///
    /// ### Why is this bad?
    ///
    /// The `.includes()` method is more readable and less error-prone than `.indexOf()`.
    ///
    /// ### Example
    /// ```javascript
    /// // bad
    /// if (str.indexOf('foo') !== -1) { }
    ///
    /// // good
    /// if (str.includes('foo')) { }
    /// ```
    PreferIncludes,
    style
);

impl Rule for PreferIncludes {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::BinaryExpression(bin_expr) = node.kind() else {
            return;
        };

        let Expression::CallExpression(left_call_expr) = &bin_expr.left.without_parenthesized()
        else {
            return;
        };

        if !is_method_call(left_call_expr, None, Some(&["indexOf"]), None, Some(2)) {
            return;
        }

        if matches!(
            bin_expr.operator,
            BinaryOperator::StrictInequality
                | BinaryOperator::Inequality
                | BinaryOperator::GreaterThan
                | BinaryOperator::StrictEquality
                | BinaryOperator::Equality
        ) {
            if !is_negative_one(bin_expr.right.without_parenthesized()) {
                return;
            }

            ctx.diagnostic(PreferIncludesDiagnostic(
                call_expr_method_callee_info(left_call_expr).unwrap().0,
            ));
        }

        if matches!(bin_expr.operator, BinaryOperator::GreaterEqualThan | BinaryOperator::LessThan)
        {
            let Expression::NumberLiteral(num_lit) = bin_expr.right.without_parenthesized() else {
                return;
            };

            if num_lit.raw != "0" {
                return;
            }
            ctx.diagnostic(PreferIncludesDiagnostic(
                call_expr_method_callee_info(left_call_expr).unwrap().0,
            ));
        }
    }
}

fn is_negative_one(expr: &Expression) -> bool {
    let Expression::UnaryExpression(unary_expr) = expr else {
        return false;
    };

    if unary_expr.operator != UnaryOperator::UnaryNegation {
        return false;
    }

    let Expression::NumberLiteral(num_lit) = unary_expr.argument.without_parenthesized() else {
        return false;
    };

    num_lit.raw == "1"
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"str.indexOf('foo') !== -n",
        r"str.indexOf('foo') !== 1",
        r"str.indexOf('foo') === -2",
        r"!str.indexOf('foo') === 1",
        r"!str.indexOf('foo') === -n",
        r"str.includes('foo')",
        r"'foobar'.includes('foo')",
        r"[1,2,3].includes(4)",
        r"null.indexOf('foo') !== 1",
        r"f(0) < 0",
        r"something.indexOf(foo, 0, another) !== -1",
    ];

    let fail = vec![
        r"'foobar'.indexOf('foo') !== -1",
        r"str.indexOf('foo') != -1",
        r"str.indexOf('foo') > -1",
        r"str.indexOf('foo') == -1",
        r"'foobar'.indexOf('foo') >= 0",
        r"[1,2,3].indexOf(4) !== -1",
        r"str.indexOf('foo') < 0",
        r"''.indexOf('foo') < 0",
        r"(a || b).indexOf('foo') === -1",
        r"foo.indexOf(bar, 0) !== -1",
        r"foo.indexOf(bar, 1) !== -1",
    ];

    Tester::new_without_config(PreferIncludes::NAME, pass, fail).test_and_snapshot();
}
