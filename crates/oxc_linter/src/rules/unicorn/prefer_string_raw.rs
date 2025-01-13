use oxc_ast::{
    ast::{JSXAttributeValue, PropertyKey, TSEnumMemberName},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_ecmascript::StringCharAt;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::keyword::RESERVED_KEYWORDS;

use crate::{context::LintContext, rule::Rule, AstNode};

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
    /// ### Example
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
        if c == '\\' {
            if let Some(next) = chars.peek() {
                if *next == '\\' || *next == quote {
                    result.push(*next);
                    chars.next();
                    continue;
                }
            }
        }

        result.push(c);
    }

    result
}

impl Rule for PreferStringRaw {
    #[allow(clippy::cast_precision_loss)]
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::StringLiteral(string_literal) = node.kind() else {
            return;
        };

        let parent_node = ctx.nodes().parent_node(node.id());

        if let Some(parent_node) = parent_node {
            match parent_node.kind() {
                AstKind::Directive(_) => {
                    return;
                }
                AstKind::ImportDeclaration(decl) => {
                    if string_literal.span == decl.source.span {
                        return;
                    }
                }
                AstKind::ExportNamedDeclaration(decl) => {
                    if let Some(source) = &decl.source {
                        if string_literal.span == source.span {
                            return;
                        }
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
                AstKind::PropertyKey(_) => {
                    if let Some(AstKind::ObjectProperty(prop)) =
                        ctx.nodes().parent_node(parent_node.id()).map(AstNode::kind)
                    {
                        let PropertyKey::StringLiteral(key) = &prop.key else {
                            return;
                        };

                        if !prop.computed && key.span == string_literal.span {
                            return;
                        }
                    }
                }
                AstKind::JSXAttributeItem(attr) => {
                    let Some(attr) = attr.as_attribute() else {
                        return;
                    };

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
                    };

                    let TSEnumMemberName::String(id) = &member.id else {
                        return;
                    };

                    if id.span == string_literal.span {
                        return;
                    }
                }
                _ => {}
            }
        }

        let raw = ctx.source_range(string_literal.span);

        let last_char_index = raw.len() - 2;
        if raw.char_at(Some(last_char_index as f64)) == Some('\\') {
            return;
        }

        if !raw.contains(r"\\") || raw.contains('`') || raw.contains("${") {
            return;
        }

        let Some(quote) = raw.char_at(Some(0.0)) else {
            return;
        };

        let trimmed = ctx.source_range(string_literal.span.shrink(1));

        let unescaped = unescape_backslash(trimmed, quote);

        if unescaped != string_literal.value.as_ref() {
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
    ];

    Tester::new(PreferStringRaw::NAME, PreferStringRaw::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
