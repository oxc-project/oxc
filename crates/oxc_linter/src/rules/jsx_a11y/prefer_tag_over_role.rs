use crate::{context::LintContext, rule::Rule, utils::has_jsx_prop_lowercase, AstNode};
use once_cell::sync::Lazy;
use oxc_ast::{
    ast::{JSXAttributeItem, JSXAttributeValue, JSXElementName},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{self, Error},
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use phf::phf_map;

#[derive(Debug, Error, Diagnostic)]
#[error(
    "eslint-plugin-jsx-a11y(prefer-tag-over-role): Prefer `{tag}` over `role` attribute `{role}`."
)]
#[diagnostic(
    severity(warning),
    help("Replace HTML elements with `role` attribute `{role}` to corresponding semantic HTML tag `{tag}`.")
)]
struct PreferTagOverRoleDiagnostic {
    #[label]
    pub span: Span,
    pub tag: String,
    pub role: String,
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
    /// ```javascript
    /// // Bad
    /// <div role="button" />
    ///
    /// // Good
    /// <button />
    /// ```
    PreferTagOverRole,
    correctness
);

impl PreferTagOverRole {
    fn check_roles<'a>(
        role_prop: &JSXAttributeItem<'a>,
        role_to_tag: &phf::Map<&str, &str>,
        jsx_name: &JSXElementName<'a>,
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

    fn check_role<'a>(
        role: &str,
        role_to_tag: &phf::Map<&str, &str>,
        jsx_name: &JSXElementName<'a>,
        span: Span,
        ctx: &LintContext<'a>,
    ) {
        if let Some(tag) = role_to_tag.get(role) {
            match jsx_name {
                JSXElementName::Identifier(id) if id.name != *tag => {
                    ctx.diagnostic(PreferTagOverRoleDiagnostic {
                        span,
                        tag: (*tag).to_string(),
                        role: role.to_string(),
                    });
                }
                _ => {}
            }
        }
    }
}

static ROLE_TO_TAG_MAP: Lazy<phf::Map<&'static str, &'static str>> = Lazy::new(|| {
    phf_map! {
        "checkbox" => "input",
        "button" => "button",
        "heading" => "h1,h2,h3,h4,h5,h6",
        "link" => "a,area",
        "rowgroup" => "tbody,tfoot,thead",
        "banner" => "header",
    }
});

impl Rule for PreferTagOverRole {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::JSXOpeningElement(jsx_el) = node.kind() {
            if let Some(role_prop) = has_jsx_prop_lowercase(jsx_el, "role") {
                Self::check_roles(role_prop, &ROLE_TO_TAG_MAP, &jsx_el.name, ctx);
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
    Tester::new_without_config(PreferTagOverRole::NAME, pass, fail).test_and_snapshot();
}
