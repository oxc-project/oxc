use oxc_ast::{
    ast::{match_member_expression, ChainElement, Expression},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    AstNode,
};

fn no_non_null_asserted_optional_chain_diagnostic(span: Span, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("non-null assertions after an optional chain expression")
        .with_help("Optional chain expressions can return undefined by design - using a non-null assertion is unsafe and wrong. You should remove the non-null assertion.")
        .with_labels([span, span1])
}

#[derive(Debug, Default, Clone)]
pub struct NoNonNullAssertedOptionalChain;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow non-null assertions after an optional chain expression.
    ///
    /// ### Why is this bad?
    /// `?.` optional chain expressions provide undefined if an object is null or undefined.
    /// Using a `!` non-null assertion to assert the result of an `?.` optional chain expression is non-nullable is likely wrong.
    ///
    /// Most of the time, either the object was not nullable and did not need the `?.` for its property lookup, or the `!` is incorrect and introducing a type safety hole.
    ///
    /// ### Example
    /// ```ts
    /// foo?.bar!;
    /// foo?.bar()!;
    /// ```
    NoNonNullAssertedOptionalChain,
    typescript,
    correctness
);

impl Rule for NoNonNullAssertedOptionalChain {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::TSNonNullExpression(non_null_expr) = node.kind() else {
            return;
        };

        let chain_span = match non_null_expr.expression.get_inner_expression() {
            Expression::ChainExpression(chain) => match &chain.expression {
                ChainElement::ComputedMemberExpression(member) if member.optional => {
                    Some(member.object.span())
                }
                ChainElement::StaticMemberExpression(member) if member.optional => {
                    Some(member.object.span())
                }
                ChainElement::PrivateFieldExpression(member) if member.optional => {
                    Some(member.object.span())
                }
                ChainElement::CallExpression(call) if call.optional => Some(call.callee.span()),
                _ => None,
            },
            Expression::CallExpression(call) => {
                if call.optional && !is_parent_member_or_call(node, ctx) {
                    Some(call.callee.span())
                } else if let Some(member) = call.callee.as_member_expression() {
                    if member.optional() && !is_parent_member_or_call(node, ctx) {
                        Some(member.object().span())
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            expr @ match_member_expression!(Expression) => {
                let member_expr = expr.to_member_expression();
                if member_expr.optional() && !is_parent_member_or_call(node, ctx) {
                    Some(member_expr.object().span())
                } else {
                    None
                }
            }
            _ => None,
        };

        if let Some(chain_span) = chain_span {
            let chain_span_end = chain_span.end;
            let non_null_end = non_null_expr.span.end - 1;
            ctx.diagnostic(no_non_null_asserted_optional_chain_diagnostic(
                Span::new(chain_span_end, chain_span_end),
                Span::new(non_null_end, non_null_end),
            ));
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_typescript()
    }
}

fn is_parent_member_or_call(node: &AstNode<'_>, ctx: &LintContext<'_>) -> bool {
    matches!(
        ctx.nodes().parent_kind(node.id()),
        Some(AstKind::MemberExpression(_) | AstKind::CallExpression(_))
    )
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "foo.bar!;",
        "foo.bar!.baz;",
        "foo.bar!.baz();",
        "foo.bar()!;",
        "foo.bar()!();",
        "foo.bar()!.baz;",
        "foo?.bar;",
        "foo?.bar();",
        "(foo?.bar).baz!;",
        "(foo?.bar()).baz!;",
        "foo?.bar!.baz;",
        "foo?.bar!();",
        "foo?.['bar']!.baz;",
        "foo?.get()!.bar()",
    ];

    let fail = vec![
        "foo?.bar!;",
        "foo?.['bar']!;",
        "foo?.bar()!;",
        "foo.bar?.()!;",
        "(foo?.bar)!.baz",
        "(foo?.bar)!().baz",
        "(foo?.bar)!",
        "(foo?.bar)!()",
        "(foo?.bar!)",
        "(foo?.bar!)()",
    ];

    Tester::new(
        NoNonNullAssertedOptionalChain::NAME,
        NoNonNullAssertedOptionalChain::PLUGIN,
        pass,
        fail,
    )
    .test_and_snapshot();
}
