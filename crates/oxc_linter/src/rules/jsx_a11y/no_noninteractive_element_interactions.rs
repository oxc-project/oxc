use cow_utils::CowUtils;
use oxc_ast::{
    AstKind,
    ast::{JSXAttributeItem, JSXAttributeValue},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, GetSpan, Span};
use schemars::JsonSchema;
use serde::Deserialize;

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

fn no_noninteractive_element_interactions_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "Non-interactive elements should not be assigned mouse or keyboard event listeners.",
    )
    .with_help("Add an interactive role or use a semantic HTML element instead.")
    .with_label(span)
}

const DEFAULT_HANDLERS: &[&str] =
    &["onClick", "onMouseDown", "onMouseUp", "onKeyPress", "onKeyDown", "onKeyUp"];

#[derive(Debug, Default, Clone, Deserialize)]
pub struct NoNoninteractiveElementInteractions(Box<NoNoninteractiveElementInteractionsConfig>);

impl std::ops::Deref for NoNoninteractiveElementInteractions {
    type Target = NoNoninteractiveElementInteractionsConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Default, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct NoNoninteractiveElementInteractionsConfig {
    handlers: Option<Vec<CompactStr>>,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces that non-interactive HTML elements do not have interactive handlers assigned.
    ///
    /// ### Why is this bad?
    ///
    /// Non-interactive HTML elements and non-interactive ARIA roles indicate content intended solely for display,
    /// not for user interaction. Attaching mouse or keyboard event handlers to these elements creates
    /// accessibility violations by misleading users of assistive technologies.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <li onClick={() => {}} />
    /// <div role="listitem" onClick={() => {}} />
    /// <h1 onClick={() => {}} />
    /// <article onClick={() => {}} />
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <div onClick={() => {}} role="button" />
    /// <button onClick={() => {}} />
    /// <input type="text" onClick={() => {}} />
    /// <div onClick={() => {}} role="presentation" />
    /// <div />
    /// ```
    NoNoninteractiveElementInteractions,
    jsx_a11y,
    correctness,
    config = NoNoninteractiveElementInteractionsConfig
);

const NON_INTERACTIVE_ELEMENTS: [&str; 36] = [
    "article",
    "aside",
    "blockquote",
    "caption",
    "dd",
    "details",
    "dialog",
    "dl",
    "dt",
    "fieldset",
    "figcaption",
    "figure",
    "footer",
    "form",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "header",
    "hr",
    "legend",
    "li",
    "main",
    "meter",
    "nav",
    "ol",
    "output",
    "p",
    "pre",
    "progress",
    "section",
    "table",
    "ul",
    "summary",
];

const NON_INTERACTIVE_ROLES: [&str; 40] = [
    "alert",
    "alertdialog",
    "application",
    "article",
    "banner",
    "blockquote",
    "caption",
    "cell",
    "code",
    "complementary",
    "contentinfo",
    "definition",
    "deletion",
    "dialog",
    "directory",
    "document",
    "emphasis",
    "feed",
    "figure",
    "generic",
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
    "meter",
    "navigation",
    "note",
    "paragraph",
    "region",
    "status",
    "strong",
    "subscript",
    "superscript",
    "time",
];

const INTERACTIVE_ROLES: [&str; 27] = [
    "button",
    "checkbox",
    "columnheader",
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

const ABSTRACT_ROLES: [&str; 11] = [
    "command",
    "composite",
    "input",
    "landmark",
    "range",
    "roletype",
    "section",
    "sectionhead",
    "select",
    "structure",
    "widget",
];

impl Rule for NoNoninteractiveElementInteractions {
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

        let is_non_interactive_el = NON_INTERACTIVE_ELEMENTS.contains(&element_type.as_ref());

        if let Some(JSXAttributeItem::Attribute(role_attr)) =
            has_jsx_prop_ignore_case(jsx_el, "role")
        {
            if let Some(JSXAttributeValue::StringLiteral(role)) = &role_attr.value {
                let role_str = role.value.as_str().cow_to_lowercase();

                if let Some(first_role) = role_str.split_whitespace().next() {
                    if INTERACTIVE_ROLES.contains(&first_role)
                        || ABSTRACT_ROLES.contains(&first_role)
                    {
                        return;
                    }
                    if NON_INTERACTIVE_ROLES.contains(&first_role) {
                        ctx.diagnostic(no_noninteractive_element_interactions_diagnostic(
                            jsx_el.name.span(),
                        ));
                        return;
                    }
                }
            }
            if !is_non_interactive_el {
                return;
            }
        }

        if is_non_interactive_el {
            ctx.diagnostic(no_noninteractive_element_interactions_diagnostic(jsx_el.name.span()));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (r"<TestComponent onClick={() => {}} />", None),
        (r"<Button onClick={() => {}} />", None),
        (r"<a tabIndex='0' onClick={() => {}} />", None),
        (r"<a onClick={() => {}} href='http://x.y.z' />", None),
        (r"<a onClick={() => {}} href='http://x.y.z' tabIndex='0' />", None),
        (r"<input onClick={() => {}} />", None),
        (r"<input type='button' onClick={() => {}} />", None),
        (r"<button onClick={() => {}} />", None),
        (r"<select onClick={() => {}} />", None),
        (r"<textarea onClick={() => {}} />", None),
        (r"<details onClick={() => {}} />", None),
        (r"<embed onClick={() => {}} />", None),
        (r"<iframe onClick={() => {}} />", None),
        (r"<label onClick={() => {}} />", None),
        (r"<audio controls onClick={() => {}} />", None),
        (r"<video controls onClick={() => {}} />", None),
        (r"<img usemap='#map' onClick={() => {}} />", None),
        (r"<div role='button' onClick={() => {}} />", None),
        (r"<div role='checkbox' onClick={() => {}} />", None),
        (r"<div role='link' onClick={() => {}} />", None),
        (r"<div role='menuitem' onClick={() => {}} />", None),
        (r"<div role='option' onClick={() => {}} />", None),
        (r"<div role='radio' onClick={() => {}} />", None),
        (r"<div role='searchbox' onClick={() => {}} />", None),
        (r"<div role='switch' onClick={() => {}} />", None),
        (r"<div role='textbox' onClick={() => {}} />", None),
        (r"<div role='combobox' onClick={() => {}} />", None),
        (r"<div role='slider' onClick={() => {}} />", None),
        (r"<div role='spinbutton' onClick={() => {}} />", None),
        (r"<div role='tab' onClick={() => {}} />", None),
        (r"<div role='presentation' onClick={() => {}} />", None),
        (r"<div role='none' onClick={() => {}} />", None),
        (r"<div onClick={() => {}} aria-hidden />", None),
        (r"<div onClick={() => {}} aria-hidden={true} />", None),
        (r"<div />", None),
        (r"<section />", None),
        (r"<main />", None),
        (r"<article />", None),
        (r"<header />", None),
        (r"<footer />", None),
        (r"<h1 />", None),
        (r"<li />", None),
        (r"<div role='command' onClick={() => {}} />", None),
        (r"<div role='composite' onClick={() => {}} />", None),
        (r"<div role='input' onClick={() => {}} />", None),
        (r"<div role='landmark' onClick={() => {}} />", None),
        (r"<div role='range' onClick={() => {}} />", None),
        (r"<div role='roletype' onClick={() => {}} />", None),
        (r"<div role='section' onClick={() => {}} />", None),
        (r"<div role='sectionhead' onClick={() => {}} />", None),
        (r"<div role='select' onClick={() => {}} />", None),
        (r"<div role='structure' onClick={() => {}} />", None),
        (r"<div role='widget' onClick={() => {}} />", None),
        (r"<div onClick={() => {}} />", None),
        (r"<span onClick={() => {}} />", None),
        (r"<b onClick={() => {}} />", None),
        (r"<bdi onClick={() => {}} />", None),
        (r"<bdo onClick={() => {}} />", None),
        (r"<cite onClick={() => {}} />", None),
        (r"<audio onClick={() => {}} />", None),
        (r"<canvas onClick={() => {}} />", None),
        (r"<body onClick={() => {}} />", None),
        (r"<tr onClick={() => {}} />", None),
    ];

    let fail = vec![
        (r"<li onClick={() => {}} />", None),
        (r"<ul contentEditable='false' onClick={() => {}} />", None),
        (r"<article contentEditable onClick={() => {}} />", None),
        (r"<div contentEditable role='article' onKeyDown={() => {}} />", None),
        (r"<div role='listitem' onClick={() => {}} />", None),
        (r"<h1 onClick={() => {}} />", None),
        (r"<h2 onClick={() => {}} />", None),
        (r"<h3 onClick={() => {}} />", None),
        (r"<h4 onClick={() => {}} />", None),
        (r"<h5 onClick={() => {}} />", None),
        (r"<h6 onClick={() => {}} />", None),
        (r"<article onClick={() => {}} />", None),
        (r"<div role='article' onClick={() => {}} />", None),
        (r"<header onClick={() => {}} />", None),
        (r"<main onClick={() => {}} />", None),
        (r"<div role='main' onClick={() => {}} />", None),
        (r"<footer onClick={() => {}} />", None),
        (r"<section onClick={() => {}} />", None),
        (r"<nav onClick={() => {}} />", None),
        (r"<div role='navigation' onClick={() => {}} />", None),
        (r"<aside onClick={() => {}} />", None),
        (r"<div role='complementary' onClick={() => {}} />", None),
        (r"<p onClick={() => {}} />", None),
        (r"<div role='paragraph' onClick={() => {}} />", None),
        (r"<ol onClick={() => {}} />", None),
        (r"<ul onClick={() => {}} />", None),
        (r"<div role='list' onClick={() => {}} />", None),
        (r"<table onClick={() => {}} />", None),
        (r"<figure onClick={() => {}} />", None),
        (r"<div role='figure' onClick={() => {}} />", None),
        (r"<form onClick={() => {}} />", None),
        (r"<fieldset onClick={() => {}} />", None),
        (r"<blockquote onClick={() => {}} />", None),
        (r"<div role='blockquote' onClick={() => {}} />", None),
        (r"<pre onClick={() => {}} />", None),
        (r"<div role='img' onClick={() => {}} />", None),
        (r"<div role='heading' onClick={() => {}} />", None),
        (r"<div role='banner' onClick={() => {}} />", None),
        (r"<div role='contentinfo' onClick={() => {}} />", None),
        (r"<div role='region' onClick={() => {}} />", None),
        (r"<div role='status' onClick={() => {}} />", None),
        (r"<div role='log' onClick={() => {}} />", None),
        (r"<div role='alert' onClick={() => {}} />", None),
        (r"<div role='dialog' onClick={() => {}} />", None),
        (r"<div role='alertdialog' onClick={() => {}} />", None),
        (r"<li onKeyDown={() => {}} />", None),
        (r"<li onKeyUp={() => {}} />", None),
        (r"<li onKeyPress={() => {}} />", None),
        (r"<li onMouseDown={() => {}} />", None),
        (r"<li onMouseUp={() => {}} />", None),
        (r"<dd onClick={() => {}} />", None),
        (r"<dt onClick={() => {}} />", None),
        (r"<figcaption onClick={() => {}} />", None),
        (r"<caption onClick={() => {}} />", None),
        (r"<div role='caption' onClick={() => {}} />", None),
        (r"<div role='definition' onClick={() => {}} />", None),
        (r"<div role='directory' onClick={() => {}} />", None),
        (r"<div role='document' onClick={() => {}} />", None),
        (r"<div role='feed' onClick={() => {}} />", None),
        (r"<div role='group' onClick={() => {}} />", None),
        (r"<div role='marquee' onClick={() => {}} />", None),
        (r"<div role='math' onClick={() => {}} />", None),
        (r"<div role='note' onClick={() => {}} />", None),
    ];

    Tester::new(
        NoNoninteractiveElementInteractions::NAME,
        NoNoninteractiveElementInteractions::PLUGIN,
        pass,
        fail,
    )
    .test_and_snapshot();
}
