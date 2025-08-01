use cow_utils::CowUtils;
use oxc_ast::{AstKind, ast::JSXAttributeItem};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode, LintContext,
    globals::{RESERVED_HTML_TAG, is_valid_aria_property},
    rule::Rule,
    utils::{get_element_type, get_jsx_attribute_name},
};

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces that reserved DOM elements do not contain ARIA roles, states,
    /// or properties.
    ///
    /// ### Why is this bad?
    ///
    /// Certain reserved DOM elements do not support ARIA roles, states and
    /// properties. This is often because they are not visible, for example
    /// `meta`, `html`, `script`, `style`. Adding ARIA attributes to these
    /// elements is meaningless and can create confusion for screen readers.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <meta charset="UTF-8" aria-hidden="false" />
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <meta charset="UTF-8" />
    /// ```
    AriaUnsupportedElements,
    jsx_a11y,
    correctness,
    fix
);

#[derive(Debug, Default, Clone)]
pub struct AriaUnsupportedElements;

fn aria_unsupported_elements_diagnostic(span: Span, attr_name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn("This element does not support ARIA roles, states and properties.")
        .with_help(format!("Try removing the prop `{attr_name}`."))
        .with_label(span)
}

impl Rule for AriaUnsupportedElements {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::JSXOpeningElement(jsx_el) = node.kind() {
            let el_type = get_element_type(ctx, jsx_el);
            if RESERVED_HTML_TAG.contains(&el_type.as_ref()) {
                for attr in &jsx_el.attributes {
                    let attr = match attr {
                        JSXAttributeItem::Attribute(attr) => attr,
                        JSXAttributeItem::SpreadAttribute(_) => continue,
                    };
                    let attr_name = get_jsx_attribute_name(&attr.name);
                    let attr_name = attr_name.cow_to_ascii_lowercase();
                    if attr_name == "role" || is_valid_aria_property(&attr_name) {
                        ctx.diagnostic_with_fix(
                            aria_unsupported_elements_diagnostic(attr.span, &attr_name),
                            |fixer| fixer.delete(&attr.span),
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (r"<a role />", None),
        (r"<abbr role />", None),
        (r"<acronym role />", None),
        (r"<address role />", None),
        (r"<applet role />", None),
        (r"<area role />", None),
        (r"<article role />", None),
        (r"<aside role />", None),
        (r"<audio role />", None),
        (r"<b role />", None),
        (r"<base  />", None),
        (r"<bdi role />", None),
        (r"<bdo role />", None),
        (r"<big role />", None),
        (r"<blink role />", None),
        (r"<blockquote role />", None),
        (r"<body role />", None),
        (r"<br role />", None),
        (r"<button role />", None),
        (r"<canvas role />", None),
        (r"<caption role />", None),
        (r"<center role />", None),
        (r"<cite role />", None),
        (r"<code role />", None),
        (r"<col  />", None),
        (r"<colgroup  />", None),
        (r"<content role />", None),
        (r"<data role />", None),
        (r"<datalist role />", None),
        (r"<dd role />", None),
        (r"<del role />", None),
        (r"<details role />", None),
        (r"<dfn role />", None),
        (r"<dialog role />", None),
        (r"<dir role />", None),
        (r"<div role />", None),
        (r"<dl role />", None),
        (r"<dt role />", None),
        (r"<em role />", None),
        (r"<embed role />", None),
        (r"<fieldset role />", None),
        (r"<figcaption role />", None),
        (r"<figure role />", None),
        (r"<font role />", None),
        (r"<footer role />", None),
        (r"<form role />", None),
        (r"<frame role />", None),
        (r"<frameset role />", None),
        (r"<h1 role />", None),
        (r"<h2 role />", None),
        (r"<h3 role />", None),
        (r"<h4 role />", None),
        (r"<h5 role />", None),
        (r"<h6 role />", None),
        (r"<head  />", None),
        (r"<header role />", None),
        (r"<hgroup role />", None),
        (r"<hr role />", None),
        (r"<html  />", None),
        (r"<i role />", None),
        (r"<iframe role />", None),
        (r"<img role />", None),
        (r"<input role />", None),
        (r"<ins role />", None),
        (r"<kbd role />", None),
        (r"<keygen role />", None),
        (r"<label role />", None),
        (r"<legend role />", None),
        (r"<li role />", None),
        (r"<link  />", None),
        (r"<main role />", None),
        (r"<map role />", None),
        (r"<mark role />", None),
        (r"<marquee role />", None),
        (r"<menu role />", None),
        (r"<menuitem role />", None),
        (r"<meta  />", None),
        (r"<meter role />", None),
        (r"<nav role />", None),
        (r"<noembed  />", None),
        (r"<noscript  />", None),
        (r"<object role />", None),
        (r"<ol role />", None),
        (r"<optgroup role />", None),
        (r"<option role />", None),
        (r"<output role />", None),
        (r"<p role />", None),
        (r"<param  />", None),
        (r"<picture  />", None),
        (r"<pre role />", None),
        (r"<progress role />", None),
        (r"<q role />", None),
        (r"<rp role />", None),
        (r"<rt role />", None),
        (r"<rtc role />", None),
        (r"<ruby role />", None),
        (r"<s role />", None),
        (r"<samp role />", None),
        (r"<script  />", None),
        (r"<section role />", None),
        (r"<select role />", None),
        (r"<small role />", None),
        (r"<source  />", None),
        (r"<spacer role />", None),
        (r"<span role />", None),
        (r"<strike role />", None),
        (r"<strong role />", None),
        (r"<style  />", None),
        (r"<sub role />", None),
        (r"<summary role />", None),
        (r"<sup role />", None),
        (r"<table role />", None),
        (r"<tbody role />", None),
        (r"<td role />", None),
        (r"<textarea role />", None),
        (r"<tfoot role />", None),
        (r"<th role />", None),
        (r"<thead role />", None),
        (r"<time role />", None),
        (r"<title  />", None),
        (r"<tr role />", None),
        (r"<track  />", None),
        (r"<tt role />", None),
        (r"<u role />", None),
        (r"<ul role />", None),
        (r"<var role />", None),
        (r"<video role />", None),
        (r"<wbr role />", None),
        (r"<xmp role />", None),
        (r"<a aria-hidden />", None),
        (r"<abbr aria-hidden />", None),
        (r"<acronym aria-hidden />", None),
        (r"<address aria-hidden />", None),
        (r"<applet aria-hidden />", None),
        (r"<area aria-hidden />", None),
        (r"<article aria-hidden />", None),
        (r"<aside aria-hidden />", None),
        (r"<audio aria-hidden />", None),
        (r"<b aria-hidden />", None),
        (r"<base  />", None),
        (r"<bdi aria-hidden />", None),
        (r"<bdo aria-hidden />", None),
        (r"<big aria-hidden />", None),
        (r"<blink aria-hidden />", None),
        (r"<blockquote aria-hidden />", None),
        (r"<body aria-hidden />", None),
        (r"<br aria-hidden />", None),
        (r"<button aria-hidden />", None),
        (r"<canvas aria-hidden />", None),
        (r"<caption aria-hidden />", None),
        (r"<center aria-hidden />", None),
        (r"<cite aria-hidden />", None),
        (r"<code aria-hidden />", None),
        (r"<col  />", None),
        (r"<colgroup  />", None),
        (r"<content aria-hidden />", None),
        (r"<data aria-hidden />", None),
        (r"<datalist aria-hidden />", None),
        (r"<dd aria-hidden />", None),
        (r"<del aria-hidden />", None),
        (r"<details aria-hidden />", None),
        (r"<dfn aria-hidden />", None),
        (r"<dialog aria-hidden />", None),
        (r"<dir aria-hidden />", None),
        (r"<div aria-hidden />", None),
        (r"<dl aria-hidden />", None),
        (r"<dt aria-hidden />", None),
        (r"<em aria-hidden />", None),
        (r"<embed aria-hidden />", None),
        (r"<fieldset aria-hidden />", None),
        (r"<figcaption aria-hidden />", None),
        (r"<figure aria-hidden />", None),
        (r"<font aria-hidden />", None),
        (r"<footer aria-hidden />", None),
        (r"<form aria-hidden />", None),
        (r"<frame aria-hidden />", None),
        (r"<frameset aria-hidden />", None),
        (r"<h1 aria-hidden />", None),
        (r"<h2 aria-hidden />", None),
        (r"<h3 aria-hidden />", None),
        (r"<h4 aria-hidden />", None),
        (r"<h5 aria-hidden />", None),
        (r"<h6 aria-hidden />", None),
        (r"<head  />", None),
        (r"<header aria-hidden />", None),
        (r"<hgroup aria-hidden />", None),
        (r"<hr aria-hidden />", None),
        (r"<html  />", None),
        (r"<i aria-hidden />", None),
        (r"<iframe aria-hidden />", None),
        (r"<img aria-hidden />", None),
        (r"<input aria-hidden />", None),
        (r"<ins aria-hidden />", None),
        (r"<kbd aria-hidden />", None),
        (r"<keygen aria-hidden />", None),
        (r"<label aria-hidden />", None),
        (r"<legend aria-hidden />", None),
        (r"<li aria-hidden />", None),
        (r"<link  />", None),
        (r"<main aria-hidden />", None),
        (r"<map aria-hidden />", None),
        (r"<mark aria-hidden />", None),
        (r"<marquee aria-hidden />", None),
        (r"<menu aria-hidden />", None),
        (r"<menuitem aria-hidden />", None),
        (r"<meta  />", None),
        (r"<meter aria-hidden />", None),
        (r"<nav aria-hidden />", None),
        (r"<noembed  />", None),
        (r"<noscript  />", None),
        (r"<object aria-hidden />", None),
        (r"<ol aria-hidden />", None),
        (r"<optgroup aria-hidden />", None),
        (r"<option aria-hidden />", None),
        (r"<output aria-hidden />", None),
        (r"<p aria-hidden />", None),
        (r"<param  />", None),
        (r"<picture  />", None),
        (r"<pre aria-hidden />", None),
        (r"<progress aria-hidden />", None),
        (r"<q aria-hidden />", None),
        (r"<rp aria-hidden />", None),
        (r"<rt aria-hidden />", None),
        (r"<rtc aria-hidden />", None),
        (r"<ruby aria-hidden />", None),
        (r"<s aria-hidden />", None),
        (r"<samp aria-hidden />", None),
        (r"<script  />", None),
        (r"<section aria-hidden />", None),
        (r"<select aria-hidden />", None),
        (r"<small aria-hidden />", None),
        (r"<source  />", None),
        (r"<spacer aria-hidden />", None),
        (r"<span aria-hidden />", None),
        (r"<strike aria-hidden />", None),
        (r"<strong aria-hidden />", None),
        (r"<style  />", None),
        (r"<sub aria-hidden />", None),
        (r"<summary aria-hidden />", None),
        (r"<sup aria-hidden />", None),
        (r"<table aria-hidden />", None),
        (r"<tbody aria-hidden />", None),
        (r"<td aria-hidden />", None),
        (r"<textarea aria-hidden />", None),
        (r"<tfoot aria-hidden />", None),
        (r"<th aria-hidden />", None),
        (r"<thead aria-hidden />", None),
        (r"<time aria-hidden />", None),
        (r"<title  />", None),
        (r"<tr aria-hidden />", None),
        (r"<track  />", None),
        (r"<tt aria-hidden />", None),
        (r"<u aria-hidden />", None),
        (r"<ul aria-hidden />", None),
        (r"<var aria-hidden />", None),
        (r"<video aria-hidden />", None),
        (r"<wbr aria-hidden />", None),
        (r"<xmp aria-hidden />", None),
    ];

    let fail = vec![
        (r"<base role {...props} />", None),
        (r"<col role {...props} />", None),
        (r"<colgroup role {...props} />", None),
        (r"<head role {...props} />", None),
        (r"<html role {...props} />", None),
        (r"<link role {...props} />", None),
        (r"<meta role {...props} />", None),
        (r"<noembed role {...props} />", None),
        (r"<noscript role {...props} />", None),
        (r"<param role {...props} />", None),
        (r"<picture role {...props} />", None),
        (r"<script role {...props} />", None),
        (r"<source role {...props} />", None),
        (r"<style role {...props} />", None),
        (r"<title role {...props} />", None),
        (r"<track role {...props} />", None),
        (r#"<base aria-hidden aria-role="none" {...props} />"#, None),
        (r#"<col aria-hidden aria-role="none" {...props} />"#, None),
        (r#"<colgroup aria-hidden aria-role="none" {...props} />"#, None),
        (r#"<head aria-hidden aria-role="none" {...props} />"#, None),
        (r#"<html aria-hidden aria-role="none" {...props} />"#, None),
        (r#"<link aria-hidden aria-role="none" {...props} />"#, None),
        (r#"<meta aria-hidden aria-role="none" {...props} />"#, None),
        (r#"<noembed aria-hidden aria-role="none" {...props} />"#, None),
        (r#"<noscript aria-hidden aria-role="none" {...props} />"#, None),
        (r#"<param aria-hidden aria-role="none" {...props} />"#, None),
        (r#"<picture aria-hidden aria-role="none" {...props} />"#, None),
        (r#"<script aria-hidden aria-role="none" {...props} />"#, None),
        (r#"<source aria-hidden aria-role="none" {...props} />"#, None),
        (r#"<style aria-hidden aria-role="none" {...props} />"#, None),
        (r#"<title aria-hidden aria-role="none" {...props} />"#, None),
        (r#"<track aria-hidden aria-role="none" {...props} />"#, None),
    ];

    let fix = vec![
        (r"<col role {...props} />", r"<col  {...props} />"),
        (
            r#"<meta aria-hidden aria-role="none" {...props} />"#,
            r#"<meta  aria-role="none" {...props} />"#,
        ),
    ];

    Tester::new(AriaUnsupportedElements::NAME, AriaUnsupportedElements::PLUGIN, pass, fail)
        .with_jsx_a11y_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
