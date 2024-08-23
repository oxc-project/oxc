use oxc_ast::{
    ast::{JSXAttributeItem, JSXAttributeName, JSXElementName},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNodeId;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn text_encoding_identifier_case_diagnostic(span0: Span, x1: &str, x2: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Prefer `{x1}` over `{x2}`.")).with_label(span0)
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
    /// ### Example
    /// ```javascript
    /// import fs from 'node:fs/promises';
    /// async function bad() {
    ///     await fs.readFile(file, 'UTF-8');
    ///
    ///     await fs.readFile(file, 'ASCII');
    ///
    ///     const string = buffer.toString('utf-8');
    /// }
    ///
    /// async function good() {
    ///     await fs.readFile(file, 'utf8');
    ///
    ///     await fs.readFile(file, 'ascii');
    ///
    ///     const string = buffer.toString('utf8');
    /// }
    ///
    /// ```
    TextEncodingIdentifierCase,
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

    let node_lower = node.to_ascii_lowercase();

    if node_lower == "utf-8" || node_lower == "utf8" {
        return Some("utf8");
    }

    if node_lower == "ascii" {
        return Some("ascii");
    }

    None
}

fn is_jsx_meta_elem_with_charset_attr(id: AstNodeId, ctx: &LintContext) -> bool {
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

    let JSXElementName::Identifier(name) = &opening_elem.name else {
        return false;
    };

    if !name.name.eq_ignore_ascii_case("meta") {
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

    Tester::new(TextEncodingIdentifierCase::NAME, pass, fail).expect_fix(fix).test_and_snapshot();
}
