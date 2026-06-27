use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_script_url_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected `javascript:` url")
        .with_help("Execute the code directly instead.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoScriptUrl;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow `javascript:` URLs.
    ///
    /// ### Why is this bad?
    ///
    /// Using `javascript:` URLs is considered by some as a form of `eval`. Code
    /// passed in `javascript:` URLs must be parsed and evaluated by the browser
    /// in the same way that `eval` is processed. This can lead to security and
    /// performance issues.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// location.href = "javascript:void(0)";
    ///
    /// location.href = `javascript:void(0)`;
    /// ```
    NoScriptUrl,
    eslint,
    style,
    version = "0.2.15",
    short_description = "Disallow `javascript:` URLs.",
);

impl Rule for NoScriptUrl {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::StringLiteral(literal) if is_javascript_url(&literal.value) => {
                ctx.diagnostic(no_script_url_diagnostic(literal.span));
            }
            AstKind::TemplateLiteral(literal)
                if !is_tagged_template_expression(ctx, node, literal.span)
                    && literal.quasis.len() == 1
                    && is_javascript_url(&literal.quasis.first().unwrap().value.raw) =>
            {
                ctx.diagnostic(no_script_url_diagnostic(literal.span));
            }
            _ => {}
        }
    }
}

/// Whether `value` begins with a case-insensitive `javascript:` prefix.
///
/// Equivalent to `value.cow_to_ascii_lowercase().starts_with("javascript:")` but
/// allocation-free: `"javascript:"` is ASCII, so ASCII-lowercasing cannot change byte
/// length and only the first 11 bytes can affect the match. This runs on every string and
/// template literal, so it avoids both the full-length lowercase scan and the heap copy the
/// previous `cow_to_ascii_lowercase` made whenever the value contained an uppercase letter.
fn is_javascript_url(value: &str) -> bool {
    const PREFIX: &[u8] = b"javascript:";
    value.len() >= PREFIX.len() && value.as_bytes()[..PREFIX.len()].eq_ignore_ascii_case(PREFIX)
}

fn is_tagged_template_expression(ctx: &LintContext, node: &AstNode, literal_span: Span) -> bool {
    matches!(
        ctx.nodes().parent_kind(node.id()),
        AstKind::TaggedTemplateExpression(expr) if expr.quasi.span == literal_span
    )
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "var a = 'Hello World!';",
        "var a = 10;",
        "var url = 'xjavascript:'",
        "var url = `xjavascript:`",
        "var url = `${foo}javascript:`",
        "var a = foo`javaScript:`;",
        // Shorter than `javascript:` (11 bytes): exercises the length guard.
        "var a = 'js:';",
        "var url = `js:`",
        // Multi-byte chars in the first 11 bytes: byte-slice prefix compare must
        // not panic and must not match.
        "var a = 'über cool stuff';",
    ];

    let fail = vec![
        "var a = 'javascript:void(0);';",
        "var a = 'javascript:';",
        "var a = `javascript:`;",
        "var a = `JavaScript:`;",
    ];

    Tester::new(NoScriptUrl::NAME, NoScriptUrl::PLUGIN, pass, fail).test_and_snapshot();
}
