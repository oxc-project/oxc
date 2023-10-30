use oxc_ast::{
    ast::{JSXAttributeItem, JSXAttributeName, JSXElementName},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{self, Error},
};
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNodeId;
use oxc_span::{Atom, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-unicorn(text-encoding-identifier-case): Prefer `{1}` over `{2}`.")]
#[diagnostic(severity(warning))]
struct TextEncodingIdentifierCaseDiagnostic(#[label] pub Span, pub &'static str, pub Atom);

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
    /// // Fail
    /// await fs.readFile(file, 'UTF-8');
    ///
    /// await fs.readFile(file, 'ASCII');
    ///
    /// const string = buffer.toString('utf-8');
    ///
    /// // pass
    ///
    /// await fs.readFile(file, 'utf8');
    ///
    /// await fs.readFile(file, 'ascii');
    ///
    /// const string = buffer.toString('utf8');
    ///
    /// ```
    TextEncodingIdentifierCase,
    style
);

impl Rule for TextEncodingIdentifierCase {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let (str, span) = match node.kind() {
            AstKind::StringLiteral(string_lit) => (&string_lit.value, string_lit.span),
            AstKind::JSXString(jsx_string) => (&jsx_string.value, jsx_string.span),
            _ => {
                return;
            }
        };

        if str.as_str() == "utf-8" && is_jsx_meta_elem_with_charset_attr(node.id(), ctx) {
            return;
        }

        let Some(replacement) = get_replacement(str) else {
            return;
        };

        if replacement == str.as_str() {
            return;
        }

        ctx.diagnostic(TextEncodingIdentifierCaseDiagnostic(span, replacement, str.clone()));
    }
}

fn get_replacement(node: &Atom) -> Option<&'static str> {
    if !matches!(node.as_str().len(), 4 | 5) {
        return None;
    }

    let node_lower = node.as_str().to_ascii_lowercase();

    if node_lower == "utf-8" || node_lower == "utf8" {
        return Some("utf8");
    }

    if node_lower == "ascii" {
        return Some("ascii");
    }

    None
}

fn is_jsx_meta_elem_with_charset_attr(id: AstNodeId, ctx: &LintContext) -> bool {
    let Some(parent) = ctx.nodes().parent_node(id) else { return false };

    let AstKind::JSXAttributeItem(JSXAttributeItem::Attribute(jsx_attr)) = parent.kind() else {
        return false;
    };

    let JSXAttributeName::Identifier(ident) = &jsx_attr.name else { return false };
    if ident.name.to_lowercase() != "charset" {
        return false;
    }

    let Some(AstKind::JSXOpeningElement(opening_elem)) = ctx.nodes().parent_kind(parent.id())
    else {
        return false;
    };

    let JSXElementName::Identifier(name) = &opening_elem.name else { return false };

    if name.name.to_lowercase() != "meta" {
        return false;
    }

    true
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"`UTF-8`"#,
        r#""utf8""#,
        r#""utf+8""#,
        r#""   utf8   ""#,
        "\'utf8\'",
        r#""\\u0055tf8""#,
        r#"const ASCII = 1"#,
        r#"const UTF8 = 1"#,
        r#"<meta charset="utf-8" />"#,
        r#"<META CHARSET="utf-8" />"#,
    ];

    let fail = vec![
        r#""UTF-8""#,
        r#""utf-8""#,
        r#"'utf-8'"#,
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

    Tester::new_without_config(TextEncodingIdentifierCase::NAME, pass, fail).test_and_snapshot();
}
