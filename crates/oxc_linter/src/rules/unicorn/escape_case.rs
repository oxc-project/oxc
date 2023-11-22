use lazy_static::lazy_static;
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
use regex::Regex;

use crate::{context::LintContext, rule::Rule, AstNode, Fix};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-unicorn(escape-case): Use uppercase characters for the value of the escape sequence.")]
#[diagnostic(severity(warning), help(""))]
struct EscapeCaseDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct EscapeCase;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces defining escape sequence values with uppercase characters rather than lowercase ones. This promotes readability by making the escaped value more distinguishable from the identifier.
    ///
    /// ### Example
    /// ```javascript
    /// // fail
    /// const foo = '\xa9';
    /// const foo = '\ud834';
    /// const foo = '\u{1d306}';
    /// const foo = '\ca';
    /// // pass
    /// const foo = '\xA9';
    /// const foo = '\uD834';
    /// const foo = '\u{1D306}';
    /// const foo = '\cA';
    /// ```
    EscapeCase,
    pedantic
);
lazy_static! {
    // \x00
    static ref HEX_2: Regex = Regex::new(r"^[\dA-Fa-f]{2}").unwrap();
    // \u0000
    static ref HEX_4: Regex = Regex::new(r"^[\dA-Fa-f]{4}").unwrap();
    // \u{00000000}
    static ref HEX_N: Regex = Regex::new(r"^\{[\dA-Fa-f]+}").unwrap();
}

// /(?<=(?:^|[^\\])(?:\\\\)*\\)(?<data>x[\dA-Fa-f]{2}|u[\dA-Fa-f]{4}|u{[\dA-Fa-f]+})/g
fn check_case(value: &str, is_regex: bool) -> Option<String> {
    let mut in_escape = false;
    let chars: Vec<char> = value.chars().collect();
    let mut result = String::with_capacity(value.len());
    let mut index = 0;
    while index < chars.len() {
        let c = chars[index];
        index += 1;
        result.push(c);
        if in_escape {
            match c {
                'x' => {
                    if HEX_2.is_match(&value[index..]) {
                        for c in value[index..index + 2].chars() {
                            result.push(c.to_ascii_uppercase());
                        }
                        index += 2;
                    }
                }
                'u' => {
                    if HEX_4.is_match(&value[index..]) {
                        for c in value[index..index + 4].chars() {
                            result.push(c.to_ascii_uppercase());
                        }
                        index += 4;
                    } else if let Some(m) = HEX_N.find(&value[index..]) {
                        result.push('{');
                        for c in value[index + 1..index + m.end() - 1].chars() {
                            result.push(c.to_ascii_uppercase());
                        }
                        index += m.len();
                        result.push('}');
                    }
                }
                // regex control character
                'c' if is_regex => {
                    let c = chars[index];
                    if c.is_ascii_lowercase() {
                        result.push(c.to_ascii_uppercase());
                        index += 1;
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
                    ctx.diagnostic_with_fix(EscapeCaseDiagnostic(*span), || Fix::new(fixed, *span));
                }
            }
            AstKind::TemplateLiteral(TemplateLiteral { quasis, .. }) => {
                quasis.iter().for_each(|quasi| {
                    if let Some(fixed) =
                        check_case(quasi.span.source_text(ctx.source_text()), false)
                    {
                        ctx.diagnostic_with_fix(EscapeCaseDiagnostic(quasi.span), || {
                            Fix::new(fixed, quasi.span)
                        });
                    }
                });
            }
            AstKind::RegExpLiteral(regex) => {
                let text = regex.span.source_text(ctx.source_text());
                if let Some(fixed) = check_case(text, true) {
                    ctx.diagnostic_with_fix(EscapeCaseDiagnostic(regex.span), || {
                        Fix::new(fixed, regex.span)
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
    let fail = vec![];
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

    Tester::new_without_config(EscapeCase::NAME, pass, fail).expect_fix(fix).test_and_snapshot();
}
