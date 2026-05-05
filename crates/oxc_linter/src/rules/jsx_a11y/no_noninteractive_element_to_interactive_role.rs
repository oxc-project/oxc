use oxc_ast::{AstKind, ast::JSXAttributeValue};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_str::CompactStr;
use rustc_hash::FxHashMap;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    context::LintContext,
    globals::HTML_TAG,
    rule::Rule,
    utils::{
        get_element_type, has_jsx_prop_ignore_case, is_interactive_role, is_non_interactive_element,
    },
};

fn no_noninteractive_element_to_interactive_role_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Non-interactive elements should not be assigned interactive roles.")
        .with_help("Remove the interactive role or use an appropriate interactive element instead.")
        .with_label(span)
}

/// Default allowed overrides matching the `eslint-plugin-jsx-a11y` recommended config.
fn default_allowed_roles() -> FxHashMap<CompactStr, Vec<CompactStr>> {
    let mut map = FxHashMap::default();
    map.insert(
        CompactStr::new("ul"),
        vec![
            CompactStr::new("menu"),
            CompactStr::new("menubar"),
            CompactStr::new("radiogroup"),
            CompactStr::new("tablist"),
            CompactStr::new("tree"),
            CompactStr::new("treegrid"),
        ],
    );
    map.insert(
        CompactStr::new("ol"),
        vec![
            CompactStr::new("menu"),
            CompactStr::new("menubar"),
            CompactStr::new("radiogroup"),
            CompactStr::new("tablist"),
            CompactStr::new("tree"),
            CompactStr::new("treegrid"),
        ],
    );
    map.insert(
        CompactStr::new("li"),
        vec![
            CompactStr::new("menuitem"),
            CompactStr::new("menuitemcheckbox"),
            CompactStr::new("menuitemradio"),
            CompactStr::new("row"),
            CompactStr::new("tab"),
            CompactStr::new("treeitem"),
        ],
    );
    map.insert(
        CompactStr::new("fieldset"),
        vec![CompactStr::new("radiogroup"), CompactStr::new("presentation")],
    );
    map
}

#[derive(Debug, Clone)]
pub struct NoNoninteractiveElementToInteractiveRole(
    Box<NoNoninteractiveElementToInteractiveRoleConfig>,
);

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct NoNoninteractiveElementToInteractiveRoleConfig {
    /// A mapping of HTML element names to arrays of ARIA role strings that are
    /// allowed overrides for that element. For example, `{ "ul": ["menu", "tablist"] }`
    /// permits `<ul role="menu" />` without triggering the rule.
    ///
    /// Defaults are:
    /// ```json
    /// {
    ///   "ul": ["menu", "menubar", "radiogroup", "tablist", "tree", "treegrid"],
    ///   "ol": ["menu", "menubar", "radiogroup", "tablist", "tree", "treegrid"],
    ///   "li": ["menuitem", "menuitemcheckbox", "menuitemradio", "row", "tab", "treeitem"],
    ///   "fieldset": ["radiogroup", "presentation"]
    /// }
    /// ```
    allowed_roles: FxHashMap<CompactStr, Vec<CompactStr>>,
}

impl Default for NoNoninteractiveElementToInteractiveRole {
    fn default() -> Self {
        Self(Box::new(NoNoninteractiveElementToInteractiveRoleConfig {
            allowed_roles: default_allowed_roles(),
        }))
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Non-interactive HTML elements indicate _content_ and _containers_ in the user interface.
    /// Non-interactive elements include `<main>`, `<area>`, `<h1>` (through `<h6>`), `<p>`,
    /// `<img>`, `<li>`, `<ul>`, and `<ol>`.
    ///
    /// Interactive HTML elements indicate _controls_ in the user interface.
    /// Interactive elements include `<a href>`, `<button>`, `<input>`, `<select>`,
    /// `<textarea>`.
    ///
    /// [WAI-ARIA roles](https://www.w3.org/TR/wai-aria-1.1/#usage_intro) should not be used
    /// to convert a non-interactive element to an interactive element. Interactive ARIA roles
    /// include `button`, `link`, `checkbox`, `menuitem`, `menuitemcheckbox`,
    /// `menuitemradio`, `option`, `radio`, `searchbox`, `switch`, and `textbox`.
    ///
    /// ### Why is this bad?
    ///
    /// Overriding the semantic meaning of non-interactive elements with interactive roles
    /// creates confusion for assistive technology users. The element lacks the expected
    /// keyboard interaction patterns and focus management that interactive elements provide.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <h1 role="button">Click me</h1>
    /// <li role="link">Navigate</li>
    /// <article role="button">Submit</article>
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <button>Click me</button>
    /// <a href="/page">Navigate</a>
    /// <div role="button">Submit</div>
    /// <ul role="menu"><li role="menuitem">Item</li></ul>
    /// ```
    NoNoninteractiveElementToInteractiveRole,
    jsx_a11y,
    correctness,
    config = NoNoninteractiveElementToInteractiveRoleConfig,
    version = "next"
);

impl Rule for NoNoninteractiveElementToInteractiveRole {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(jsx_el) = node.kind() else {
            return;
        };

        // Only check elements that have a `role` attribute.
        let Some(role_attr_item) = has_jsx_prop_ignore_case(jsx_el, "role") else {
            return;
        };

        let oxc_ast::ast::JSXAttributeItem::Attribute(role_attr) = role_attr_item else {
            return;
        };

        // Get the role value — only string literals are checked.
        let Some(JSXAttributeValue::StringLiteral(role_value)) = &role_attr.value else {
            return;
        };

        let role = role_value.value.as_str().trim();

        // Take only the first role token (whitespace-separated).
        let Some(first_role) = role.split_whitespace().next() else {
            return;
        };

        let element_type = get_element_type(ctx, jsx_el);

        // Skip custom/unknown elements — only check known HTML tags.
        if !HTML_TAG.contains(element_type.as_ref()) {
            return;
        }

        // Skip if this element-role combination is allowed by config.
        if let Some(allowed) = self.0.allowed_roles.get(element_type.as_ref())
            && allowed.iter().any(|r: &CompactStr| r.as_str() == first_role)
        {
            return;
        }

        // Report if the element is non-interactive AND the role is interactive.
        if is_non_interactive_element(&element_type, jsx_el) && is_interactive_role(first_role) {
            ctx.diagnostic(no_noninteractive_element_to_interactive_role_diagnostic(
                role_attr.span,
            ));
        }
    }

    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        let Some(config) = value.get(0) else {
            return Ok(Self::default());
        };

        let Some(obj) = config.as_object() else {
            return Ok(Self::default());
        };

        let mut allowed_roles = FxHashMap::default();
        for (element, roles_value) in obj {
            if let Some(roles_arr) = roles_value.as_array() {
                let roles: Vec<CompactStr> =
                    roles_arr.iter().filter_map(|v| v.as_str().map(CompactStr::new)).collect();
                allowed_roles.insert(CompactStr::new(element), roles);
            }
        }

        Ok(Self(Box::new(NoNoninteractiveElementToInteractiveRoleConfig { allowed_roles })))
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    fn components_settings() -> serde_json::Value {
        serde_json::json!({
            "settings": { "jsx-a11y": {
                "components": {
                    "Article": "article",
                    "Input": "input",
                }
            } }
        })
    }

    // Default config uses recommended allowed roles.
    let pass = vec![
        // Custom components (not in HTML_TAG — skipped)
        (r"<TestComponent onClick={doFoo} />", None, None),
        (r"<Button onClick={doFoo} />", None, None),
        // Interactive elements with interactive roles (allowed)
        (r#"<a tabIndex="0" role="button" />"#, None, None),
        (r#"<a href="http://x.y.z" role="button" />"#, None, None),
        (r#"<a href="http://x.y.z" tabIndex="0" role="button" />"#, None, None),
        (r#"<area role="button" />"#, None, None),
        (r#"<area role="menuitem" />"#, None, None),
        (r#"<button className="foo" role="button" />"#, None, None),
        (r#"<body role="button" />"#, None, None),
        (r#"<frame role="button" />"#, None, None),
        (r#"<td role="button" />"#, None, None),
        (r#"<frame role="menuitem" />"#, None, None),
        (r#"<td role="menuitem" />"#, None, None),
        // All flavors of input (interactive elements)
        (r#"<input role="button" />"#, None, None),
        (r#"<input type="button" role="button" />"#, None, None),
        (r#"<input type="checkbox" role="button" />"#, None, None),
        (r#"<input type="color" role="button" />"#, None, None),
        (r#"<input type="date" role="button" />"#, None, None),
        (r#"<input type="datetime" role="button" />"#, None, None),
        (r#"<input type="datetime-local" role="button" />"#, None, None),
        (r#"<input type="email" role="button" />"#, None, None),
        (r#"<input type="file" role="button" />"#, None, None),
        (r#"<input type="hidden" role="button" />"#, None, None),
        (r#"<input type="image" role="button" />"#, None, None),
        (r#"<input type="month" role="button" />"#, None, None),
        (r#"<input type="number" role="button" />"#, None, None),
        (r#"<input type="password" role="button" />"#, None, None),
        (r#"<input type="radio" role="button" />"#, None, None),
        (r#"<input type="range" role="button" />"#, None, None),
        (r#"<input type="reset" role="button" />"#, None, None),
        (r#"<input type="search" role="button" />"#, None, None),
        (r#"<input type="submit" role="button" />"#, None, None),
        (r#"<input type="tel" role="button" />"#, None, None),
        (r#"<input type="text" role="button" />"#, None, None),
        (r#"<input type="time" role="button" />"#, None, None),
        (r#"<input type="url" role="button" />"#, None, None),
        (r#"<input type="week" role="button" />"#, None, None),
        (r#"<input type="hidden" role="img" />"#, None, None),
        (r#"<menuitem role="button" />"#, None, None),
        (r#"<option className="foo" role="button" />"#, None, None),
        (r#"<select className="foo" role="button" />"#, None, None),
        (r#"<textarea className="foo" role="button" />"#, None, None),
        (r#"<tr role="button" />"#, None, None),
        (r#"<tr role="presentation" />"#, None, None),
        // Non-interactive roles on interactive elements (allowed)
        (r#"<a tabIndex="0" role="img" />"#, None, None),
        (r#"<a href="http://x.y.z" role="img" />"#, None, None),
        // Non-interactive roles on non-interactive elements (allowed)
        (r#"<article role="listitem" />"#, None, None),
        (r#"<dd role="listitem" />"#, None, None),
        (r#"<dfn role="listitem" />"#, None, None),
        (r#"<dt role="listitem" />"#, None, None),
        (r#"<fieldset role="listitem" />"#, None, None),
        (r#"<figure role="listitem" />"#, None, None),
        (r#"<form role="listitem" />"#, None, None),
        (r#"<h1 role="listitem" />"#, None, None),
        (r#"<h2 role="listitem" />"#, None, None),
        (r#"<h3 role="listitem" />"#, None, None),
        (r#"<h4 role="listitem" />"#, None, None),
        (r#"<h5 role="listitem" />"#, None, None),
        (r#"<h6 role="listitem" />"#, None, None),
        (r#"<hr role="listitem" />"#, None, None),
        (r#"<img role="listitem" />"#, None, None),
        (r#"<li role="listitem" />"#, None, None),
        (r#"<li role="presentation" />"#, None, None),
        (r#"<main role="listitem" />"#, None, None),
        (r#"<nav role="listitem" />"#, None, None),
        (r#"<ol role="listitem" />"#, None, None),
        (r#"<table role="listitem" />"#, None, None),
        (r#"<tbody role="listitem" />"#, None, None),
        (r#"<tfoot role="listitem" />"#, None, None),
        (r#"<thead role="listitem" />"#, None, None),
        (r#"<ul role="listitem" />"#, None, None),
        // Static/unknown HTML elements with interactive roles (not non-interactive — skipped)
        (r#"<div role="button" />"#, None, None),
        (r#"<div className="foo" role="button" />"#, None, None),
        (r#"<span role="button" />"#, None, None),
        (r#"<header role="button" />"#, None, None),
        (r#"<b role="button" />"#, None, None),
        (r#"<i role="button" />"#, None, None),
        // Non-interactive roles on div (allowed — div is not non-interactive)
        (r#"<div role="alert" />"#, None, None),
        (r#"<div role="article" />"#, None, None),
        (r#"<div role="listitem" />"#, None, None),
        (r#"<div role="presentation" />"#, None, None),
        // Abstract roles (allowed — not interactive)
        (r#"<div role="command" />"#, None, None),
        (r#"<div role="composite" />"#, None, None),
        (r#"<div role="widget" />"#, None, None),
        // Custom component not in settings (skipped)
        (r#"<Article role="button" />"#, None, None),
        // Custom component mapped via settings to interactive element (allowed)
        (r#"<Input role="button" />"#, None, Some(components_settings())),
        // Recommended allowed overrides (default config)
        (r#"<ul role="menu" />"#, None, None),
        (r#"<ul role="menubar" />"#, None, None),
        (r#"<ul role="radiogroup" />"#, None, None),
        (r#"<ul role="tablist" />"#, None, None),
        (r#"<ul role="tree" />"#, None, None),
        (r#"<ul role="treegrid" />"#, None, None),
        (r#"<ol role="menu" />"#, None, None),
        (r#"<ol role="menubar" />"#, None, None),
        (r#"<ol role="radiogroup" />"#, None, None),
        (r#"<ol role="tablist" />"#, None, None),
        (r#"<ol role="tree" />"#, None, None),
        (r#"<ol role="treegrid" />"#, None, None),
        (r#"<li role="tab" />"#, None, None),
        (r#"<li role="menuitem" />"#, None, None),
        (r#"<li role="menuitemcheckbox" />"#, None, None),
        (r#"<li role="menuitemradio" />"#, None, None),
        (r#"<li role="row" />"#, None, None),
        (r#"<li role="treeitem" />"#, None, None),
        (r#"<fieldset role="radiogroup" />"#, None, None),
        (r#"<fieldset role="presentation" />"#, None, None),
    ];

    let fail = vec![
        // Non-interactive elements assigned interactive roles
        (r#"<address role="button" />"#, None, None),
        (r#"<article role="button" />"#, None, None),
        (r#"<aside role="button" />"#, None, None),
        (r#"<blockquote role="button" />"#, None, None),
        (r#"<br role="button" />"#, None, None),
        (r#"<caption role="button" />"#, None, None),
        (r#"<code role="button" />"#, None, None),
        (r#"<dd role="button" />"#, None, None),
        (r#"<del role="button" />"#, None, None),
        (r#"<details role="button" />"#, None, None),
        (r#"<dfn role="button" />"#, None, None),
        (r#"<dir role="button" />"#, None, None),
        (r#"<dl role="button" />"#, None, None),
        (r#"<dt role="button" />"#, None, None),
        (r#"<em role="button" />"#, None, None),
        (r#"<fieldset role="button" />"#, None, None),
        (r#"<figcaption role="button" />"#, None, None),
        (r#"<figure role="button" />"#, None, None),
        (r#"<footer role="button" />"#, None, None),
        (r#"<form role="button" />"#, None, None),
        (r#"<h1 role="button" />"#, None, None),
        (r#"<h2 role="button" />"#, None, None),
        (r#"<h3 role="button" />"#, None, None),
        (r#"<h4 role="button" />"#, None, None),
        (r#"<h5 role="button" />"#, None, None),
        (r#"<h6 role="button" />"#, None, None),
        (r#"<hr role="button" />"#, None, None),
        (r#"<html role="button" />"#, None, None),
        (r#"<iframe role="button" />"#, None, None),
        (r#"<img role="button" />"#, None, None),
        (r#"<ins role="button" />"#, None, None),
        (r#"<label role="button" />"#, None, None),
        (r#"<legend role="button" />"#, None, None),
        (r#"<li role="button" />"#, None, None),
        (r#"<main role="button" />"#, None, None),
        (r#"<mark role="button" />"#, None, None),
        (r#"<marquee role="button" />"#, None, None),
        (r#"<menu role="button" />"#, None, None),
        (r#"<meter role="button" />"#, None, None),
        (r#"<nav role="button" />"#, None, None),
        (r#"<ol role="button" />"#, None, None),
        (r#"<optgroup role="button" />"#, None, None),
        (r#"<output role="button" />"#, None, None),
        (r#"<pre role="button" />"#, None, None),
        (r#"<progress role="button" />"#, None, None),
        (r#"<ruby role="button" />"#, None, None),
        (r#"<strong role="button" />"#, None, None),
        (r#"<sub role="button" />"#, None, None),
        (r#"<sup role="button" />"#, None, None),
        (r#"<table role="button" />"#, None, None),
        (r#"<tbody role="button" />"#, None, None),
        (r#"<tfoot role="button" />"#, None, None),
        (r#"<thead role="button" />"#, None, None),
        (r#"<time role="button" />"#, None, None),
        (r#"<ul role="button" />"#, None, None),
        (r#"<p role="button" />"#, None, None),
        (r#"<section role="button" aria-label="Aardvark" />"#, None, None),
        // Non-interactive elements with menuitem role
        (r#"<main role="menuitem" />"#, None, None),
        (r#"<article role="menuitem" />"#, None, None),
        (r#"<dd role="menuitem" />"#, None, None),
        (r#"<dfn role="menuitem" />"#, None, None),
        (r#"<dt role="menuitem" />"#, None, None),
        (r#"<fieldset role="menuitem" />"#, None, None),
        (r#"<figure role="menuitem" />"#, None, None),
        (r#"<form role="menuitem" />"#, None, None),
        (r#"<h1 role="menuitem" />"#, None, None),
        (r#"<h2 role="menuitem" />"#, None, None),
        (r#"<h3 role="menuitem" />"#, None, None),
        (r#"<h4 role="menuitem" />"#, None, None),
        (r#"<h5 role="menuitem" />"#, None, None),
        (r#"<h6 role="menuitem" />"#, None, None),
        (r#"<hr role="menuitem" />"#, None, None),
        (r#"<img role="menuitem" />"#, None, None),
        (r#"<nav role="menuitem" />"#, None, None),
        (r#"<ol role="menuitem" />"#, None, None),
        (r#"<table role="menuitem" />"#, None, None),
        (r#"<tbody role="menuitem" />"#, None, None),
        (r#"<tfoot role="menuitem" />"#, None, None),
        (r#"<thead role="menuitem" />"#, None, None),
        // Custom component mapped via settings to non-interactive element
        (r#"<Article role="button" />"#, None, Some(components_settings())),
    ];

    // Strict mode tests: recommended allowed overrides become invalid.
    let strict_fail = vec![
        (r#"<ul role="menu" />"#, Some(serde_json::json!([{}])), None),
        (r#"<ul role="menubar" />"#, Some(serde_json::json!([{}])), None),
        (r#"<ul role="radiogroup" />"#, Some(serde_json::json!([{}])), None),
        (r#"<ul role="tablist" />"#, Some(serde_json::json!([{}])), None),
        (r#"<ul role="tree" />"#, Some(serde_json::json!([{}])), None),
        (r#"<ul role="treegrid" />"#, Some(serde_json::json!([{}])), None),
        (r#"<ol role="menu" />"#, Some(serde_json::json!([{}])), None),
        (r#"<ol role="menubar" />"#, Some(serde_json::json!([{}])), None),
        (r#"<ol role="radiogroup" />"#, Some(serde_json::json!([{}])), None),
        (r#"<ol role="tablist" />"#, Some(serde_json::json!([{}])), None),
        (r#"<ol role="tree" />"#, Some(serde_json::json!([{}])), None),
        (r#"<ol role="treegrid" />"#, Some(serde_json::json!([{}])), None),
        (r#"<li role="tab" />"#, Some(serde_json::json!([{}])), None),
        (r#"<li role="menuitem" />"#, Some(serde_json::json!([{}])), None),
        (r#"<li role="row" />"#, Some(serde_json::json!([{}])), None),
        (r#"<li role="treeitem" />"#, Some(serde_json::json!([{}])), None),
    ];

    let mut all_fail = fail;
    all_fail.extend(strict_fail);

    Tester::new(
        NoNoninteractiveElementToInteractiveRole::NAME,
        NoNoninteractiveElementToInteractiveRole::PLUGIN,
        pass,
        all_fail,
    )
    .test_and_snapshot();
}
