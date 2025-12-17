use oxc_allocator::{GetAddress, UnstableAddress};
use oxc_ast::{
    AstKind,
    ast::{JSXAttributeValue, PropertyKey, TSEnumMemberName, TSLiteral},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::keyword::RESERVED_KEYWORDS;

use crate::{AstNode, context::LintContext, rule::Rule};

fn prefer_string_raw(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(r"`String.raw` should be used to avoid escaping `\`.").with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferStringRaw;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefers use of String.raw to avoid escaping \.
    ///
    /// ### Why is this bad?
    ///
    /// Excessive backslashes can make string values less readable which can be avoided by using `String.raw`.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// const file = "C:\\windows\\style\\path\\to\\file.js";
    /// const regexp = new RegExp('foo\\.bar');
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// const file = String.raw`C:\windows\style\path\to\file.js`;
    /// const regexp = new RegExp(String.raw`foo\.bar`);
    /// ```
    PreferStringRaw,
    unicorn,
    style,
    fix,
);

fn unescape_backslash(input: &str, quote: char) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\'
            && let Some(next) = chars.peek()
            && (*next == '\\' || *next == quote)
        {
            result.push(*next);
            chars.next();
            continue;
        }

        result.push(c);
    }

    result
}

impl Rule for PreferStringRaw {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::StringLiteral(string_literal) = node.kind() else {
            return;
        };

        let parent_node = ctx.nodes().parent_node(node.id());

        match parent_node.kind() {
            // Skip string literals in type annotations - String.raw cannot be used in type positions
            AstKind::TSLiteralType(ts_literal_type) => {
                if let TSLiteral::StringLiteral(lit) = &ts_literal_type.literal
                    && lit.address() == string_literal.unstable_address()
                {
                    return;
                }
            }
            AstKind::Directive(_) => return,
            AstKind::ImportDeclaration(decl) => {
                if string_literal.span == decl.source.span {
                    return;
                }
            }
            AstKind::ExportNamedDeclaration(decl) => {
                if let Some(source) = &decl.source
                    && string_literal.span == source.span
                {
                    return;
                }
            }
            AstKind::ExportAllDeclaration(decl) => {
                if string_literal.span == decl.source.span {
                    return;
                }
            }
            AstKind::ObjectProperty(prop) => {
                let PropertyKey::StringLiteral(key) = &prop.key else {
                    return;
                };

                if !prop.computed && string_literal.span == key.span {
                    return;
                }
            }
            AstKind::JSXAttribute(attr) => {
                let Some(JSXAttributeValue::StringLiteral(value)) = &attr.value else {
                    return;
                };

                if value.span == string_literal.span {
                    return;
                }
            }
            AstKind::TSEnumMember(member) => {
                if member.span == string_literal.span {
                    return;
                }

                let TSEnumMemberName::String(id) = &member.id else {
                    return;
                };

                if id.span == string_literal.span {
                    return;
                }
            }
            _ => {}
        }

        let raw = ctx.source_range(string_literal.span);

        // Must contain escaped backslashes to be worth converting
        if !raw.contains(r"\\") {
            return;
        }

        let value = string_literal.value.as_str();

        // Cannot use String.raw if the value ends with an odd number of backslashes,
        // as the final backslash would escape the closing backtick.
        // e.g., String.raw`foo\` is invalid because \` escapes the backtick
        // but String.raw`foo\\` is valid (two literal backslashes)
        if has_odd_trailing_backslashes(value) {
            return;
        }

        // Cannot use String.raw if the value contains backticks (would need escaping)
        // or template literal syntax ${...} (would be interpreted as interpolation)
        if value.contains('`') || value.contains("${") {
            return;
        }

        let quote = raw.chars().next().unwrap_or('"');

        let trimmed = ctx.source_range(string_literal.span.shrink(1));

        let unescaped = unescape_backslash(trimmed, quote);

        if unescaped != value {
            return;
        }

        ctx.diagnostic_with_fix(prefer_string_raw(string_literal.span), |fixer| {
            let end = string_literal.span.start;
            let before = ctx.source_range(oxc_span::Span::new(0, end));

            let mut fix = format!("String.raw`{unescaped}`");

            if ends_with_keyword(before) {
                fix = format!(" {fix}");
            }

            fixer.replace(string_literal.span, fix)
        });
    }

    fn should_run(&self, ctx: &crate::context::ContextHost) -> bool {
        !ctx.source_type().is_typescript_definition()
    }
}

/// Returns true if the string ends with an odd number of backslashes.
/// This is used to detect cases where using String.raw would be invalid,
/// as an odd number of trailing backslashes would escape the closing backtick.
fn has_odd_trailing_backslashes(s: &str) -> bool {
    let count = s.chars().rev().take_while(|&c| c == '\\').count();
    count % 2 == 1
}

fn ends_with_keyword(source: &str) -> bool {
    for keyword in &RESERVED_KEYWORDS {
        if source.ends_with(keyword) {
            return true;
        }
    }

    if source.ends_with("of") {
        return true;
    }

    false
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass: Vec<&str> = vec![
        r"const file = String.raw`C:\windows\style\path\to\file.js`;",
        r"const regexp = new RegExp(String.raw`foo\.bar`);",
        r"a = '\''",
        r"'a\\b'",
        r#"import foo from "./foo\\bar.js";"#,
        r#"export {foo} from "./foo\\bar.js";"#,
        r#"export * from "./foo\\bar.js";"#,
        r"a = {'a\\b': 1}",
        "
            a = '\\\\a \\
                b'
         ",
        r"a = 'a\\b\u{51}c'",
        "a = 'a\\\\b`'",
        "a = 'a\\\\b${foo}'",
        r#"<Component attribute="a\\b" />"#,
        r#"
             enum Files {
                Foo = "C:\\\\path\\\\to\\\\foo.js",
             }
        "#,
        r#"
             enum Foo {
                "\\\\a\\\\b" = "baz",
             }
        "#,
        r"const a = 'a\\';",
        // Non-ASCII characters with trailing backslash - should NOT be fixed
        // (regression test for byte-length vs char-length bug)
        r"const a = 'c:\\someöäü\\';",
        r"const a = 'c:\\日本語\\';",
        // String literals in type annotations should not be changed (String.raw is not valid in type positions)
        r#"declare const POSIX_REGEX_SOURCE: { ascii: "\\x00-\\x7F"; };"#,
        r#"type Foo = { path: "C:\\windows\\path"; };"#,
        r#"interface Bar { regex: "foo\\.bar"; }"#,
        r#"let x: "a\\b";"#,
    ];

    let fail = vec![
        r#"const file = "C:\\windows\\style\\path\\to\\file.js";"#,
        r"const regexp = new RegExp('foo\\.bar');",
        r"a = 'a\\b'",
        r"a = {['a\\b']: b}",
        r"function a() {return'a\\b'}",
        r"function* a() {yield'a\\b'}",
        r"function a() {throw'a\\b'}",
        r"if (typeof'a\\b' === 'string') {}",
        r"const a = () => void'a\\b';",
        r"const foo = 'foo \\x46';",
        r"for (const f of'a\\b') {}",
        // Non-ASCII characters without trailing backslash - should be fixed
        r"const a = 'c:\\someöäü\\path';",
    ];

    let fix = vec![
        (
            r#"const file = "C:\\windows\\style\\path\\to\\file.js";"#,
            r"const file = String.raw`C:\windows\style\path\to\file.js`;",
            None,
        ),
        (
            r"const regexp = new RegExp('foo\\.bar');",
            r"const regexp = new RegExp(String.raw`foo\.bar`);",
            None,
        ),
        (r"a = 'a\\b'", r"a = String.raw`a\b`", None),
        (r"a = {['a\\b']: b}", r"a = {[String.raw`a\b`]: b}", None),
        (r"function a() {return'a\\b'}", r"function a() {return String.raw`a\b`}", None),
        (r"const foo = 'foo \\x46';", r"const foo = String.raw`foo \x46`;", None),
        (r"for (const f of'a\\b') {}", r"for (const f of String.raw`a\b`) {}", None),
        (r"a = 'a\\b'", r"a = String.raw`a\b`", None),
        (r"a = {['a\\b']: b}", r"a = {[String.raw`a\b`]: b}", None),
        (r"function a() {return'a\\b'}", r"function a() {return String.raw`a\b`}", None),
        (r"function* a() {yield'a\\b'}", r"function* a() {yield String.raw`a\b`}", None),
        (r"function a() {throw'a\\b'}", r"function a() {throw String.raw`a\b`}", None),
        (
            r"if (typeof'a\\b' === 'string') {}",
            r"if (typeof String.raw`a\b` === 'string') {}",
            None,
        ),
        (r"const a = () => void'a\\b';", r"const a = () => void String.raw`a\b`;", None),
        (r"const foo = 'foo \\x46';", r"const foo = String.raw`foo \x46`;", None),
        (r"for (const f of'a\\b') {}", r"for (const f of String.raw`a\b`) {}", None),
        // Non-ASCII characters
        (r"const a = 'c:\\someöäü\\path';", r"const a = String.raw`c:\someöäü\path`;", None),
    ];

    Tester::new(PreferStringRaw::NAME, PreferStringRaw::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();

    let dts_pass = vec![r#"declare const POSIX_REGEX_SOURCE: { ascii: "\\x00-\\x7F"; };"#];
    let dts_fail: Vec<&str> = vec![];

    Tester::new(PreferStringRaw::NAME, PreferStringRaw::PLUGIN, dts_pass, dts_fail)
        .change_rule_path("test.d.ts")
        .intentionally_allow_no_fix_tests()
        .test();
}
