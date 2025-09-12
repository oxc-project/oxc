use oxc_ast::{
    AstKind,
    ast::{ChainElement, Expression, match_member_expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    rule::Rule,
};

fn no_non_null_asserted_optional_chain_diagnostic(
    chain_span: Span,
    assertion_span: Span,
) -> OxcDiagnostic {
    OxcDiagnostic::warn("Optional chain expressions can return undefined by design: using a non-null assertion is unsafe and wrong.")
        .with_help("Remove the non-null assertion.")
        .with_label(assertion_span.primary_label("non-null assertion made after optional chain"))
        .and_label(chain_span.label("optional chain used"))
}

#[derive(Debug, Default, Clone)]
pub struct NoNonNullAssertedOptionalChain;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow non-null assertions after an optional chain expression.
    ///
    /// ### Why is this bad?
    ///
    /// By design, optional chain expressions (`?.`) provide `undefined` as the expression's value, if the object being
    /// accessed is `null` or `undefined`, instead of throwing an error. Using a non-null assertion (`!`) to assert the
    /// result of an optional chain expression is contradictory and likely wrong, as it indicates the code is both expecting
    /// the value to be potentially `null` or `undefined` and non-null at the same time.
    ///
    /// In most cases, either:
    /// 1. The object is not nullable and did not need the `?.` for its property lookup
    /// 2. The non-null assertion is incorrect and introduces a type safety hole.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// foo?.bar!;
    /// foo?.bar()!;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// foo?.bar;
    /// foo.bar!;
    /// ```
    NoNonNullAssertedOptionalChain,
    typescript,
    correctness,
    suggestion
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
            let diagnostic = no_non_null_asserted_optional_chain_diagnostic(
                Span::sized(chain_span_end, 1),
                Span::sized(non_null_end, 1),
            );
            ctx.diagnostic_with_suggestion(diagnostic, |fixer| {
                fixer.delete_range(Span::sized(non_null_end, 1))
            });
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_typescript()
    }
}

fn is_parent_member_or_call(node: &AstNode<'_>, ctx: &LintContext<'_>) -> bool {
    let parent_kind = ctx.nodes().parent_kind(node.id());
    matches!(parent_kind, AstKind::CallExpression(_)) || parent_kind.is_member_expression_kind()
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

    let fix = vec![
        ("foo?.bar!", "foo?.bar"),
        ("foo?.['bar']!", "foo?.['bar']"),
        ("foo?.bar()!", "foo?.bar()"),
        ("(foo?.bar)!.baz", "(foo?.bar).baz"),
        ("(foo?.bar)!().baz", "(foo?.bar)().baz"),
        ("(foo?.bar)!", "(foo?.bar)"),
        ("(foo?.bar)!()", "(foo?.bar)()"),
        ("(foo?.bar!)", "(foo?.bar)"),
        ("(foo?.bar!)()", "(foo?.bar)()"),
    ];

    Tester::new(
        NoNonNullAssertedOptionalChain::NAME,
        NoNonNullAssertedOptionalChain::PLUGIN,
        pass,
        fail,
    )
    .expect_fix(fix)
    .test_and_snapshot();
}
