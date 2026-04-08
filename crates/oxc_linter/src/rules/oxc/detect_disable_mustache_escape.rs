use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn detect_disable_mustache_escape_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Mustache escaping is being disabled")
        .with_help("Setting escapeMarkup to false or escape to false disables HTML escaping and can lead to XSS vulnerabilities.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct DetectDisableMustacheEscape;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Detects when HTML escaping is disabled in template engines by setting
    /// `escapeMarkup: false` or `escape: false`.
    ///
    /// ### Why is this bad?
    ///
    /// Disabling HTML escaping in template engines allows raw HTML to be injected
    /// into the output, which can lead to Cross-Site Scripting (XSS) vulnerabilities.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// const options = { escapeMarkup: false };
    /// const opts = { escape: false };
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const options = { escapeMarkup: true };
    /// const options = { escape: myEscapeFunction };
    /// ```
    DetectDisableMustacheEscape,
    oxc,
    suspicious,
    none
);

impl Rule for DetectDisableMustacheEscape {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ObjectProperty(prop) = node.kind() else {
            return;
        };

        let key_name = match &prop.key {
            oxc_ast::ast::PropertyKey::StaticIdentifier(ident) => &ident.name,
            _ => return,
        };

        if key_name != "escapeMarkup" && key_name != "escape" {
            return;
        }

        if let Expression::BooleanLiteral(lit) = &prop.value
            && !lit.value
        {
            ctx.diagnostic(detect_disable_mustache_escape_diagnostic(prop.span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "const options = { escapeMarkup: true }",
        "const options = { escape: true }",
        "const options = { escapeMarkup: myFunc }",
        "const options = { unrelated: false }",
    ];

    let fail = vec!["const options = { escapeMarkup: false }", "const options = { escape: false }"];

    Tester::new(DetectDisableMustacheEscape::NAME, DetectDisableMustacheEscape::PLUGIN, pass, fail)
        .test_and_snapshot();
}
