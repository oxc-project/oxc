use crate::{context::LintContext, rule::Rule, AstNode};
use lazy_static::lazy_static;
use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use regex::Regex;

fn no_octal_escape_diagnostic(span: Span, sequence: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Don't use octal: '\\{sequence}'. Use '\\u....' instead."))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoOctalEscape;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows the use of octal escape sequences in string literals.
    ///
    /// ### Why is this bad?
    ///
    /// Octal escape sequences are deprecated and can lead to unexpected behavior. Modern code should use Unicode or hexadecimal escape sequences instead.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// const str = "\251";
    /// const str = "\1";
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const str = "\u00A9";
    /// const str = "\xA9";
    /// ```
    NoOctalEscape,
    eslint,
    correctness,
    pending
);

impl Rule for NoOctalEscape {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::StringLiteral(literal) = node.kind() {
            if let Some(raw) = &literal.raw {
                if let Some(captures) = OCTAL_ESCAPE_PATTERN.captures(raw) {
                    if let Some(sequence) = captures.get(1) {
                        ctx.diagnostic(no_octal_escape_diagnostic(literal.span, sequence.as_str()));
                    }
                }
            }
        }
    }
}

lazy_static! {
    static ref OCTAL_ESCAPE_PATTERN: Regex =
        Regex::new(r"(?s)^(?:[^\\]|\\.)*?\\([0-3][0-7]{1,2}|[4-7][0-7]|(08|09)|[1-7])").unwrap();
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"var foo = "\x51";"#,
        r#"var foo = "foo \\251 bar";"#,
        r"var foo = /([abc]) \1/g;",
        r"var foo = '\0';",
        r"'\0'",
        r"'\8'",
        r"'\9'",
        r"'\0 '",
        r"' \0'",
        r"'a\0'",
        r"'\0a'",
        r"'a\8a'",
        r"'\0\8'",
        r"'\8\0'",
        r"'\80'",
        r"'\81'",
        r"'\\'",
        r"'\\0'",
        r"'\\08'",
        r"'\\1'",
        r"'\\01'",
        r"'\\12'",
        r"'\\\0'",
        r"'\\\8'",
        r"'\0\\'",
        "'0'",
        "'1'",
        "'8'",
        "'01'",
        "'08'",
        "'80'",
        "'12'",
        r"'\a'",
        r"'\n'",
    ];

    let fail = vec![
        r#"var foo = "foo \01 bar";"#,
        r#"var foo = "foo \000 bar";"#,
        r#"var foo = "foo \377 bar";"#,
        r#"var foo = "foo \378 bar";"#,
        r#"var foo = "foo \37a bar";"#,
        r#"var foo = "foo \381 bar";"#,
        r#"var foo = "foo \3a1 bar";"#,
        r#"var foo = "foo \251 bar";"#,
        r#"var foo = "foo \258 bar";"#,
        r#"var foo = "foo \25a bar";"#,
        r#"var foo = "\3s51";"#,
        r#"var foo = "\77";"#,
        r#"var foo = "\78";"#,
        r#"var foo = "\5a";"#,
        r#"var foo = "\751";"#,
        r#"var foo = "foo \400 bar";"#,
        r#"var foo = "\t\1";"#,
        r#"var foo = "\\\751";"#,
        r"'\0\1'",
        r"'\0 \1'",
        // oxc itself throws the error below:
        // Octal escape sequences are not allowed. Use the syntax '\x01'.
        // r"\0\01'",
        r"'\0 \01'",
        r"'\0a\1'",
        r"'\0a\01'",
        r"'\0\08'",
        r"'\1'",
        r"'\2'",
        r"'\7'",
        r"'\00'",
        r"'\01'",
        r"'\02'",
        r"'\07'",
        r"'\08'",
        r"'\09'",
        r"'\10'",
        r"'\12'",
        r"' \1'",
        r"'\1 '",
        r"'a\1'",
        r"'\1a'",
        r"'a\1a'",
        r"' \01'",
        r"'\01 '",
        r"'a\01'",
        r"'\01a'",
        r"'a\01a'",
        r"'a\08a'",
        r"'\n\1'",
        r"'\n\01'",
        r"'\n\08'",
        r"'\\\1'",
        r"'\\\01'",
        r"'\\\08'",
        r"'\\n\1'",
        r"'\01\02'",
        r"'\02\01'",
        r"'\01\2'",
        r"'\2\01'",
        r"'\08\1'",
        r"'foo \1 bar \2'",
    ];

    Tester::new(NoOctalEscape::NAME, NoOctalEscape::PLUGIN, pass, fail).test_and_snapshot();
}
