use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use oxc_ast::{AstKind, ast::JSXAttributeName};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::NodeId;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

fn text_encoding_identifier_case_diagnostic(
    span: Span,
    good_encoding: &str,
    bad_encoding: &str,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Prefer `{good_encoding}` over `{bad_encoding}`.")).with_label(span)
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct TextEncodingIdentifierCase {
    /// If `true`, prefer `utf-8` over `utf8`.
    with_dash: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule enforces consistent casing for text encoding identifiers, specifically:
    /// - `'utf8'` instead of `'UTF-8'` or `'utf-8'` (or `'utf-8'` if `withDash` is enabled)
    /// - `'ascii'` instead of `'ASCII'`
    ///
    /// ### Why is this bad?
    ///
    /// Inconsistent casing of encoding identifiers reduces code readability and
    /// can lead to subtle confusion across a codebase. Although casing is not
    /// strictly enforced by ECMAScript or Node.js, using lowercase is the
    /// conventional and widely recognized style.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// import fs from 'node:fs/promises';
    /// async function bad() {
    ///   await fs.readFile(file, 'UTF-8');
    ///   await fs.readFile(file, 'ASCII');
    ///   const string = buffer.toString('utf-8');
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// import fs from 'node:fs/promises';
    /// async function good() {
    ///   await fs.readFile(file, 'utf8');
    ///   await fs.readFile(file, 'ascii');
    ///   const string = buffer.toString('utf8');
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule with `{ "withDash": true }`:
    /// ```javascript
    /// import fs from 'node:fs/promises';
    /// async function good() {
    ///   await fs.readFile(file, 'utf-8');
    ///   await fs.readFile(file, 'ascii');
    ///   const string = buffer.toString('utf-8');
    /// }
    /// ```
    TextEncodingIdentifierCase,
    unicorn,
    style,
    fix,
    config = TextEncodingIdentifierCase
);

impl Rule for TextEncodingIdentifierCase {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let (s, span) = match node.kind() {
            AstKind::StringLiteral(string_lit) => (string_lit.value.as_str(), string_lit.span),
            AstKind::JSXText(jsx_text) => (jsx_text.value.as_str(), jsx_text.span),
            _ => return,
        };

        let with_dash = self.with_dash || should_enforce_dash(node.id(), ctx);

        if let Some(replacement) = get_replacement(s, with_dash)
            && replacement != s
        {
            ctx.diagnostic_with_fix(
                text_encoding_identifier_case_diagnostic(span, replacement, s),
                |fixer| fixer.replace(Span::new(span.start + 1, span.end - 1), replacement),
            );
        }
    }
}

fn get_replacement(encoding: &str, with_dash: bool) -> Option<&'static str> {
    if encoding.eq_ignore_ascii_case("utf8") || encoding.eq_ignore_ascii_case("utf-8") {
        Some(if with_dash { "utf-8" } else { "utf8" })
    } else if encoding.eq_ignore_ascii_case("ascii") {
        Some("ascii")
    } else {
        None
    }
}

/// Check if this context requires `utf-8` (with dash) regardless of the option.
fn should_enforce_dash(id: NodeId, ctx: &LintContext) -> bool {
    is_jsx_element_with_charset_attr(id, ctx, "meta", &["charset"])
        || is_jsx_element_with_charset_attr(id, ctx, "form", &["acceptcharset", "accept-charset"])
        || is_text_decoder_argument(id, ctx)
}

/// Check if this is the first argument to `new TextDecoder()`.
fn is_text_decoder_argument(id: NodeId, ctx: &LintContext) -> bool {
    let parent = ctx.nodes().parent_node(id);
    let AstKind::NewExpression(new_expr) = parent.kind() else {
        return false;
    };

    // Check that this is the first argument
    let Some(first_arg) = new_expr.arguments.first() else {
        return false;
    };
    if !first_arg.span().contains_inclusive(ctx.nodes().get_node(id).span()) {
        return false;
    }

    // Check that the callee is `TextDecoder`
    new_expr.callee.get_identifier_reference().is_some_and(|ident| ident.name == "TextDecoder")
}

fn is_jsx_element_with_charset_attr(
    id: NodeId,
    ctx: &LintContext,
    element: &str,
    attributes: &[&str],
) -> bool {
    let parent = ctx.nodes().parent_node(id);
    let AstKind::JSXAttribute(jsx_attr) = parent.kind() else {
        return false;
    };
    let JSXAttributeName::Identifier(ident) = &jsx_attr.name else {
        return false;
    };

    if !attributes.iter().any(|attr| ident.name.eq_ignore_ascii_case(attr)) {
        return false;
    }
    let AstKind::JSXOpeningElement(opening_elem) = ctx.nodes().parent_kind(parent.id()) else {
        return false;
    };
    opening_elem
        .name
        .get_identifier_name()
        .is_some_and(|tag_name| tag_name.eq_ignore_ascii_case(element))
}

#[test]
fn test() {
    use crate::tester::Tester;
    use serde_json::json;

    let with_dash = Some(json!([{ "withDash": true }]));
    let no_dash = Some(json!([{ "withDash": false }]));

    let pass = vec![
        // Default (no dash)
        (r"`UTF-8`", None),
        (r#""utf8""#, None),
        (r#""utf+8""#, None),
        (r#""   utf8   ""#, None),
        ("\'utf8\'", None),
        (r#""\\u0055tf8""#, None),
        (r"const ASCII = 1", None),
        (r"const UTF8 = 1", None),
        // with dash
        (r"`Utf-8`;", with_dash.clone()),
        (r#""utf-8";"#, with_dash.clone()),
        (r#""   Utf8   ";"#, with_dash.clone()),
        (r"'utf-8';", with_dash.clone()),
        (r"const utf8 = 2;", with_dash.clone()),
        // Cases that always require utf-8 (with dash)
        (r#"<meta charset="utf-8" />"#, None),
        (r#"<META CHARSET="utf-8" />"#, None),
        (r#"<form acceptCharset="utf-8" />"#, None),
        (r#"<form accept-charset="utf-8" />"#, None),
        (r#"new TextDecoder("utf-8")"#, None),
        // Non-matching elements with utf8 (no dash)
        (r#"<not-meta charset="utf8" />"#, None),
        (r#"<not-meta notCharset="utf8" />"#, None),
        // with_dash: true
        (r#"<meta charset="utf-8" />"#, with_dash.clone()),
        (r#"<form acceptCharset="utf-8" />"#, with_dash.clone()),
        (r#"new TextDecoder("utf-8")"#, with_dash.clone()),
        (r#"<not-meta charset="utf-8" />"#, with_dash.clone()),
        (r#"<not-meta notCharset="utf-8" />"#, with_dash.clone()),
        (r#"<meta charset="utf-8" />"#, with_dash.clone()),
        (r#"<meta charset="utf-8" />"#, no_dash.clone()),
        (r#"<META CHARSET="utf-8" />"#, with_dash.clone()),
        (r#"<META CHARSET="utf-8" />"#, no_dash.clone()),
        (r#"<form acceptCharset="utf-8" />"#, with_dash.clone()),
        (r#"<form acceptCharset="utf-8" />"#, no_dash.clone()),
        (r#"<form accept-charset="utf-8" />"#, with_dash.clone()),
        (r#"<form accept-charset="utf-8" />"#, no_dash.clone()),
        (r#"new TextDecoder("utf-8")"#, with_dash.clone()),
        (r#"new TextDecoder("utf-8")"#, no_dash.clone()),
        (r#"<not-meta charset="utf-8" />"#, with_dash.clone()),
        (r#"<not-meta notCharset="utf-8" />"#, with_dash.clone()),
        (r#"<not-meta charset="utf8" />"#, no_dash.clone()),
        (r#"<not-meta notCharset="utf8" />"#, no_dash.clone()),
    ];

    let fail = vec![
        // Default (no dash)
        (r#""UTF-8""#, None),
        (r#""utf-8""#, None),
        (r"'utf-8'", None),
        (r#""Utf8""#, None),
        (r#""ASCII""#, None),
        (r#"fs.readFile?.(file, "UTF-8")"#, None),
        (r#"fs?.readFile(file, "UTF-8")"#, None),
        (r#"readFile(file, "UTF-8")"#, None),
        (r#"fs.readFile(...file, "UTF-8")"#, None),
        (r#"new fs.readFile(file, "UTF-8")"#, None),
        (r#"fs.readFile(file, {encoding: "UTF-8"})"#, None),
        (r#"fs.readFile("UTF-8")"#, None),
        (r#"fs.readFile(file, "UTF-8", () => {})"#, None),
        (r#"fs.readFileSync(file, "UTF-8")"#, None),
        (r#"fs[readFile](file, "UTF-8")"#, None),
        (r#"fs["readFile"](file, "UTF-8")"#, None),
        (r#"import fs from 'fs'; await fs.readFile(file, "UTF-8",)"#, None),
        (r#"fs.promises.readFile(file, "UTF-8",)"#, None),
        (r#"whatever.readFile(file, "UTF-8",)"#, None),
        // with_dash: true
        (r#""UTF-8";"#, with_dash.clone()),
        (r#""UTF8";"#, with_dash.clone()),
        (r#""utf8";"#, with_dash.clone()),
        (r"'utf8';", with_dash.clone()),
        (r#""Utf8";"#, with_dash.clone()),
        (r#""ASCII";"#, with_dash.clone()),
        (r#"fs.readFile(file, "utf8",);"#, with_dash.clone()),
        (r#"whatever.readFile(file, "UTF8",)"#, with_dash.clone()),
        // JSX cases that need utf-8 but have wrong value
        (r#"<meta charset="ASCII" />"#, with_dash.clone()),
        (r#"<meta charset="ASCII" />"#, no_dash.clone()),
        (r#"<META CHARSET="ASCII" />"#, with_dash.clone()),
        (r#"<META CHARSET="ASCII" />"#, no_dash.clone()),
        (r#"<meta charset="utf8" />"#, with_dash.clone()),
        (r#"<meta charset="utf8" />"#, no_dash.clone()),
        (r#"<meta charset="UTF-8" />"#, with_dash.clone()),
        (r#"<meta charset="UTF-8" />"#, no_dash.clone()),
        (r#"<form acceptCharset="utf8" />"#, with_dash.clone()),
        (r#"<form acceptCharset="utf8" />"#, no_dash.clone()),
        (r#"<form accept-charset="UTF-8" />"#, with_dash.clone()),
        (r#"<form accept-charset="UTF-8" />"#, no_dash.clone()),
        (r#"new TextDecoder("UTF-8")"#, with_dash.clone()),
        (r#"new TextDecoder("UTF-8")"#, no_dash.clone()),
        (r#"new TextDecoder("UTF-8", options)"#, with_dash.clone()),
        (r#"new TextDecoder("UTF-8", options)"#, no_dash.clone()),
        (r#"<not-meta charset="utf8" />"#, with_dash.clone()),
        (r#"<not-meta notCharset="utf8" />"#, with_dash.clone()),
        (r#"<not-meta charset="utf-8" />"#, no_dash.clone()),
        (r#"<not-meta notCharset="utf-8" />"#, no_dash),
    ];

    let fix = vec![
        // Default (no dash)
        (r#""UTF-8""#, r#""utf8""#, None),
        (r#""utf-8""#, r#""utf8""#, None),
        (r"'utf-8'", r"'utf8'", None),
        (r#""Utf8""#, r#""utf8""#, None),
        (r#""ASCII""#, r#""ascii""#, None),
        (r#"fs.readFile?.(file, "UTF-8")"#, r#"fs.readFile?.(file, "utf8")"#, None),
        (r#"fs?.readFile(file, "UTF-8")"#, r#"fs?.readFile(file, "utf8")"#, None),
        (r#"readFile(file, "UTF-8")"#, r#"readFile(file, "utf8")"#, None),
        (r#"fs.readFile(...file, "UTF-8")"#, r#"fs.readFile(...file, "utf8")"#, None),
        (r#"new fs.readFile(file, "UTF-8")"#, r#"new fs.readFile(file, "utf8")"#, None),
        (
            r#"fs.readFile(file, {encoding: "UTF-8"})"#,
            r#"fs.readFile(file, {encoding: "utf8"})"#,
            None,
        ),
        (r#"fs.readFile("UTF-8")"#, r#"fs.readFile("utf8")"#, None),
        (r#"fs.readFile(file, "UTF-8", () => {})"#, r#"fs.readFile(file, "utf8", () => {})"#, None),
        (r#"fs.readFileSync(file, "UTF-8")"#, r#"fs.readFileSync(file, "utf8")"#, None),
        (r#"fs[readFile](file, "UTF-8")"#, r#"fs[readFile](file, "utf8")"#, None),
        (r#"fs["readFile"](file, "UTF-8")"#, r#"fs["readFile"](file, "utf8")"#, None),
        (
            r#"import fs from 'fs'; await fs.readFile(file, "UTF-8",)"#,
            r#"import fs from 'fs'; await fs.readFile(file, "utf8",)"#,
            None,
        ),
        (r#"fs.promises.readFile(file, "UTF-8",)"#, r#"fs.promises.readFile(file, "utf8",)"#, None),
        (r#"whatever.readFile(file, "UTF-8",)"#, r#"whatever.readFile(file, "utf8",)"#, None),
        // JSX cases that need utf-8
        (r#"<meta charset="utf8" />"#, r#"<meta charset="utf-8" />"#, None),
        (r#"<meta charset="UTF-8" />"#, r#"<meta charset="utf-8" />"#, None),
        (r#"<form acceptCharset="utf8" />"#, r#"<form acceptCharset="utf-8" />"#, None),
        (r#"<form accept-charset="UTF-8" />"#, r#"<form accept-charset="utf-8" />"#, None),
        (r#"new TextDecoder("UTF-8")"#, r#"new TextDecoder("utf-8")"#, None),
        (r#"new TextDecoder("UTF-8", options)"#, r#"new TextDecoder("utf-8", options)"#, None),
        // JSX non-matching elements (should use utf8)
        (r#"<not-meta charset="utf-8" />"#, r#"<not-meta charset="utf8" />"#, None),
        (r#"<not-meta notCharset="utf-8" />"#, r#"<not-meta notCharset="utf8" />"#, None),
        (r#"<meta not-charset="utf-8" />"#, r#"<meta not-charset="utf8" />"#, None),
        (r#"<meta charset="ASCII" />"#, r#"<meta charset="ascii" />"#, None),
        (r#"<META CHARSET="ASCII" />"#, r#"<META CHARSET="ascii" />"#, None),
        // with_dash: true
        (r#""UTF-8";"#, r#""utf-8";"#, with_dash.clone()),
        (r#""UTF8";"#, r#""utf-8";"#, with_dash.clone()),
        (r#""utf8";"#, r#""utf-8";"#, with_dash.clone()),
        (r"'utf8';", r"'utf-8';", with_dash.clone()),
        (r#""Utf8";"#, r#""utf-8";"#, with_dash.clone()),
        (r#""ASCII";"#, r#""ascii";"#, with_dash.clone()),
        (r#"fs.readFile(file, "utf8",);"#, r#"fs.readFile(file, "utf-8",);"#, with_dash.clone()),
        (
            r#"whatever.readFile(file, "UTF8",)"#,
            r#"whatever.readFile(file, "utf-8",)"#,
            with_dash.clone(),
        ),
        (r#"<meta charset="utf8" />"#, r#"<meta charset="utf-8" />"#, with_dash.clone()),
        (r#"<meta charset="UTF-8" />"#, r#"<meta charset="utf-8" />"#, with_dash.clone()),
        (
            r#"<form acceptCharset="utf8" />"#,
            r#"<form acceptCharset="utf-8" />"#,
            with_dash.clone(),
        ),
        (r#"new TextDecoder("UTF-8")"#, r#"new TextDecoder("utf-8")"#, with_dash.clone()),
        (r#"<not-meta charset="utf8" />"#, r#"<not-meta charset="utf-8" />"#, with_dash.clone()),
        (r#"<not-meta notCharset="utf8" />"#, r#"<not-meta notCharset="utf-8" />"#, with_dash),
    ];

    Tester::new(TextEncodingIdentifierCase::NAME, TextEncodingIdentifierCase::PLUGIN, pass, fail)
        .change_rule_path_extension("jsx")
        .expect_fix(fix)
        .test_and_snapshot();
}
