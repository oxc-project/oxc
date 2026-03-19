use oxc_ast::{
    AstKind,
    ast::{ChainElement, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::{BinaryOperator, UnaryOperator};

use crate::{
    AstNode,
    ast_util::{call_expr_method_callee_info, is_method_call},
    context::LintContext,
    rule::Rule,
};

fn prefer_includes_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "Prefer `includes()` over `indexOf()` when checking for existence or non-existence.",
    )
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferIncludes;

#[derive(Debug, Clone, Copy)]
enum ComparisonKind {
    // `indexOf(...) != -1` / `!== -1`
    ExistsOrUndefined,
    // `indexOf(...) > -1` / `>= 0`
    ExistsOnly,
    // `indexOf(...) == -1` / `=== -1` / `< 0`
    NotExistsOnly,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefer `includes()` over `indexOf()` when checking for existence or non-existence.
    /// All built-ins have `.includes()` in addition to `.indexOf()`.
    ///
    /// ### Why is this bad?
    ///
    /// The `.includes()` method is more readable and less error-prone than `.indexOf()`.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// if (str.indexOf('foo') !== -1) { }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// if (str.includes('foo')) { }
    /// ```
    PreferIncludes,
    unicorn,
    style,
    suggestion
);

impl Rule for PreferIncludes {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::BinaryExpression(bin_expr) = node.kind() else {
            return;
        };

        let left_call_expr = match bin_expr.left.without_parentheses() {
            Expression::CallExpression(call_expr) => call_expr,
            Expression::ChainExpression(chain_expr) => {
                let ChainElement::CallExpression(call_expr) = &chain_expr.expression else {
                    return;
                };
                call_expr
            }
            _ => return,
        };

        if !is_method_call(left_call_expr, None, Some(&["indexOf"]), None, Some(2)) {
            return;
        }

        let comparison_kind: Option<ComparisonKind> = if matches!(
            bin_expr.operator,
            BinaryOperator::StrictInequality | BinaryOperator::Inequality
        ) {
            if is_negative_one(bin_expr.right.without_parentheses()) {
                Some(ComparisonKind::ExistsOrUndefined)
            } else {
                None
            }
        } else if bin_expr.operator == BinaryOperator::GreaterThan {
            if is_negative_one(bin_expr.right.without_parentheses()) {
                Some(ComparisonKind::ExistsOnly)
            } else {
                None
            }
        } else if matches!(
            bin_expr.operator,
            BinaryOperator::StrictEquality | BinaryOperator::Equality
        ) {
            if is_negative_one(bin_expr.right.without_parentheses()) {
                Some(ComparisonKind::NotExistsOnly)
            } else {
                None
            }
        } else if bin_expr.operator == BinaryOperator::GreaterEqualThan {
            let Expression::NumericLiteral(num_lit) = bin_expr.right.without_parentheses() else {
                return;
            };
            if num_lit.raw.as_ref().unwrap() == "0" {
                Some(ComparisonKind::ExistsOnly)
            } else {
                None
            }
        } else if bin_expr.operator == BinaryOperator::LessThan {
            let Expression::NumericLiteral(num_lit) = bin_expr.right.without_parentheses() else {
                return;
            };
            if num_lit.raw.as_ref().unwrap() == "0" {
                Some(ComparisonKind::NotExistsOnly)
            } else {
                None
            }
        } else {
            None
        };

        let Some(comparison_kind) = comparison_kind else {
            return;
        };

        let callee_info = call_expr_method_callee_info(left_call_expr).unwrap();
        let callee_span = callee_info.0;

        // Get the object (receiver) text.
        let Some(member_expr) = left_call_expr.callee.get_inner_expression().as_member_expression()
        else {
            return;
        };

        let object_text = ctx.source_range(member_expr.object().span()).to_string();
        let member_operator = if member_expr.optional() { "?." } else { "." };
        let call_operator = if left_call_expr.optional { "?.(" } else { "(" };
        let has_optional_chain = left_call_expr.optional || member_expr.optional();

        // Get arguments text
        let args_text = left_call_expr
            .arguments
            .iter()
            .map(|arg| ctx.source_range(arg.span()))
            .collect::<Vec<_>>()
            .join(", ");

        let fix_span = bin_expr.span;
        ctx.diagnostic_with_suggestion(prefer_includes_diagnostic(callee_span), |fixer| {
            let includes_call =
                format!("{object_text}{member_operator}includes{call_operator}{args_text})");

            let replacement = if has_optional_chain {
                match comparison_kind {
                    ComparisonKind::ExistsOrUndefined => format!("{includes_call} !== false"),
                    ComparisonKind::ExistsOnly => format!("{includes_call} === true"),
                    ComparisonKind::NotExistsOnly => format!("{includes_call} === false"),
                }
            } else {
                match comparison_kind {
                    ComparisonKind::NotExistsOnly => format!("!{includes_call}"),
                    ComparisonKind::ExistsOrUndefined | ComparisonKind::ExistsOnly => includes_call,
                }
            };
            fixer.replace(fix_span, replacement)
        });
    }
}

fn is_negative_one(expr: &Expression) -> bool {
    let Expression::UnaryExpression(unary_expr) = expr else {
        return false;
    };

    if unary_expr.operator != UnaryOperator::UnaryNegation {
        return false;
    }

    let Expression::NumericLiteral(num_lit) = unary_expr.argument.without_parentheses() else {
        return false;
    };

    num_lit.raw.as_ref().unwrap() == "1"
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
        r"foo?.indexOf('x') !== -1",
        r"foo.indexOf?.('x') === -1",
        r"foo?.indexOf?.('x') == -1",
        r"foo?.indexOf('x') > -1",
        r"foo?.indexOf('x') >= 0",
        r"foo?.indexOf('x') < 0",
    ];

    let fix = vec![
        (r"'foobar'.indexOf('foo') !== -1", r"'foobar'.includes('foo')"),
        (r"str.indexOf('foo') != -1", r"str.includes('foo')"),
        (r"str.indexOf('foo') > -1", r"str.includes('foo')"),
        (r"str.indexOf('foo') == -1", r"!str.includes('foo')"),
        (r"'foobar'.indexOf('foo') >= 0", r"'foobar'.includes('foo')"),
        (r"[1,2,3].indexOf(4) !== -1", r"[1,2,3].includes(4)"),
        (r"str.indexOf('foo') < 0", r"!str.includes('foo')"),
        (r"''.indexOf('foo') < 0", r"!''.includes('foo')"),
        (r"foo.indexOf(bar, 0) !== -1", r"foo.includes(bar, 0)"),
        (r"foo.indexOf(bar, 1) !== -1", r"foo.includes(bar, 1)"),
        (r"foo?.indexOf('x') !== -1", r"foo?.includes('x') !== false"),
        (r"foo.indexOf?.('x') === -1", r"foo.includes?.('x') === false"),
        (r"foo?.indexOf?.('x') == -1", r"foo?.includes?.('x') === false"),
        (r"foo?.indexOf('x') > -1", r"foo?.includes('x') === true"),
        (r"foo?.indexOf('x') >= 0", r"foo?.includes('x') === true"),
        (r"foo?.indexOf('x') < 0", r"foo?.includes('x') === false"),
    ];

    Tester::new(PreferIncludes::NAME, PreferIncludes::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
