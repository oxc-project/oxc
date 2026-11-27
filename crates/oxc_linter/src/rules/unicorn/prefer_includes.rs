use itertools::Itertools;

use oxc_ast::{
    AstKind,
    ast::{Argument, ChainElement, Expression, TSType},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::IsGlobalReference;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::{BinaryOperator, UnaryOperator};

use crate::{
    AstNode,
    ast_util::{call_expr_method_callee_info, is_method_call},
    context::LintContext,
    rule::Rule,
    utils::is_regexp_callee,
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
    // `indexOf(...) == -1` / `=== -1` / `<= -1` / `< 0`
    NotExistsOnly,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefer `includes()` over `indexOf()` and `lastIndexOf()` when checking for existence/non-existence.
    /// All built-ins have `.includes()` in addition to `.indexOf()` and `.lastIndexOf()`.
    ///
    /// ### Why is this bad?
    ///
    /// The `.includes()` method is more readable and less error-prone than `.indexOf()` or `.lastIndexOf()`.
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
    recommended,
    suggestion,
    version = "0.0.18",
    short_description = "Prefer `includes()` over `indexOf()` and `lastIndexOf()` when checking for existence/non-existence.",
);

const METHODS: [&str; 2] = ["indexOf", "lastIndexOf"];

impl Rule for PreferIncludes {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::BinaryExpression(bin_expr) = node.kind()
            && let Some(left_call_expr) = (match bin_expr.left.without_parentheses() {
                Expression::CallExpression(call_expr) => Some(call_expr),
                Expression::ChainExpression(chain_expr) => match &chain_expr.expression {
                    ChainElement::CallExpression(call_expr) => Some(call_expr),
                    _ => None,
                },
                _ => None,
            })
            && is_method_call(left_call_expr, None, Some(&METHODS), None, Some(2))
        {
            let comparison_kind = match bin_expr.operator {
                BinaryOperator::StrictInequality | BinaryOperator::Inequality
                    if is_negative_one(bin_expr.right.without_parentheses()) =>
                {
                    Some(ComparisonKind::ExistsOrUndefined)
                }
                BinaryOperator::GreaterThan
                    if is_negative_one(bin_expr.right.without_parentheses()) =>
                {
                    Some(ComparisonKind::ExistsOnly)
                }
                BinaryOperator::StrictEquality
                | BinaryOperator::Equality
                | BinaryOperator::LessEqualThan
                    if is_negative_one(bin_expr.right.without_parentheses()) =>
                {
                    Some(ComparisonKind::NotExistsOnly)
                }
                BinaryOperator::GreaterEqualThan
                    if bin_expr.right.without_parentheses().is_specific_raw_number_literal("0") =>
                {
                    Some(ComparisonKind::ExistsOnly)
                }
                BinaryOperator::LessThan
                    if bin_expr.right.without_parentheses().is_specific_raw_number_literal("0") =>
                {
                    Some(ComparisonKind::NotExistsOnly)
                }
                _ => None,
            };

            if let Some(comparison_kind) = comparison_kind
                && let Some((callee_span, method_name)) =
                    call_expr_method_callee_info(left_call_expr)
                && let Some(member_expr) =
                    left_call_expr.callee.without_parentheses().as_member_expression()
            {
                let args = &left_call_expr.arguments;
                let receiver = member_expr.object();
                if left_call_expr.type_arguments.is_some()
                    || (method_name == "lastIndexOf"
                        && (args.len() > 1
                            || matches!(args.first(), Some(Argument::SpreadElement(_)))))
                    || args.first().and_then(Argument::as_expression).is_some_and(|a| {
                        is_nan_expr(a, ctx) || is_unsafe_regexp_search(a, receiver, ctx)
                    })
                {
                    return;
                }

                let object_text = ctx.source_range(receiver.span());
                let member_operator = if member_expr.optional() { "?." } else { "." };
                let call_operator = if left_call_expr.optional { "?.(" } else { "(" };
                let has_optional_chain =
                    matches!(bin_expr.left.without_parentheses(), Expression::ChainExpression(_));

                let fix_span = bin_expr.span;
                let diagnostic = if method_name == "lastIndexOf" {
                    OxcDiagnostic::warn(
                        "Prefer `includes()` over `lastIndexOf()` when checking for existence or non-existence.",
                    )
                    .with_label(callee_span)
                } else {
                    prefer_includes_diagnostic(callee_span)
                };
                ctx.diagnostic_with_suggestion(diagnostic, |fixer| {
                    let args_text = left_call_expr
                        .arguments
                        .iter()
                        .map(|arg| ctx.source_range(arg.span()))
                        .join(", ");

                    let includes_call = format!(
                        "{object_text}{member_operator}includes{call_operator}{args_text})"
                    );

                    let replacement = if has_optional_chain {
                        match comparison_kind {
                            ComparisonKind::ExistsOrUndefined => {
                                format!("{includes_call} !== false")
                            }
                            ComparisonKind::ExistsOnly => format!("{includes_call} === true"),
                            ComparisonKind::NotExistsOnly => format!("{includes_call} === false"),
                        }
                    } else {
                        match comparison_kind {
                            ComparisonKind::NotExistsOnly => format!("!{includes_call}"),
                            ComparisonKind::ExistsOrUndefined | ComparisonKind::ExistsOnly => {
                                includes_call
                            }
                        }
                    };
                    let wrap = has_optional_chain
                        && bin_expr.operator.is_compare()
                        && match ctx.nodes().parent_kind(node.id()) {
                            AstKind::TSAsExpression(_) | AstKind::TSSatisfiesExpression(_) => true,
                            AstKind::BinaryExpression(p) => {
                                p.operator.is_equality()
                                    || p.operator.is_compare()
                                    || p.operator.is_relational()
                            }
                            _ => false,
                        };
                    let replacement = if wrap { format!("({replacement})") } else { replacement };
                    fixer.replace(fix_span, replacement)
                });
            }
        }
    }
}

fn is_negative_one(expr: &Expression) -> bool {
    if let Expression::UnaryExpression(unary_expr) = expr
        && unary_expr.operator == UnaryOperator::UnaryNegation
    {
        unary_expr.argument.without_parentheses().is_specific_raw_number_literal("1")
    } else {
        false
    }
}

fn is_nan_expr(expr: &Expression, ctx: &LintContext) -> bool {
    let expr = expr.get_inner_expression();
    match expr {
        Expression::Identifier(id) => id.name == "NaN" && id.is_global_reference(ctx.scoping()),
        Expression::UnaryExpression(u) => {
            matches!(u.operator, UnaryOperator::UnaryPlus | UnaryOperator::UnaryNegation)
                && is_nan_expr(&u.argument, ctx)
        }
        _ => expr.get_member_expr().is_some_and(|m| {
            m.static_property_name().is_some_and(|n| n == "NaN")
                && matches!(m.object().get_inner_expression(), Expression::Identifier(id) if id.name == "Number" && id.is_global_reference(ctx.scoping()))
        }),
    }
}

fn is_unsafe_regexp_search<'b>(
    arg: &'b Expression<'b>,
    recv: &Expression<'_>,
    ctx: &'b LintContext<'_>,
) -> bool {
    let ts_type = match recv.without_parentheses() {
        Expression::ArrayExpression(_) => return false,
        Expression::TSAsExpression(t) => Some(&t.type_annotation),
        Expression::TSSatisfiesExpression(t) => Some(&t.type_annotation),
        Expression::TSTypeAssertion(t) => Some(&t.type_annotation),
        _ => None,
    };
    if ts_type.is_some_and(|t| matches!(t, TSType::TSArrayType(_))) {
        return false;
    }
    let callee = match arg.get_inner_expression() {
        Expression::RegExpLiteral(_) => return true,
        Expression::NewExpression(n) => &n.callee,
        Expression::CallExpression(c) => &c.callee,
        Expression::ChainExpression(c) => match &c.expression {
            ChainElement::CallExpression(c) => &c.callee,
            _ => return false,
        },
        _ => return false,
    };
    is_regexp_callee(callee.get_inner_expression(), ctx)
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
        r"(obj?.indexOf)('x') !== -1;",
        r"((obj?.['indexOf']))('x') === -1;",
        r"(obj?.indexOf)?.('x') >= 0;",
        r"((obj?.indexOf)!)('x') < 0;",
        r"array.lastIndexOf('foo', 0) !== -1",
        r"array.lastIndexOf(...args) !== -1",
        r"array.lastIndexOf(NaN) !== -1",
        r"array.indexOf(Number.NaN) === -1",
        r"array.indexOf(Number['NaN']) !== -1",
        r"array.indexOf(Number?.NaN) !== -1",
        r"array.indexOf(-NaN) !== -1",
        r"str.indexOf(/a/) !== -1",
        r"str.lastIndexOf(new RegExp('a')) !== -1",
        r"str.indexOf(RegExp('a')) !== -1",
        r"str.indexOf(globalThis.RegExp('a')) !== -1",
        r"arr.indexOf<string>('a') !== -1",
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
        r"array.indexOf('foo') <= -1",
        r"array.lastIndexOf('foo') !== -1",
        r"array.lastIndexOf('foo') <= -1",
        r"array?.lastIndexOf('foo') === -1",
        r"foo?.bar.indexOf('x') <= -1",
        r"a === foo?.indexOf('x') <= -1",
        r"a === foo?.indexOf('x') <= -1 as boolean",
        r"foo?.indexOf('x') <= -1 in obj",
        r"array.lastIndexOf() !== -1",
        r"[pattern].indexOf(/a/) !== -1",
        r"(values as RegExp[]).lastIndexOf((pattern as RegExp)) !== -1",
        r"(values satisfies RegExp[]).lastIndexOf(/a/) !== -1",
        r"function f(NaN: number) { return array.indexOf(NaN) !== -1; }",
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
        (r"(str.indexOf)('foo') !== -1", r"str.includes('foo')"),
        (r"((str['indexOf']))('foo') === -1", r"!str.includes('foo')"),
        (r"(foo.indexOf)?.('x') === -1", r"foo.includes?.('x') === false"),
        (r"array.indexOf('foo') <= -1", r"!array.includes('foo')"),
        (r"array.lastIndexOf('foo') !== -1", r"array.includes('foo')"),
        (r"array.lastIndexOf('foo') <= -1", r"!array.includes('foo')"),
        (r"array?.lastIndexOf('foo') === -1", r"array?.includes('foo') === false"),
        (r"foo?.bar.indexOf('x') <= -1", r"foo?.bar.includes('x') === false"),
        (r"a === foo?.indexOf('x') <= -1", r"a === (foo?.includes('x') === false)"),
        (
            r"a === foo?.indexOf('x') <= -1 as boolean",
            r"a === (foo?.includes('x') === false) as boolean",
        ),
        (r"foo?.indexOf('x') <= -1 in obj", r"(foo?.includes('x') === false) in obj"),
        (r"array.lastIndexOf() !== -1", r"array.includes()"),
        (r"[pattern].indexOf(/a/) !== -1", r"[pattern].includes(/a/)"),
        (
            r"(values as RegExp[]).lastIndexOf((pattern as RegExp)) !== -1",
            r"(values as RegExp[]).includes((pattern as RegExp))",
        ),
        (
            r"(values satisfies RegExp[]).lastIndexOf(/a/) !== -1",
            r"(values satisfies RegExp[]).includes(/a/)",
        ),
        (
            r"function f(NaN: number) { return array.indexOf(NaN) !== -1; }",
            r"function f(NaN: number) { return array.includes(NaN); }",
        ),
    ];

    Tester::new(PreferIncludes::NAME, PreferIncludes::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
