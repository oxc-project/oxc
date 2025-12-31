use cow_utils::CowUtils;
use schemars::JsonSchema;
use serde::Deserialize;

use oxc_ast::{
    AstKind,
    ast::{JSXAttributeItem, JSXAttributeValue},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, GetSpan, Span};

use crate::{
    AstNode,
    context::LintContext,
    globals::HTML_TAG,
    rule::{DefaultRuleConfig, Rule},
    utils::{
        get_element_type, has_jsx_prop, has_jsx_prop_ignore_case, is_hidden_from_screen_reader,
        is_interactive_element, is_presentation_role,
    },
};

fn no_static_element_interactions_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Static HTML elements with event handlers require a role.")
        .with_help("Add a role attribute to this element, or use a semantic HTML element instead.")
        .with_label(span)
}

const DEFAULT_HANDLERS: &[&str] =
    &["onClick", "onMouseDown", "onMouseUp", "onKeyPress", "onKeyDown", "onKeyUp"];

#[derive(Debug, Default, Clone, Deserialize)]
pub struct NoStaticElementInteractions(Box<NoStaticElementInteractionsConfig>);

impl std::ops::Deref for NoStaticElementInteractions {
    type Target = NoStaticElementInteractionsConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Default, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct NoStaticElementInteractionsConfig {
    /// An array of event handler names that should trigger this rule (e.g., `onClick`, `onKeyDown`).
    handlers: Option<Vec<CompactStr>>,
    /// If `true`, role attribute values that are JSX expressions (e.g., `role={ROLE}`) are allowed.
    /// If `false`, only string literal role values are permitted.
    allow_expression_values: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces that static HTML elements with event handlers must have appropriate ARIA roles.
    ///
    /// ### Why is this bad?
    ///
    /// Static HTML elements do not have semantic meaning in accessibility contexts.
    /// When these elements receive click or keyboard event handlers, they must declare a role
    /// to indicate their interactive purpose to assistive technologies.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <div onClick={() => {}} />
    /// <span onKeyDown={handleKeyDown} />
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <button onClick={() => {}} />
    /// <div onClick={() => {}} role="button" />
    /// <input type="text" onClick={() => {}} />
    /// ```
    NoStaticElementInteractions,
    jsx_a11y,
    correctness,
    config = NoStaticElementInteractionsConfig,
);

const INTERACTIVE_ROLES: [&str; 26] = [
    "button",
    "checkbox",
    "columnheader",
    "combobox",
    "gridcell",
    "link",
    "listbox",
    "menu",
    "menubar",
    "menuitem",
    "menuitemcheckbox",
    "menuitemradio",
    "option",
    "radio",
    "radiogroup",
    "row",
    "rowheader",
    "scrollbar",
    "searchbox",
    "separator",
    "slider",
    "spinbutton",
    "switch",
    "tab",
    "textbox",
    "treeitem",
];

const NON_INTERACTIVE_ROLES: [&str; 43] = [
    "alert",
    "alertdialog",
    "application",
    "article",
    "banner",
    "blockquote",
    "caption",
    "cell",
    "complementary",
    "contentinfo",
    "definition",
    "deletion",
    "dialog",
    "directory",
    "document",
    "feed",
    "figure",
    "form",
    "group",
    "heading",
    "img",
    "insertion",
    "list",
    "listitem",
    "log",
    "main",
    "marquee",
    "math",
    "navigation",
    "note",
    "paragraph",
    "region",
    "row",
    "rowgroup",
    "search",
    "status",
    "table",
    "tabpanel",
    "term",
    "time",
    "timer",
    "toolbar",
    "tooltip",
];

impl Rule for NoStaticElementInteractions {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        Ok(serde_json::from_value::<DefaultRuleConfig<Self>>(value)
            .unwrap_or_default()
            .into_inner())
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(jsx_el) = node.kind() else {
            return;
        };

        let has_handler = match &self.handlers {
            Some(handlers) => {
                handlers.iter().any(|handler| has_jsx_prop(jsx_el, handler.as_str()).is_some())
            }
            None => DEFAULT_HANDLERS.iter().any(|handler| has_jsx_prop(jsx_el, handler).is_some()),
        };

        if !has_handler {
            return;
        }

        let element_type = get_element_type(ctx, jsx_el);

        if !HTML_TAG.contains(element_type.as_ref()) {
            return;
        }

        if is_hidden_from_screen_reader(ctx, jsx_el) || is_presentation_role(jsx_el) {
            return;
        }

        if is_interactive_element(&element_type, jsx_el) {
            return;
        }

        let Some(JSXAttributeItem::Attribute(role_attr)) = has_jsx_prop_ignore_case(jsx_el, "role")
        else {
            ctx.diagnostic(no_static_element_interactions_diagnostic(jsx_el.name.span()));
            return;
        };

        let Some(role_value) = &role_attr.value else {
            ctx.diagnostic(no_static_element_interactions_diagnostic(jsx_el.name.span()));
            return;
        };

        match role_value {
            JSXAttributeValue::StringLiteral(role) => {
                let role_str = role.value.as_str().cow_to_lowercase();
                let roles: Vec<&str> = role_str.split_whitespace().collect();

                if let Some(first_role) = roles.first() {
                    if INTERACTIVE_ROLES.contains(first_role) {
                        return;
                    }
                    if NON_INTERACTIVE_ROLES.contains(first_role) {
                        ctx.diagnostic(no_static_element_interactions_diagnostic(
                            jsx_el.name.span(),
                        ));
                        return;
                    }
                }
            }
            JSXAttributeValue::ExpressionContainer(_) => {
                if self.allow_expression_values {
                    return;
                }
            }
            _ => {}
        }

        ctx.diagnostic(no_static_element_interactions_diagnostic(jsx_el.name.span()));
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (r"<div />;", None),
        (r"<div className='foo' />;", None),
        (r"<div className='foo' onClick={() => {}} role='button' />;", None),
        (r"<div className='foo' onKeyDown={() => {}} role='button' />;", None),
        (r"<div onClick={() => {}} role='button' />;", None),
        (r"<button onClick={() => {}} className='foo' />;", None),
        (r"<input type='text' onClick={() => {}} />;", None),
        (r"<input onClick={() => {}} />;", None),
        (r"<button onClick={() => {}} className='foo' />;", None),
        (r"<select onClick={() => {}} />;", None),
        (r"<textarea onClick={() => {}} />;", None),
        (r"<a tabIndex='0' onClick={() => {}} href='http://x.y.z' />;", None),
        (r"<input type='hidden' onClick={() => {}} />;", None),
        (r"<div onClick={() => {}} role='presentation' />;", None),
        (r"<div onClick={() => {}} role='none' />;", None),
        (r"<div onMouseDown={() => {}} role='button' />;", None),
        (r"<div onMouseUp={() => {}} role='button' />;", None),
        (r"<div onKeyPress={() => {}} role='button' />;", None),
        (r"<div onKeyDown={() => {}} role='button' />;", None),
        (r"<div onKeyUp={() => {}} role='button' />;", None),
        (r"<div onClick={() => {}} aria-hidden />;", None),
        (r"<div onClick={() => {}} aria-hidden={true} />;", None),
        (r"<div onClick={() => {}} aria-hidden={false} role='button' />;", None),
        (r"<input onClick={() => {}} type='submit' />;", None),
        (r"<TestComponent onClick={() => {}} />;", None),
        (r"<Button onClick={() => {}} />;", None),
        (
            r"<div onClick={() => {}} role={ROLE} />",
            Some(serde_json::json!([{ "allowExpressionValues": true }])),
        ),
        (
            r#"<div onClick={() => {}} role={isButton ? "button" : "link"} />"#,
            Some(serde_json::json!([{ "allowExpressionValues": true }])),
        ),
        (r"<div onDblClick={() => {}} />", Some(serde_json::json!([{ "handlers": ["onClick"] }]))),
        (r"<div onClick={() => {}} role='checkbox' />;", None),
        (r"<div onClick={() => {}} role='link' />;", None),
        (r"<div onClick={() => {}} role='menuitem' />;", None),
        (r"<div onClick={() => {}} role='option' />;", None),
        (r"<div onClick={() => {}} role='radio' />;", None),
        (r"<div onClick={() => {}} role='searchbox' />;", None),
        (r"<div onClick={() => {}} role='switch' />;", None),
        (r"<div onClick={() => {}} role='textbox' />;", None),
        (r"<div onClick={() => {}} role='combobox' />;", None),
        (r"<div onClick={() => {}} role='slider' />;", None),
        (r"<div onClick={() => {}} role='spinbutton' />;", None),
        (r"<div onClick={() => {}} role='tab' />;", None),
    ];

    let fail = vec![
        (r"<div onClick={() => {}} />;", None),
        (r"<div onMouseDown={() => {}} />;", None),
        (r"<div onMouseUp={() => {}} />;", None),
        (r"<div onKeyPress={() => {}} />;", None),
        (r"<div onKeyDown={() => {}} />;", None),
        (r"<div onKeyUp={() => {}} />;", None),
        (r"<section onClick={() => {}} />;", None),
        (r"<main onClick={() => {}} />;", None),
        (r"<article onClick={() => {}} />;", None),
        (r"<header onClick={() => {}} />;", None),
        (r"<footer onClick={() => {}} />;", None),
        (r"<div onClick={() => {}} role='article' />;", None),
        (r"<div onClick={() => {}} role='navigation' />;", None),
        (r"<div onClick={() => {}} role='main' />;", None),
        (r"<div onClick={() => {}} aria-hidden={false} />;", None),
        (r"<a onClick={() => {}} />;", None),
        (r#"<a tabIndex="0" onClick={() => {}} />"#, None),
        (
            r"<div onClick={() => {}} role={ROLE} />",
            Some(serde_json::json!([{ "allowExpressionValues": false }])),
        ),
        (
            r#"<div onClick={() => {}} role={isButton ? "button" : "link"} />"#,
            Some(serde_json::json!([{ "allowExpressionValues": false }])),
        ),
        (
            r"<div onDblClick={() => {}} />",
            Some(serde_json::json!([{ "handlers": ["onClick", "onDblClick"] }])),
        ),
        (r"<span onClick={() => {}} />;", None),
        (r"<div onClick={() => {}} role='document' />;", None),
        (r"<div onClick={() => {}} role='list' />;", None),
        (r"<div onClick={() => {}} role='listitem' />;", None),
        (r"<div onClick={() => {}} role='heading' />;", None),
        (r"<div onClick={() => {}} role='img' />;", None),
        (r"<div onClick={() => {}} role='form' />;", None),
        (r"<div onClick={() => {}} role='region' />;", None),
        (r"<div onClick={() => {}} role='banner' />;", None),
        (r"<div onClick={() => {}} role='contentinfo' />;", None),
        (r"<div onClick={() => {}} role='complementary' />;", None),
        (r"<div onClick={() => {}} role='status' />;", None),
        (r"<div onClick={() => {}} role='log' />;", None),
        (r"<div onClick={() => {}} role='timer' />;", None),
        (r"<div onClick={() => {}} role='alert' />;", None),
        (r"<div onClick={() => {}} role='dialog' />;", None),
        (r"<div onClick={() => {}} role='alertdialog' />;", None),
        (r"<div onClick={() => {}} role='tabpanel' />;", None),
        (r"<div onClick={() => {}} role='tooltip' />;", None),
    ];

    Tester::new(NoStaticElementInteractions::NAME, NoStaticElementInteractions::PLUGIN, pass, fail)
        .test_and_snapshot();
}
