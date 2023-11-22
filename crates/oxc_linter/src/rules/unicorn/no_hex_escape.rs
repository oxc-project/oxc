use oxc_ast::{
    ast::{StringLiteral, TemplateLiteral},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode, Fix};

#[derive(Debug, Error, Diagnostic)]
#[error(
    "eslint-plugin-unicorn(no-hex-escape): Use Unicode escapes instead of hexadecimal escapes."
)]
#[diagnostic(severity(warning))]
struct NoHexEscapeDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoHexEscape;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces a convention of using [Unicode escapes](https://mathiasbynens.be/notes/javascript-escapes#unicode) instead of [hexadecimal escapes](https://mathiasbynens.be/notes/javascript-escapes#hexadecimal) for consistency and clarity.
    ///
    /// ### Example
    /// ```javascript
    /// // fail
    /// const foo = '\x1B';
    /// const foo = `\x1B${bar}`;
    ///
    /// // pass
    /// const foo = '\u001B';
    /// const foo = `\u001B${bar}`;
    /// ```
    NoHexEscape,
    pedantic
);

// \x -> \u00
fn check_escape(value: &str) -> Option<String> {
    let mut in_escape = false;
    let mut matched = Vec::new();
    for (index, c) in value.chars().enumerate() {
        if c == '\\' && !in_escape {
            in_escape = true;
        } else if c == 'x' && in_escape {
            matched.push(index);
            in_escape = false;
        } else {
            in_escape = false;
        }
    }
    if matched.is_empty() {
        None
    } else {
        let mut fixed = String::with_capacity(value.len() + matched.len() * 2);
        let mut last = 0;
        for index in matched {
            fixed.push_str(&value[last..index - 1]);
            fixed.push_str("\\u00");
            last = index + 1;
        }
        fixed.push_str(&value[last..]);
        Some(fixed)
    }
}
impl Rule for NoHexEscape {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::StringLiteral(StringLiteral { span, .. }) => {
                let text = span.source_text(ctx.source_text());
                if let Some(fixed) = check_escape(&text[1..text.len() - 1]) {
                    ctx.diagnostic_with_fix(NoHexEscapeDiagnostic(*span), || {
                        Fix::new(format!("'{fixed}'"), *span)
                    });
                }
            }
            AstKind::TemplateLiteral(TemplateLiteral { quasis, .. }) => {
                quasis.iter().for_each(|quasi| {
                    if let Some(fixed) = check_escape(quasi.span.source_text(ctx.source_text())) {
                        ctx.diagnostic_with_fix(NoHexEscapeDiagnostic(quasi.span), || {
                            Fix::new(fixed, quasi.span)
                        });
                    }
                });
            }
            AstKind::RegExpLiteral(regex) => {
                let text = regex.span.source_text(ctx.source_text());
                if let Some(fixed) = check_escape(&text[1..text.len() - 1]) {
                    ctx.diagnostic_with_fix(NoHexEscapeDiagnostic(regex.span), || {
                        Fix::new(format!("/{fixed}/"), regex.span)
                    });
                }
            }
            _ => {}
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"const foo = 'foo'",
        r"const foo = '\u00b1'",
        r"const foo = '\u00b1\u00b1'",
        r"const foo = 'foo\u00b1'",
        r"const foo = 'foo\u00b1foo'",
        r"const foo = '\u00b1foo'",
        r"const foo = '\\xb1'",
        r"const foo = '\\\\xb1'",
        r"const foo = 'foo\\xb1'",
        r"const foo = 'foo\\\\xb1'",
        r"const foo = '\\xd8\\x3d\\xdc\\xa9'",
        r"const foo = 'foo\\x12foo\\x34'",
        r"const foo = '\\\\xd8\\\\x3d\\\\xdc\\\\xa9'",
        r"const foo = 'foo\\\\x12foo\\\\x34'",
        r"const foo = 42",
        r"const foo = `foo`",
        r"const foo = `\u00b1`",
        r"const foo = `\u00b1\u00b1`",
        r"const foo = `foo\u00b1`",
        r"const foo = `foo\u00b1foo`",
        r"const foo = `\u00b1foo`",
        r"const foo = `42`",
        r"const foo = `\\xb1`",
        r"const foo = `\\\\xb1`",
        r"const foo = `foo\\xb1`",
        r"const foo = `foo\\\\xb1`",
        r"const foo = `\\xd8\\x3d\\xdc\\xa9`",
        r"const foo = `foo\\x12foo\\x34`",
        r"const foo = `\\\\xd8\\\\x3d\\\\xdc\\\\xa9`",
        r"const foo = `foo\\\\x12foo\\\\x34`",
    ];
    let fail = vec![r#"const foo = "\xb1""#];
    let fix = vec![
        (r"const foo = '\xb1'", r"const foo = '\u00b1'", None),
        (r"const foo = '\\\xb1'", r"const foo = '\\\u00b1'", None),
        (r"const foo = '\xb1\xb1'", r"const foo = '\u00b1\u00b1'", None),
        (r"const foo = '\\\xb1\\\xb1'", r"const foo = '\\\u00b1\\\u00b1'", None),
        (r"const foo = '\\\xb1\\\\xb1'", r"const foo = '\\\u00b1\\\\xb1'", None),
        (r"const foo = '\\\\\xb1\\\xb1'", r"const foo = '\\\\\u00b1\\\u00b1'", None),
        (r"const foo = '\xb1foo'", r"const foo = '\u00b1foo'", None),
        (r"const foo = '\xd8\x3d\xdc\xa9'", r"const foo = '\u00d8\u003d\u00dc\u00a9'", None),
        (r"const foo = 'foo\xb1'", r"const foo = 'foo\u00b1'", None),
        (r"const foo = 'foo\\\xb1'", r"const foo = 'foo\\\u00b1'", None),
        (r"const foo = 'foo\\\\\xb1'", r"const foo = 'foo\\\\\u00b1'", None),
        (r"const foo = 'foo\x12foo\x34'", r"const foo = 'foo\u0012foo\u0034'", None),
        (r"const foo = '42\x1242\x34'", r"const foo = '42\u001242\u0034'", None),
        (r"const foo = '42\\\x1242\\\x34'", r"const foo = '42\\\u001242\\\u0034'", None),
        (r"const foo = /^[\x20-\x7E]*$/", r"const foo = /^[\u0020-\u007E]*$/", None),
        (r"const foo = `\xb1`", r"const foo = `\u00b1`", None),
        (r"const foo = `\\\xb1`", r"const foo = `\\\u00b1`", None),
        (r"const foo = `\xb1\xb1`", r"const foo = `\u00b1\u00b1`", None),
        (r"const foo = `\\\xb1\\\xb1`", r"const foo = `\\\u00b1\\\u00b1`", None),
        (r"const foo = `\\\\\xb1\\\xb1`", r"const foo = `\\\\\u00b1\\\u00b1`", None),
        (r"const foo = `\\\\\xb1\\\\xb1`", r"const foo = `\\\\\u00b1\\\\xb1`", None),
        (r"const foo = `\xb1foo`", r"const foo = `\u00b1foo`", None),
        (r"const foo = `\xd8\x3d\xdc\xa9`", r"const foo = `\u00d8\u003d\u00dc\u00a9`", None),
        (r"const foo = `foo\xb1`", r"const foo = `foo\u00b1`", None),
        (r"const foo = `foo\\\xb1`", r"const foo = `foo\\\u00b1`", None),
        (r"const foo = `foo\\\\\xb1`", r"const foo = `foo\\\\\u00b1`", None),
        (r"const foo = `foo\x12foo\x34`", r"const foo = `foo\u0012foo\u0034`", None),
        (r"const foo = `42\x1242\x34`", r"const foo = `42\u001242\u0034`", None),
        (r"const foo = `42\\\x1242\\\x34`", r"const foo = `42\\\u001242\\\u0034`", None),
        (r"const foo = `\xb1${foo}\xb1${foo}`", r"const foo = `\u00b1${foo}\u00b1${foo}`", None),
    ];

    Tester::new_without_config(NoHexEscape::NAME, pass, fail).expect_fix(fix).test_and_snapshot();
}

#[test]
fn test_check_escape() {
    let result = check_escape(r"\x1B").unwrap();
    assert_eq!(result, r"\u001B");
    let result = check_escape(r"a\x1B").unwrap();
    assert_eq!(result, r"a\u001B");
    assert!(check_escape(r"\\x1B").is_none());
    let result = check_escape(r"\\\x1B").unwrap();
    assert_eq!(result, r"\\\u001B");
    let result = check_escape(r"\\\a\x1B").unwrap();
    assert_eq!(result, r"\\\a\u001B");
    assert!(check_escape(r"\\xb1").is_none());
}
