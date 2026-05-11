use std::borrow::Cow;

use cow_utils::CowUtils;
use oxc_ast::{
    AstKind,
    ast::{JSXAttributeValue, JSXExpression, JSXOpeningElement},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_str::CompactStr;
use rustc_hash::FxHashMap;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    context::LintContext,
    globals::HTML_TAG,
    rule::{DefaultRuleConfig, Rule},
    utils::{
        KEYBOARD_EVENT_HANDLERS, MOUSE_EVENT_HANDLERS, get_element_type, get_prop_value,
        get_string_literal_prop_value, has_jsx_prop, has_jsx_prop_ignore_case, is_abstract_role,
        is_hidden_from_screen_reader, is_interactive_element, is_interactive_role,
        is_non_interactive_element, is_non_interactive_role, is_presentation_role,
    },
};

fn no_noninteractive_element_interactions_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "Non-interactive elements should not be assigned mouse or keyboard event listeners.",
    )
    .with_help(
        "Move the handler to an interactive element, or use an appropriate interactive role.",
    )
    .with_label(span)
}

const FOCUS_EVENT_HANDLERS: &[&str] = &["onFocus", "onBlur"];
const IMAGE_EVENT_HANDLERS: &[&str] = &["onLoad", "onError"];
const DEFAULT_HANDLER_GROUPS: &[&[&str]] =
    &[FOCUS_EVENT_HANDLERS, IMAGE_EVENT_HANDLERS, KEYBOARD_EVENT_HANDLERS, MOUSE_EVENT_HANDLERS];

const RECOMMENDED_HANDLERS: &[&str] = &[
    "onClick",
    "onError",
    "onLoad",
    "onMouseDown",
    "onMouseUp",
    "onKeyPress",
    "onKeyDown",
    "onKeyUp",
];

const KEYBOARD_HANDLER_EXCEPTIONS: &[&str] = &["onKeyUp", "onKeyDown", "onKeyPress"];
const LOAD_ERROR_HANDLER_EXCEPTIONS: &[&str] = &["onError", "onLoad"];

fn compact_handlers(handlers: &[&str]) -> Vec<CompactStr> {
    handlers.iter().map(|handler| CompactStr::new(*handler)).collect()
}

fn recommended_handler_exceptions() -> FxHashMap<CompactStr, Vec<CompactStr>> {
    let mut exceptions = FxHashMap::default();
    exceptions.insert(CompactStr::new("alert"), compact_handlers(KEYBOARD_HANDLER_EXCEPTIONS));
    exceptions.insert(CompactStr::new("body"), compact_handlers(LOAD_ERROR_HANDLER_EXCEPTIONS));
    exceptions.insert(CompactStr::new("dialog"), compact_handlers(KEYBOARD_HANDLER_EXCEPTIONS));
    exceptions.insert(CompactStr::new("iframe"), compact_handlers(LOAD_ERROR_HANDLER_EXCEPTIONS));
    exceptions.insert(CompactStr::new("img"), compact_handlers(LOAD_ERROR_HANDLER_EXCEPTIONS));
    exceptions
}

#[derive(Debug, Clone, Deserialize)]
pub struct NoNoninteractiveElementInteractions(Box<NoNoninteractiveElementInteractionsConfig>);

impl Default for NoNoninteractiveElementInteractions {
    fn default() -> Self {
        Self(Box::new(NoNoninteractiveElementInteractionsConfig {
            handlers: Some(compact_handlers(RECOMMENDED_HANDLERS)),
            handler_exceptions: recommended_handler_exceptions(),
        }))
    }
}

#[derive(Debug, Default, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct NoNoninteractiveElementInteractionsConfig {
    /// An array of event handler names that should trigger this rule.
    handlers: Option<Vec<CompactStr>>,
    /// A mapping of HTML element names to handler names that should be ignored for that element.
    #[serde(flatten)]
    handler_exceptions: FxHashMap<CompactStr, Vec<CompactStr>>,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prevents non-interactive HTML elements and elements with non-interactive ARIA roles from
    /// being assigned mouse or keyboard event handlers.
    ///
    /// ### Why is this bad?
    ///
    /// Non-interactive elements such as `<main>`, `<h1>`, `<p>`, `<img>`, `<li>`, `<ul>`, and
    /// `<ol>` represent content or containers. Adding interaction handlers to them can make the
    /// UI difficult or impossible to operate with assistive technology.
    ///
    /// Move the handler to an interactive element, such as `<button>` or `<a href>`, or use an
    /// element with an appropriate interactive role and keyboard behavior.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <li onClick={() => {}} />
    /// <div role="listitem" onKeyDown={() => {}} />
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <button onClick={() => {}} />
    /// <div role="button" onClick={() => {}} />
    /// <div onClick={() => {}} role="presentation" />
    /// ```
    NoNoninteractiveElementInteractions,
    jsx_a11y,
    correctness,
    config = NoNoninteractiveElementInteractionsConfig,
    version = "next",
);

impl Rule for NoNoninteractiveElementInteractions {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(jsx_el) = node.kind() else {
            return;
        };

        let element_type = get_element_type(ctx, jsx_el);
        if !HTML_TAG.contains(element_type.as_ref()) {
            return;
        }

        if !self.has_interactive_handler(jsx_el, element_type.as_ref()) {
            return;
        }

        let role = first_role(jsx_el);
        if is_content_editable(jsx_el)
            || is_hidden_from_screen_reader(ctx, jsx_el)
            || is_presentation_role(jsx_el)
            || role.as_deref().is_some_and(|role| matches!(role, "presentation" | "none"))
        {
            return;
        }

        if role.as_deref().is_some_and(is_interactive_role) || is_abstract_role(ctx, jsx_el) {
            return;
        }

        let is_non_interactive_element = is_non_interactive_element(element_type.as_ref(), jsx_el);
        // Oxc's `is_interactive_element` follows HTML interactive-content semantics, which are
        // broader than this rule's accessibility semantics. Some elements such as `<iframe>`,
        // `<label>`, and `<details>` are still non-interactive for this rule.
        if is_interactive_element(element_type.as_ref(), jsx_el) && !is_non_interactive_element {
            return;
        }

        if !is_non_interactive_element && !role.as_deref().is_some_and(is_non_interactive_role) {
            return;
        }

        ctx.diagnostic(no_noninteractive_element_interactions_diagnostic(jsx_el.name.span()));
    }
}

impl NoNoninteractiveElementInteractions {
    fn has_interactive_handler(&self, jsx_el: &JSXOpeningElement, element_type: &str) -> bool {
        let ignored_handlers =
            self.0.handler_exceptions.get(element_type).map_or([].as_slice(), Vec::as_slice);

        match &self.0.handlers {
            Some(handlers) => handlers
                .iter()
                .any(|handler| has_active_handler(jsx_el, handler.as_str(), ignored_handlers)),
            None => DEFAULT_HANDLER_GROUPS
                .iter()
                .flat_map(|handlers| handlers.iter())
                .any(|handler| has_active_handler(jsx_el, handler, ignored_handlers)),
        }
    }
}

fn has_active_handler(
    jsx_el: &JSXOpeningElement,
    handler: &str,
    ignored_handlers: &[CompactStr],
) -> bool {
    if ignored_handlers.iter().any(|ignored| ignored.as_str() == handler) {
        return false;
    }

    has_jsx_prop(jsx_el, handler)
        .and_then(get_prop_value)
        .is_some_and(|value| !is_nullish_value(value))
}

fn is_nullish_value(value: &JSXAttributeValue) -> bool {
    matches!(
        value,
        JSXAttributeValue::ExpressionContainer(container)
            if matches!(container.expression, JSXExpression::NullLiteral(_))
                || container.expression.is_undefined()
    )
}

fn is_content_editable(jsx_el: &JSXOpeningElement) -> bool {
    has_jsx_prop(jsx_el, "contentEditable")
        .and_then(get_string_literal_prop_value)
        .is_some_and(|value| value == "true")
}

fn first_role<'a, 'b>(jsx_el: &'b JSXOpeningElement<'a>) -> Option<Cow<'b, str>> {
    has_jsx_prop_ignore_case(jsx_el, "role")
        .and_then(get_string_literal_prop_value)
        .and_then(|role| role.split_whitespace().next())
        .map(|role| role.cow_to_lowercase())
}

#[test]
fn test() {
    use crate::{
        rule::RuleMeta,
        tester::{TestCase, Tester},
    };

    fn components_settings() -> serde_json::Value {
        serde_json::json!({
            "settings": { "jsx-a11y": {
                "components": {
                    "Button": "button",
                    "Image": "img",
                }
            } }
        })
    }

    fn strict_config() -> serde_json::Value {
        serde_json::json!([{
            "body": ["onError", "onLoad"],
            "iframe": ["onError", "onLoad"],
            "img": ["onError", "onLoad"]
        }])
    }

    let mut pass: Vec<TestCase> = vec![
        (r"<TestComponent onClick={doFoo} />", None, None).into(),
        (r"<Button onClick={doFoo} />", None, None).into(),
        (r"<Button onClick={doFoo} />", None, Some(components_settings())).into(),
        (r"<button onClick={() => void 0} />", None, None).into(),
        (r#"<a href="/" onClick={() => void 0} />"#, None, None).into(),
        (r"<a onClick={() => void 0} />", None, None).into(),
        (r#"<input type="text" onClick={() => void 0} />"#, None, None).into(),
        (r#"<input type="hidden" onClick={() => void 0} />"#, None, None).into(),
        (r"<div onClick={() => void 0} />", None, None).into(),
        (r#"<div role="button" onClick={() => void 0} />"#, None, None).into(),
        (r#"<li role="button" onClick={() => void 0} />"#, None, None).into(),
        (r#"<button role="listitem" onClick={() => void 0} />"#, None, None).into(),
        (r#"<div role="presentation" onClick={() => void 0} />"#, None, None).into(),
        (r#"<div role="none" onClick={() => void 0} />"#, None, None).into(),
        (r"<main onClick={null} />", None, None).into(),
        (r"<main onClick={undefined} />", None, None).into(),
        (r"<main onFocus={() => void 0} />", None, None).into(),
        (r#"<img onLoad={() => void 0} alt="" />"#, None, None).into(),
        (r"<iframe onLoad={() => void 0} />", None, None).into(),
        (r"<dialog onKeyDown={() => void 0} />", None, None).into(),
        (r#"<article contentEditable="true" onClick={() => void 0} />"#, None, None).into(),
        (
            r"<main onKeyDown={() => void 0} />",
            Some(serde_json::json!([{ "handlers": ["onClick"] }])),
            None,
        )
            .into(),
        (
            r"<main onClick={() => void 0} />",
            Some(serde_json::json!([{ "handlers": ["onClick"], "main": ["onClick"] }])),
            None,
        )
            .into(),
        (r#"<img onLoad={() => void 0} alt="" />"#, Some(strict_config()), None).into(),
    ];

    for input_type in [
        "button",
        "checkbox",
        "color",
        "date",
        "datetime",
        "datetime-local",
        "email",
        "file",
        "image",
        "month",
        "number",
        "password",
        "radio",
        "range",
        "reset",
        "search",
        "submit",
        "tel",
        "time",
        "url",
        "week",
    ] {
        pass.push(format!(r#"<input type="{input_type}" onClick={{() => void 0}} />"#).into());
    }

    for tag in [
        "area", "audio", "canvas", "menuitem", "option", "select", "summary", "td", "textarea",
        "th", "tr", "video",
    ] {
        pass.push(format!("<{tag} onClick={{() => void 0}} />").into());
    }

    pass.extend(
        [
            r"<main onClick={() => void 0} aria-hidden />",
            r"<main onClick={() => void 0} aria-hidden={true} />",
            r"<body onLoad={() => void 0} />",
            r#"<iframe onError={() => void 0} />"#,
            r#"<img onError={() => void 0} alt="" />"#,
        ]
        .into_iter()
        .map(TestCase::from),
    );

    for role in [
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
    ] {
        pass.push(format!(r#"<div role="{role}" onClick={{() => void 0}} />"#).into());
    }

    for handler in [
        "onAbort",
        "onAnimationEnd",
        "onAnimationIteration",
        "onAnimationStart",
        "onBlur",
        "onCanPlay",
        "onCanPlayThrough",
        "onChange",
        "onCompositionEnd",
        "onCompositionStart",
        "onCompositionUpdate",
        "onContextMenu",
        "onCopy",
        "onCut",
        "onDblClick",
        "onDoubleClick",
        "onDrag",
        "onDragEnd",
        "onDragEnter",
        "onDragExit",
        "onDragLeave",
        "onDragOver",
        "onDragStart",
        "onDrop",
        "onDurationChange",
        "onEmptied",
        "onEncrypted",
        "onEnded",
        "onFocus",
        "onInput",
        "onLoadStart",
        "onLoadedData",
        "onLoadedMetadata",
        "onMouseEnter",
        "onMouseLeave",
        "onMouseMove",
        "onMouseOut",
        "onMouseOver",
        "onPaste",
        "onPause",
        "onPlay",
        "onPlaying",
        "onProgress",
        "onRateChange",
        "onScroll",
        "onSeeked",
        "onSeeking",
        "onSelect",
        "onStalled",
        "onSubmit",
        "onSuspend",
        "onTimeUpdate",
        "onTouchCancel",
        "onTouchEnd",
        "onTouchMove",
        "onTouchStart",
        "onTransitionEnd",
        "onVolumeChange",
        "onWaiting",
        "onWheel",
    ] {
        pass.push(format!(r#"<div role="article" {handler}={{() => void 0}} />"#).into());
    }

    let mut fail: Vec<TestCase> = vec![
        (r"<main onClick={() => void 0} />", None, None).into(),
        (r"<li onClick={() => void 0} />", None, None).into(),
        (r#"<img onClick={() => void 0} alt="" />"#, None, None).into(),
        (r"<iframe onClick={() => void 0} />", None, None).into(),
        (r"<label onClick={() => void 0} />", None, None).into(),
        (r#"<section aria-label="Aardvark" onClick={() => void 0} />"#, None, None).into(),
        (r#"<div role="listitem" onClick={() => void 0} />"#, None, None).into(),
        (r#"<div role="article" onKeyDown={() => void 0} />"#, None, None).into(),
        (r"<Image onClick={() => void 0} />", None, Some(components_settings())).into(),
        (r"<article contentEditable onClick={() => void 0} />", None, None).into(),
        (r#"<ul contentEditable="false" onClick={() => void 0} />"#, None, None).into(),
        (r"<dialog onClick={() => void 0} />", None, None).into(),
        (r"<main onFocus={() => void 0} />", Some(strict_config()), None).into(),
        (r#"<div role="article" onBlur={() => void 0} />"#, Some(strict_config()), None).into(),
        (r#"<div role="article" onContextMenu={() => void 0} />"#, Some(strict_config()), None)
            .into(),
        (r#"<img onLoad={() => void 0} alt="" />"#, Some(serde_json::json!([{}])), None).into(),
        (
            r"<main onClick={() => void 0} />",
            Some(serde_json::json!([{ "handlers": ["onClick"] }])),
            None,
        )
            .into(),
        (r"<dialog onKeyDown={() => void 0} />", Some(strict_config()), None).into(),
        (r"<main onClick={() => void 0} role={undefined} />", None, None).into(),
        (r"<main onClick={() => void 0} {...props} />", None, None).into(),
    ];

    for tag in [
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
        "ins",
        "legend",
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
        "strong",
        "sub",
        "sup",
        "table",
        "tbody",
        "tfoot",
        "thead",
        "time",
        "ul",
    ] {
        fail.push(format!("<{tag} onClick={{() => void 0}} />").into());
    }

    for role in [
        "alert",
        "alertdialog",
        "application",
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
        "log",
        "main",
        "marquee",
        "math",
        "navigation",
        "note",
        "paragraph",
        "progressbar",
        "region",
        "rowgroup",
        "search",
        "status",
        "table",
        "tabpanel",
        "term",
        "time",
        "timer",
        "tooltip",
    ] {
        fail.push(format!(r#"<div role="{role}" onClick={{() => void 0}} />"#).into());
    }

    for handler in ["onError", "onLoad", "onMouseDown", "onMouseUp"] {
        fail.push(format!(r#"<div role="article" {handler}={{() => void 0}} />"#).into());
    }

    for handler in [
        "onDoubleClick",
        "onDrag",
        "onDragEnd",
        "onDragEnter",
        "onDragExit",
        "onDragLeave",
        "onDragOver",
        "onDragStart",
        "onDrop",
        "onMouseEnter",
        "onMouseLeave",
        "onMouseMove",
        "onMouseOut",
        "onMouseOver",
    ] {
        let code = format!(r#"<div role="article" {handler}={{() => void 0}} />"#);
        fail.push((code.as_str(), Some(strict_config()), None).into());
    }

    Tester::new(
        NoNoninteractiveElementInteractions::NAME,
        NoNoninteractiveElementInteractions::PLUGIN,
        pass,
        fail,
    )
    .test_and_snapshot();
}
