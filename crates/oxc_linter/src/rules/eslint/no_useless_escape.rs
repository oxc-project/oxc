use memchr::memmem;
use oxc_ast::{ast::RegExpFlags, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_regular_expression::{
    ast::{Character, CharacterClass},
    visit::{RegExpAstKind, Visit},
};
use oxc_semantic::NodeId;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_useless_escape_diagnostic(escape_char: char, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Unnecessary escape character {escape_char:?}")).with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoUselessEscape;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow unnecessary escape characters
    ///
    /// ### Why is this bad?
    ///
    ///
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule:
    ///
    /// ```javascript
    /// /*eslint no-useless-escape: "error"*/
    ///
    /// "\'";
    /// '\"';
    /// "\#";
    /// "\e";
    /// `\"`;
    /// `\"${foo}\"`;
    /// `\#{foo}`;
    /// /\!/;
    /// /\@/;
    /// /[\[]/;
    /// /[a-z\-]/;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    ///
    /// ```javascript
    /// /*eslint no-useless-escape: "error"*/
    ///
    /// "\"";
    /// '\'';
    /// "\x12";
    /// "\u00a9";
    /// "\371";
    /// "xs\u2111";
    /// `\``;
    /// `\${${foo}}`;
    /// `$\{${foo}}`;
    /// /\\/g;
    /// /\t/g;
    /// /\w\$\*\^\./;
    /// /[[]/;
    /// /[\]]/;
    /// /[a-z-]/;
    /// ```
    NoUselessEscape,
    eslint,
    correctness,
    fix
);

impl Rule for NoUselessEscape {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::RegExpLiteral(literal)
                if literal.regex.pattern.len() + literal.regex.flags.iter().count()
                    != literal.span.size() as usize =>
            {
                if let Some(pattern) = literal.regex.pattern.as_pattern() {
                    let mut finder = UselessEscapeFinder {
                        useless_escape_spans: vec![],
                        character_classes: vec![],
                        unicode_sets: literal.regex.flags.contains(RegExpFlags::V),
                        source_text: ctx.source_text(),
                    };
                    finder.visit_pattern(pattern);
                    for span in finder.useless_escape_spans {
                        let c = span.source_text(ctx.source_text()).chars().last().unwrap();
                        ctx.diagnostic_with_fix(no_useless_escape_diagnostic(c, span), |fixer| {
                            fixer.replace(span, c.to_string())
                        });
                    }
                }
            }
            AstKind::StringLiteral(literal) => check(
                ctx,
                node.id(),
                literal.span.start,
                &check_string(literal.span.source_text(ctx.source_text())),
            ),
            AstKind::TemplateLiteral(literal) if !matches!(ctx.nodes().parent_kind(node.id()), Some(AstKind::TaggedTemplateExpression(expr)) if expr.quasi.span == literal.span) => {
                for template_element in &literal.quasis {
                    check(
                        ctx,
                        node.id(),
                        template_element.span.start - 1,
                        &check_template(template_element.span.source_text(ctx.source_text())),
                    );
                }
            }
            _ => {}
        }
    }
}

fn is_within_jsx_attribute_item(id: NodeId, ctx: &LintContext) -> bool {
    if matches!(ctx.nodes().parent_kind(id), Some(AstKind::JSXAttributeItem(_))) {
        return true;
    }
    false
}

#[allow(clippy::cast_possible_truncation)]
fn check(ctx: &LintContext<'_>, node_id: NodeId, start: u32, offsets: &[usize]) {
    let source_text = ctx.source_text();
    for offset in offsets {
        let offset = start as usize + offset;
        let c = source_text[offset..].chars().next().unwrap();
        let offset = offset as u32;
        let len = c.len_utf8() as u32;

        if !is_within_jsx_attribute_item(node_id, ctx) {
            let span = Span::new(offset - 1, offset + len);
            ctx.diagnostic_with_fix(no_useless_escape_diagnostic(c, span), |fixer| {
                fixer.replace(span, c.to_string())
            });
        }
    }
}

const REGEX_GENERAL_ESCAPES: &str = "\\bcdDfnpPrsStvwWxu0123456789]";
const REGEX_NON_CHARCLASS_ESCAPES: &str = "\\bcdDfnpPrsStvwWxu0123456789]^/.$*+?[{}|()Bk";
const REGEX_CLASSSET_CHARACTER_ESCAPES: &str = "\\bcdDfnpPrsStvwWxu0123456789]q/[{}|()-";
const REGEX_CLASS_SET_RESERVED_DOUBLE_PUNCTUATOR: &str = "!#$%&*+,.:;<=>?@^`~";

fn check_character(
    source_text: &str,
    character: &Character,
    character_class: Option<&CharacterClass>,
    unicode_sets: bool,
) -> Option<Span> {
    let char_text = character.span.source_text(source_text);
    // The character is escaped if it has at least two characters and the first character is a backslash
    let is_escaped = char_text.starts_with('\\') && char_text.len() >= 2;
    if !is_escaped {
        return None;
    }
    let span = character.span;
    let escape_char = char_text.chars().nth(1).unwrap();
    let escapes = if character_class.is_some() {
        if unicode_sets {
            REGEX_CLASSSET_CHARACTER_ESCAPES
        } else {
            REGEX_GENERAL_ESCAPES
        }
    } else {
        REGEX_NON_CHARCLASS_ESCAPES
    };
    if escapes.contains(escape_char) {
        return None;
    }

    if let Some(class) = character_class {
        if escape_char == '^' {
            /* The '^' character is also a special case; it must always be escaped outside of character classes, but
             * it only needs to be escaped in character classes if it's at the beginning of the character class. To
             * account for this, consider it to be a valid escape character outside of character classes, and filter
             * out '^' characters that appear at the start of a character class.
             * (From ESLint source: https://github.com/eslint/eslint/blob/v9.9.1/lib/rules/no-useless-escape.js)
             */
            if class.span.start + 1 == span.start {
                return None;
            }
        }
        if unicode_sets {
            if REGEX_CLASS_SET_RESERVED_DOUBLE_PUNCTUATOR.contains(escape_char) {
                if let Some(prev_char) = source_text.chars().nth(span.end as usize) {
                    // Escaping is valid when it is a reserved double punctuator
                    if prev_char == escape_char {
                        return None;
                    }
                }
                if let Some(prev_prev_char) = source_text.chars().nth(span.start as usize - 1) {
                    if prev_prev_char == escape_char {
                        if escape_char != '^' {
                            return None;
                        }

                        // Escaping caret is unnecessary if the previous character is a `negate` caret(`^`).
                        if !class.negative {
                            return None;
                        }

                        let caret_index = class.span.start + 1;
                        if caret_index < span.start - 1 {
                            return None;
                        }
                    }
                }
            }
        } else if escape_char == '-' {
            /* The '-' character is a special case, because it's only valid to escape it if it's in a character
             * class, and is not at either edge of the character class. To account for this, don't consider '-'
             * characters to be valid in general, and filter out '-' characters that appear in the middle of a
             * character class.
             * (From ESLint source: https://github.com/eslint/eslint/blob/v9.9.1/lib/rules/no-useless-escape.js)
             */
            if class.span.start + 1 != span.start && span.end != class.span.end - 1 {
                return None;
            }
        }
    }

    Some(span)
}

const VALID_STRING_ESCAPES: &str = "\\nrvtbfux\n\r\u{2028}\u{2029}";

fn check_string(string: &str) -> Vec<usize> {
    if string.len() <= 1 {
        return vec![];
    }

    let quote_char = string.chars().next().unwrap();
    let bytes = &string[1..string.len() - 1].as_bytes();
    let escapes = memmem::find_iter(bytes, "\\").collect::<Vec<_>>();

    if escapes.is_empty() {
        return vec![];
    }

    let mut offsets = vec![];
    let mut prev_offset = None; // for checking double escape `\\`
    for offset in escapes {
        // Safety:
        // The offset comes from a utf8 checked string

        let s = unsafe { std::str::from_utf8_unchecked(&bytes[offset..]) };
        if let Some(c) = s.chars().nth(1) {
            if !(c == quote_char
                || (offset > 0 && prev_offset == Some(offset - 1))
                || c.is_ascii_digit()
                || VALID_STRING_ESCAPES.contains(c))
            {
                // +1 for skipping the first string quote `"`
                // +1 for skipping the escape char `\\`
                offsets.push(offset + 2);
            }
        }
        prev_offset.replace(offset);
    }

    offsets
}

fn check_template(string: &str) -> Vec<usize> {
    if string.len() <= 1 {
        return vec![];
    }

    let mut offsets = vec![];
    let mut in_escape = false;
    let mut prev_non_escape_char = '`';
    let mut byte_offset = 1;

    let mut chars = string.chars().peekable();

    while let Some(c) = chars.next() {
        byte_offset += c.len_utf8();

        if in_escape {
            in_escape = false;
            match c {
                c if c.is_ascii_digit() || c == '`' => { /* noop */ }
                '{' => {
                    if prev_non_escape_char != '$' {
                        offsets.push(byte_offset - c.len_utf8());
                    }
                }
                '$' => {
                    if chars.peek().is_some_and(|c| *c != '{') {
                        offsets.push(byte_offset - c.len_utf8());
                    }
                }
                c if !VALID_STRING_ESCAPES.contains(c) => {
                    offsets.push(byte_offset - c.len_utf8());
                }
                _ => {}
            }
            prev_non_escape_char = c;
        } else if c == '\\' {
            in_escape = true;
        } else {
            prev_non_escape_char = c;
        }
    }

    offsets
}

struct UselessEscapeFinder<'a> {
    useless_escape_spans: Vec<Span>,
    character_classes: Vec<&'a CharacterClass<'a>>,
    unicode_sets: bool,
    source_text: &'a str,
}

impl<'a> Visit<'a> for UselessEscapeFinder<'a> {
    fn enter_node(&mut self, kind: RegExpAstKind<'a>) {
        if let RegExpAstKind::CharacterClass(class) = kind {
            self.character_classes.push(class);
        }
    }

    fn leave_node(&mut self, kind: RegExpAstKind<'a>) {
        if let RegExpAstKind::CharacterClass(_) = kind {
            self.character_classes.pop();
        }
    }

    fn visit_character(&mut self, ch: &Character) {
        let character_class = self.character_classes.last().copied();
        if let Some(span) =
            check_character(self.source_text, ch, character_class, self.unicode_sets)
        {
            self.useless_escape_spans.push(span);
        }
    }
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
        // Carets
        "/[^^]/u", // { "ecmaVersion": 2015 },
        // ES2024
        r"/[\q{abc}]/v", // { "ecmaVersion": 2024 },
        r"/[\(]/v",      // { "ecmaVersion": 2024 },
        r"/[\)]/v",      // { "ecmaVersion": 2024 },
        r"/[\{]/v",      // { "ecmaVersion": 2024 },
        r"/[\]]/v",      // { "ecmaVersion": 2024 },
        r"/[\}]/v",      // { "ecmaVersion": 2024 },
        r"/[\/]/v",      // { "ecmaVersion": 2024 },
        r"/[\-]/v",      // { "ecmaVersion": 2024 },
        r"/[\|]/v",      // { "ecmaVersion": 2024 },
        r"/[\$$]/v",     // { "ecmaVersion": 2024 },
        r"/[\&&]/v",     // { "ecmaVersion": 2024 },
        r"/[\!!]/v",     // { "ecmaVersion": 2024 },
        r"/[\##]/v",     // { "ecmaVersion": 2024 },
        r"/[\%%]/v",     // { "ecmaVersion": 2024 },
        r"/[\**]/v",     // { "ecmaVersion": 2024 },
        r"/[\++]/v",     // { "ecmaVersion": 2024 },
        r"/[\,,]/v",     // { "ecmaVersion": 2024 },
        r"/[\..]/v",     // { "ecmaVersion": 2024 },
        r"/[\::]/v",     // { "ecmaVersion": 2024 },
        r"/[\;;]/v",     // { "ecmaVersion": 2024 },
        r"/[\<<]/v",     // { "ecmaVersion": 2024 },
        r"/[\==]/v",     // { "ecmaVersion": 2024 },
        r"/[\>>]/v",     // { "ecmaVersion": 2024 },
        r"/[\??]/v",     // { "ecmaVersion": 2024 },
        r"/[\@@]/v",     // { "ecmaVersion": 2024 },
        "/[\\``]/v",     // { "ecmaVersion": 2024 },
        r"/[\~~]/v",     // { "ecmaVersion": 2024 },
        r"/[^\^^]/v",    // { "ecmaVersion": 2024 },
        r"/[_\^^]/v",    // { "ecmaVersion": 2024 },
        r"/[$\$]/v",     // { "ecmaVersion": 2024 },
        r"/[&\&]/v",     // { "ecmaVersion": 2024 },
        r"/[!\!]/v",     // { "ecmaVersion": 2024 },
        r"/[#\#]/v",     // { "ecmaVersion": 2024 },
        r"/[%\%]/v",     // { "ecmaVersion": 2024 },
        r"/[*\*]/v",     // { "ecmaVersion": 2024 },
        r"/[+\+]/v",     // { "ecmaVersion": 2024 },
        r"/[,\,]/v",     // { "ecmaVersion": 2024 },
        r"/[.\.]/v",     // { "ecmaVersion": 2024 },
        r"/[:\:]/v",     // { "ecmaVersion": 2024 },
        r"/[;\;]/v",     // { "ecmaVersion": 2024 },
        r"/[<\<]/v",     // { "ecmaVersion": 2024 },
        r"/[=\=]/v",     // { "ecmaVersion": 2024 },
        r"/[>\>]/v",     // { "ecmaVersion": 2024 },
        r"/[?\?]/v",     // { "ecmaVersion": 2024 },
        r"/[@\@]/v",     // { "ecmaVersion": 2024 },
        "/[`\\`]/v",     // { "ecmaVersion": 2024 },
        r"/[~\~]/v",     // { "ecmaVersion": 2024 },
        r"/[^^\^]/v",    // { "ecmaVersion": 2024 },
        r"/[_^\^]/v",    // { "ecmaVersion": 2024 },
        r"/[\&&&\&]/v",  // { "ecmaVersion": 2024 },
        r"/[[\-]\-]/v",  // { "ecmaVersion": 2024 },
        r"/[\^]/v",      // { "ecmaVersion": 2024 }
        r"/[\s\-(]/",    // https://github.com/oxc-project/oxc/issues/5227
        r"/\c/",         // https://github.com/oxc-project/oxc/issues/6046
        r"/\\c/",        // https://github.com/oxc-project/oxc/issues/6046
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
        r#"var stringLiteralWithNextLine = "line 1\line 2";"#,
        r"var stringLiteralWithNextLine = `line 1\line 2`;",
        r#""use\ strict";"#,
        // spellchecker:off
        r#"({ foo() { "foo"; "bar"; "ba\z" } })"#, // { "ecmaVersion": 6 }
        // spellchecker:on
        // Carets
        r"/[^\^]/",
        r"/[^\^]/u", // { "ecmaVersion": 2015 },
        // ES2024
        r"/[\$]/v",            // { "ecmaVersion": 2024 },
        r"/[\&\&]/v",          // { "ecmaVersion": 2024 },
        r"/[\!\!]/v",          // { "ecmaVersion": 2024 },
        r"/[\#\#]/v",          // { "ecmaVersion": 2024 },
        r"/[\%\%]/v",          // { "ecmaVersion": 2024 },
        r"/[\*\*]/v",          // { "ecmaVersion": 2024 },
        r"/[\+\+]/v",          // { "ecmaVersion": 2024 },
        r"/[\,\,]/v",          // { "ecmaVersion": 2024 },
        r"/[\.\.]/v",          // { "ecmaVersion": 2024 },
        r"/[\:\:]/v",          // { "ecmaVersion": 2024 },
        r"/[\;\;]/v",          // { "ecmaVersion": 2024 },
        r"/[\<\<]/v",          // { "ecmaVersion": 2024 },
        r"/[\=\=]/v",          // { "ecmaVersion": 2024 },
        r"/[\>\>]/v",          // { "ecmaVersion": 2024 },
        r"/[\?\?]/v",          // { "ecmaVersion": 2024 },
        r"/[\@\@]/v",          // { "ecmaVersion": 2024 },
        "/[\\`\\`]/v",         // { "ecmaVersion": 2024 },
        r"/[\~\~]/v",          // { "ecmaVersion": 2024 },
        r"/[^\^\^]/v",         // { "ecmaVersion": 2024 },
        r"/[_\^\^]/v",         // { "ecmaVersion": 2024 },
        r"/[\&\&&\&]/v",       // { "ecmaVersion": 2024 },
        r"/[\p{ASCII}--\.]/v", // { "ecmaVersion": 2024 },
        r"/[\p{ASCII}&&\.]/v", // { "ecmaVersion": 2024 },
        r"/[\.--[.&]]/v",      // { "ecmaVersion": 2024 },
        r"/[\.&&[.&]]/v",      // { "ecmaVersion": 2024 },
        r"/[\.--\.--\.]/v",    // { "ecmaVersion": 2024 },
        r"/[\.&&\.&&\.]/v",    // { "ecmaVersion": 2024 },
        r"/[[\.&]--[\.&]]/v",  // { "ecmaVersion": 2024 },
        r"/[[\.&]&&[\.&]]/v",  // { "ecmaVersion": 2024 }
    ];

    let fix = vec![
        ("var foo = /\\#/;", "var foo = /#/;", None),
        ("var foo = /\\;/;", "var foo = /;/;", None),
        ("var foo = \"\\'\";", "var foo = \"'\";", None),
        ("var foo = \"\\#/\";", "var foo = \"#/\";", None),
        ("var foo = \"\\a\"", "var foo = \"a\"", None),
        ("var foo = \"\\B\";", "var foo = \"B\";", None),
        ("var foo = \"\\@\";", "var foo = \"@\";", None),
        ("var foo = \"foo \\a bar\";", "var foo = \"foo a bar\";", None),
        ("var foo = '\\\"';", "var foo = '\"';", None),
        ("var foo = '\\#';", "var foo = '#';", None),
        ("var foo = '\\$';", "var foo = '$';", None),
        ("var foo = '\\p';", "var foo = 'p';", None),
        ("var foo = '\\p\\a\\@';", "var foo = 'pa@';", None),
        ("<foo attr={\"\\d\"}/>", "<foo attr={\"d\"}/>", None),
        ("var foo = '\\`';", "var foo = '`';", None),
        ("var foo = `\\\"`;", "var foo = `\"`;", None),
        ("var foo = `\\'`;", "var foo = `'`;", None),
        ("var foo = `\\#`;", "var foo = `#`;", None),
        ("var foo = '\\`foo\\`';", "var foo = '`foo`';", None),
        ("var foo = `\\\"${foo}\\\"`;", "var foo = `\"${foo}\"`;", None),
        ("var foo = `\\'${foo}\\'`;", "var foo = `'${foo}'`;", None),
        ("var foo = `\\#${foo}`;", "var foo = `#${foo}`;", None),
        ("let foo = '\\ ';", "let foo = ' ';", None),
        ("let foo = /\\ /;", "let foo = / /;", None),
        ("var foo = `\\$\\{{${foo}`;", "var foo = `$\\{{${foo}`;", None),
        (r#""use\ strict";"#, r#""use strict";"#, None),
        // spellchecker:off
        (r#"({ foo() { "foo"; "bar"; "ba\z" } })"#, r#"({ foo() { "foo"; "bar"; "baz" } })"#, None), // { "ecmaVersion": 6 }
        // spellchecker:on
        // Carets
        (r"/[^\^]/", r"/[^^]/", None),
        (r"/[^\^]/u", r"/[^^]/u", None), // { "ecmaVersion": 2015 },
        // ES2024
        (r"/[\$]/v", r"/[$]/v", None),       // { "ecmaVersion": 2024 },
        (r"/[\&\&]/v", r"/[&\&]/v", None),   // { "ecmaVersion": 2024 },
        (r"/[\!\!]/v", r"/[!\!]/v", None),   // { "ecmaVersion": 2024 },
        (r"/[\#\#]/v", r"/[#\#]/v", None),   // { "ecmaVersion": 2024 },
        (r"/[\%\%]/v", r"/[%\%]/v", None),   // { "ecmaVersion": 2024 },
        (r"/[\*\*]/v", r"/[*\*]/v", None),   // { "ecmaVersion": 2024 },
        (r"/[\+\+]/v", r"/[+\+]/v", None),   // { "ecmaVersion": 2024 },
        (r"/[\,\,]/v", r"/[,\,]/v", None),   // { "ecmaVersion": 2024 },
        (r"/[\.\.]/v", r"/[.\.]/v", None),   // { "ecmaVersion": 2024 },
        (r"/[\:\:]/v", r"/[:\:]/v", None),   // { "ecmaVersion": 2024 },
        (r"/[\;\;]/v", r"/[;\;]/v", None),   // { "ecmaVersion": 2024 },
        (r"/[\<\<]/v", r"/[<\<]/v", None),   // { "ecmaVersion": 2024 },
        (r"/[\=\=]/v", r"/[=\=]/v", None),   // { "ecmaVersion": 2024 },
        (r"/[\>\>]/v", r"/[>\>]/v", None),   // { "ecmaVersion": 2024 },
        (r"/[\?\?]/v", r"/[?\?]/v", None),   // { "ecmaVersion": 2024 },
        (r"/[\@\@]/v", r"/[@\@]/v", None),   // { "ecmaVersion": 2024 },
        (r"/[\`\`]/v", r"/[`\`]/v", None),   // { "ecmaVersion": 2024 },
        (r"/[\~\~]/v", r"/[~\~]/v", None),   // { "ecmaVersion": 2024 },
        (r"/[^\^\^]/v", r"/[^^\^]/v", None), // { "ecmaVersion": 2024 },
        (r"/[_\^\^]/v", r"/[_^\^]/v", None), // { "ecmaVersion": 2024 },
        (r"/[\&\&&\&]/v", r"/[&\&&\&]/v", None), // { "ecmaVersion": 2024 },
        (r"/[\p{ASCII}--\.]/v", r"/[\p{ASCII}--.]/v", None), // { "ecmaVersion": 2024 },
        (r"/[\p{ASCII}&&\.]/v", r"/[\p{ASCII}&&.]/v", None), // { "ecmaVersion": 2024 },
        (r"/[\.--[.&]]/v", r"/[.--[.&]]/v", None), // { "ecmaVersion": 2024 },
        (r"/[\.&&[.&]]/v", r"/[.&&[.&]]/v", None), // { "ecmaVersion": 2024 },
        (r"/[\.--\.--\.]/v", r"/[.--.--.]/v", None), // { "ecmaVersion": 2024 },
        (r"/[\.&&\.&&\.]/v", r"/[.&&.&&.]/v", None), // { "ecmaVersion": 2024 },
        (r"/[[\.&]--[\.&]]/v", r"/[[.&]--[.&]]/v", None), // { "ecmaVersion": 2024 },
        (r"/[[\.&]&&[\.&]]/v", r"/[[.&]&&[.&]]/v", None), // { "ecmaVersion": 2024 }
        (
            // https://github.com/oxc-project/oxc/issues/5227
            r"const regex = /(https?:\/\/github\.com\/(([^\s]+)\/([^\s]+))\/([^\s]+\/)?(issues|pull)\/([0-9]+))|(([^\s]+)\/([^\s]+))?#([1-9][0-9]*)($|[\s\:\;\-\(\=])/;",
            r"const regex = /(https?:\/\/github\.com\/(([^\s]+)\/([^\s]+))\/([^\s]+\/)?(issues|pull)\/([0-9]+))|(([^\s]+)\/([^\s]+))?#([1-9][0-9]*)($|[\s:;\-(=])/;",
            None,
        ),
    ];

    Tester::new(NoUselessEscape::NAME, NoUselessEscape::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
