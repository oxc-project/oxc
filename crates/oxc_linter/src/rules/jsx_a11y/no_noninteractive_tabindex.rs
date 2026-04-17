use oxc_ast::{
    AstKind,
    ast::{JSXAttributeItem, JSXAttributeValue},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_str::CompactStr;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    context::LintContext,
    globals::HTML_TAG,
    rule::Rule,
    utils::{get_element_type, has_jsx_prop_ignore_case, is_interactive_element, parse_jsx_value},
};

fn no_noninteractive_tabindex_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("`tabIndex` should only be declared on interactive elements.")
        .with_help("The `tabIndex` attribute should be removed.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoNoninteractiveTabindex(Box<NoNoninteractiveTabindexConfig>);

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
struct NoNoninteractiveTabindexConfig {
    /// An array of custom HTML elements that should be considered interactive.
    tags: Vec<CompactStr>,
    /// An array of ARIA roles that should be considered interactive.
    roles: Vec<CompactStr>,
    /// If `true`, allows tabIndex values to be expression values (e.g., variables, ternaries). If `false`, only string literal values are allowed.
    allow_expression_values: bool,
}

impl Default for NoNoninteractiveTabindexConfig {
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
    ///
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
    jsx_a11y,
    correctness,
    config = NoNoninteractiveTabindexConfig,
    version = "0.15.4",
);

// https://www.w3.org/TR/wai-aria/#widget_roles
// NOTE: "tabpanel" is not included here because it's technically a section role. It can optionally be considered interactive within the context of a tablist, because its visibility is dynamically controlled by an element with the "tab" aria role. It's included in the recommended jsx-a11y config for this reason.
const INTERACTIVE_HTML_ROLES: [&str; 29] = [
    "button",
    "checkbox",
    "combobox",
    "grid",
    "gridcell",
    "link",
    "listbox",
    "menu",
    "menubar",
    "menuitem",
    "menuitemcheckbox",
    "menuitemradio",
    "option",
    "progressbar",
    "radio",
    "radiogroup",
    "scrollbar",
    "searchbox",
    "separator",
    "slider",
    "spinbutton",
    "switch",
    "tab",
    "tablist",
    "textbox",
    "toolbar",
    "tree",
    "treegrid",
    "treeitem",
];

impl Rule for NoNoninteractiveTabindex {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(jsx_el) = node.kind() else {
            return;
        };

        let Some(JSXAttributeItem::Attribute(tabindex_attr)) =
            has_jsx_prop_ignore_case(jsx_el, "tabIndex")
        else {
            return;
        };

        let Some(tabindex_value) = &tabindex_attr.value else {
            return;
        };

        let Ok(tabindex) = parse_jsx_value(tabindex_value) else {
            if matches!(tabindex_value, JSXAttributeValue::ExpressionContainer(_))
                && !self.0.allow_expression_values
            {
                ctx.diagnostic(no_noninteractive_tabindex_diagnostic(tabindex_attr.span));
            }
            return;
        };

        if tabindex < 0.0 || tabindex.fract() != 0.0 {
            return;
        }

        let component = &get_element_type(ctx, jsx_el);

        if self.0.tags.iter().any(|tag| tag == component.as_ref()) {
            return;
        }

        if !HTML_TAG.contains(component.as_ref()) {
            return;
        }

        if is_interactive_element(component, jsx_el) {
            return;
        }

        let Some(JSXAttributeItem::Attribute(role_attr)) = has_jsx_prop_ignore_case(jsx_el, "role")
        else {
            // if the component is not an interactive element and has no role, the tabindex is invalid.
            ctx.diagnostic(no_noninteractive_tabindex_diagnostic(tabindex_attr.span));
            return;
        };

        if let Some(role) = role_attr.value.as_ref() {
            match role {
                JSXAttributeValue::StringLiteral(role) => {
                    let is_interactive_role =
                        role.value.split_whitespace().next().is_some_and(|role| {
                            INTERACTIVE_HTML_ROLES.contains(&role)
                                || self.0.roles.iter().any(|allowed_role| allowed_role == role)
                        });

                    if is_interactive_role {
                        return;
                    }
                }
                JSXAttributeValue::ExpressionContainer(_) if self.0.allow_expression_values => {
                    return;
                }
                _ => {}
            }
        }

        ctx.diagnostic(no_noninteractive_tabindex_diagnostic(tabindex_attr.span));
    }

    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        let default = Self::default();

        let Some(config) = value.get(0) else {
            return Ok(default);
        };

        Ok(Self(Box::new(NoNoninteractiveTabindexConfig {
            roles: config
                .get("roles")
                .and_then(serde_json::Value::as_array)
                .map_or(default.0.roles, |v| {
                    v.iter().map(|v| CompactStr::new(v.as_str().unwrap())).collect()
                }),
            tags: config
                .get("tags")
                .and_then(serde_json::Value::as_array)
                .map_or(default.0.tags, |v| {
                    v.iter().map(|v| CompactStr::new(v.as_str().unwrap())).collect()
                }),
            allow_expression_values: config
                .get("allowExpressionValues")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(default.0.allow_expression_values),
        })))
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    fn settings() -> serde_json::Value {
        serde_json::json!({
            "settings": { "jsx-a11y": {
                "components": {
                    "Article": "article",
                    "MyButton": "button",
                }
            } }
        })
    }

    let pass = vec![
        (r"<MyButton tabIndex={0} />", None, None),
        (r"<button />", None, None),
        (r#"<button tabIndex="0" />"#, None, None),
        (r"<button tabIndex={0} />", None, None),
        (r"<div />", None, None),
        (r#"<div tabIndex="-1" />"#, None, None),
        (r#"<div role="button" tabIndex="0" />"#, None, None),
        (r#"<div role="article" tabIndex="-1" />"#, None, None),
        (r#"<article tabIndex="-1" />"#, None, None),
        (r#"<Article tabIndex="-1" />"#, None, Some(settings())),
        (r"<MyButton tabIndex={0} />", None, Some(settings())),
        (r#"<div role="tabpanel" tabIndex="0" />"#, None, None),
        (r#"<div role={ROLE_BUTTON} onClick={() => {}} tabIndex="0" />;"#, None, None),
        (
            r"<div tabIndex={someVar} />",
            Some(serde_json::json!([{ "allowExpressionValues": true }])),
            None,
        ),
        (
            r"<div tabIndex={-1} />",
            Some(serde_json::json!([{ "allowExpressionValues": false }])),
            None,
        ),
        (
            r#"<div role={BUTTON} onClick={() => {}} tabIndex="0" />;"#,
            Some(serde_json::json!([{ "allowExpressionValues": true }])),
            None,
        ),
        (
            r#"<div role={isButton ? "button" : "link"} onClick={() => {}} tabIndex="0" />;"#,
            Some(serde_json::json!([{ "allowExpressionValues": true }])),
            None,
        ),
        (
            r#"<div role={isButton ? "button" : LINK} onClick={() => {}} tabIndex="0" />;"#,
            Some(serde_json::json!([{ "allowExpressionValues": true }])),
            None,
        ),
        (
            r#"<div role={isButton ? BUTTON : LINK} onClick={() => {}} tabIndex="0"/>;"#,
            Some(serde_json::json!([{ "allowExpressionValues": true }])),
            None,
        ),
        // Composite widget roles should be considered interactive
        (r#"<div role="combobox" tabIndex="0" />"#, None, None),
        (r#"<div role="grid" tabIndex="0" />"#, None, None),
        (r#"<div role="listbox" tabIndex="0" />"#, None, None),
        (r#"<div role="menu" tabIndex="0" />"#, None, None),
        (r#"<div role="menubar" tabIndex="0" />"#, None, None),
        (r#"<div role="radiogroup" tabIndex="0" />"#, None, None),
        (r#"<div role="tablist" tabIndex="0" />"#, None, None),
        (r#"<div role="tree" tabIndex="0" />"#, None, None),
        (r#"<div role="treegrid" tabIndex="0" />"#, None, None),
        (r#"<div role="toolbar" tabIndex="0" />"#, None, None),
    ];

    let fail = vec![
        (r#"<div tabIndex="0" />"#, None, None),
        (r"<div tabIndex={0} />", None, None),
        (r#"<div role="article" tabIndex="0" />"#, None, None),
        (r"<article tabIndex={0} />", None, None),
        (r"<Article tabIndex={0} />", None, Some(settings())),
        (r#"<article tabIndex="0" />"#, None, None),
        (
            r#"<div role="tabpanel" tabIndex="0" />"#,
            Some(serde_json::json!([{ "roles": [], "allowExpressionValues": false }])),
            None,
        ),
        (
            r#"<div role={ROLE_BUTTON} onClick={() => {}} tabIndex="0" />;"#,
            Some(serde_json::json!([{ "roles": [], "allowExpressionValues": false }])),
            None,
        ),
        (
            r#"<div role={BUTTON} onClick={() => {}} tabIndex="0" />;"#,
            Some(serde_json::json!([{ "allowExpressionValues": false }])),
            None,
        ),
        (
            r#"<div role={isButton ? "button" : "link"} onClick={() => {}} tabIndex="0" />;"#,
            Some(serde_json::json!([{ "allowExpressionValues": false }])),
            None,
        ),
        (
            r"<div tabIndex={someVar} />",
            Some(serde_json::json!([{ "allowExpressionValues": false }])),
            None,
        ),
    ];

    Tester::new(NoNoninteractiveTabindex::NAME, NoNoninteractiveTabindex::PLUGIN, pass, fail)
        .test_and_snapshot();
}
