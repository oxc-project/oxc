use cow_utils::CowUtils;
use oxc_ast::{
    ast::{JSXAttributeItem, JSXAttributeName},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::NodeId;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn text_encoding_identifier_case_diagnostic(
    span: Span,
    good_encoding: &str,
    bad_encoding: &str,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Prefer `{good_encoding}` over `{bad_encoding}`.")).with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct TextEncodingIdentifierCase;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule aims to enforce consistent case for text encoding identifiers.
    ///
    /// Enforces `'utf8'` for UTF-8 encoding
    /// Enforces `'ascii'` for ASCII encoding.
    ///
    /// ### Why is this bad?
    ///
    /// - Inconsistency in text encoding identifiers can make the code harder to read and understand.
    /// - The ECMAScript specification does not define the case sensitivity of text encoding identifiers, but it is common practice to use lowercase.
    ///
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// import fs from 'node:fs/promises';
    /// async function bad() {
    ///     await fs.readFile(file, 'UTF-8');
    ///
    ///     await fs.readFile(file, 'ASCII');
    ///
    ///     const string = buffer.toString('utf-8');
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// async function good() {
    ///     await fs.readFile(file, 'utf8');
    ///
    ///     await fs.readFile(file, 'ascii');
    ///
    ///     const string = buffer.toString('utf8');
    /// }
    /// ```
    TextEncodingIdentifierCase,
    unicorn,
    style,
    fix
);

impl Rule for TextEncodingIdentifierCase {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let (s, span) = match node.kind() {
            AstKind::StringLiteral(string_lit) => (&string_lit.value, string_lit.span),
            AstKind::JSXText(jsx_text) => (&jsx_text.value, jsx_text.span),
            _ => {
                return;
            }
        };
        let s = s.as_str();

        if s == "utf-8" && is_jsx_meta_elem_with_charset_attr(node.id(), ctx) {
            return;
        }

        let Some(replacement) = get_replacement(s) else {
            return;
        };

        if replacement == s {
            return;
        }

        ctx.diagnostic_with_fix(
            text_encoding_identifier_case_diagnostic(span, replacement, s),
            |fixer| fixer.replace(Span::new(span.start + 1, span.end - 1), replacement),
        );
    }
}

fn get_replacement(node: &str) -> Option<&'static str> {
    if !matches!(node.len(), 4 | 5) {
        return None;
    }

    let node_lower = node.cow_to_ascii_lowercase();

    if node_lower == "utf-8" || node_lower == "utf8" {
        return Some("utf8");
    }

    if node_lower == "ascii" {
        return Some("ascii");
    }

    None
}

fn is_jsx_meta_elem_with_charset_attr(id: NodeId, ctx: &LintContext) -> bool {
    let Some(parent) = ctx.nodes().parent_node(id) else {
        return false;
    };

    let AstKind::JSXAttributeItem(JSXAttributeItem::Attribute(jsx_attr)) = parent.kind() else {
        return false;
    };

    let JSXAttributeName::Identifier(ident) = &jsx_attr.name else {
        return false;
    };
    if !ident.name.eq_ignore_ascii_case("charset") {
        return false;
    }

    let Some(AstKind::JSXOpeningElement(opening_elem)) = ctx.nodes().parent_kind(parent.id())
    else {
        return false;
    };

    let Some(tag_name) = opening_elem.name.get_identifier_name() else { return false };

    if !tag_name.eq_ignore_ascii_case("meta") {
        return false;
    }

    true
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"`UTF-8`",
        r#""utf8""#,
        r#""utf+8""#,
        r#""   utf8   ""#,
        "\'utf8\'",
        r#""\\u0055tf8""#,
        r"const ASCII = 1",
        r"const UTF8 = 1",
        r#"<meta charset="utf-8" />"#,
        r#"<META CHARSET="utf-8" />"#,
    ];

    let fail = vec![
        r#""UTF-8""#,
        r#""utf-8""#,
        r"'utf-8'",
        r#""Utf8""#,
        r#""ASCII""#,
        r#"fs.readFile?.(file, "UTF-8")"#,
        r#"fs?.readFile(file, "UTF-8")"#,
        r#"readFile(file, "UTF-8")"#,
        r#"fs.readFile(...file, "UTF-8")"#,
        r#"new fs.readFile(file, "UTF-8")"#,
        r#"fs.readFile(file, {encoding: "UTF-8"})"#,
        r#"fs.readFile("UTF-8")"#,
        r#"fs.readFile(file, "UTF-8", () => {})"#,
        r#"fs.readFileSync(file, "UTF-8")"#,
        r#"fs[readFile](file, "UTF-8")"#,
        r#"fs["readFile"](file, "UTF-8")"#,
        r#"await fs.readFile(file, "UTF-8",)"#,
        r#"fs.promises.readFile(file, "UTF-8",)"#,
        r#"whatever.readFile(file, "UTF-8",)"#,
        r#"<not-meta charset="utf-8" />"#,
        r#"<meta not-charset="utf-8" />"#,
        r#"<meta charset="ASCII" />"#,
        r#"<META CHARSET="ASCII" />"#,
    ];

    let fix = vec![
        (r#""UTF-8""#, r#""utf8""#),
        (r#""utf-8""#, r#""utf8""#),
        (r"'utf-8'", r"'utf8'"),
        (r#""Utf8""#, r#""utf8""#),
        (r#""ASCII""#, r#""ascii""#),
        (r#"fs.readFile?.(file, "UTF-8")"#, r#"fs.readFile?.(file, "utf8")"#),
        (r#"fs?.readFile(file, "UTF-8")"#, r#"fs?.readFile(file, "utf8")"#),
        (r#"readFile(file, "UTF-8")"#, r#"readFile(file, "utf8")"#),
        (r#"fs.readFile(...file, "UTF-8")"#, r#"fs.readFile(...file, "utf8")"#),
        (r#"new fs.readFile(file, "UTF-8")"#, r#"new fs.readFile(file, "utf8")"#),
        (r#"fs.readFile(file, {encoding: "UTF-8"})"#, r#"fs.readFile(file, {encoding: "utf8"})"#),
        (r#"fs.readFile("UTF-8")"#, r#"fs.readFile("utf8")"#),
        (r#"fs.readFile(file, "UTF-8", () => {})"#, r#"fs.readFile(file, "utf8", () => {})"#),
        (r#"fs.readFileSync(file, "UTF-8")"#, r#"fs.readFileSync(file, "utf8")"#),
        (r#"fs[readFile](file, "UTF-8")"#, r#"fs[readFile](file, "utf8")"#),
        (r#"fs["readFile"](file, "UTF-8")"#, r#"fs["readFile"](file, "utf8")"#),
        (r#"await fs.readFile(file, "UTF-8",)"#, r#"await fs.readFile(file, "utf8",)"#),
        (r#"fs.promises.readFile(file, "UTF-8",)"#, r#"fs.promises.readFile(file, "utf8",)"#),
        (r#"whatever.readFile(file, "UTF-8",)"#, r#"whatever.readFile(file, "utf8",)"#),
        (r#"<not-meta charset="utf-8" />"#, r#"<not-meta charset="utf8" />"#),
        (r#"<meta not-charset="utf-8" />"#, r#"<meta not-charset="utf8" />"#),
        (r#"<meta charset="ASCII" />"#, r#"<meta charset="ascii" />"#),
        (r#"<META CHARSET="ASCII" />"#, r#"<META CHARSET="ascii" />"#),
    ];

    Tester::new(TextEncodingIdentifierCase::NAME, TextEncodingIdentifierCase::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
