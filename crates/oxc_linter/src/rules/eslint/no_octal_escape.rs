use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_octal_escape_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Don't use octal escape sequences.")
        .with_help("Use Unicode or hexadecimal escape sequences instead of octal escape sequences.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoOctalEscape;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows octal escape sequences in string literals.
    ///
    /// ### Why is this bad?
    ///
    /// As of the ECMAScript 5 specification, octal escape sequences in string
    /// literals are deprecated and should not be used. Unicode escape sequences
    /// or hexadecimal escape sequences should be used instead.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// var foo = "Copyright \251";
    /// var bar = "\1";
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// var foo = "Copyright \u00A9";
    /// var bar = "\x01";
    /// ```
    NoOctalEscape,
    eslint,
    correctness
);

impl Rule for NoOctalEscape {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::StringLiteral(literal) = node.kind() else {
            return;
        };

        let raw = ctx.source_range(literal.span);
        if raw.len() < 2 {
            return;
        }
        // Strip quotes
        let inner = &raw[1..raw.len() - 1];

        let bytes = inner.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'\\' && i + 1 < bytes.len() {
                let next = bytes[i + 1];
                if next >= b'0' && next <= b'7' {
                    // This is an octal escape — but \0 not followed by a digit is allowed (null char)
                    if next == b'0'
                        && (i + 2 >= bytes.len() || bytes[i + 2] < b'0' || bytes[i + 2] > b'7')
                    {
                        i += 2;
                        continue;
                    }
                    let start = literal.span.start + 1 + i as u32;
                    // Find the end of the octal sequence (up to 3 digits)
                    let mut end = i + 2;
                    while end < bytes.len()
                        && end < i + 4
                        && bytes[end] >= b'0'
                        && bytes[end] <= b'7'
                    {
                        end += 1;
                    }
                    let span_end = literal.span.start + 1 + end as u32;
                    ctx.diagnostic(no_octal_escape_diagnostic(Span::new(start, span_end)));
                    return;
                }
                i += 2;
            } else {
                i += 1;
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"var foo = "\x51";"#,
        r#"var foo = "hello \u0051";"#,
        r#"var foo = "\0";"#,
        r"var foo = '\0';",
        r#"var foo = "hello";"#,
        r"var foo = '\n';",
        r"var foo = '\\1';",
    ];

    let fail = vec![
        r"var foo = '\1';",
        r"var foo = '\2';",
        r"var foo = '\7';",
        r#"var foo = "\00";"#,
        r"var foo = '\251';",
        r"var foo = '\377';",
        r"var foo = '\012';",
        r#"var foo = "Copyright \251";"#,
        r#"var foo = "\1";"#,
    ];

    Tester::new(NoOctalEscape::NAME, NoOctalEscape::PLUGIN, pass, fail).test_and_snapshot();
}
