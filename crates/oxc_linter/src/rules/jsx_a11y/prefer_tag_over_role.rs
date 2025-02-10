use lazy_static::lazy_static;
use phf::phf_map;

use oxc_ast::{
    ast::{JSXAttributeItem, JSXAttributeValue},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{get_element_type, has_jsx_prop_ignore_case},
    AstNode,
};

fn prefer_tag_over_role_diagnostic(span: Span, tag: &str, role: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Prefer `{tag}` over `role` attribute `{role}`."))
        .with_help(format!("Replace HTML elements with `role` attribute `{role}` to corresponding semantic HTML tag `{tag}`."))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferTagOverRole;

declare_oxc_lint!(
    /// ### What it does
    /// Enforces using semantic HTML tags over `role` attribute.
    ///
    /// ### Why is this bad?
    /// Using semantic HTML tags can improve accessibility and readability of the code.
    ///
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <div role="button" />
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <button />
    /// ```
    PreferTagOverRole,
    jsx_a11y,
    correctness
);

impl PreferTagOverRole {
    fn check_roles<'a>(
        role_prop: &JSXAttributeItem<'a>,
        role_to_tag: &phf::Map<&str, &str>,
        jsx_name: &str,
        ctx: &LintContext<'a>,
    ) {
        if let JSXAttributeItem::Attribute(attr) = role_prop {
            if let Some(JSXAttributeValue::StringLiteral(role_values)) = &attr.value {
                let roles = role_values.value.split_whitespace();
                for role in roles {
                    Self::check_role(role, role_to_tag, jsx_name, attr.span, ctx);
                }
            }
        }
    }

    fn check_role(
        role: &str,
        role_to_tag: &phf::Map<&str, &str>,
        jsx_name: &str,
        span: Span,
        ctx: &LintContext,
    ) {
        if let Some(tag) = role_to_tag.get(role) {
            if jsx_name != *tag {
                ctx.diagnostic(prefer_tag_over_role_diagnostic(span, tag, role));
            }
        }
    }
}

lazy_static! {
    static ref ROLE_TO_TAG_MAP: phf::Map<&'static str, &'static str> = phf_map! {
        "checkbox" => "input",
        "button" => "button",
        "heading" => "h1,h2,h3,h4,h5,h6",
        "link" => "a,area",
        "rowgroup" => "tbody,tfoot,thead",
        "banner" => "header",
    };
}

impl Rule for PreferTagOverRole {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::JSXOpeningElement(jsx_el) = node.kind() {
            let name = get_element_type(ctx, jsx_el);
            if let Some(role_prop) = has_jsx_prop_ignore_case(jsx_el, "role") {
                Self::check_roles(role_prop, &ROLE_TO_TAG_MAP, &name, ctx);
            }
        }
    }
}
#[test]
fn test() {
    use crate::tester::Tester;
    let pass = vec![
        "<div />",
        "<div role=\"unknown\" />",
        "<div role=\"also unknown\" />",
        "<other />",
        "<img role=\"img\" />",
        "<input role=\"checkbox\" />",
    ];
    let fail: Vec<&str> = vec![
        r#"<div role="checkbox" />"#,
        r#"<div role="button checkbox" />"#,
        r#"<div role="heading" />"#,
        r#"<div role="link" />"#,
        r#"<div role="rowgroup" />"#,
        r#"<span role="checkbox" />"#,
        r#"<other role="checkbox" />"#,
        r#"<other role="checkbox" />"#,
        r#"<div role="banner" />"#,
    ];
    Tester::new(PreferTagOverRole::NAME, PreferTagOverRole::PLUGIN, pass, fail).test_and_snapshot();
}
