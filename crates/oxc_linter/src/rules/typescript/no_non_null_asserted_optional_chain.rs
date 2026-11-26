use oxc_ast::{AstKind, ast::Expression};
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
        .with_label(chain_span.primary_label("optional chain used"))
        .and_label(assertion_span.label("non-null assertion made after optional chain"))
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
    recommended,
    suggestion,
    version = "0.0.6",
    short_description = "Disallow non-null assertions after an optional chain expression.",
);

impl Rule for NoNonNullAssertedOptionalChain {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::TSNonNullExpression(non_null_expr) = node.kind() else {
            return;
        };

        let chain_span = match non_null_expr.expression.without_parentheses() {
            Expression::ChainExpression(chain) => chain.span,
            _ if matches!(ctx.nodes().parent_kind(node.id()), AstKind::ChainExpression(_)) => {
                non_null_expr.expression.span()
            }
            _ => return,
        };

        let assertion_span = Span::sized(non_null_expr.span.end - 1, 1);
        ctx.diagnostic_with_suggestion(
            no_non_null_asserted_optional_chain_diagnostic(chain_span, assertion_span),
            |fixer| fixer.delete_range(assertion_span),
        );
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_typescript()
    }
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
        "(foo?.bar as string)!;",
        "foo?.bar!!.baz;",
        "// eslint-disable-next-line typescript/no-non-null-asserted-optional-chain\nfoo\n  ?.bar!;",
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
        "foo?.bar!!;",
        "foo\u{a0}?.bar!;",
        "foo\n  ?.bar!;",
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
        ("foo?.bar!!", "foo?.bar!"),
        ("foo\u{a0}?.bar!", "foo\u{a0}?.bar"),
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
