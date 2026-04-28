use oxc_ast::AstKind;
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
    rule::{DefaultRuleConfig, Rule},
    utils::{
        get_element_type, get_string_literal_prop_value, has_jsx_prop_ignore_case,
        is_interactive_element, is_non_interactive_role, is_presentation_role,
    },
};

fn no_interactive_element_to_noninteractive_role_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Interactive elements should not be assigned non-interactive roles.")
        .with_help("Use a non-interactive element instead, or remove the role attribute.")
        .with_label(span)
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct NoInteractiveElementToNoninteractiveRole(
    Box<NoInteractiveElementToNoninteractiveRoleConfig>,
);

#[derive(Debug, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct NoInteractiveElementToNoninteractiveRoleConfig {
    /// A map of element/component names to arrays of ARIA roles that are allowed
    /// for that element even if they are non-interactive roles.
    pub allowed_roles: FxHashMap<CompactStr, Vec<CompactStr>>,
}

impl Default for NoInteractiveElementToNoninteractiveRoleConfig {
    fn default() -> Self {
        let mut allowed_roles = FxHashMap::default();
        allowed_roles.insert(CompactStr::new("tr"), vec!["none".into(), "presentation".into()]);
        allowed_roles.insert(CompactStr::new("canvas"), vec!["img".into()]);
        Self { allowed_roles }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce that interactive HTML elements are not assigned non-interactive
    /// ARIA roles.
    ///
    /// Interactive HTML elements indicate _controls_ in the user interface.
    /// Interactive elements include `<a href>`, `<button>`, `<input>`,
    /// `<select>`, `<textarea>`.
    ///
    /// Non-interactive HTML elements and non-interactive ARIA roles indicate
    /// _content_ and _containers_ in the user interface. Non-interactive
    /// elements include `<main>`, `<area>`, `<h1>` (,`<h2>`, etc), `<img>`,
    /// `<li>`, `<ul>` and `<ol>`.
    ///
    /// ### Why is this bad?
    ///
    /// [WAI-ARIA roles](https://www.w3.org/TR/wai-aria-1.1/#usage_intro)
    /// should not be used to convert an interactive element to a
    /// non-interactive element. Non-interactive ARIA roles include `article`,
    /// `banner`, `complementary`, `img`, `listitem`, `main`, `region` and
    /// `tooltip`.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <input role="img" />
    /// <button role="article" />
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <div role="article">
    ///   <button>Save</button>
    /// </div>
    ///
    /// <div
    ///   role="button"
    ///   onClick={() => {}}
    ///   onKeyPress={() => {}}
    ///   tabIndex="0">
    ///   <div role="img" aria-label="Save" />
    /// </div>
    /// ```
    ///
    NoInteractiveElementToNoninteractiveRole,
    jsx_a11y,
    correctness,
    config = NoInteractiveElementToNoninteractiveRoleConfig,
    version = "next",
);

impl Rule for NoInteractiveElementToNoninteractiveRole {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(jsx_el) = node.kind() else {
            return;
        };
        let Some(role) =
            has_jsx_prop_ignore_case(jsx_el, "role").and_then(get_string_literal_prop_value)
        else {
            return;
        };
        let element_type = get_element_type(ctx, jsx_el);

        if let Some(allowed) = self.0.allowed_roles.get(element_type.as_ref())
            && allowed.iter().any(|r| r.as_str() == role)
        {
            return;
        }

        if is_interactive_element(&element_type, jsx_el)
            && (is_non_interactive_role(role) || is_presentation_role(jsx_el))
        {
            ctx.diagnostic(no_interactive_element_to_noninteractive_role_diagnostic(jsx_el.span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    fn settings() -> serde_json::Value {
        serde_json::json!({
            "settings": { "jsx-a11y": {
                "components": {
                    "Button": "button",
                    "Link": "a",
                }
            }}
        })
    }

    let pass: Vec<(&str, Option<serde_json::Value>, Option<serde_json::Value>)> = vec![
        ("<TestComponent onClick={doFoo} />", None, None),
        ("<Button onClick={doFoo} />", None, None),
        (r#"<a href="http://x.y.z" role="button" />"#, None, None),
        (r#"<a href="http://x.y.z" tabIndex="0" role="button" />"#, None, None),
        (r#"<button className="foo" role="button" />"#, None, None),
        (r#"<input role="button" />"#, None, None),
        (r#"<input type="button" role="button" />"#, None, None),
        (r#"<input type="checkbox" role="button" />"#, None, None),
        (r#"<input type="color" role="button" />"#, None, None),
        (r#"<input type="date" role="button" />"#, None, None),
        (r#"<input type="datetime" role="button" />"#, None, None),
        (r#"<input type="datetime-local" role="button" />"#, None, None),
        (r#"<input type="email" role="button" />"#, None, None),
        (r#"<input type="file" role="button" />"#, None, None),
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
        (r#"<input type="hidden" role="button" />"#, None, None),
        (r#"<menuitem role="button" />;"#, None, None),
        (r#"<option className="foo" role="button" />"#, None, None),
        (r#"<select className="foo" role="button" />"#, None, None),
        (r#"<textarea className="foo" role="button" />"#, None, None),
        (r#"<tr role="button" />;"#, None, None),
        (r#"<a role="button" />"#, None, None),
        (r#"<a role="img" />;"#, None, None),
        (r#"<a tabIndex="0" role="button" />"#, None, None),
        (r#"<a tabIndex="0" role="img" />"#, None, None),
        (r#"<acronym role="button" />;"#, None, None),
        (r#"<address role="button" />;"#, None, None),
        (r#"<applet role="button" />;"#, None, None),
        (r#"<aside role="button" />;"#, None, None),
        (r#"<audio role="button" />;"#, None, None),
        (r#"<b role="button" />;"#, None, None),
        (r#"<base role="button" />;"#, None, None),
        (r#"<bdi role="button" />;"#, None, None),
        (r#"<bdo role="button" />;"#, None, None),
        (r#"<big role="button" />;"#, None, None),
        (r#"<blink role="button" />;"#, None, None),
        (r#"<blockquote role="button" />;"#, None, None),
        (r#"<body role="button" />;"#, None, None),
        (r#"<br role="button" />;"#, None, None),
        (r#"<canvas role="button" />;"#, None, None),
        (r#"<caption role="button" />;"#, None, None),
        (r#"<center role="button" />;"#, None, None),
        (r#"<cite role="button" />;"#, None, None),
        (r#"<code role="button" />;"#, None, None),
        (r#"<col role="button" />;"#, None, None),
        (r#"<colgroup role="button" />;"#, None, None),
        (r#"<content role="button" />;"#, None, None),
        (r#"<data role="button" />;"#, None, None),
        (r#"<datalist role="button" />;"#, None, None),
        (r#"<del role="button" />;"#, None, None),
        (r#"<details role="button" />;"#, None, None),
        (r#"<dir role="button" />;"#, None, None),
        (r#"<div role="button" />;"#, None, None),
        (r#"<div className="foo" role="button" />;"#, None, None),
        (r#"<div className="foo" {...props} role="button" />;"#, None, None),
        (r#"<div aria-hidden role="button" />;"#, None, None),
        (r#"<div aria-hidden={true} role="button" />;"#, None, None),
        (r#"<div role={undefined} role="button" />;"#, None, None),
        (r#"<div {...props} role="button" />;"#, None, None),
        (r#"<div onKeyUp={() => void 0} aria-hidden={false} role="button" />;"#, None, None),
        (r#"<dl role="button" />;"#, None, None),
        (r#"<em role="button" />;"#, None, None),
        (r#"<embed role="button" />;"#, None, None),
        (r#"<figcaption role="button" />;"#, None, None),
        (r#"<font role="button" />;"#, None, None),
        (r#"<footer role="button" />;"#, None, None),
        (r#"<frameset role="button" />;"#, None, None),
        (r#"<head role="button" />;"#, None, None),
        (r#"<header role="button" />;"#, None, None),
        (r#"<hgroup role="button" />;"#, None, None),
        (r#"<html role="button" />;"#, None, None),
        (r#"<i role="button" />;"#, None, None),
        (r#"<iframe role="button" />;"#, None, None),
        (r#"<ins role="button" />;"#, None, None),
        (r#"<kbd role="button" />;"#, None, None),
        (r#"<keygen role="button" />;"#, None, None),
        (r#"<label role="button" />;"#, None, None),
        (r#"<legend role="button" />;"#, None, None),
        (r#"<link role="button" />;"#, None, None),
        (r#"<map role="button" />;"#, None, None),
        (r#"<mark role="button" />;"#, None, None),
        (r#"<marquee role="button" />;"#, None, None),
        (r#"<menu role="button" />;"#, None, None),
        (r#"<meta role="button" />;"#, None, None),
        (r#"<meter role="button" />;"#, None, None),
        (r#"<noembed role="button" />;"#, None, None),
        (r#"<noscript role="button" />;"#, None, None),
        (r#"<object role="button" />;"#, None, None),
        (r#"<optgroup role="button" />;"#, None, None),
        (r#"<output role="button" />;"#, None, None),
        (r#"<p role="button" />;"#, None, None),
        (r#"<param role="button" />;"#, None, None),
        (r#"<picture role="button" />;"#, None, None),
        (r#"<pre role="button" />;"#, None, None),
        (r#"<progress role="button" />;"#, None, None),
        (r#"<q role="button" />;"#, None, None),
        (r#"<rp role="button" />;"#, None, None),
        (r#"<rt role="button" />;"#, None, None),
        (r#"<rtc role="button" />;"#, None, None),
        (r#"<ruby role="button" />;"#, None, None),
        (r#"<s role="button" />;"#, None, None),
        (r#"<samp role="button" />;"#, None, None),
        (r#"<script role="button" />;"#, None, None),
        (r#"<section role="button" />;"#, None, None),
        (r#"<small role="button" />;"#, None, None),
        (r#"<source role="button" />;"#, None, None),
        (r#"<spacer role="button" />;"#, None, None),
        (r#"<span role="button" />;"#, None, None),
        (r#"<strike role="button" />;"#, None, None),
        (r#"<strong role="button" />;"#, None, None),
        (r#"<style role="button" />;"#, None, None),
        (r#"<sub role="button" />;"#, None, None),
        (r#"<summary role="button" />;"#, None, None),
        (r#"<sup role="button" />;"#, None, None),
        (r#"<th role="button" />;"#, None, None),
        (r#"<time role="button" />;"#, None, None),
        (r#"<title role="button" />;"#, None, None),
        (r#"<track role="button" />;"#, None, None),
        (r#"<tt role="button" />;"#, None, None),
        (r#"<u role="button" />;"#, None, None),
        (r#"<var role="button" />;"#, None, None),
        (r#"<video role="button" />;"#, None, None),
        (r#"<wbr role="button" />;"#, None, None),
        (r#"<xmp role="button" />;"#, None, None),
        (r#"<div role="button" />;"#, None, None),
        (r#"<div role="checkbox" />;"#, None, None),
        (r#"<div role="columnheader" />;"#, None, None),
        (r#"<div role="combobox" />;"#, None, None),
        (r#"<div role="grid" />;"#, None, None),
        (r#"<div role="gridcell" />;"#, None, None),
        (r#"<div role="link" />;"#, None, None),
        (r#"<div role="listbox" />;"#, None, None),
        (r#"<div role="menu" />;"#, None, None),
        (r#"<div role="menubar" />;"#, None, None),
        (r#"<div role="menuitem" />;"#, None, None),
        (r#"<div role="menuitemcheckbox" />;"#, None, None),
        (r#"<div role="menuitemradio" />;"#, None, None),
        (r#"<div role="option" />;"#, None, None),
        (r#"<div role="progressbar" />;"#, None, None),
        (r#"<div role="radio" />;"#, None, None),
        (r#"<div role="radiogroup" />;"#, None, None),
        (r#"<div role="row" />;"#, None, None),
        (r#"<div role="rowheader" />;"#, None, None),
        (r#"<div role="searchbox" />;"#, None, None),
        (r#"<div role="slider" />;"#, None, None),
        (r#"<div role="spinbutton" />;"#, None, None),
        (r#"<div role="switch" />;"#, None, None),
        (r#"<div role="tab" />;"#, None, None),
        (r#"<div role="textbox" />;"#, None, None),
        (r#"<div role="treeitem" />;"#, None, None),
        (r#"<div role="presentation" />;"#, None, None),
        (r#"<div role="command" />;"#, None, None),
        (r#"<div role="composite" />;"#, None, None),
        (r#"<div role="input" />;"#, None, None),
        (r#"<div role="landmark" />;"#, None, None),
        (r#"<div role="range" />;"#, None, None),
        (r#"<div role="roletype" />;"#, None, None),
        (r#"<div role="section" />;"#, None, None),
        (r#"<div role="sectionhead" />;"#, None, None),
        (r#"<div role="select" />;"#, None, None),
        (r#"<div role="structure" />;"#, None, None),
        (r#"<div role="tablist" />;"#, None, None),
        (r#"<div role="toolbar" />;"#, None, None),
        (r#"<div role="tree" />;"#, None, None),
        (r#"<div role="treegrid" />;"#, None, None),
        (r#"<div role="widget" />;"#, None, None),
        (r#"<div role="window" />;"#, None, None),
        (r#"<main role="button" />;"#, None, None),
        (r#"<area role="button" />;"#, None, None),
        (r#"<article role="button" />;"#, None, None),
        (r#"<dd role="button" />;"#, None, None),
        (r#"<dfn role="button" />;"#, None, None),
        (r#"<dt role="button" />;"#, None, None),
        (r#"<fieldset role="button" />;"#, None, None),
        (r#"<figure role="button" />;"#, None, None),
        (r#"<form role="button" />;"#, None, None),
        (r#"<frame role="button" />;"#, None, None),
        (r#"<h1 role="button" />;"#, None, None),
        (r#"<h2 role="button" />;"#, None, None),
        (r#"<h3 role="button" />;"#, None, None),
        (r#"<h4 role="button" />;"#, None, None),
        (r#"<h5 role="button" />;"#, None, None),
        (r#"<h6 role="button" />;"#, None, None),
        (r#"<hr role="button" />;"#, None, None),
        (r#"<img role="button" />;"#, None, None),
        (r#"<input type="hidden" role="img" />"#, None, None),
        (r#"<canvas role="img" />;"#, None, None),
        (r#"<li role="button" />;"#, None, None),
        (r#"<li role="presentation" />;"#, None, None),
        (r#"<tr role="none" />;"#, None, None),
        (r#"<tr role="presentation" />;"#, None, None),
        (r#"<nav role="button" />;"#, None, None),
        (r#"<ol role="button" />;"#, None, None),
        (r#"<table role="button" />;"#, None, None),
        (r#"<tbody role="button" />;"#, None, None),
        (r#"<td role="button" />;"#, None, None),
        (r#"<tfoot role="button" />;"#, None, None),
        (r#"<thead role="button" />;"#, None, None),
        (r#"<ul role="button" />;"#, None, None),
        (r#"<div role="alert" />;"#, None, None),
        (r#"<div role="alertdialog" />;"#, None, None),
        (r#"<div role="application" />;"#, None, None),
        (r#"<div role="article" />;"#, None, None),
        (r#"<div role="banner" />;"#, None, None),
        (r#"<div role="cell" />;"#, None, None),
        (r#"<div role="complementary" />;"#, None, None),
        (r#"<div role="contentinfo" />;"#, None, None),
        (r#"<div role="definition" />;"#, None, None),
        (r#"<div role="dialog" />;"#, None, None),
        (r#"<div role="directory" />;"#, None, None),
        (r#"<div role="document" />;"#, None, None),
        (r#"<div role="feed" />;"#, None, None),
        (r#"<div role="figure" />;"#, None, None),
        (r#"<div role="form" />;"#, None, None),
        (r#"<div role="group" />;"#, None, None),
        (r#"<div role="heading" />;"#, None, None),
        (r#"<div role="img" />;"#, None, None),
        (r#"<div role="list" />;"#, None, None),
        (r#"<div role="listitem" />;"#, None, None),
        (r#"<div role="log" />;"#, None, None),
        (r#"<div role="main" />;"#, None, None),
        (r#"<div role="marquee" />;"#, None, None),
        (r#"<div role="math" />;"#, None, None),
        (r#"<div role="navigation" />;"#, None, None),
        (r#"<div role="note" />;"#, None, None),
        (r#"<div role="region" />;"#, None, None),
        (r#"<div role="rowgroup" />;"#, None, None),
        (r#"<div role="search" />;"#, None, None),
        (r#"<div role="separator" />;"#, None, None),
        (r#"<div role="scrollbar" />;"#, None, None),
        (r#"<div role="status" />;"#, None, None),
        (r#"<div role="table" />;"#, None, None),
        (r#"<div role="tabpanel" />;"#, None, None),
        (r#"<div role="term" />;"#, None, None),
        (r#"<div role="timer" />;"#, None, None),
        (r#"<div role="tooltip" />;"#, None, None),
        (r#"<div mynamespace:role="term" />"#, None, None),
        (r#"<input mynamespace:role="img" />"#, None, None),
        (r#"<Link href="http://x.y.z" role="img" />"#, None, None),
        (r#"<Link href="http://x.y.z" />"#, None, Some(settings())),
        (r"<Button onClick={doFoo} />", None, Some(settings())),
    ];

    let fail: Vec<(&str, Option<serde_json::Value>, Option<serde_json::Value>)> = vec![
        (r#"<a href="http://x.y.z" role="img" />"#, None, None),
        (r#"<a href="http://x.y.z" tabIndex="0" role="img" />"#, None, None),
        (r#"<input role="img" />"#, None, None),
        (r#"<input type="img" role="img" />"#, None, None),
        (r#"<input type="checkbox" role="img" />"#, None, None),
        (r#"<input type="color" role="img" />"#, None, None),
        (r#"<input type="date" role="img" />"#, None, None),
        (r#"<input type="datetime" role="img" />"#, None, None),
        (r#"<input type="datetime-local" role="img" />"#, None, None),
        (r#"<input type="email" role="img" />"#, None, None),
        (r#"<input type="file" role="img" />"#, None, None),
        (r#"<input type="image" role="img" />"#, None, None),
        (r#"<input type="month" role="img" />"#, None, None),
        (r#"<input type="number" role="img" />"#, None, None),
        (r#"<input type="password" role="img" />"#, None, None),
        (r#"<input type="radio" role="img" />"#, None, None),
        (r#"<input type="range" role="img" />"#, None, None),
        (r#"<input type="reset" role="img" />"#, None, None),
        (r#"<input type="search" role="img" />"#, None, None),
        (r#"<input type="submit" role="img" />"#, None, None),
        (r#"<input type="tel" role="img" />"#, None, None),
        (r#"<input type="text" role="img" />"#, None, None),
        (r#"<input type="time" role="img" />"#, None, None),
        (r#"<input type="url" role="img" />"#, None, None),
        (r#"<input type="week" role="img" />"#, None, None),
        (r#"<menuitem role="img" />;"#, None, None),
        (r#"<option className="foo" role="img" />"#, None, None),
        (r#"<select className="foo" role="img" />"#, None, None),
        (r#"<textarea className="foo" role="img" />"#, None, None),
        (r#"<tr role="img" />;"#, None, None),
        (r#"<a href="http://x.y.z" role="listitem" />"#, None, None),
        (r#"<a href="http://x.y.z" tabIndex="0" role="listitem" />"#, None, None),
        (r#"<input role="listitem" />"#, None, None),
        (r#"<input type="listitem" role="listitem" />"#, None, None),
        (r#"<input type="checkbox" role="listitem" />"#, None, None),
        (r#"<input type="color" role="listitem" />"#, None, None),
        (r#"<input type="date" role="listitem" />"#, None, None),
        (r#"<input type="datetime" role="listitem" />"#, None, None),
        (r#"<input type="datetime-local" role="listitem" />"#, None, None),
        (r#"<input type="email" role="listitem" />"#, None, None),
        (r#"<input type="file" role="listitem" />"#, None, None),
        (r#"<input type="image" role="listitem" />"#, None, None),
        (r#"<input type="month" role="listitem" />"#, None, None),
        (r#"<input type="number" role="listitem" />"#, None, None),
        (r#"<input type="password" role="listitem" />"#, None, None),
        (r#"<input type="radio" role="listitem" />"#, None, None),
        (r#"<input type="range" role="listitem" />"#, None, None),
        (r#"<input type="reset" role="listitem" />"#, None, None),
        (r#"<input type="search" role="listitem" />"#, None, None),
        (r#"<input type="submit" role="listitem" />"#, None, None),
        (r#"<input type="tel" role="listitem" />"#, None, None),
        (r#"<input type="text" role="listitem" />"#, None, None),
        (r#"<input type="time" role="listitem" />"#, None, None),
        (r#"<input type="url" role="listitem" />"#, None, None),
        (r#"<input type="week" role="listitem" />"#, None, None),
        (r#"<menuitem role="listitem" />;"#, None, None),
        (r#"<option className="foo" role="listitem" />"#, None, None),
        (r#"<select className="foo" role="listitem" />"#, None, None),
        (r#"<textarea className="foo" role="listitem" />"#, None, None),
        (r#"<tr role="listitem" />;"#, None, None),
        (r#"<Link href="http://x.y.z" role="img" />"#, None, Some(settings())),
    ];

    Tester::new(
        NoInteractiveElementToNoninteractiveRole::NAME,
        NoInteractiveElementToNoninteractiveRole::PLUGIN,
        pass,
        fail,
    )
    .test_and_snapshot();
}
