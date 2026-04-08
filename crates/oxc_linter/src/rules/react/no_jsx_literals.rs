use oxc_ast::AstKind;
use oxc_ast::ast::JSXChild;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_jsx_literals_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("String literals are not allowed in JSX.")
        .with_help("Wrap the string in a JSX expression: `{'text'}` or use a variable/constant.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoJsxLiterals;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows string literals in JSX, requiring them to be wrapped in
    /// JSX expressions or extracted into variables.
    ///
    /// ### Why is this bad?
    ///
    /// String literals in JSX can make internationalization (i18n) difficult.
    /// Requiring all strings to be wrapped or extracted encourages the use
    /// of translation functions.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <div>Hello World</div>
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <div>{t('hello_world')}</div>
    /// <div>{'Hello World'}</div>
    /// ```
    NoJsxLiterals,
    react,
    restriction,
    pending
);

impl Rule for NoJsxLiterals {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXElement(jsx) = node.kind() else {
            return;
        };

        for child in &jsx.children {
            if let JSXChild::Text(text) = child {
                let trimmed = text.value.as_str().trim();
                // Only flag non-whitespace-only text
                if !trimmed.is_empty() {
                    ctx.diagnostic(no_jsx_literals_diagnostic(text.span));
                }
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "<div>{t('hello')}</div>",
        "<div>{'Hello World'}</div>",
        "<div>{variable}</div>",
        "<div> </div>",
    ];

    let fail = vec!["<div>Hello World</div>", "<span>Some text</span>"];

    Tester::new(NoJsxLiterals::NAME, NoJsxLiterals::PLUGIN, pass, fail).test_and_snapshot();
}
