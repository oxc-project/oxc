use oxc_ast::{
    ast::{StringLiteral, TemplateLiteral},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_regular_expression::{
    ast::{Character, CharacterKind},
    visit::Visit,
};
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_hex_escape_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Use Unicode escapes instead of hexadecimal escapes.").with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoHexEscape;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces a convention of using [Unicode escapes](https://mathiasbynens.be/notes/javascript-escapes#unicode) instead of [hexadecimal escapes](https://mathiasbynens.be/notes/javascript-escapes#hexadecimal) for consistency and clarity.
    ///
    /// ### Why is this bad?
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// const foo = '\x1B';
    /// const foo = `\x1B${bar}`;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// const foo = '\u001B';
    /// const foo = `\u001B${bar}`;
    /// ```
    NoHexEscape,
    unicorn,
    pedantic,
    fix
);

// \x -> \u00
fn check_escape(value: &str) -> Option<String> {
    let mut in_escape = false;
    let mut matched = Vec::new();
    for (index, c) in value.char_indices() {
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
        let mut fixed: String = String::with_capacity(value.len() + matched.len() * 2);
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
                    ctx.diagnostic_with_fix(no_hex_escape_diagnostic(*span), |fixer| {
                        fixer.replace(*span, format!("'{fixed}'"))
                    });
                }
            }
            AstKind::TemplateLiteral(TemplateLiteral { quasis, .. }) => {
                quasis.iter().for_each(|quasi| {
                    if let Some(fixed) = check_escape(quasi.span.source_text(ctx.source_text())) {
                        ctx.diagnostic_with_fix(no_hex_escape_diagnostic(quasi.span), |fixer| {
                            fixer.replace(quasi.span, fixed)
                        });
                    }
                });
            }
            AstKind::RegExpLiteral(regex) => {
                let Some(pattern) = regex.regex.pattern.as_pattern() else {
                    return;
                };

                let mut finder = HexEscapeFinder { hex_escapes: vec![] };
                finder.visit_pattern(pattern);

                for span in finder.hex_escapes {
                    let unicode_escape =
                        format!(r"\u00{}", &span.source_text(ctx.source_text())[2..]);

                    ctx.diagnostic_with_fix(no_hex_escape_diagnostic(span), |fixer| {
                        fixer.replace(span, unicode_escape)
                    });
                }
            }
            _ => {}
        }
    }
}

struct HexEscapeFinder {
    hex_escapes: Vec<Span>,
}

impl Visit<'_> for HexEscapeFinder {
    fn visit_character(&mut self, ch: &Character) {
        if ch.kind == CharacterKind::HexadecimalEscape {
            self.hex_escapes.push(ch.span);
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

    let fail = vec![
        r#"const foo = "\xb1""#,
        r"wrapId(/(^|[<nonId>])(?:алг|арг(?:\x20*рез)?|ввод|ВКЛЮЧИТЬ|вс[её]|выбор|вывод|выход|дано|для|до|дс|если|иначе|исп|использовать|кон(?:(?:\x20+|_)исп)?|кц(?:(?:\x20+|_)при)?|надо|нач|нс|нц|от|пауза|пока|при|раза?|рез|стоп|таб|то|утв|шаг)(?=[<nonId>]|$)/.source)",
    ];

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
        (
            r#"const unicodeMatch = "".toString().match(/[^\x00-\xFF]+/g);"#,
            r#"const unicodeMatch = "".toString().match(/[^\u0000-\u00FF]+/g);"#,
            None,
        ),
        (
            r#"const unicodeMatch = "".toString().match(/[^\x00-\xFF]+/gim);"#,
            r#"const unicodeMatch = "".toString().match(/[^\u0000-\u00FF]+/gim);"#,
            None,
        ),
    ];

    Tester::new(NoHexEscape::NAME, NoHexEscape::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
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
