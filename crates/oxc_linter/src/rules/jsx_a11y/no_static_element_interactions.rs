use cow_utils::CowUtils;
use schemars::JsonSchema;
use serde::Deserialize;

use oxc_ast::{
    AstKind,
    ast::{JSXAttributeItem, JSXAttributeValue, JSXExpression},
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
        get_element_type, get_prop_value, has_jsx_prop, has_jsx_prop_ignore_case, is_abstract_role,
        is_hidden_from_screen_reader, is_interactive_element, is_interactive_role,
        is_non_interactive_element, is_non_interactive_role, is_presentation_role,
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
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
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
    nursery,
    config = NoStaticElementInteractionsConfig,
);

impl Rule for NoStaticElementInteractions {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(jsx_el) = node.kind() else {
            return;
        };

        // Note: We skip the handler if it exists with a `null` value, e.g. `<div onClick={null} />`.
        let has_handler = match &self.handlers {
            Some(handlers) => handlers.iter().any(|handler| {
                has_jsx_prop(jsx_el, handler.as_str())
                    .and_then(get_prop_value)
                    .is_some_and(|value| !is_null_value(value))
            }),
            None => DEFAULT_HANDLERS.iter().any(|handler| {
                has_jsx_prop(jsx_el, handler)
                    .and_then(get_prop_value)
                    .is_some_and(|value| !is_null_value(value))
            }),
        };

        if !has_handler {
            return;
        }

        let element_type = get_element_type(ctx, jsx_el);

        // Do not test custom JSX elements.
        if !HTML_TAG.contains(element_type.as_ref()) {
            return;
        }

        if is_hidden_from_screen_reader(ctx, jsx_el) || is_presentation_role(jsx_el) {
            return;
        }

        if is_interactive_element(&element_type, jsx_el) {
            return;
        }

        if is_non_interactive_element(&element_type, jsx_el) {
            return;
        }

        // This rule has no opinion on abstract roles, so just ignore them.
        if is_abstract_role(ctx, jsx_el) {
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
                    if is_interactive_role(first_role) {
                        return;
                    }
                    if is_non_interactive_role(first_role) {
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

fn is_null_value(value: &JSXAttributeValue) -> bool {
    matches!(
        value,
        JSXAttributeValue::ExpressionContainer(container)
            if matches!(container.expression, JSXExpression::NullLiteral(_))
    )
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (r"<div />", None),
        (r#"<div className="foo" />"#, None),
        (r#"<div className="foo" {...props} />"#, None),
        (r"<div onClick={() => void 0} aria-hidden />", None),
        (r"<div onClick={() => void 0} aria-hidden={true} />", None),
        (r"<div onClick={null} />", None),
        // All flavors of input
        (r"<input onClick={() => void 0} />", None),
        (r#"<input type="button" onClick={() => void 0} />"#, None),
        (r#"<input type="checkbox" onClick={() => void 0} />"#, None),
        (r#"<input type="color" onClick={() => void 0} />"#, None),
        (r#"<input type="date" onClick={() => void 0} />"#, None),
        (r#"<input type="datetime" onClick={() => void 0} />"#, None),
        (r#"<input type="datetime-local" onClick={() => void 0} />"#, None),
        (r#"<input type="email" onClick={() => void 0} />"#, None),
        (r#"<input type="file" onClick={() => void 0} />"#, None),
        (r#"<input type="hidden" onClick={() => void 0} />"#, None),
        (r#"<input type="image" onClick={() => void 0} />"#, None),
        (r#"<input type="month" onClick={() => void 0} />"#, None),
        (r#"<input type="number" onClick={() => void 0} />"#, None),
        (r#"<input type="password" onClick={() => void 0} />"#, None),
        (r#"<input type="radio" onClick={() => void 0} />"#, None),
        (r#"<input type="range" onClick={() => void 0} />"#, None),
        (r#"<input type="reset" onClick={() => void 0} />"#, None),
        (r#"<input type="search" onClick={() => void 0} />"#, None),
        (r#"<input type="submit" onClick={() => void 0} />"#, None),
        (r#"<input type="tel" onClick={() => void 0} />"#, None),
        (r#"<input type="text" onClick={() => void 0} />"#, None),
        (r#"<input type="time" onClick={() => void 0} />"#, None),
        (r#"<input type="url" onClick={() => void 0} />"#, None),
        (r#"<input type="week" onClick={() => void 0} />"#, None),
        // End all flavors of input
        (r#"<button onClick={() => void 0} className="foo" />"#, None),
        (r"<datalist onClick={() => {}} />;", None),
        (r"<menuitem onClick={() => {}} />;", None),
        (r#"<option onClick={() => void 0} className="foo" />"#, None),
        (r#"<select onClick={() => void 0} className="foo" />"#, None),
        (r#"<textarea onClick={() => void 0} className="foo" />"#, None),
        (r#"<a onClick={() => void 0} href="http://x.y.z" />"#, None),
        (r#"<a onClick={() => void 0} href="http://x.y.z" tabIndex="0" />"#, None),
        (r"<audio onClick={() => {}} />;", None),
        (r"<form onClick={() => {}} />;", None),
        (r"<form onSubmit={() => {}} />;", None),
        // HTML elements attributed with an interactive role
        (r#"<div role="button" onClick={() => {}} />;"#, None),
        (r#"<div role="checkbox" onClick={() => {}} />;"#, None),
        (r#"<div role="columnheader" onClick={() => {}} />;"#, None),
        (r#"<div role="combobox" onClick={() => {}} />;"#, None),
        (r#"<div role="form" onClick={() => {}} />;"#, None),
        (r#"<div role="gridcell" onClick={() => {}} />;"#, None),
        (r#"<div role="link" onClick={() => {}} />;"#, None),
        (r#"<div role="menuitem" onClick={() => {}} />;"#, None),
        (r#"<div role="menuitemcheckbox" onClick={() => {}} />;"#, None),
        (r#"<div role="menuitemradio" onClick={() => {}} />;"#, None),
        (r#"<div role="option" onClick={() => {}} />;"#, None),
        (r#"<div role="radio" onClick={() => {}} />;"#, None),
        (r#"<div role="rowheader" onClick={() => {}} />;"#, None),
        (r#"<div role="searchbox" onClick={() => {}} />;"#, None),
        (r#"<div role="slider" onClick={() => {}} />;"#, None),
        (r#"<div role="spinbutton" onClick={() => {}} />;"#, None),
        (r#"<div role="switch" onClick={() => {}} />;"#, None),
        (r#"<div role="tab" onClick={() => {}} />;"#, None),
        (r#"<div role="textbox" onClick={() => {}} />;"#, None),
        (r#"<div role="treeitem" onClick={() => {}} />;"#, None),
        // Presentation is a special case role that indicates intentional static semantics
        (r#"<div role="presentation" onClick={() => {}} />;"#, None),
        (r#"<div role="presentation" onKeyDown={() => {}} />;"#, None),
        // HTML elements with an inherent, non-interactive role
        (r"<address onClick={() => {}} />;", None),
        (r"<article onClick={() => {}} />;", None),
        (r"<article onDblClick={() => void 0} />;", None),
        (r"<aside onClick={() => {}} />;", None),
        (r"<blockquote onClick={() => {}} />;", None),
        (r"<br onClick={() => {}} />;", None),
        (r"<canvas onClick={() => {}} />;", None),
        (r"<caption onClick={() => {}} />;", None),
        (r"<code onClick={() => {}} />;", None),
        (r"<dd onClick={() => {}} />;", None),
        (r"<del onClick={() => {}} />;", None),
        (r"<details onClick={() => {}} />;", None),
        (r"<dfn onClick={() => {}} />;", None),
        (r"<dir onClick={() => {}} />;", None),
        (r"<dl onClick={() => {}} />;", None),
        (r"<dt onClick={() => {}} />;", None),
        (r"<em onClick={() => {}} />;", None),
        (r"<embed onClick={() => {}} />;", None),
        (r"<fieldset onClick={() => {}} />;", None),
        (r"<figcaption onClick={() => {}} />;", None),
        (r"<figure onClick={() => {}} />;", None),
        (r"<footer onClick={() => {}} />;", None),
        (r"<h1 onClick={() => {}} />;", None),
        (r"<h2 onClick={() => {}} />;", None),
        (r"<h3 onClick={() => {}} />;", None),
        (r"<h4 onClick={() => {}} />;", None),
        (r"<h5 onClick={() => {}} />;", None),
        (r"<h6 onClick={() => {}} />;", None),
        (r"<hr onClick={() => {}} />;", None),
        (r"<html onClick={() => {}} />;", None),
        (r"<iframe onClick={() => {}} />;", None),
        (r"<img onClick={() => {}} />;", None),
        (r"<ins onClick={() => {}} />;", None),
        (r"<label onClick={() => {}} />;", None),
        (r"<legend onClick={() => {}} />;", None),
        (r"<li onClick={() => {}} />;", None),
        (r"<main onClick={() => void 0} />;", None),
        (r"<mark onClick={() => {}} />;", None),
        (r"<marquee onClick={() => {}} />;", None),
        (r"<menu onClick={() => {}} />;", None),
        (r"<meter onClick={() => {}} />;", None),
        (r"<nav onClick={() => {}} />;", None),
        (r"<ol onClick={() => {}} />;", None),
        (r"<optgroup onClick={() => {}} />;", None),
        (r"<output onClick={() => {}} />;", None),
        (r"<p onClick={() => {}} />;", None),
        (r"<pre onClick={() => {}} />;", None),
        (r"<progress onClick={() => {}} />;", None),
        (r"<ruby onClick={() => {}} />;", None),
        (r#"<section onClick={() => {}} aria-label="Aa" />;"#, None),
        (r#"<section onClick={() => {}} aria-labelledby="js_1" />;"#, None),
        (r"<strong onClick={() => {}} />;", None),
        (r"<sub onClick={() => {}} />;", None),
        (r"<summary onClick={() => {}} />;", None),
        (r"<sup onClick={() => {}} />;", None),
        (r"<table onClick={() => {}} />;", None),
        (r"<tbody onClick={() => {}} />;", None),
        (r"<tfoot onClick={() => {}} />;", None),
        (r"<th onClick={() => {}} />;", None),
        (r"<thead onClick={() => {}} />;", None),
        (r"<time onClick={() => {}} />;", None),
        (r"<tr onClick={() => {}} />;", None),
        (r"<video onClick={() => {}} />;", None),
        (r"<ul onClick={() => {}} />;", None),
        // HTML elements attributed with an abstract role
        (r#"<div role="command" onClick={() => {}} />;"#, None),
        (r#"<div role="composite" onClick={() => {}} />;"#, None),
        (r#"<div role="input" onClick={() => {}} />;"#, None),
        (r#"<div role="landmark" onClick={() => {}} />;"#, None),
        (r#"<div role="range" onClick={() => {}} />;"#, None),
        (r#"<div role="roletype" onClick={() => {}} />;"#, None),
        (r#"<div role="sectionhead" onClick={() => {}} />;"#, None),
        (r#"<div role="select" onClick={() => {}} />;"#, None),
        (r#"<div role="structure" onClick={() => {}} />;"#, None),
        (r#"<div role="widget" onClick={() => {}} />;"#, None),
        (r#"<div role="window" onClick={() => {}} />;"#, None),
        // HTML elements attributed with a non-interactive role
        (r#"<div role="alert" onClick={() => {}} />;"#, None),
        (r#"<div role="alertdialog" onClick={() => {}} />;"#, None),
        (r#"<div role="application" onClick={() => {}} />;"#, None),
        (r#"<div role="article" onClick={() => {}} />;"#, None),
        (r#"<div role="banner" onClick={() => {}} />;"#, None),
        (r#"<div role="cell" onClick={() => {}} />;"#, None),
        (r#"<div role="complementary" onClick={() => {}} />;"#, None),
        (r#"<div role="contentinfo" onClick={() => {}} />;"#, None),
        (r#"<div role="definition" onClick={() => {}} />;"#, None),
        (r#"<div role="dialog" onClick={() => {}} />;"#, None),
        (r#"<div role="directory" onClick={() => {}} />;"#, None),
        (r#"<div role="document" onClick={() => {}} />;"#, None),
        (r#"<div role="feed" onClick={() => {}} />;"#, None),
        (r#"<div role="figure" onClick={() => {}} />;"#, None),
        (r#"<div role="grid" onClick={() => {}} />;"#, None),
        (r#"<div role="group" onClick={() => {}} />;"#, None),
        (r#"<div role="heading" onClick={() => {}} />;"#, None),
        (r#"<div role="img" onClick={() => {}} />;"#, None),
        (r#"<div role="list" onClick={() => {}} />;"#, None),
        (r#"<div role="listbox" onClick={() => {}} />;"#, None),
        (r#"<div role="listitem" onClick={() => {}} />;"#, None),
        (r#"<div role="log" onClick={() => {}} />;"#, None),
        (r#"<div role="main" onClick={() => {}} />;"#, None),
        (r#"<div role="marquee" onClick={() => {}} />;"#, None),
        (r#"<div role="math" onClick={() => {}} />;"#, None),
        (r#"<div role="menu" onClick={() => {}} />;"#, None),
        (r#"<div role="menubar" onClick={() => {}} />;"#, None),
        (r#"<div role="navigation" onClick={() => {}} />;"#, None),
        (r#"<div role="note" onClick={() => {}} />;"#, None),
        (r#"<div role="progressbar" onClick={() => {}} />;"#, None),
        (r#"<div role="radiogroup" onClick={() => {}} />;"#, None),
        (r#"<div role="region" onClick={() => {}} />;"#, None),
        (r#"<div role="row" onClick={() => {}} />;"#, None),
        (r#"<div role="rowgroup" onClick={() => {}} />;"#, None),
        (r#"<div role="section" onClick={() => {}} />;"#, None),
        (r#"<div role="search" onClick={() => {}} />;"#, None),
        (r#"<div role="separator" onClick={() => {}} />;"#, None),
        (r#"<div role="scrollbar" onClick={() => {}} />;"#, None),
        (r#"<div role="status" onClick={() => {}} />;"#, None),
        (r#"<div role="table" onClick={() => {}} />;"#, None),
        (r#"<div role="tablist" onClick={() => {}} />;"#, None),
        (r#"<div role="tabpanel" onClick={() => {}} />;"#, None),
        (r#"<div role="term" onClick={() => {}} />;"#, None),
        (r#"<div role="timer" onClick={() => {}} />;"#, None),
        (r#"<div role="toolbar" onClick={() => {}} />;"#, None),
        (r#"<div role="tooltip" onClick={() => {}} />;"#, None),
        (r#"<div role="tree" onClick={() => {}} />;"#, None),
        (r#"<div role="treegrid" onClick={() => {}} />;"#, None),
        // All the possible handlers
        ("<div onCopy={() => {}} />;", None),
        ("<div onCut={() => {}} />;", None),
        ("<div onPaste={() => {}} />;", None),
        ("<div onCompositionEnd={() => {}} />;", None),
        ("<div onCompositionStart={() => {}} />;", None),
        ("<div onCompositionUpdate={() => {}} />;", None),
        ("<div onChange={() => {}} />;", None),
        ("<div onInput={() => {}} />;", None),
        ("<div onSubmit={() => {}} />;", None),
        ("<div onSelect={() => {}} />;", None),
        ("<div onTouchCancel={() => {}} />;", None),
        ("<div onTouchEnd={() => {}} />;", None),
        ("<div onTouchMove={() => {}} />;", None),
        ("<div onTouchStart={() => {}} />;", None),
        ("<div onScroll={() => {}} />;", None),
        ("<div onWheel={() => {}} />;", None),
        ("<div onAbort={() => {}} />;", None),
        ("<div onCanPlay={() => {}} />;", None),
        ("<div onCanPlayThrough={() => {}} />;", None),
        ("<div onDurationChange={() => {}} />;", None),
        ("<div onEmptied={() => {}} />;", None),
        ("<div onEncrypted={() => {}} />;", None),
        ("<div onEnded={() => {}} />;", None),
        ("<div onError={() => {}} />;", None),
        ("<div onLoadedData={() => {}} />;", None),
        ("<div onLoadedMetadata={() => {}} />;", None),
        ("<div onLoadStart={() => {}} />;", None),
        ("<div onPause={() => {}} />;", None),
        ("<div onPlay={() => {}} />;", None),
        ("<div onPlaying={() => {}} />;", None),
        ("<div onProgress={() => {}} />;", None),
        ("<div onRateChange={() => {}} />;", None),
        ("<div onSeeked={() => {}} />;", None),
        ("<div onSeeking={() => {}} />;", None),
        ("<div onStalled={() => {}} />;", None),
        ("<div onSuspend={() => {}} />;", None),
        ("<div onTimeUpdate={() => {}} />;", None),
        ("<div onVolumeChange={() => {}} />;", None),
        ("<div onWaiting={() => {}} />;", None),
        ("<div onLoad={() => {}} />;", None),
        ("<div onError={() => {}} />;", None),
        ("<div onAnimationStart={() => {}} />;", None),
        ("<div onAnimationEnd={() => {}} />;", None),
        ("<div onAnimationIteration={() => {}} />;", None),
        // other tests
        (r#"<div className="foo" onClick={() => {}} role="button" />;"#, None),
        (r#"<div className="foo" onKeyDown={() => {}} role="button" />;"#, None),
        (r#"<div onClick={() => {}} role="button" />;"#, None),
        (r#"<button onClick={() => {}} className="foo" />;"#, None),
        (r#"<input type="text" onClick={() => {}} />;"#, None),
        (r"<input onClick={() => {}} />;", None),
        (r#"<button onClick={() => {}} className="foo" />;"#, None),
        (r"<select onClick={() => {}} />;", None),
        (r"<textarea onClick={() => {}} />;", None),
        (r#"<input type="hidden" onClick={() => {}} />;"#, None),
        (r#"<div onClick={() => {}} role="presentation" />;"#, None),
        (r#"<div onClick={() => {}} role="none" />;"#, None),
        (r#"<div onMouseDown={() => {}} role="button" />;"#, None),
        (r#"<div onMouseUp={() => {}} role="button" />;"#, None),
        (r#"<div onKeyPress={() => {}} role="button" />;"#, None),
        (r#"<div onKeyDown={() => {}} role="button" />;"#, None),
        (r#"<div onKeyUp={() => {}} role="button" />;"#, None),
        (r"<div onClick={() => {}} aria-hidden />;", None),
        (r"<div onClick={() => {}} aria-hidden={true} />;", None),
        (r#"<div onClick={() => {}} aria-hidden={false} role="button" />;"#, None),
        (r#"<input onClick={() => {}} type="submit" />;"#, None),
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
        // Test that it works regardless of order.
        (r#"<div onClick={() => {}} role="tab" />;"#, None),
        // Test that it works regardless of extra attributes.
        (r#"<div role="tab" onClick={() => {}} style="color: red;" />;"#, None),
    ];

    let fail = vec![
        (r"<div onClick={() => {}} aria-hidden={false} />;", None),
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
        // Start of ported tests.
        (r"<div onClick={() => void 0} />;", None),
        (r"<div onClick={() => void 0} role={undefined} />;", None),
        (r"<div onClick={() => void 0} {...props} />;", None),
        (r"<div onKeyUp={() => void 0} aria-hidden={false} />;", None),
        // Static elements; no inherent role
        (r"<a onClick={() => {}} />;", None),
        (r"<a onClick={() => void 0} />", None),
        (r#"<a tabIndex="0" onClick={() => void 0} />"#, None),
        (r"<acronym onClick={() => {}} />;", None),
        (r"<applet onClick={() => {}} />;", None),
        (r"<area onClick={() => {}} />;", None),
        (r"<b onClick={() => {}} />;", None),
        (r"<base onClick={() => {}} />;", None),
        (r"<bdi onClick={() => {}} />;", None),
        (r"<bdo onClick={() => {}} />;", None),
        (r"<big onClick={() => {}} />;", None),
        (r"<blink onClick={() => {}} />;", None),
        (r"<body onClick={() => {}} />;", None),
        (r"<center onClick={() => {}} />;", None),
        (r"<cite onClick={() => {}} />;", None),
        (r"<col onClick={() => {}} />;", None),
        (r"<colgroup onClick={() => {}} />;", None),
        (r"<content onClick={() => {}} />;", None),
        (r"<data onClick={() => {}} />;", None),
        (r"<font onClick={() => {}} />;", None),
        (r"<frame onClick={() => {}} />;", None),
        (r"<frameset onClick={() => {}} />;", None),
        (r"<head onClick={() => {}} />;", None),
        (r"<header onClick={() => {}} />;", None),
        (r"<hgroup onClick={() => {}} />;", None),
        (r"<i onClick={() => {}} />;", None),
        (r"<kbd onClick={() => {}} />;", None),
        (r"<keygen onClick={() => {}} />;", None),
        (r"<map onClick={() => {}} />;", None),
        (r"<meta onClick={() => {}} />;", None),
        (r"<noembed onClick={() => {}} />;", None),
        (r"<noscript onClick={() => {}} />;", None),
        (r"<object onClick={() => {}} />;", None),
        (r"<param onClick={() => {}} />;", None),
        (r"<picture onClick={() => {}} />;", None),
        (r"<q onClick={() => {}} />;", None),
        (r"<rp onClick={() => {}} />;", None),
        (r"<rt onClick={() => {}} />;", None),
        (r"<rtc onClick={() => {}} />;", None),
        (r"<s onClick={() => {}} />;", None),
        (r"<samp onClick={() => {}} />;", None),
        (r"<script onClick={() => {}} />;", None),
        (r"<section onClick={() => {}} />;", None),
        (r"<small onClick={() => {}} />;", None),
        (r"<source onClick={() => {}} />;", None),
        (r"<spacer onClick={() => {}} />;", None),
        (r"<span onClick={() => {}} />;", None),
        (r"<strike onClick={() => {}} />;", None),
        (r"<style onClick={() => {}} />;", None),
        (r"<title onClick={() => {}} />;", None),
        (r"<track onClick={() => {}} />;", None),
        (r"<tt onClick={() => {}} />;", None),
        (r"<u onClick={() => {}} />;", None),
        (r"<var onClick={() => {}} />;", None),
        (r"<wbr onClick={() => {}} />;", None),
        (r"<xmp onClick={() => {}} />;", None),
        // Handlers
        (r"<div onKeyDown={() => {}} />;", None),
        (r"<div onKeyPress={() => {}} />;", None),
        (r"<div onKeyUp={() => {}} />;", None),
        (r"<div onClick={() => {}} />;", None),
        (r"<div onMouseDown={() => {}} />;", None),
        (r"<div onMouseUp={() => {}} />;", None),
        // More possible handlers, these only fail in "strict mode" which does not allow expressions and has handlers unset, which results in all
        // focus, keyboard, and mouse handlers being activated.
        // (r"<div onContextMenu={() => {}} />;", None),
        // (r"<div onDblClick={() => {}} />;", None),
        // (r"<div onDoubleClick={() => {}} />;", None),
        // (r"<div onDrag={() => {}} />;", None),
        // (r"<div onDragEnd={() => {}} />;", None),
        // (r"<div onDragEnter={() => {}} />;", None),
        // (r"<div onDragExit={() => {}} />;", None),
        // (r"<div onDragLeave={() => {}} />;", None),
        // (r"<div onDragOver={() => {}} />;", None),
        // (r"<div onDragStart={() => {}} />;", None),
        // (r"<div onDrop={() => {}} />;", None),
        // (r"<div onMouseEnter={() => {}} />;", None),
        // (r"<div onMouseLeave={() => {}} />;", None),
        // (r"<div onMouseMove={() => {}} />;", None),
        // (r"<div onMouseOut={() => {}} />;", None),
        // (r"<div onMouseOver={() => {}} />;", None),
    ];

    Tester::new(NoStaticElementInteractions::NAME, NoStaticElementInteractions::PLUGIN, pass, fail)
        .test_and_snapshot();
}
