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
    suggestion,
    version = "0.0.6",
    short_description = "Disallow non-null assertions after an optional chain expression.",
);

impl Rule for NoNonNullAssertedOptionalChain {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::TSNonNullExpression(non_null_expr) = node.kind() else {
            return;
        };

        let expression = non_null_expr.expression.get_inner_expression();
        let chain_span = match expression {
            Expression::ChainExpression(chain) => find_optional_chain_span(&chain.expression),
            Expression::CallExpression(_)
            | Expression::ComputedMemberExpression(_)
            | Expression::StaticMemberExpression(_)
            | Expression::PrivateFieldExpression(_)
                if !is_parent_member_or_call(node, ctx) =>
            {
                find_optional_chain_span_in_expression(expression)
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

fn find_optional_chain_span(chain: &ChainElement<'_>) -> Option<Span> {
    let expression = match chain {
        ChainElement::CallExpression(call) => {
            if call.optional {
                return Some(call.callee.span());
            }
            &call.callee
        }
        ChainElement::TSNonNullExpression(non_null) => &non_null.expression,
        ChainElement::ComputedMemberExpression(member) => {
            if member.optional {
                return Some(member.object.span());
            }
            &member.object
        }
        ChainElement::StaticMemberExpression(member) => {
            if member.optional {
                return Some(member.object.span());
            }
            &member.object
        }
        ChainElement::PrivateFieldExpression(member) => {
            if member.optional {
                return Some(member.object.span());
            }
            &member.object
        }
    };

    find_optional_chain_span_in_expression(expression)
}

fn find_optional_chain_span_in_expression(mut expression: &Expression<'_>) -> Option<Span> {
    loop {
        if matches!(expression, Expression::ParenthesizedExpression(_)) {
            return None;
        }

        match expression.get_inner_expression() {
            Expression::ChainExpression(chain) => {
                return find_optional_chain_span(&chain.expression);
            }
            Expression::CallExpression(call) => {
                if call.optional {
                    return Some(call.callee.span());
                }
                expression = &call.callee;
            }
            expr @ match_member_expression!(Expression) => {
                let member = expr.to_member_expression();
                if member.optional() {
                    return Some(member.object().span());
                }
                expression = member.object();
            }
            _ => return None,
        }
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
        "foo?.bar().baz()!;",
        "foo?.bar.baz!;",
        "foo.bar?.()!;",
        "foo.bar?.().baz!;",
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
        ("foo?.bar().baz()!", "foo?.bar().baz()"),
        ("foo?.bar.baz!", "foo?.bar.baz"),
        ("foo.bar?.().baz!", "foo.bar?.().baz"),
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
