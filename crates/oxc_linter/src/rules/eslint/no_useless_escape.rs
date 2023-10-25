use oxc_ast::AstKind;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-useless-escape): Unnecessary escape character {0:?}")]
#[diagnostic(severity(warning))]
struct NoUselessEscapeDiagnostic(char, #[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoUselessEscape;

declare_oxc_lint!(
    /// ### What it does
    ///
    ///
    /// ### Why is this bad?
    ///
    ///
    /// ### Example
    /// ```javascript
    /// ```
    NoUselessEscape,
    correctness
);

impl Rule for NoUselessEscape {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if matches!(ctx.nodes().parent_kind(node.id()), Some(AstKind::JSXAttributeItem(_))) {
            return;
        }

        match node.kind() {
            AstKind::RegExpLiteral(literal)
                if literal.regex.pattern.len() + literal.regex.flags.iter().count()
                    != literal.span.size() as usize =>
            {
                check(
                    ctx,
                    literal.span.start,
                    &check_regexp(literal.span.source_text(ctx.source_text())),
                );
            }
            AstKind::StringLiteral(literal) => check(
                ctx,
                literal.span.start,
                &check_string(literal.span.source_text(ctx.source_text())),
            ),
            AstKind::TemplateLiteral(literal) if !matches!(ctx.nodes().parent_kind(node.id()), Some(AstKind::TaggedTemplateExpression(expr)) if expr.quasi.span == literal.span) => {
                for template_element in &literal.quasis {
                    check(
                        ctx,
                        template_element.span.start - 1,
                        &check_template(template_element.span.source_text(ctx.source_text())),
                    );
                }
            }
            _ => {}
        }
    }
}

#[allow(clippy::cast_possible_truncation)]
fn check(ctx: &LintContext<'_>, start: u32, offsets: &[usize]) {
    let source_text = ctx.source_text();
    for offset in offsets {
        let offset = start as usize + offset;
        let c = source_text[offset..].chars().next().unwrap();
        let offset = offset as u32;
        let len = c.len_utf8() as u32;
        ctx.diagnostic(NoUselessEscapeDiagnostic(c, Span::new(offset - 1, offset + len)));
    }
}

const REGEX_GENERAL_ESCAPES: &str = "\\bcdDfnpPrsStvwWxu0123456789]";
const REGEX_NON_CHARCLASS_ESCAPES: &str = "\\bcdDfnpPrsStvwWxu0123456789]^/.$*+?[{}|()Bk";

fn check_regexp(regex: &str) -> Vec<usize> {
    let mut offsets = vec![];
    let mut in_escape = false;
    let mut in_character_class = false;
    let mut start_char_class = false;
    let mut offset = 1;

    // Skip the leading and trailing `/`
    let mut chars = regex[1..regex.len() - 1].chars().peekable();
    while let Some(c) = chars.next() {
        if in_escape {
            in_escape = false;
            match c {
                '-' if in_character_class
                    && !start_char_class
                    && !chars.peek().is_some_and(|c| *c == ']') =>
                { /* noop */ }
                '^' if start_char_class => { /* noop */ }
                _ => {
                    let escapes = if in_character_class {
                        REGEX_GENERAL_ESCAPES
                    } else {
                        REGEX_NON_CHARCLASS_ESCAPES
                    };
                    if !escapes.contains(c) {
                        offsets.push(offset);
                    }
                }
            }
        } else if c == '/' && !in_character_class {
            break;
        } else if c == '[' {
            in_character_class = true;
            start_char_class = true;
        } else if c == '\\' {
            in_escape = true;
        } else if c == ']' {
            in_character_class = false;
        } else {
            start_char_class = false;
        }
        offset += c.len_utf8();
    }

    offsets
}

const VALID_STRING_ESCAPES: &str = "\\nrvtbfux\n\r\u{2028}\u{2029}";

fn check_string(string: &str) -> Vec<usize> {
    if string.len() <= 1 {
        return vec![];
    }

    let mut offsets = vec![];
    let quote_string = string.chars().next();
    let mut in_escape = false;
    let mut offset = 0;
    for c in string[1..].chars() {
        offset += c.len_utf8();
        if in_escape {
            in_escape = false;
            match c {
                c if c.is_ascii_digit() || quote_string == Some(c) => { /* noop */ }
                c if !VALID_STRING_ESCAPES.contains(c) => {
                    offsets.push(offset);
                }
                _ => {}
            }
        } else if c == '\\' {
            in_escape = true;
        }
    }
    offsets
}

fn check_template(string: &str) -> Vec<usize> {
    if string.len() <= 1 {
        return vec![];
    }

    let mut offsets = vec![];
    let mut in_escape = false;
    let mut prev_char = '`';
    let mut offset = 0;

    let mut chars = string.chars().peekable();
    while let Some(c) = chars.next() {
        offset += c.len_utf8();
        if in_escape {
            in_escape = false;
            match c {
                c if c.is_ascii_digit() || c == '`' => { /* noop */ }
                '{' => {
                    if prev_char != '$' {
                        offsets.push(offset);
                    }
                }
                '$' => {
                    if chars.peek().is_some_and(|c| *c != '{') {
                        offsets.push(offset);
                    }
                }
                c if !VALID_STRING_ESCAPES.contains(c) => {
                    offsets.push(offset);
                }
                _ => {}
            }
        } else if c == '\\' {
            in_escape = true;
        } else {
            prev_char = c;
        }
    }

    offsets
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "var foo = /\\./",
        "var foo = /\\//g",
        "var foo = /\"\"/",
        "var foo = /''/",
        "var foo = /([A-Z])\\t+/g",
        "var foo = /([A-Z])\\n+/g",
        "var foo = /([A-Z])\\v+/g",
        "var foo = /\\D/",
        "var foo = /\\W/",
        "var foo = /\\w/",
        "var foo = /\\B/",
        "var foo = /\\\\/g",
        "var foo = /\\w\\$\\*\\./",
        "var foo = /\\^\\+\\./",
        "var foo = /\\|\\}\\{\\./",
        "var foo = /]\\[\\(\\)\\//",
        "var foo = \"\\x123\"",
        "var foo = \"\\u00a9\"",
        "var foo = \"\\377\"",
        "var foo = \"\\\"\"",
        "var foo = \"xs\\u2111\"",
        "var foo = \"foo \\\\ bar\";",
        "var foo = \"\\t\";",
        "var foo = \"foo \\b bar\";",
        "var foo = '\\n';",
        "var foo = 'foo \\r bar';",
        "var foo = '\\v';",
        "var foo = '\\f';",
        "var foo = '\\\n';",
        "var foo = '\\\r\n';",
        "<foo attr=\"\\d\"/>",
        "<div> Testing: \\ </div>",
        "<div> Testing: &#x5C </div>",
        "<foo attr='\\d'></foo>",
        "<> Testing: \\ </>",
        "<> Testing: &#x5C </>",
        "var foo = `\\x123`",
        "var foo = `\\u00a9`",
        "var foo = `xs\\u2111`",
        "var foo = `foo \\\\ bar`;",
        "var foo = `\\t`;",
        "var foo = `foo \\b bar`;",
        "var foo = `\\n`;",
        "var foo = `foo \\r bar`;",
        "var foo = `\\v`;",
        "var foo = `\\f`;",
        "var foo = `\\\n`;",
        "var foo = `\\\r\n`;",
        "var foo = `${foo} \\x123`",
        "var foo = `${foo} \\u00a9`",
        "var foo = `${foo} xs\\u2111`",
        "var foo = `${foo} \\\\ ${bar}`;",
        "var foo = `${foo} \\b ${bar}`;",
        "var foo = `${foo}\\t`;",
        "var foo = `${foo}\\n`;",
        "var foo = `${foo}\\r`;",
        "var foo = `${foo}\\v`;",
        "var foo = `${foo}\\f`;",
        "var foo = `${foo}\\\n`;",
        "var foo = `${foo}\\\r\n`;",
        "var foo = `\\``",
        "var foo = `\\`${foo}\\``",
        "var foo = `\\${{${foo}`;",
        "var foo = `$\\{{${foo}`;",
        "var foo = String.raw`\\.`",
        "var foo = myFunc`\\.`",
        "var foo = /[\\d]/",
        "var foo = /[a\\-b]/",
        "var foo = /foo\\?/",
        "var foo = /example\\.com/",
        "var foo = /foo\\|bar/",
        "var foo = /\\^bar/",
        "var foo = /[\\^bar]/",
        "var foo = /\\(bar\\)/",
        r"var foo = /[[\]]/",
        "var foo = /[[]\\./",
        "var foo = /[\\]\\]]/",
        "var foo = /\\[abc]/",
        "var foo = /\\[foo\\.bar]/",
        "var foo = /vi/m",
        "var foo = /\\B/",
        "var foo = /\\0/",
        "var foo = /\\1/",
        "var foo = /(a)\\1/",
        "var foo = /(a)\\12/",
        "var foo = /[\\0]/",
        "var foo = 'foo \\  bar'",
        "var foo = 'foo \\  bar'",
        r"/]/",
        r"/\]/",
        r"/\]/u",
        "var foo = /foo\\]/",
        "var foo = /[[]\\]/",
        "var foo = /\\[foo\\.bar\\]/",
        // ES2018
        "var foo = /(?<a>)\\k<a>/",
        "var foo = /(\\\\?<a>)/",
        "var foo = /\\p{ASCII}/u",
        "var foo = /\\P{ASCII}/u",
        "var foo = /[\\p{ASCII}]/u",
        "var foo = /[\\P{ASCII}]/u",
        "`${/\\s+/g}`",
    ];

    let fail = vec![
        "var foo = /\\#/;",
        "var foo = /\\;/;",
        "var foo = \"\\'\";",
        "var foo = \"\\#/\";",
        "var foo = \"\\a\"",
        "var foo = \"\\B\";",
        "var foo = \"\\@\";",
        "var foo = \"foo \\a bar\";",
        "var foo = '\\\"';",
        "var foo = '\\#';",
        "var foo = '\\$';",
        "var foo = '\\p';",
        "var foo = '\\p\\a\\@';",
        "<foo attr={\"\\d\"}/>",
        "var foo = '\\`';",
        "var foo = `\\\"`;",
        "var foo = `\\'`;",
        "var foo = `\\#`;",
        "var foo = '\\`foo\\`';",
        "var foo = `\\\"${foo}\\\"`;",
        "var foo = `\\'${foo}\\'`;",
        "var foo = `\\#${foo}`;",
        "let foo = '\\ ';",
        "let foo = /\\ /;",
        "var foo = `\\$\\{{${foo}`;",
        "var foo = `\\$a${foo}`;",
        "var foo = `a\\{{${foo}`;",
        "var foo = /[ab\\-]/",
        "var foo = /[\\-ab]/",
        "var foo = /[ab\\?]/",
        "var foo = /[ab\\.]/",
        "var foo = /[a\\|b]/",
        "var foo = /\\-/",
        "var foo = /[\\-]/",
        "var foo = /[ab\\$]/",
        "var foo = /[\\(paren]/",
        "var foo = /[\\[]/",
        "var foo = /[\\/]/",
        "var foo = /[\\B]/",
        "var foo = /[a][\\-b]/",
        "var foo = /\\-[]/",
        "var foo = /[a\\^]/",
        "`multiline template\nliteral with useless \\escape`",
        "`multiline template\r\nliteral with useless \\escape`",
        "`template literal with line continuation \\\nand useless \\escape`",
        "`template literal with line continuation \\\r\nand useless \\escape`",
        "`template literal with mixed linebreaks \r\r\n\n\\and useless escape`",
        "`template literal with mixed linebreaks in line continuations \\\n\\\r\\\r\n\\and useless escape`",
        "`\\a```",
        r"var foo = /\（([^\）\（]+)\）$|\(([^\)\)]+)\)$/;",
    ];

    Tester::new_without_config(NoUselessEscape::NAME, pass, fail).test_and_snapshot();
}
