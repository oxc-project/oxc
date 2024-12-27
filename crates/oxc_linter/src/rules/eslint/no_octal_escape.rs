use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use regex::Regex;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_octal_escape_diagnostic(span: Span, sequence: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Don't use octal: '{}'. Use '\\u....' instead.", sequence))
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
    correctness,
);

impl Rule for NoOctalEscape {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::StringLiteral(literal) = node.kind() {
            let octal_escape_pattern =
            // /^(?:[^\\]|\\.)*?\\([0-3][0-7]{1,2}|[4-7][0-7]|0(?=[89])|[1-7])/su
                Regex::new(r"^(?:[^\\]|\\.)*?\\([0-3][0-7]{1,2}|[4-7][0-7]|0[0-7]|[1-7])")
                    .unwrap();

            if let Some(captures) = octal_escape_pattern.captures(&literal.value) {
                if let Some(sequence) = captures.get(1) {
                    let escape_seq = sequence.as_str();
                    if escape_seq.starts_with('0')
                        && escape_seq.len() == 2
                        && &escape_seq[1..] >= "8"
                    {
                        ctx.diagnostic(no_octal_escape_diagnostic(literal.span, sequence.as_str()));
                    }
                }
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "var foo = \"\x51\";",
        "let foo = \"foo \\251 bar\"",
        "var foo = /([abc]) \\1/g;",
        "var foo = '\0';",
        "'\\0'",
        "'\\8'",
        "'\\9'",
        "'\\0 '",
        "' \\0'",
        "'a\\0'",
        "'\\0a'",
        "'a\\8a'",
        "'\\0\\8'",
        "'\\8\\0'",
        "'\\80'",
        "'\\81'",
        "'\\\\'",
        "'\\\\0'",
        "'\\\\08'",
        "'\\\\1'",
        "'\\\\01'",
        "'\\\\12'",
        "'\\\\\\0'",
        "'\\\\\\8'",
        "'\\0\\\\'",
        "'0'",
        "'1'",
        "'8'",
        "'01'",
        "'08'",
        "'80'",
        "'12'",
        "'\\a'",
        "'\\n'",
    ];

    let fail = vec![
        "var foo = \"foo \\01 bar\";",
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
        "'\0\\1'",
        "'\0 \\1'",
        "'\0\01'",
        "'\0 \01'",
        "'\0a\\1'",
        "'\0a\01'",
        "'\0\08'",
        "'\\1'",
        "'\\2'",
        "'\\7'",
        "'\00'",
        "'\01'",
        "'\02'",
        "'\07'",
        "'\08'",
        "'\09'",
        "'\\10'",
        "'\\12'",
        "' \\1'",
        "'\\1 '",
        "'a\\1'",
        "'\\1a'",
        "'a\\1a'",
        "' \01'",
        "'\01 '",
        "'a\01'",
        "'\01a'",
        "'a\01a'",
        "'a\08a'",
        "'\n\\1'",
        "'\n\01'",
        "'\n\08'",
        "'\\\\1'",
        "'\\\01'",
        "'\\\08'",
        "'\\
			\\1'",
        "'\01\02'",
        "'\02\01'",
        "'\01\\2'",
        "'\\2\01'",
        "'\08\\1'",
        "'foo \\1 bar \\2'",
    ];

    Tester::new(NoOctalEscape::NAME, NoOctalEscape::CATEGORY, pass, fail).test_and_snapshot();
}
