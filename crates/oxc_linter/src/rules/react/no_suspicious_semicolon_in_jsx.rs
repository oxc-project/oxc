use oxc_ast::AstKind;
use oxc_ast::ast::JSXChild;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_suspicious_semicolon_in_jsx_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Suspicious semicolon as JSX text content.")
        .with_help("This semicolon appears as visible text in the rendered output. If you meant to end a statement, move it outside the JSX expression.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoSuspiciousSemicolonInJsx;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Detects semicolons that appear as text content inside JSX elements.
    ///
    /// ### Why is this bad?
    ///
    /// A semicolon inside JSX is rendered as visible text, not as a statement
    /// terminator. This is almost always a mistake where the developer
    /// accidentally placed a semicolon inside a JSX expression.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// const Component = () => (
    ///   <div>;
    ///     content
    ///   </div>
    /// );
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// const Component = () => (
    ///   <div>
    ///     content
    ///   </div>
    /// );
    /// ```
    NoSuspiciousSemicolonInJsx,
    react,
    suspicious,
    pending
);

impl Rule for NoSuspiciousSemicolonInJsx {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXElement(jsx) = node.kind() else {
            return;
        };

        for child in &jsx.children {
            if let JSXChild::Text(text) = child {
                let value = text.value.as_str().trim();
                if value == ";" {
                    ctx.diagnostic(no_suspicious_semicolon_in_jsx_diagnostic(text.span));
                }
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec!["<div>content</div>;", "<div>content</div>", "<div> </div>"];

    let fail = vec!["<div>;</div>"];

    Tester::new(NoSuspiciousSemicolonInJsx::NAME, NoSuspiciousSemicolonInJsx::PLUGIN, pass, fail)
        .test_and_snapshot();
}
