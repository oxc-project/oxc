use oxc_ast::{
    ast::{JSXAttributeItem, JSXAttributeValue},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, Span};
use phf::phf_set;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{get_element_type, has_jsx_prop_ignore_case},
    AstNode,
};

fn no_noninteractive_tabindex_diagnostic(span: Span) -> OxcDiagnostic {
    // See <https://oxc.rs/docs/contribute/linter/adding-rules.html#diagnostics> for details
    OxcDiagnostic::warn("Tabindex should only be declared on interactive elements")
        .with_help("Tabindex attribute should be removed")
        .with_label(span)
}

#[derive(Debug, Clone)]
pub struct NoNoninteractiveTabindex {
    tags: Vec<CompactStr>,
    roles: Vec<CompactStr>,
    allow_expression_values: bool,
}

impl Default for NoNoninteractiveTabindex {
    fn default() -> Self {
        Self {
            roles: vec![CompactStr::new("tabpanel")],
            allow_expression_values: true,
            tags: vec![],
        }
    }
}

declare_oxc_lint!(
    /// ### What it does
    /// This rule checks that non-interactive elements don't have a tabIndex which would make them interactive via keyboard navigation.
    ///
    /// ### Why is this bad?
    ///
    /// Tab key navigation should be limited to elements on the page that can be interacted with.
    /// Thus it is not necessary to add a tabindex to items in an unordered list, for example,
    /// to make them navigable through assistive technology.
    ///
    /// These applications already afford page traversal mechanisms based on the HTML of the page.
    /// Generally, we should try to reduce the size of the page's tab ring rather than increasing it.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <div tabIndex="0" />
    /// <div role="article" tabIndex="0" />
    /// <article tabIndex="0" />
    /// <article tabIndex={0} />
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <div />
    /// <MyButton tabIndex={0} />
    /// <button />
    /// <button tabIndex="0" />
    /// <button tabIndex={0} />
    /// <div />
    /// <div tabIndex="-1" />
    /// <div role="button" tabIndex="0" />
    /// <div role="article" tabIndex="-1" />
    /// <article tabIndex="-1" />
    /// ```
    NoNoninteractiveTabindex,
    correctness,
);

const INTERACTIVE_HTML_ELEMENTS: phf::set::Set<&'static str> = phf_set! {
    "a", "audio", "button", "details", "embed", "iframe", "img", "input", "label", "select", "textarea", "video"
};

const INTERACTIVE_HTML_ROLES: phf::set::Set<&'static str> = phf_set! {
    "button", "checkbox", "gridcell", "link", "menuitem", "menuitemcheckbox", "menuitemradio", "option", "progressbar", "radio", "textbox"
};

impl Rule for NoNoninteractiveTabindex {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(jsx_el) = node.kind() else {
            return;
        };

        if let Some(JSXAttributeItem::Attribute(tabindex_attr)) =
            has_jsx_prop_ignore_case(jsx_el, "tabIndex")
        {
            if let Some(JSXAttributeValue::StringLiteral(tabindex)) = &tabindex_attr.value {
                if tabindex.value == "-1" {
                    return;
                }

                let component = &get_element_type(ctx, jsx_el);

                if INTERACTIVE_HTML_ELEMENTS.contains(component) {
                    return;
                }

                println!("component: {component}");

                if let Some(JSXAttributeItem::Attribute(role_attr)) =
                    has_jsx_prop_ignore_case(jsx_el, "role")
                {
                    if self.allow_expression_values {
                        return;
                    }

                    println!("role_attr: {role_attr:?}");
                    if let Some(JSXAttributeValue::StringLiteral(role)) = &role_attr.value {
                        println!("role: {role:?}");

                        if !INTERACTIVE_HTML_ROLES.contains(role.value.as_str())
                            && !self.roles.contains(&CompactStr::new(role.value.as_str()))
                        {
                            ctx.diagnostic(no_noninteractive_tabindex_diagnostic(
                                tabindex_attr.span,
                            ));
                        }
                    } else {
                        ctx.diagnostic(no_noninteractive_tabindex_diagnostic(tabindex_attr.span));
                    }
                } else {
                    ctx.diagnostic(no_noninteractive_tabindex_diagnostic(tabindex_attr.span));
                }
            }
        }
    }

    fn from_configuration(value: serde_json::Value) -> Self {
        let default = Self::default();

        if let Some(config) = value.get(0) {
            Self {
                roles: config
                    .get("roles")
                    .and_then(serde_json::Value::as_array)
                    .map_or(default.roles, |v| {
                        v.iter().map(|v| CompactStr::new(v.as_str().unwrap())).collect()
                    }),
                tags: config
                    .get("tags")
                    .and_then(serde_json::Value::as_array)
                    .map_or(default.tags, |v| {
                        v.iter().map(|v| CompactStr::new(v.as_str().unwrap())).collect()
                    }),
                allow_expression_values: config
                    .get("allow_expression_values")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(default.allow_expression_values),
            }
        } else {
            default
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (r#"<div role="tabpanel" tabIndex="0" />"#, None),
        (r#"<div role={ROLE_BUTTON} onClick={() => {}} tabIndex="0" />;"#, None),
        (
            r#"<div role={BUTTON} onClick={() => {}} tabIndex="0" />;"#,
            Some(serde_json::json!([{ "allowExpressionValues": true }])),
        ),
        (
            r#"<div role={isButton ? "button" : "link"} onClick={() => {}} tabIndex="0" />;"#,
            Some(serde_json::json!([{ "allowExpressionValues": true }])),
        ),
        (
            r#"<div role={isButton ? "button" : LINK} onClick={() => {}} tabIndex="0" />;"#,
            Some(serde_json::json!([{ "allowExpressionValues": true }])),
        ),
        (
            r#"<div role={isButton ? BUTTON : LINK} onClick={() => {}} tabIndex="0"/>;"#,
            Some(serde_json::json!([{ "allowExpressionValues": true }])),
        ),
    ];

    let fail = vec![
        (
            r#"<div role="tabpanel" tabIndex="0" />"#,
            Some(serde_json::json!([{ "roles": [], "allowExpressionValues": false }])),
        ),
        (
            r#"<div role={ROLE_BUTTON} onClick={() => {}} tabIndex="0" />;"#,
            Some(serde_json::json!([{ "roles": [], "allowExpressionValues": false }])),
        ),
        (
            r#"<div role={BUTTON} onClick={() => {}} tabIndex="0" />;"#,
            Some(serde_json::json!([{ "allowExpressionValues": false }])),
        ),
        (
            r#"<div role={isButton ? "button" : "link"} onClick={() => {}} tabIndex="0" />;"#,
            Some(serde_json::json!([{ "allowExpressionValues": false }])),
        ),
    ];

    Tester::new(NoNoninteractiveTabindex::NAME, NoNoninteractiveTabindex::CATEGORY, pass, fail)
        .test_and_snapshot();
}
