use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn use_consistent_curly_braces_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unnecessary curly braces around JSX string expression.")
        .with_help(
            "Remove the curly braces around this string literal. Use `text` instead of `{'text'}`.",
        )
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct UseConsistentCurlyBraces;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces consistent use of curly braces in JSX expressions.
    /// Specifically, flags unnecessary curly braces around string literals.
    ///
    /// ### Why is this bad?
    ///
    /// `{'text'}` is equivalent to `text` in JSX but adds visual noise.
    /// Consistent usage improves readability.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <div>{'Hello'}</div>
    /// <Component title={'World'} />
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <div>Hello</div>
    /// <Component title="World" />
    /// <div>{variable}</div>
    /// ```
    UseConsistentCurlyBraces,
    unicorn,
    style,
    pending
);

impl Rule for UseConsistentCurlyBraces {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXExpressionContainer(container) = node.kind() else {
            return;
        };

        let Some(expr) = container.expression.as_expression() else {
            return;
        };

        // Flag string literals in JSX expression containers
        if matches!(expr, Expression::StringLiteral(_)) {
            ctx.diagnostic(use_consistent_curly_braces_diagnostic(container.span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "<div>Hello</div>",
        "<div>{variable}</div>",
        "<div>{1 + 2}</div>",
        r#"<Component title="World" />"#,
    ];

    let fail = vec!["<div>{'Hello'}</div>"];

    Tester::new(UseConsistentCurlyBraces::NAME, UseConsistentCurlyBraces::PLUGIN, pass, fail)
        .test_and_snapshot();
}
