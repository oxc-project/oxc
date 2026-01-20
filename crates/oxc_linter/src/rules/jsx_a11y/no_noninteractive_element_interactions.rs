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
        get_element_type, get_string_literal_prop_value, has_jsx_prop, has_jsx_prop_ignore_case,
        is_hidden_from_screen_reader, is_presentation_role,
    },
};

use oxc_ast::ast::JSXOpeningElement;

fn no_noninteractive_element_interactions_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "Non-interactive elements should not be assigned mouse or keyboard event listeners.",
    )
    .with_help("Add an interactive role or use a semantic HTML element instead.")
    .with_label(span)
}

const DEFAULT_HANDLERS: &[&str] =
    &["onClick", "onMouseDown", "onMouseUp", "onKeyPress", "onKeyDown", "onKeyUp"];

fn is_content_editable(node: &JSXOpeningElement) -> bool {
    has_jsx_prop_ignore_case(node, "contentEditable")
        .and_then(|item| get_string_literal_prop_value(item))
        .is_some_and(|value| value == "true")
}

fn is_null_handler_value(value: &JSXAttributeValue) -> bool {
    matches!(
        value,
        JSXAttributeValue::ExpressionContainer(container)
            if matches!(&container.expression, oxc_ast::ast::JSXExpression::NullLiteral(_))
    )
}

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
    /// An array of event handler names that should trigger this rule (e.g., `onClick`, `onKeyDown`).
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

const NON_INTERACTIVE_ELEMENTS: &[&str] = &[
    "address",
    "article",
    "aside",
    "blockquote",
    "br",
    "caption",
    "code",
    "dd",
    "del",
    "details",
    "dfn",
    "dir",
    "dl",
    "dt",
    "em",
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
    "hr",
    "html",
    "iframe",
    "img",
    "ins",
    "label",
    "legend",
    "li",
    "main",
    "mark",
    "marquee",
    "menu",
    "meter",
    "nav",
    "ol",
    "optgroup",
    "output",
    "p",
    "pre",
    "progress",
    "ruby",
    "section",
    "strong",
    "sub",
    "sup",
    "table",
    "tbody",
    "tfoot",
    "thead",
    "time",
    "ul",
];

const NON_INTERACTIVE_ROLES: &[&str] = &[
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
    "form",
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
    "rowgroup",
    "search",
    "separator",
    "status",
    "strong",
    "subscript",
    "superscript",
    "table",
    "tabpanel",
    "term",
    "time",
    "timer",
    "tooltip",
];

const INTERACTIVE_ROLES: &[&str] = &[
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
    "progressbar",
    "radio",
    "radiogroup",
    "row",
    "rowheader",
    "scrollbar",
    "searchbox",
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

const ABSTRACT_ROLES: &[&str] = &[
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
    "window",
];

fn is_truly_interactive_element(element_type: &str, jsx_el: &JSXOpeningElement) -> bool {
    match element_type {
        "button" | "select" | "textarea" => true,
        "input" => {
            if let Some(input_type) = has_jsx_prop(jsx_el, "type")
                && get_string_literal_prop_value(input_type)
                    .is_some_and(|val| val.eq_ignore_ascii_case("hidden"))
            {
                return false;
            }
            true
        }
        "a" | "area" => has_jsx_prop(jsx_el, "href").is_some(),
        "audio" | "video" => has_jsx_prop(jsx_el, "controls").is_some(),
        "img" => has_jsx_prop(jsx_el, "usemap").is_some(),
        _ => false,
    }
}

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
            Some(handlers) => handlers.iter().any(|handler| {
                has_jsx_prop(jsx_el, handler.as_str()).is_some_and(|attr| {
                    attr.as_attribute().is_some_and(|a| {
                        a.value.as_ref().is_some_and(|v| !is_null_handler_value(v))
                    })
                })
            }),
            None => DEFAULT_HANDLERS.iter().any(|handler| {
                has_jsx_prop(jsx_el, handler).is_some_and(|attr| {
                    attr.as_attribute().is_some_and(|a| {
                        a.value.as_ref().is_some_and(|v| !is_null_handler_value(v))
                    })
                })
            }),
        };

        if !has_handler {
            return;
        }

        let element_type = get_element_type(ctx, jsx_el);

        if !HTML_TAG.contains(element_type.as_ref()) {
            return;
        }

        if is_content_editable(jsx_el) {
            return;
        }

        if is_hidden_from_screen_reader(ctx, jsx_el) {
            return;
        }

        if is_presentation_role(jsx_el) {
            return;
        }

        if let Some(JSXAttributeItem::Attribute(role_attr)) =
            has_jsx_prop_ignore_case(jsx_el, "role")
            && let Some(JSXAttributeValue::StringLiteral(role)) = &role_attr.value
        {
            let role_str = role.value.as_str().cow_to_lowercase();

            if let Some(first_role) = role_str.split_whitespace().next() {
                if INTERACTIVE_ROLES.contains(&first_role) || ABSTRACT_ROLES.contains(&first_role) {
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

        if is_truly_interactive_element(&element_type, jsx_el) {
            return;
        }

        if NON_INTERACTIVE_ELEMENTS.contains(&element_type.as_ref()) {
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
        (r"<input onClick={() => {}} />", None),
        (r"<input type='button' onClick={() => {}} />", None),
        (r"<input type='checkbox' onClick={() => {}} />", None),
        (r"<input type='color' onClick={() => {}} />", None),
        (r"<input type='date' onClick={() => {}} />", None),
        (r"<input type='datetime' onClick={() => {}} />", None),
        (r"<input type='datetime-local' onClick={() => {}} />", None),
        (r"<input type='email' onClick={() => {}} />", None),
        (r"<input type='file' onClick={() => {}} />", None),
        (r"<input type='image' onClick={() => {}} />", None),
        (r"<input type='month' onClick={() => {}} />", None),
        (r"<input type='number' onClick={() => {}} />", None),
        (r"<input type='password' onClick={() => {}} />", None),
        (r"<input type='radio' onClick={() => {}} />", None),
        (r"<input type='range' onClick={() => {}} />", None),
        (r"<input type='reset' onClick={() => {}} />", None),
        (r"<input type='search' onClick={() => {}} />", None),
        (r"<input type='submit' onClick={() => {}} />", None),
        (r"<input type='tel' onClick={() => {}} />", None),
        (r"<input type='text' onClick={() => {}} />", None),
        (r"<input type='time' onClick={() => {}} />", None),
        (r"<input type='url' onClick={() => {}} />", None),
        (r"<input type='week' onClick={() => {}} />", None),
        (r"<input type='hidden' onClick={() => {}} />", None),
        (r"<a onClick={() => {}} />", None),
        (r"<a tabIndex='0' onClick={() => {}} />", None),
        (r"<a onClick={() => {}} href='http://x.y.z' />", None),
        (r"<a onClick={() => {}} href='http://x.y.z' tabIndex='0' />", None),
        (r"<area onClick={() => {}} />", None),
        (r"<body onClick={() => {}} />", None),
        (r"<button onClick={() => {}} className='foo' />", None),
        (r"<menuitem onClick={() => {}} />", None),
        (r"<option onClick={() => {}} className='foo' />", None),
        (r"<select onClick={() => {}} className='foo' />", None),
        (r"<textarea onClick={() => {}} className='foo' />", None),
        (r"<tr onClick={() => {}} />", None),
        (r"<acronym onClick={() => {}} />", None),
        (r"<applet onClick={() => {}} />", None),
        (r"<audio onClick={() => {}} />", None),
        (r"<b onClick={() => {}} />", None),
        (r"<base onClick={() => {}} />", None),
        (r"<bdi onClick={() => {}} />", None),
        (r"<bdo onClick={() => {}} />", None),
        (r"<big onClick={() => {}} />", None),
        (r"<blink onClick={() => {}} />", None),
        (r"<canvas onClick={() => {}} />", None),
        (r"<center onClick={() => {}} />", None),
        (r"<cite onClick={() => {}} />", None),
        (r"<col onClick={() => {}} />", None),
        (r"<colgroup onClick={() => {}} />", None),
        (r"<content onClick={() => {}} />", None),
        (r"<data onClick={() => {}} />", None),
        (r"<datalist onClick={() => {}} />", None),
        (r"<div />", None),
        (r"<div className='foo' />", None),
        (r"<div onClick={() => {}} aria-hidden />", None),
        (r"<div onClick={() => {}} aria-hidden={true} />", None),
        (r"<div onClick={() => {}} />", None),
        (r"<div onClick={() => {}} role={undefined} />", None),
        (r"<div onClick={null} />", None),
        (r"<div onKeyUp={() => {}} aria-hidden={false} />", None),
        (r"<embed onClick={() => {}} />", None),
        (r"<font onClick={() => {}} />", None),
        (r"<frame onClick={() => {}} />", None),
        (r"<frameset onClick={() => {}} />", None),
        (r"<head onClick={() => {}} />", None),
        (r"<header onClick={() => {}} />", None),
        (r"<hgroup onClick={() => {}} />", None),
        (r"<i onClick={() => {}} />", None),
        (r"<img onLoad={() => {}} />", None),
        (r"<kbd onClick={() => {}} />", None),
        (r"<keygen onClick={() => {}} />", None),
        (r"<link onClick={() => {}} />", None),
        (r"<main onClick={null} />", None),
        (r"<map onClick={() => {}} />", None),
        (r"<meta onClick={() => {}} />", None),
        (r"<noembed onClick={() => {}} />", None),
        (r"<noscript onClick={() => {}} />", None),
        (r"<object onClick={() => {}} />", None),
        (r"<param onClick={() => {}} />", None),
        (r"<picture onClick={() => {}} />", None),
        (r"<q onClick={() => {}} />", None),
        (r"<rp onClick={() => {}} />", None),
        (r"<rt onClick={() => {}} />", None),
        (r"<rtc onClick={() => {}} />", None),
        (r"<s onClick={() => {}} />", None),
        (r"<samp onClick={() => {}} />", None),
        (r"<script onClick={() => {}} />", None),
        (r"<small onClick={() => {}} />", None),
        (r"<source onClick={() => {}} />", None),
        (r"<spacer onClick={() => {}} />", None),
        (r"<span onClick={() => {}} />", None),
        (r"<strike onClick={() => {}} />", None),
        (r"<style onClick={() => {}} />", None),
        (r"<summary onClick={() => {}} />", None),
        (r"<th onClick={() => {}} />", None),
        (r"<title onClick={() => {}} />", None),
        (r"<track onClick={() => {}} />", None),
        (r"<td onClick={() => {}} />", None),
        (r"<tt onClick={() => {}} />", None),
        (r"<u onClick={() => {}} />", None),
        (r"<var onClick={() => {}} />", None),
        (r"<video onClick={() => {}} />", None),
        (r"<wbr onClick={() => {}} />", None),
        (r"<xmp onClick={() => {}} />", None),
        (r"<div role='button' onClick={() => {}} />", None),
        (r"<div role='checkbox' onClick={() => {}} />", None),
        (r"<div role='columnheader' onClick={() => {}} />", None),
        (r"<div role='combobox' onClick={() => {}} />", None),
        (r"<div role='grid' onClick={() => {}} />", None),
        (r"<div role='gridcell' onClick={() => {}} />", None),
        (r"<div role='link' onClick={() => {}} />", None),
        (r"<div role='listbox' onClick={() => {}} />", None),
        (r"<div role='menu' onClick={() => {}} />", None),
        (r"<div role='menubar' onClick={() => {}} />", None),
        (r"<div role='menuitem' onClick={() => {}} />", None),
        (r"<div role='menuitemcheckbox' onClick={() => {}} />", None),
        (r"<div role='menuitemradio' onClick={() => {}} />", None),
        (r"<div role='option' onClick={() => {}} />", None),
        (r"<div role='progressbar' onClick={() => {}} />", None),
        (r"<div role='radio' onClick={() => {}} />", None),
        (r"<div role='radiogroup' onClick={() => {}} />", None),
        (r"<div role='row' onClick={() => {}} />", None),
        (r"<div role='rowheader' onClick={() => {}} />", None),
        (r"<div role='scrollbar' onClick={() => {}} />", None),
        (r"<div role='searchbox' onClick={() => {}} />", None),
        (r"<div role='slider' onClick={() => {}} />", None),
        (r"<div role='spinbutton' onClick={() => {}} />", None),
        (r"<div role='switch' onClick={() => {}} />", None),
        (r"<div role='tab' onClick={() => {}} />", None),
        (r"<div role='textbox' onClick={() => {}} />", None),
        (r"<div role='treeitem' onClick={() => {}} />", None),
        (r"<div role='tablist' onClick={() => {}} />", None),
        (r"<div role='toolbar' onClick={() => {}} />", None),
        (r"<div role='tree' onClick={() => {}} />", None),
        (r"<div role='treegrid' onClick={() => {}} />", None),
        (r"<div role='presentation' onClick={() => {}} />", None),
        (r"<div role='none' onClick={() => {}} />", None),
        (r"<div role='command' onClick={() => {}} />", None),
        (r"<div role='composite' onClick={() => {}} />", None),
        (r"<div role='input' onClick={() => {}} />", None),
        (r"<div role='landmark' onClick={() => {}} />", None),
        (r"<div role='range' onClick={() => {}} />", None),
        (r"<div role='roletype' onClick={() => {}} />", None),
        (r"<div role='sectionhead' onClick={() => {}} />", None),
        (r"<div role='select' onClick={() => {}} />", None),
        (r"<div role='structure' onClick={() => {}} />", None),
        (r"<div role='widget' onClick={() => {}} />", None),
        (r"<div role='window' onClick={() => {}} />", None),
        (r"<div role='article' onCopy={() => {}} />", None),
        (r"<div role='article' onCut={() => {}} />", None),
        (r"<div role='article' onPaste={() => {}} />", None),
        (r"<div role='article' onCompositionEnd={() => {}} />", None),
        (r"<div role='article' onCompositionStart={() => {}} />", None),
        (r"<div role='article' onCompositionUpdate={() => {}} />", None),
        (r"<div role='article' onChange={() => {}} />", None),
        (r"<div role='article' onInput={() => {}} />", None),
        (r"<div role='article' onSubmit={() => {}} />", None),
        (r"<div role='article' onSelect={() => {}} />", None),
        (r"<div role='article' onTouchCancel={() => {}} />", None),
        (r"<div role='article' onTouchEnd={() => {}} />", None),
        (r"<div role='article' onTouchMove={() => {}} />", None),
        (r"<div role='article' onTouchStart={() => {}} />", None),
        (r"<div role='article' onScroll={() => {}} />", None),
        (r"<div role='article' onWheel={() => {}} />", None),
        (r"<div role='article' onAbort={() => {}} />", None),
        (r"<div role='article' onCanPlay={() => {}} />", None),
        (r"<div role='article' onCanPlayThrough={() => {}} />", None),
        (r"<div role='article' onDurationChange={() => {}} />", None),
        (r"<div role='article' onEmptied={() => {}} />", None),
        (r"<div role='article' onEncrypted={() => {}} />", None),
        (r"<div role='article' onEnded={() => {}} />", None),
        (r"<div role='article' onLoadedData={() => {}} />", None),
        (r"<div role='article' onLoadedMetadata={() => {}} />", None),
        (r"<div role='article' onLoadStart={() => {}} />", None),
        (r"<div role='article' onPause={() => {}} />", None),
        (r"<div role='article' onPlay={() => {}} />", None),
        (r"<div role='article' onPlaying={() => {}} />", None),
        (r"<div role='article' onProgress={() => {}} />", None),
        (r"<div role='article' onRateChange={() => {}} />", None),
        (r"<div role='article' onSeeked={() => {}} />", None),
        (r"<div role='article' onSeeking={() => {}} />", None),
        (r"<div role='article' onStalled={() => {}} />", None),
        (r"<div role='article' onSuspend={() => {}} />", None),
        (r"<div role='article' onTimeUpdate={() => {}} />", None),
        (r"<div role='article' onVolumeChange={() => {}} />", None),
        (r"<div role='article' onWaiting={() => {}} />", None),
        (r"<div role='article' onAnimationStart={() => {}} />", None),
        (r"<div role='article' onAnimationEnd={() => {}} />", None),
        (r"<div role='article' onAnimationIteration={() => {}} />", None),
        (r"<div role='article' onTransitionEnd={() => {}} />", None),
        (r#"<article contentEditable="true" onClick={() => {}} />"#, None),
        (r#"<ul contentEditable="true" onClick={() => {}} />"#, None),
        (r"<section />", None),
        (r"<main />", None),
        (r"<article />", None),
        (r"<footer />", None),
        (r"<h1 />", None),
        (r"<li />", None),
        (r"<img usemap='#map' onClick={() => {}} />", None),
        (r"<audio controls onClick={() => {}} />", None),
        (r"<video controls onClick={() => {}} />", None),
        (r"<form onSubmit={() => {}} />", None),
    ];

    let fail = vec![
        (r"<main onClick={() => {}} />", None),
        (r"<address onClick={() => {}} />", None),
        (r"<article onClick={() => {}} />", None),
        (r"<aside onClick={() => {}} />", None),
        (r"<blockquote onClick={() => {}} />", None),
        (r"<br onClick={() => {}} />", None),
        (r"<caption onClick={() => {}} />", None),
        (r"<code onClick={() => {}} />", None),
        (r"<dd onClick={() => {}} />", None),
        (r"<del onClick={() => {}} />", None),
        (r"<details onClick={() => {}} />", None),
        (r"<dfn onClick={() => {}} />", None),
        (r"<dl onClick={() => {}} />", None),
        (r"<dir onClick={() => {}} />", None),
        (r"<dt onClick={() => {}} />", None),
        (r"<em onClick={() => {}} />", None),
        (r"<fieldset onClick={() => {}} />", None),
        (r"<figcaption onClick={() => {}} />", None),
        (r"<figure onClick={() => {}} />", None),
        (r"<footer onClick={() => {}} />", None),
        (r"<form onClick={() => {}} />", None),
        (r"<h1 onClick={() => {}} />", None),
        (r"<h2 onClick={() => {}} />", None),
        (r"<h3 onClick={() => {}} />", None),
        (r"<h4 onClick={() => {}} />", None),
        (r"<h5 onClick={() => {}} />", None),
        (r"<h6 onClick={() => {}} />", None),
        (r"<hr onClick={() => {}} />", None),
        (r"<html onClick={() => {}} />", None),
        (r"<iframe onClick={() => {}} />", None),
        (r"<img onClick={() => {}} />", None),
        (r"<ins onClick={() => {}} />", None),
        (r"<label onClick={() => {}} />", None),
        (r"<legend onClick={() => {}} />", None),
        (r"<li onClick={() => {}} />", None),
        (r"<mark onClick={() => {}} />", None),
        (r"<marquee onClick={() => {}} />", None),
        (r"<menu onClick={() => {}} />", None),
        (r"<meter onClick={() => {}} />", None),
        (r"<nav onClick={() => {}} />", None),
        (r"<ol onClick={() => {}} />", None),
        (r"<optgroup onClick={() => {}} />", None),
        (r"<output onClick={() => {}} />", None),
        (r"<p onClick={() => {}} />", None),
        (r"<pre onClick={() => {}} />", None),
        (r"<progress onClick={() => {}} />", None),
        (r"<ruby onClick={() => {}} />", None),
        (r"<section onClick={() => {}} />", None),
        (r"<strong onClick={() => {}} />", None),
        (r"<sub onClick={() => {}} />", None),
        (r"<sup onClick={() => {}} />", None),
        (r"<table onClick={() => {}} />", None),
        (r"<tbody onClick={() => {}} />", None),
        (r"<tfoot onClick={() => {}} />", None),
        (r"<thead onClick={() => {}} />", None),
        (r"<time onClick={() => {}} />", None),
        (r"<ul onClick={() => {}} />", None),
        (r#"<ul contentEditable="false" onClick={() => {}} />"#, None),
        (r"<article contentEditable onClick={() => {}} />", None),
        (r"<div contentEditable role='article' onKeyDown={() => {}} />", None),
        (r"<div role='alert' onClick={() => {}} />", None),
        (r"<div role='alertdialog' onClick={() => {}} />", None),
        (r"<div role='application' onClick={() => {}} />", None),
        (r"<div role='banner' onClick={() => {}} />", None),
        (r"<div role='cell' onClick={() => {}} />", None),
        (r"<div role='complementary' onClick={() => {}} />", None),
        (r"<div role='contentinfo' onClick={() => {}} />", None),
        (r"<div role='definition' onClick={() => {}} />", None),
        (r"<div role='dialog' onClick={() => {}} />", None),
        (r"<div role='directory' onClick={() => {}} />", None),
        (r"<div role='document' onClick={() => {}} />", None),
        (r"<div role='feed' onClick={() => {}} />", None),
        (r"<div role='figure' onClick={() => {}} />", None),
        (r"<div role='form' onClick={() => {}} />", None),
        (r"<div role='group' onClick={() => {}} />", None),
        (r"<div role='heading' onClick={() => {}} />", None),
        (r"<div role='img' onClick={() => {}} />", None),
        (r"<div role='list' onClick={() => {}} />", None),
        (r"<div role='listitem' onClick={() => {}} />", None),
        (r"<div role='log' onClick={() => {}} />", None),
        (r"<div role='main' onClick={() => {}} />", None),
        (r"<div role='marquee' onClick={() => {}} />", None),
        (r"<div role='math' onClick={() => {}} />", None),
        (r"<div role='navigation' onClick={() => {}} />", None),
        (r"<div role='note' onClick={() => {}} />", None),
        (r"<div role='region' onClick={() => {}} />", None),
        (r"<div role='rowgroup' onClick={() => {}} />", None),
        (r"<div role='search' onClick={() => {}} />", None),
        (r"<div role='separator' onClick={() => {}} />", None),
        (r"<div role='status' onClick={() => {}} />", None),
        (r"<div role='table' onClick={() => {}} />", None),
        (r"<div role='tabpanel' onClick={() => {}} />", None),
        (r"<div role='term' onClick={() => {}} />", None),
        (r"<div role='timer' onClick={() => {}} />", None),
        (r"<div role='tooltip' onClick={() => {}} />", None),
        (r"<div role='article' onClick={() => {}} />", None),
        (r"<div role='blockquote' onClick={() => {}} />", None),
        (r"<div role='caption' onClick={() => {}} />", None),
        (r"<div role='paragraph' onClick={() => {}} />", None),
        (r"<div role='article' onKeyDown={() => {}} />", None),
        (r"<div role='article' onKeyPress={() => {}} />", None),
        (r"<div role='article' onKeyUp={() => {}} />", None),
        (r"<div role='article' onClick={() => {}} />", None),
        (r"<div role='article' onMouseDown={() => {}} />", None),
        (r"<div role='article' onMouseUp={() => {}} />", None),
        (r"<li onKeyDown={() => {}} />", None),
        (r"<li onKeyUp={() => {}} />", None),
        (r"<li onKeyPress={() => {}} />", None),
        (r"<li onMouseDown={() => {}} />", None),
        (r"<li onMouseUp={() => {}} />", None),
    ];

    Tester::new(
        NoNoninteractiveElementInteractions::NAME,
        NoNoninteractiveElementInteractions::PLUGIN,
        pass,
        fail,
    )
    .test_and_snapshot();
}
