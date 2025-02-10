use std::ops::Deref;

use oxc_ast::{
    ast::{
        match_expression, Expression, JSXAttributeItem, JSXAttributeName, JSXAttributeValue,
        JSXExpression, StringLiteral,
    },
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, GetSpan, Span};

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    AstNode,
};

fn target_blank_without_noreferrer(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Using target=`_blank` without rel=`noreferrer` (which implies rel=`noopener`) is a security risk in older browsers: see https://mathiasbynens.github.io/rel-noopener/#recommendations")
.with_help("add rel=`noreferrer` to the element")
.with_label(span)
}

fn target_blank_without_noopener(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Using target=`_blank` without rel=`noreferrer` or rel=`noopener` (the former implies the latter and is preferred due to wider support) is a security risk: see https://mathiasbynens.github.io/rel-noopener/#recommendations")
.with_help("add rel=`noreferrer` or rel=`noopener` to the element")
.with_label(span)
}

fn explicit_props_in_spread_attributes(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("all spread attributes are treated as if they contain an unsafe combination of props, unless specifically overridden by props after the last spread attribute prop.")
.with_help("add rel=`noreferrer` to the element")
.with_label(span)
}

#[derive(Debug, Clone)]
pub struct JsxNoTargetBlank {
    enforce_dynamic_links: EnforceDynamicLinksEnum,
    warn_on_spread_attributes: bool,
    allow_referrer: bool,
    links: bool,
    forms: bool,
}

#[derive(Debug, Clone)]
enum EnforceDynamicLinksEnum {
    Always,
    Never,
}

impl Default for JsxNoTargetBlank {
    fn default() -> Self {
        Self {
            enforce_dynamic_links: EnforceDynamicLinksEnum::Always,
            warn_on_spread_attributes: false,
            allow_referrer: false,
            links: true,
            forms: false,
        }
    }
}

impl JsxNoTargetBlank {
    fn diagnostic(&self, span: Span, ctx: &LintContext) {
        if self.allow_referrer {
            ctx.diagnostic(target_blank_without_noopener(span));
        } else {
            ctx.diagnostic(target_blank_without_noreferrer(span));
        }
    }

    fn check_is_link(&self, tag_name: &str, ctx: &LintContext) -> bool {
        if !self.links {
            return false;
        }
        if tag_name == "a" {
            return true;
        }
        return ctx.settings().react.get_link_component_attrs(tag_name).is_some();
    }

    fn check_is_forms(&self, tag_name: &str, ctx: &LintContext) -> bool {
        if !self.forms {
            return false;
        }
        if tag_name == "form" {
            return true;
        }
        return ctx.settings().react.get_form_component_attrs(tag_name).is_some();
    }
}

declare_oxc_lint!(
    /// ### What it does
    /// This rule aims to prevent user generated link hrefs and form actions from creating security vulnerabilities by
    /// requiring `rel='noreferrer'` for external link hrefs and form actions, and optionally any dynamically generated
    /// link hrefs and form actions.
    ///
    /// ### Why is this bad?
    ///
    /// When creating a JSX element that has an `a` tag, it is often desired to have the link open in a new tab using the
    /// `target='_blank'` attribute. Using this attribute unaccompanied by `rel='noreferrer'`, however, is a severe security
    /// vulnerability (see [`noreferrer` docs] and [`noopener` docs] for more details).
    /// This rules requires that you accompany `target='_blank'` attributes with `rel='noreferrer'`.
    ///
    /// ### Example
    /// ```jsx
    /// /// correct
    /// var Hello = <p target="_blank"></p>
    /// var Hello = <a target="_blank" rel="noreferrer" href="https://example.com"></a>
    /// var Hello = <a target="_blank" rel="noopener noreferrer" href="https://example.com"></a>
    /// var Hello = <a target="_blank" href="relative/path/in/the/host"></a>
    /// var Hello = <a target="_blank" href="/absolute/path/in/the/host"></a>
    /// var Hello = <a></a>
    /// /// incorrect
    /// var Hello = <a target='_blank' href="https://example.com/"></a>
    /// var Hello = <a target='_blank' href={dynamicLink}></a>
    /// ```
    ///
    /// [`noreferrer` docs]: https://html.spec.whatwg.org/multipage/links.html#link-type-noreferrer
    /// [`noopener` docs]: https://html.spec.whatwg.org/multipage/links.html#link-type-noopener
    JsxNoTargetBlank,
    react,
    correctness
);

impl Rule for JsxNoTargetBlank {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::JSXOpeningElement(jsx_ele) = node.kind() {
            let Some(tag_name) = &jsx_ele.name.get_identifier_name() else {
                return;
            };
            if self.check_is_link(tag_name, ctx) || self.check_is_forms(tag_name, ctx) {
                let mut target_blank_tuple = (false, "", false, false);
                let mut rel_valid_tuple = (false, "", false, false);
                let mut is_href_valid = true;
                let mut has_href_value = false;
                let mut is_warn_on_spread_attributes = false;
                let mut target_span = None;
                let mut spread_span = Span::default();

                jsx_ele.attributes.iter().for_each(|attribute| match attribute {
                    JSXAttributeItem::Attribute(attribute) => {
                        if let JSXAttributeName::Identifier(identifier) = &attribute.deref().name {
                            let attribute_name = identifier.name.as_str();
                            if attribute_name == "target" {
                                if let Some(val) = attribute.deref().value.as_ref() {
                                    target_blank_tuple = check_target(val);
                                    target_span = attribute.value.as_ref().map(GetSpan::span);
                                }
                            } else if attribute_name == "href"
                                || attribute_name == "action"
                                || ctx
                                    .settings()
                                    .react
                                    .get_link_component_attrs(tag_name)
                                    .is_some_and(|link_attribute| {
                                        link_attribute.contains(&CompactStr::new(attribute_name))
                                    })
                                || ctx
                                    .settings()
                                    .react
                                    .get_form_component_attrs(tag_name)
                                    .is_some_and(|form_attribute| {
                                        form_attribute.contains(&CompactStr::new(attribute_name))
                                    })
                            {
                                if let Some(val) = attribute.value.as_ref() {
                                    has_href_value = true;
                                    is_href_valid = check_href(val, &self.enforce_dynamic_links);
                                }
                            } else if attribute_name == "rel" {
                                if let Some(val) = attribute.value.as_ref() {
                                    rel_valid_tuple = check_rel(val, self.allow_referrer);
                                }
                            };
                        }
                    }
                    JSXAttributeItem::SpreadAttribute(_) => {
                        if self.warn_on_spread_attributes {
                            is_warn_on_spread_attributes = true;
                            spread_span = attribute.span();
                            target_blank_tuple = (false, "", false, false);
                            rel_valid_tuple = (false, "", false, false);
                            is_href_valid = false;
                            has_href_value = true;
                        };
                    }
                });

                if is_warn_on_spread_attributes {
                    if (has_href_value && is_href_valid) || rel_valid_tuple.0 {
                        return;
                    }
                    ctx.diagnostic(explicit_props_in_spread_attributes(spread_span));
                    return;
                }

                let span = target_span.unwrap_or(jsx_ele.span);
                if !is_href_valid {
                    if !target_blank_tuple.1.is_empty() && target_blank_tuple.1 == rel_valid_tuple.1
                    {
                        if (target_blank_tuple.2 && !rel_valid_tuple.2)
                            || (target_blank_tuple.3 && !rel_valid_tuple.3)
                        {
                            self.diagnostic(span, ctx);
                        }
                        return;
                    }

                    if target_blank_tuple.0 && !rel_valid_tuple.0 {
                        self.diagnostic(span, ctx);
                    }
                }
            }
        }
    }

    fn from_configuration(value: serde_json::Value) -> Self {
        let value = value.as_array().and_then(|arr| arr.first()).and_then(|val| val.as_object());

        Self {
            enforce_dynamic_links: value
                .and_then(|val| val.get("enforceDynamicLinks").and_then(serde_json::Value::as_str))
                .map_or(EnforceDynamicLinksEnum::Always, |str| {
                    if str == "always" {
                        EnforceDynamicLinksEnum::Always
                    } else {
                        EnforceDynamicLinksEnum::Never
                    }
                }),
            warn_on_spread_attributes: value
                .and_then(|val| {
                    val.get("warnOnSpreadAttributes").and_then(serde_json::Value::as_bool)
                })
                .unwrap_or(false),
            links: value
                .and_then(|val| val.get("links").and_then(serde_json::Value::as_bool))
                .unwrap_or(true),
            forms: value
                .and_then(|val| val.get("forms").and_then(serde_json::Value::as_bool))
                .unwrap_or(false),
            allow_referrer: value
                .and_then(|val| val.get("allowReferrer").and_then(serde_json::Value::as_bool))
                .unwrap_or(false),
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_jsx()
    }
}

fn check_is_external_link(link: &str) -> bool {
    link.contains("//")
}

fn match_href_expression(
    expr: &Expression,
    is_external_link: &mut bool,
    is_dynamic_link: &mut bool,
) {
    match expr {
        Expression::StringLiteral(str) => *is_external_link = check_is_external_link(&str.value),
        Expression::Identifier(_) => *is_dynamic_link = true,
        Expression::ConditionalExpression(expr) => {
            match_href_expression(&expr.consequent, is_external_link, is_dynamic_link);
            match_href_expression(&expr.alternate, is_external_link, is_dynamic_link);
        }
        _ => {}
    }
}

fn check_href(
    attribute_value: &JSXAttributeValue,
    enforce_dynamic_links: &EnforceDynamicLinksEnum,
) -> bool {
    let mut is_dynamic_link = false;
    let mut is_external_link = false;
    let is_enforce_dynamic_links_never =
        matches!(enforce_dynamic_links, EnforceDynamicLinksEnum::Never);
    match attribute_value {
        JSXAttributeValue::StringLiteral(str) => {
            is_external_link = check_is_external_link(&str.value);
        }
        JSXAttributeValue::ExpressionContainer(expr) => {
            if let Some(expr) = expr.expression.as_expression() {
                match_href_expression(expr, &mut is_external_link, &mut is_dynamic_link);
            }
        }
        _ => {}
    };
    if is_enforce_dynamic_links_never {
        // correct:
        // 1. <a target="_blank" href="./link.js"></a>
        // 2. <a target="_blank" href={ dynamicLink }></a>
        // wrong:
        // 1. <a target="_blank" href="https://test.com"></a>
        return !is_external_link || is_dynamic_link;
    }
    // correct:
    // 1. <a target="_blank" href="./link.js"></a>
    // wrong:
    // 1. <a target="_blank" href="https://test.com"></a>
    // 2. <a target="_blank" href={ dynamicLink }></a>
    !(is_external_link || is_dynamic_link)
}

fn check_rel_val(str: &StringLiteral, allow_referrer: bool) -> bool {
    let mut splits = str.value.as_str().split(' ');
    if allow_referrer {
        return splits.any(|str| {
            if str == "noopener" {
                return true;
            }
            if str == "noreferrer" {
                return true;
            }
            false
        });
    }
    splits.any(|str| str.eq_ignore_ascii_case("noreferrer"))
}

fn match_rel_expression<'a>(
    expr: &'a Expression<'a>,
    allow_referrer: bool,
) -> (bool, &'a str, bool, bool) {
    let default = (false, "", false, false);
    match expr {
        Expression::StringLiteral(str) => (check_rel_val(str, allow_referrer), "", false, false),
        Expression::ConditionalExpression(expr) => {
            let consequent = match_rel_expression(&expr.consequent, allow_referrer);
            let alternate = match_rel_expression(&expr.alternate, allow_referrer);
            if let Expression::Identifier(identifier) = &expr.test {
                return (
                    consequent.0 && alternate.0,
                    identifier.name.as_str(),
                    consequent.0,
                    alternate.0,
                );
            }
            (consequent.0 && alternate.0, "", consequent.0, alternate.0)
        }
        _ => default,
    }
}

fn check_rel<'a>(
    attribute_value: &'a JSXAttributeValue<'a>,
    allow_referrer: bool,
) -> (bool, &'a str, bool, bool) {
    let default = (false, "", false, false);
    match attribute_value {
        JSXAttributeValue::StringLiteral(str) => {
            (check_rel_val(str, allow_referrer), "", false, false)
        }
        JSXAttributeValue::ExpressionContainer(expr) => match &expr.expression {
            JSXExpression::EmptyExpression(_) => default,
            expr @ match_expression!(JSXExpression) => {
                match_rel_expression(expr.to_expression(), allow_referrer)
            }
        },
        _ => default,
    }
}

fn match_target_expression<'a>(expr: &'a Expression<'a>) -> (bool, &'a str, bool, bool) {
    let default = (false, "", false, false);
    match expr {
        Expression::StringLiteral(str) => {
            (str.value.eq_ignore_ascii_case("_blank"), "", false, false)
        }
        Expression::ConditionalExpression(expr) => {
            let consequent = match_target_expression(&expr.consequent);
            let alternate = match_target_expression(&expr.alternate);
            if let Expression::Identifier(identifier) = &expr.test {
                return (
                    consequent.0 || alternate.0,
                    identifier.name.as_str(),
                    consequent.0,
                    alternate.0,
                );
            }
            (consequent.0 || alternate.0, "", consequent.0, alternate.0)
        }
        _ => default,
    }
}

fn check_target<'a>(attribute_value: &'a JSXAttributeValue<'a>) -> (bool, &'a str, bool, bool) {
    let default = (false, "", false, false);
    match attribute_value {
        JSXAttributeValue::StringLiteral(str) => {
            (str.value.eq_ignore_ascii_case("_blank"), "", false, false)
        }
        JSXAttributeValue::ExpressionContainer(expr) => {
            if let Some(expr) = expr.expression.as_expression() {
                match_target_expression(expr)
            } else {
                default
            }
        }
        _ => default,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (r#"<a href="foobar"></a>"#, None, None),
        (r"<a randomTag></a>", None, None),
        (r"<a target />", None, None),
        (r#"<a href="foobar" target="_blank" rel="noopener noreferrer"></a>"#, None, None),
        (r#"<a href="foobar" target="_blank" rel="noreferrer"></a>"#, None, None),
        (r#"<a href="foobar" target="_blank" rel={"noopener noreferrer"}></a>"#, None, None),
        (r#"<a href="foobar" target="_blank" rel={"noreferrer"}></a>"#, None, None),
        (r#"<a href={"foobar"} target={"_blank"} rel={"noopener noreferrer"}></a>"#, None, None),
        (r#"<a href={"foobar"} target={"_blank"} rel={"noreferrer"}></a>"#, None, None),
        (r"<a href={'foobar'} target={'_blank'} rel={'noopener noreferrer'}></a>", None, None),
        (r"<a href={'foobar'} target={'_blank'} rel={'noreferrer'}></a>", None, None),
        (r"<a href={`foobar`} target={`_blank`} rel={`noopener noreferrer`}></a>", None, None),
        (r"<a href={`foobar`} target={`_blank`} rel={`noreferrer`}></a>", None, None),
        (r#"<a target="_blank" {...spreadProps} rel="noopener noreferrer"></a>"#, None, None),
        (r#"<a target="_blank" {...spreadProps} rel="noreferrer"></a>"#, None, None),
        (
            r#"<a {...spreadProps} target="_blank" rel="noopener noreferrer" href="https://example.com">s</a>"#,
            None,
            None,
        ),
        (
            r#"<a {...spreadProps} target="_blank" rel="noreferrer" href="https://example.com">s</a>"#,
            None,
            None,
        ),
        (r#"<a target="_blank" rel="noopener noreferrer" {...spreadProps}></a>"#, None, None),
        (r#"<a target="_blank" rel="noreferrer" {...spreadProps}></a>"#, None, None),
        (r#"<p target="_blank"></p>"#, None, None),
        (r#"<a href="foobar" target="_BLANK" rel="NOOPENER noreferrer"></a>"#, None, None),
        (r#"<a href="foobar" target="_BLANK" rel="NOREFERRER"></a>"#, None, None),
        (r#"<a target="_blank" rel={relValue}></a>"#, None, None),
        (r#"<a target={targetValue} rel="noopener noreferrer"></a>"#, None, None),
        (r#"<a target={targetValue} rel="noreferrer"></a>"#, None, None),
        (r#"<a target={targetValue} rel={"noopener noreferrer"}></a>"#, None, None),
        (r#"<a target={targetValue} rel={"noreferrer"}></a>"#, None, None),
        (r#"<a target={targetValue} href="relative/path"></a>"#, None, None),
        (r#"<a target={targetValue} href="/absolute/path"></a>"#, None, None),
        (r#"<a target={'targetValue'} href="/absolute/path"></a>"#, None, None),
        (r#"<a target={"targetValue"} href="/absolute/path"></a>"#, None, None),
        (r#"<a target={null} href="//example.com"></a>"#, None, None),
        (
            r#"<a {...someObject} href="/absolute/path"></a>"#,
            Some(
                serde_json::json!([{ "enforceDynamicLinks": "always", "warnOnSpreadAttributes": true }]),
            ),
            None,
        ),
        (
            r#"<a {...someObject} rel="noreferrer"></a>"#,
            Some(
                serde_json::json!([{ "enforceDynamicLinks": "always", "warnOnSpreadAttributes": true }]),
            ),
            None,
        ),
        (
            r#"<a {...someObject} rel="noreferrer" target="_blank"></a>"#,
            Some(
                serde_json::json!([{ "enforceDynamicLinks": "always", "warnOnSpreadAttributes": true }]),
            ),
            None,
        ),
        (
            r#"<a {...someObject} href="foobar" target="_blank"></a>"#,
            Some(
                serde_json::json!([{ "enforceDynamicLinks": "always", "warnOnSpreadAttributes": true }]),
            ),
            None,
        ),
        (
            r#"<a target="_blank" href={ dynamicLink }></a>"#,
            Some(serde_json::json!([{ "enforceDynamicLinks": "never" }])),
            None,
        ),
        (
            r#"<a target={"_blank"} href={ dynamicLink }></a>"#,
            Some(serde_json::json!([{ "enforceDynamicLinks": "never" }])),
            None,
        ),
        (
            r"<a target={'_blank'} href={ dynamicLink }></a>",
            Some(serde_json::json!([{ "enforceDynamicLinks": "never" }])),
            None,
        ),
        (
            r#"<Link target="_blank" href={ dynamicLink }></Link>"#,
            Some(serde_json::json!([{ "enforceDynamicLinks": "never" }])),
            Some(serde_json::json!({ "settings": { "react": { "linkComponents": ["Link"] } } })),
        ),
        (
            r#"<Link target="_blank" to={ dynamicLink }></Link>"#,
            Some(serde_json::json!([{ "enforceDynamicLinks": "never" }])),
            Some(
                serde_json::json!({"settings": { "react": { "linkComponents": [{ "name": "Link", "linkAttribute": "to" }] } }}),
            ),
        ),
        (
            r#"<Link target="_blank" to={ dynamicLink }></Link>"#,
            Some(serde_json::json!([{ "enforceDynamicLinks": "never" }])),
            Some(
                serde_json::json!({ "settings": { "react": { "linkComponents": [{ "name": "Link", "linkAttribute": ["to"] }] } }}),
            ),
        ),
        (
            r#"<a href="foobar" target="_blank" rel="noopener"></a>"#,
            Some(serde_json::json!([{ "allowReferrer": true }])),
            None,
        ),
        (
            r#"<a href="foobar" target="_blank" rel="noreferrer"></a>"#,
            Some(serde_json::json!([{ "allowReferrer": true }])),
            None,
        ),
        (r"<a target={3} />", None, None),
        (r#"<a href="some-link" {...otherProps} target="some-non-blank-target"></a>"#, None, None),
        (r#"<a href="some-link" target="some-non-blank-target" {...otherProps}></a>"#, None, None),
        (
            r#"<a target="_blank" href="/absolute/path"></a>"#,
            Some(serde_json::json!([{ "forms": false }])),
            None,
        ),
        (
            r#"<a target="_blank" href="/absolute/path"></a>"#,
            Some(serde_json::json!([{ "forms": false, "links": true }])),
            None,
        ),
        (
            r#"<form action="https://example.com" target="_blank"></form>"#,
            Some(serde_json::json!([])),
            None,
        ),
        (
            r#"<form action="https://example.com" target="_blank" rel="noopener noreferrer"></form>"#,
            Some(serde_json::json!([{ "forms": true }])),
            None,
        ),
        (
            r#"<form action="https://example.com" target="_blank" rel="noopener noreferrer"></form>"#,
            Some(serde_json::json!([{ "forms": true, "links": false }])),
            None,
        ),
        (r#"<a href target="_blank"/>"#, None, None),
        (
            r#"<a href={href} target={isExternal ? "_blank" : undefined} rel="noopener noreferrer" />"#,
            None,
            None,
        ),
        (
            r#"<a href={href} target={isExternal ? undefined : "_blank"} rel={isExternal ? "noreferrer" : "noopener noreferrer"} />"#,
            None,
            None,
        ),
        (
            r#"<a href={href} target={isExternal ? undefined : "_blank"} rel={isExternal ? "noreferrer noopener" : "noreferrer"} />"#,
            None,
            None,
        ),
        (
            r#"<a href={href} target="_blank" rel={isExternal ? "noreferrer" : "noopener"} />"#,
            Some(serde_json::json!([{ "allowReferrer": true }])),
            None,
        ),
        (
            r#"<a href={href} target={isExternal ? "_blank" : undefined} rel={isExternal ? "noreferrer" : undefined} />"#,
            None,
            None,
        ),
        (
            r#"<a href={href} target={isSelf ? "_self" : "_blank"} rel={isSelf ? undefined : "noreferrer"} />"#,
            None,
            None,
        ),
        (
            r#"<a href={href} target={isSelf ? "_self" : ""} rel={isSelf ? undefined : ""} />"#,
            None,
            None,
        ),
        (
            r#"<a href={href} target={isExternal ? "_blank" : undefined} rel={isExternal ? "noopener noreferrer" : undefined} />"#,
            None,
            None,
        ),
        (r"<form action={action} />", Some(serde_json::json!([{ "forms": true }])), None),
        (
            r"<form action={action} {...spread} />",
            Some(serde_json::json!([{ "forms": true }])),
            None,
        ),
    ];

    let fail = vec![
        (r#"<a target="_blank" href="https://example.com/1"></a>"#, None, None),
        (r#"<a target="_blank" rel="" href="https://example.com/2"></a>"#, None, None),
        (r#"<a target="_blank" rel={0} href="https://example.com/3"></a>"#, None, None),
        (r#"<a target="_blank" rel={1} href="https://example.com/3"></a>"#, None, None),
        (r#"<a target="_blank" rel={false} href="https://example.com/4"></a>"#, None, None),
        (r#"<a target="_blank" rel={null} href="https://example.com/5"></a>"#, None, None),
        (
            r#"<a target="_blank" rel="noopenernoreferrer" href="https://example.com/6"></a>"#,
            None,
            None,
        ),
        (r#"<a target="_blank" rel="no referrer" href="https://example.com/7"></a>"#, None, None),
        (r#"<a target="_BLANK" href="https://example.com/8"></a>"#, None, None),
        (r#"<a target="_blank" href="//example.com/9"></a>"#, None, None),
        (r#"<a target="_blank" href="//example.com/10" rel={true}></a>"#, None, None),
        (r#"<a target="_blank" href="//example.com/11" rel={3}></a>"#, None, None),
        (r#"<a target="_blank" href="//example.com/12" rel={null}></a>"#, None, None),
        (r#"<a target="_blank" href="//example.com/13" rel={getRel()}></a>"#, None, None),
        (
            r#"<a target="_blank" href="//example.com/14" rel={"noopenernoreferrer"}></a>"#,
            None,
            None,
        ),
        (
            r#"<a target={"_blank"} href={"//example.com/15"} rel={"noopenernoreferrer"}></a>"#,
            None,
            None,
        ),
        (
            r#"<a target={"_blank"} href={"//example.com/16"} rel={"noopenernoreferrernoreferrernoreferrernoreferrernoreferrer"}></a>"#,
            None,
            None,
        ),
        (r#"<a target="_blank" href="//example.com/17" rel></a>"#, None, None),
        (r#"<a target="_blank" href={ dynamicLink }></a>"#, None, None),
        (r#"<a target={'_blank'} href="//example.com/18"></a>"#, None, None),
        (r#"<a target={"_blank"} href="//example.com/19"></a>"#, None, None),
        (
            r#"<a href="https://example.com/20" target="_blank" rel></a>"#,
            Some(serde_json::json!([{ "allowReferrer": true }])),
            None,
        ),
        (
            r#"<a href="https://example.com/20" target="_blank"></a>"#,
            Some(serde_json::json!([{ "allowReferrer": true }])),
            None,
        ),
        (
            r#"<a target="_blank" href={ dynamicLink }></a>"#,
            Some(serde_json::json!([{ "enforceDynamicLinks": "always" }])),
            None,
        ),
        (
            r"<a {...someObject}></a>",
            Some(
                serde_json::json!([{ "enforceDynamicLinks": "always", "warnOnSpreadAttributes": true }]),
            ),
            None,
        ),
        (
            r#"<a {...someObject} target="_blank"></a>"#,
            Some(
                serde_json::json!([{ "enforceDynamicLinks": "always", "warnOnSpreadAttributes": true }]),
            ),
            None,
        ),
        (
            r#"<a href="foobar" {...someObject} target="_blank"></a>"#,
            Some(
                serde_json::json!([{ "enforceDynamicLinks": "always", "warnOnSpreadAttributes": true }]),
            ),
            None,
        ),
        (
            r#"<a href="foobar" target="_blank" rel="noreferrer" {...someObject}></a>"#,
            Some(
                serde_json::json!([{ "enforceDynamicLinks": "always", "warnOnSpreadAttributes": true }]),
            ),
            None,
        ),
        (
            r#"<a href="foobar" target="_blank" {...someObject}></a>"#,
            Some(
                serde_json::json!([{ "enforceDynamicLinks": "always", "warnOnSpreadAttributes": true }]),
            ),
            None,
        ),
        (
            r#"<Link target="_blank" href={ dynamicLink }></Link>"#,
            Some(serde_json::json!([{ "enforceDynamicLinks": "always"}])),
            Some(serde_json::json!({ "settings": { "react": { "linkComponents": ["Link"] } } })),
        ),
        (
            r#"<Link target="_blank" to={ dynamicLink }></Link>"#,
            Some(serde_json::json!([{ "enforceDynamicLinks": "always" }])),
            Some(
                serde_json::json!({ "settings": { "react": { "linkComponents": [{ "name": "Link", "linkAttribute": "to" }] } } }),
            ),
        ),
        (
            r#"<a href="some-link" {...otherProps} target="some-non-blank-target"></a>"#,
            Some(serde_json::json!([{ "warnOnSpreadAttributes": true }])),
            None,
        ),
        (
            r#"<a href="some-link" target="some-non-blank-target" {...otherProps}></a>"#,
            Some(serde_json::json!([{ "warnOnSpreadAttributes": true }])),
            None,
        ),
        (
            r#"<a target="_blank" href="//example.com" rel></a>"#,
            Some(serde_json::json!([{ "links": true }])),
            None,
        ),
        (
            r#"<a target="_blank" href="//example.com" rel></a>"#,
            Some(serde_json::json!([{ "links": true, "forms": true }])),
            None,
        ),
        (
            r#"<a target="_blank" href="//example.com" rel></a>"#,
            Some(serde_json::json!([{ "links": true, "forms": false }])),
            None,
        ),
        (
            r#"<form method="POST" action="https://example.com" target="_blank"></form>"#,
            Some(serde_json::json!([{ "forms": true }])),
            None,
        ),
        (
            r#"<form method="POST" action="https://example.com" rel="" target="_blank"></form>"#,
            Some(serde_json::json!([{ "forms": true }])),
            None,
        ),
        (
            r#"<form method="POST" action="https://example.com" rel="noopenernoreferrer" target="_blank"></form>"#,
            Some(serde_json::json!([{ "forms": true }])),
            None,
        ),
        (
            r#"<form method="POST" action="https://example.com" rel="noopenernoreferrer" target="_blank"></form>"#,
            Some(serde_json::json!([{ "forms": true, "links": false }])),
            None,
        ),
        (
            r#"<a href={href} target="_blank" rel={isExternal ? "undefined" : "undefined"} />"#,
            None,
            None,
        ),
        (
            r#"<a href={href} target="_blank" rel={isExternal ? "noopener" : undefined} />"#,
            None,
            None,
        ),
        (
            r#"<a href={href} target="_blank" rel={isExternal ? "undefined" : "noopener"} />"#,
            None,
            None,
        ),
        (
            r#"<a href={href} target={isExternal ? "_blank" : undefined} rel={isExternal ? undefined : "noopener noreferrer"} />"#,
            None,
            None,
        ),
        (
            r#"<a href={href} target="_blank" rel={isExternal ? 3 : "noopener noreferrer"} />"#,
            None,
            None,
        ),
        (
            r#"<a href={href} target="_blank" rel={isExternal ? "noopener noreferrer" : "3"} />"#,
            None,
            None,
        ),
        (
            r#"<a href={href} target="_blank" rel={isExternal ? "noopener" : "2"} />"#,
            Some(serde_json::json!([{ "allowReferrer": true }])),
            None,
        ),
        (
            r#"<form action={action} target="_blank" />"#,
            Some(serde_json::json!([{ "allowReferrer": true, "forms": true }])),
            None,
        ),
        (
            r#"<form action={action} target="_blank" />"#,
            Some(serde_json::json!([{ "forms": true }])),
            None,
        ),
        (
            r"<form action={action} {...spread} />",
            Some(serde_json::json!([{ "forms": true, "warnOnSpreadAttributes": true }])),
            None,
        ),
    ];

    Tester::new(JsxNoTargetBlank::NAME, JsxNoTargetBlank::PLUGIN, pass, fail).test_and_snapshot();
}
