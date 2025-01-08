use std::str::Chars;

use oxc_ast::{
    ast::{StringLiteral, TemplateLiteral},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn escape_case_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Use uppercase characters for the value of the escape sequence.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct EscapeCase;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces defining escape sequence values with uppercase characters rather than lowercase ones. This promotes readability by making the escaped value more distinguishable from the identifier.
    ///
    /// ### Why is this bad?
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// const foo = '\xa9';
    /// const foo = '\ud834';
    /// const foo = '\u{1d306}';
    /// const foo = '\ca';
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// const foo = '\xA9';
    /// const foo = '\uD834';
    /// const foo = '\u{1D306}';
    /// const foo = '\cA';
    /// ```
    EscapeCase,
    unicorn,
    pedantic,
    fix
);

fn is_hex_char(c: char) -> bool {
    c.is_ascii_hexdigit()
}
fn is_hex(iter: &Chars, count: i32) -> bool {
    let mut iter = iter.clone();
    for _ in 0..count {
        if !matches!(iter.next(), Some(c) if is_hex_char(c)) {
            return false;
        }
    }
    true
}

// /(?<=(?:^|[^\\])(?:\\\\)*\\)(?<data>x[\dA-Fa-f]{2}|u[\dA-Fa-f]{4}|u{[\dA-Fa-f]+})/g
fn check_case(value: &str, is_regex: bool) -> Option<String> {
    let mut in_escape = false;
    let mut result = String::with_capacity(value.len());
    let mut p = value.chars();
    while let Some(c) = p.next() {
        result.push(c);
        if in_escape {
            match c {
                'x' => {
                    if is_hex(&p, 2) {
                        for _ in 0..2 {
                            result.push(p.next().unwrap().to_ascii_uppercase());
                        }
                    }
                }
                'u' => {
                    let mut iter = p.clone();
                    let c = iter.next();
                    if c == Some('{') {
                        let mut is_match = false;
                        let mut chars = vec![];
                        for c in iter {
                            if c == '}' {
                                is_match = true;
                                break;
                            } else if is_hex_char(c) {
                                chars.push(c);
                            } else {
                                break;
                            }
                        }
                        if is_match {
                            p.next();
                            result.push('{');
                            for _ in 0..chars.len() {
                                result.push(p.next().unwrap().to_ascii_uppercase());
                            }
                            p.next();
                            result.push('}');
                        }
                    } else if is_hex(&p, 4) {
                        for _ in 0..4 {
                            result.push(p.next().unwrap().to_ascii_uppercase());
                        }
                    }
                }
                'c' if is_regex => {
                    if matches!(p.clone().next(), Some(c) if c.is_ascii_lowercase()) {
                        result.push(p.next().unwrap().to_ascii_uppercase());
                    }
                }
                _ => {}
            }
            in_escape = false;
        } else if c == '\\' {
            in_escape = true;
        }
    }

    if result == value {
        None
    } else {
        Some(result)
    }
}
impl Rule for EscapeCase {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::StringLiteral(StringLiteral { span, .. }) => {
                let text = span.source_text(ctx.source_text());
                if let Some(fixed) = check_case(text, false) {
                    ctx.diagnostic_with_fix(escape_case_diagnostic(*span), |fixer| {
                        fixer.replace(*span, fixed)
                    });
                }
            }
            AstKind::TemplateLiteral(TemplateLiteral { quasis, .. }) => {
                quasis.iter().for_each(|quasi| {
                    if let Some(fixed) =
                        check_case(quasi.span.source_text(ctx.source_text()), false)
                    {
                        ctx.diagnostic_with_fix(escape_case_diagnostic(quasi.span), |fixer| {
                            fixer.replace(quasi.span, fixed)
                        });
                    }
                });
            }
            AstKind::RegExpLiteral(regex) => {
                let text = regex.span.source_text(ctx.source_text());
                if let Some(fixed) = check_case(text, true) {
                    ctx.diagnostic_with_fix(escape_case_diagnostic(regex.span), |fixer| {
                        fixer.replace(regex.span, fixed)
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
        r#"const foo = "\xA9一二三\xA9";"#,
        r#"const foo = "\xA9";"#,
        r#"const foo = "\uD834";"#,
        r#"const foo = "\u{1D306}";"#,
        r#"const foo = "\uD834foo";"#,
        r#"const foo = "foo\uD834";"#,
        r#"const foo = "foo \uD834";"#,
        r#"const foo = "foo \u2500";"#,
        r#"const foo = "foo \x46";"#,
        r#"const foo = "foo\\xbar";"#,
        r#"const foo = "foo\\ubarbaz";"#,
        r#"const foo = "foo\\\\xbar";"#,
        r#"const foo = "foo\\\\ubarbaz";"#,
        r#"const foo = "\ca";"#,
        r"const foo = `\xA9`;",
        r"const foo = `\uD834`;",
        r"const foo = `\u{1D306}`;",
        r"const foo = `\uD834foo`;",
        r"const foo = `foo\uD834`;",
        r"const foo = `foo \uD834`;",
        r#"const foo = `${"\uD834 foo"} \uD834`;"#,
        r"const foo = `foo \u2500`;",
        r"const foo = `foo \x46`;",
        r"const foo = `foo\\xbar`;",
        r"const foo = `foo\\ubarbaz`;",
        r"const foo = `foo\\\\xbar`;",
        r"const foo = `foo\\\\ubarbaz`;",
        r"const foo = `\ca`;",
        r"const foo = /foo\xA9/",
        r"const foo = /foo\uD834/",
        r"const foo = /foo\u{1D306}/u",
        r"const foo = /foo\cA/",
        r"const foo = /foo\\xa9/;",
        r"const foo = /foo\\\\xa9/;",
        r"const foo = /foo\\uD834/",
        r"const foo = /foo\\u{1}/u",
        r"const foo = /foo\\cA/",
        r#"const foo = new RegExp("/\xA9")"#,
        r#"const foo = new RegExp("/\uD834/")"#,
        r#"const foo = new RegExp("/\u{1D306}/", "u")"#,
        r#"const foo = new RegExp("/\ca/")"#,
        r#"const foo = new RegExp("/\cA/")"#,
    ];
    let fail = vec![
        r#"const foo = "\xAab\xaab\xAAb\uAaAab\uaaaab\uAAAAb\u{AaAa}b\u{aaaa}b\u{AAAA}";"#,
        r"const foo = `\xAab\xaab\xAA${foo}\uAaAab\uaaaab\uAAAAb\u{AaAa}${foo}\u{aaaa}b\u{AAAA}`;",
        r"const foo = `\ud834${foo}\ud834${foo}\ud834`;",
        r#"const foo = new RegExp("/\u{1d306}/", "u")"#,
    ];
    let fix = vec![
        (r#"const foo = "\xa9";"#, r#"const foo = "\xA9";"#, None),
        (r#"const foo = "\xAa";"#, r#"const foo = "\xAA";"#, None),
        (r#"const foo = "\uAaAa";"#, r#"const foo = "\uAAAA";"#, None),
        (r#"const foo = "\u{AaAa}";"#, r#"const foo = "\u{AAAA}";"#, None),
        (
            r#"const foo = "\xAab\xaab\xAAb\uAaAab\uaaaab\uAAAAb\u{AaAa}b\u{aaaa}b\u{AAAA}";"#,
            r#"const foo = "\xAAb\xAAb\xAAb\uAAAAb\uAAAAb\uAAAAb\u{AAAA}b\u{AAAA}b\u{AAAA}";"#,
            None,
        ),
        (r#"const foo = "\ud834";"#, r#"const foo = "\uD834";"#, None),
        (r#"const foo = "\u{1d306}";"#, r#"const foo = "\u{1D306}";"#, None),
        (r#"const foo = "\ud834foo";"#, r#"const foo = "\uD834foo";"#, None),
        (r#"const foo = "foo\ud834";"#, r#"const foo = "foo\uD834";"#, None),
        (r#"const foo = "foo \ud834";"#, r#"const foo = "foo \uD834";"#, None),
        (r#"const foo = "\\\ud834foo";"#, r#"const foo = "\\\uD834foo";"#, None),
        (r#"const foo = "foo\\\ud834";"#, r#"const foo = "foo\\\uD834";"#, None),
        (r#"const foo = "foo \\\ud834";"#, r#"const foo = "foo \\\uD834";"#, None),
        (r"const foo = `\xa9`;", r"const foo = `\xA9`;", None),
        (r"const foo = `\ud834`;", r"const foo = `\uD834`;", None),
        (r"const foo = `\u{1d306}`;", r"const foo = `\u{1D306}`;", None),
        (r"const foo = `\ud834foo`;", r"const foo = `\uD834foo`;", None),
        (r"const foo = `foo\ud834`;", r"const foo = `foo\uD834`;", None),
        (r"const foo = `foo \ud834`;", r"const foo = `foo \uD834`;", None),
        (
            r#"const foo = `${"\ud834 foo"} \ud834`;"#,
            r#"const foo = `${"\uD834 foo"} \uD834`;"#,
            None,
        ),
        (
            r"const foo = `\ud834${foo}\ud834${foo}\ud834`;",
            r"const foo = `\uD834${foo}\uD834${foo}\uD834`;",
            None,
        ),
        (r"const foo = `\\\ud834foo`;", r"const foo = `\\\uD834foo`;", None),
        (r"const foo = `foo\\\ud834`;", r"const foo = `foo\\\uD834`;", None),
        (r"const foo = `foo \\\ud834`;", r"const foo = `foo \\\uD834`;", None),
        (r"const foo = `\xAa`;", r"const foo = `\xAA`;", None),
        (r"const foo = `\uAaAa`;", r"const foo = `\uAAAA`;", None),
        (r"const foo = `\u{AaAa}`;", r"const foo = `\u{AAAA}`;", None),
        (
            r"const foo = `\xAab\xaab\xAA${foo}\uAaAab\uaaaab\uAAAAb\u{AaAa}${foo}\u{aaaa}b\u{AAAA}`;",
            r"const foo = `\xAAb\xAAb\xAA${foo}\uAAAAb\uAAAAb\uAAAAb\u{AAAA}${foo}\u{AAAA}b\u{AAAA}`;",
            None,
        ),
        (r"const foo = /\xa9/;", r"const foo = /\xA9/;", None),
        (r"const foo = /\ud834/", r"const foo = /\uD834/", None),
        (r"const foo = /\u{1d306}/u", r"const foo = /\u{1D306}/u", None),
        (r"const foo = /\ca/", r"const foo = /\cA/", None),
        (r"const foo = /foo\\\xa9/;", r"const foo = /foo\\\xA9/;", None),
        (r"const foo = /foo\\\\\xa9/;", r"const foo = /foo\\\\\xA9/;", None),
        (r"const foo = /\xAa/;", r"const foo = /\xAA/;", None),
        (r"const foo = /\uAaAa/;", r"const foo = /\uAAAA/;", None),
        (r"const foo = /\u{AaAa}/;", r"const foo = /\u{AAAA}/;", None),
        (
            r"const foo = /\xAab\xaab\xAAb\uAaAab\uaaaab\uAAAAb\u{AaAa}b\u{aaaa}b\u{AAAA}b\ca/;",
            r"const foo = /\xAAb\xAAb\xAAb\uAAAAb\uAAAAb\uAAAAb\u{AAAA}b\u{AAAA}b\u{AAAA}b\cA/;",
            None,
        ),
        (r#"const foo = new RegExp("/\xa9")"#, r#"const foo = new RegExp("/\xA9")"#, None),
        (r#"const foo = new RegExp("/\ud834/")"#, r#"const foo = new RegExp("/\uD834/")"#, None),
        (
            r#"const foo = new RegExp("/\u{1d306}/", "u")"#,
            r#"const foo = new RegExp("/\u{1D306}/", "u")"#,
            None,
        ),
    ];

    Tester::new(EscapeCase::NAME, EscapeCase::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
