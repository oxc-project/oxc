use std::ops::Deref;

use oxc_ast::{
    ast::{JSXAttributeItem, JSXAttributeValue, JSXExpression},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, GetSpan, Span};
use serde_json::Value;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{get_element_type, has_jsx_prop_ignore_case},
    AstNode,
};

fn missing_href_attribute(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Missing `href` attribute for the `a` element.")
        .with_help("Provide an `href` for the `a` element.")
        .with_label(span)
}

fn incorrect_href(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Use of incorrect `href` for the 'a' element.")
        .with_help("Provide a correct `href` for the `a` element.")
        .with_label(span)
}

fn cant_be_anchor(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("The `a` element has `href` and `onClick`.")
        .with_help("Use a `button` element instead of an `a` element.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct AnchorIsValid(Box<AnchorIsValidConfig>);

#[derive(Debug, Default, Clone)]
pub struct AnchorIsValidConfig {
    /// Unique and sorted list of valid hrefs
    valid_hrefs: Vec<CompactStr>,
}

impl Deref for AnchorIsValid {
    type Target = AnchorIsValidConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    /// The HTML `<a>` element, with a valid href attribute, is formally defined as representing a **hyperlink**.
    /// That is, a link between one HTML document and another, or between one location inside an HTML document and another location inside the same document.
    ///
    /// While before it was possible to attach logic to an anchor element, with the advent of JSX libraries,
    /// it's now  easier to attach logic to any HTML element, anchors included.
    ///
    /// This rule is designed to prevent users to attach logic at the click of anchors, and also makes
    /// sure that the `href` provided to the anchor element is valid. If the anchor has logic attached to it,
    /// the rules suggests to turn it to a `button`, because that's likely what the user wants.
    ///
    /// Anchor `<a></a>` elements should be used for navigation, while `<button></button>` should be
    /// used for user interaction.
    ///
    /// Consider the following:
    ///
    /// ```jsx
    /// <>
    ///     <a href="javascript:void(0)" onClick={foo}>Perform action</a>
    ///     <a href="#" onClick={foo}>Perform action</a>
    ///     <a onClick={foo}>Perform action</a>
    /// </>
    /// ````
    ///
    /// All these anchor implementations indicate that the element is only used to execute JavaScript code. All the above should be replaced with:
    ///
    /// ```jsx
    /// <button onClick={foo}>Perform action</button>
    /// ```
    /// `
    /// ### Why is this bad?
    /// There are **many reasons** why an anchor should not have a logic and have a correct `href` attribute:
    /// - it can disrupt the correct flow of the user navigation e.g. a user that wants to open the link
    /// in another tab, but the default "click" behaviour is prevented
    /// - it can source of invalid links, and crawlers can't navigate the website, risking to penalise SEO ranking
    ///
    /// ### Example
    ///
    /// #### Valid
    ///
    /// ```jsx
    /// <>
    ///     <a href={`https://www.javascript.com`}>navigate here</a>
    ///     <a href={somewhere}>navigate here</a>
    ///     <a {...spread}>navigate here</a>
    /// </>
    /// ```
    ///
    /// #### Invalid
    ///
    /// ```jsx
    /// <>
    ///     <a href={null}>navigate here</a>
    ///     <a href={undefined}>navigate here</a>
    ///     <a href>navigate here</a>
    ///     <a href="javascript:void(0)">navigate here</a>
    ///     <a href="https://example.com" onClick={something}>navigate here</a>
    /// </>
    /// ```
    ///
    /// ### Reference
    ///
    /// - [WCAG 2.1.1](https://www.w3.org/WAI/WCAG21/Understanding/keyboard)
    AnchorIsValid,
    jsx_a11y,
    correctness
);

impl Rule for AnchorIsValid {
    fn from_configuration(value: serde_json::Value) -> Self {
        let Some(valid_hrefs) = value.get("validHrefs").and_then(Value::as_array) else {
            return Self::default();
        };
        Self(Box::new(valid_hrefs.iter().collect()))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::JSXElement(jsx_el) = node.kind() {
            let name = get_element_type(ctx, &jsx_el.opening_element);

            if name != "a" {
                return;
            };
            // Don't eagerly get `span` here, to avoid that work unless rule fails
            let get_span = || jsx_el.opening_element.name.span();
            if let Some(href_attr) = has_jsx_prop_ignore_case(&jsx_el.opening_element, "href") {
                let JSXAttributeItem::Attribute(attr) = href_attr else {
                    return;
                };

                // Check if the 'a' element has a correct href attribute
                let Some(value) = attr.value.as_ref() else {
                    ctx.diagnostic(incorrect_href(get_span()));
                    return;
                };

                let is_empty = self.check_value_is_empty_or_invalid(value);
                if is_empty {
                    if has_jsx_prop_ignore_case(&jsx_el.opening_element, "onclick").is_some() {
                        ctx.diagnostic(cant_be_anchor(get_span()));
                        return;
                    }
                    ctx.diagnostic(incorrect_href(get_span()));
                    return;
                }
                return;
            }
            // Exclude '<a {...props} />' case
            let has_spread_attr = jsx_el.opening_element.attributes.iter().any(|attr| match attr {
                JSXAttributeItem::SpreadAttribute(_) => true,
                JSXAttributeItem::Attribute(_) => false,
            });
            if has_spread_attr {
                return;
            }
            ctx.diagnostic(missing_href_attribute(get_span()));
        }
    }
}

impl AnchorIsValid {
    fn check_value_is_empty_or_invalid(&self, value: &JSXAttributeValue) -> bool {
        match value {
            JSXAttributeValue::Element(_) => false,
            JSXAttributeValue::StringLiteral(str_lit) => self.is_invalid_href(&str_lit.value),
            JSXAttributeValue::ExpressionContainer(exp) => match &exp.expression {
                JSXExpression::Identifier(ident) => ident.name == "undefined",
                JSXExpression::NullLiteral(_) => true,
                JSXExpression::StringLiteral(str_lit) => self.is_invalid_href(&str_lit.value),
                JSXExpression::TemplateLiteral(temp_lit) => {
                    if !temp_lit.expressions.is_empty() {
                        return false;
                    }

                    let Some(quasi) = temp_lit.quasi() else {
                        return false;
                    };
                    self.is_invalid_href(&quasi)
                }
                _ => false,
            },
            JSXAttributeValue::Fragment(_) => true,
        }
    }
}

impl AnchorIsValidConfig {
    fn new(mut valid_hrefs: Vec<CompactStr>) -> Self {
        valid_hrefs.sort_unstable();
        valid_hrefs.dedup();
        Self { valid_hrefs }
    }

    fn is_invalid_href(&self, href: &str) -> bool {
        if self.contains(href) {
            return false;
        }

        href.is_empty() || href == "#" || href == "javascript:void(0)"
    }

    fn contains(&self, href: &str) -> bool {
        self.valid_hrefs.binary_search_by(|valid_href| valid_href.as_str().cmp(href)).is_ok()
    }
}

impl<'v> FromIterator<&'v Value> for AnchorIsValidConfig {
    fn from_iter<T: IntoIterator<Item = &'v Value>>(iter: T) -> Self {
        let hrefs = iter.into_iter().filter_map(Value::as_str).map(CompactStr::from).collect();
        Self::new(hrefs)
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    // let components = vec![1];
    // let specialLink = vec![1];
    // let componentsAndSpecialLink = vec![1];
    // let invalidHrefAspect = vec![1];
    // let preferButtonAspect = vec![1];
    // let preferButtonInvalidHrefAspect = vec![1];
    // let noHrefAspect = vec![1];
    // let noHrefPreferButtonAspect = vec![1];
    // let componentsAndSpecialLinkAndInvalidHrefAspect = vec![1];
    // let noHrefInvalidHrefAspect = vec![1];
    // let componentsAndSpecialLinkAndNoHrefAspect = vec![1];

    // https://raw.githubusercontent.com/jsx-eslint/eslint-plugin-jsx-a11y/main/__tests__/src/rules/anchor-is-valid-test.js
    let pass = vec![
        (r"<Anchor />", None, None),
        (r"<a {...props} />", None, None),
        (r"<a href='foo' />", Some(serde_json::json!({ "validHrefs": ["foo"] })), None),
        (r"<a href={foo} />", None, None),
        (r"<a href='/foo' />", Some(serde_json::json!({ "validHrefs": ["/foo"] })), None),
        (
            r"<a href='https://foo.bar.com' />",
            Some(serde_json::json!({ "validHrefs": ["https://foo.bar.com"] })),
            None,
        ),
        (r"<div href='foo' />", None, None),
        (
            r"<a href='javascript' />",
            Some(serde_json::json!({ "validHrefs": ["javascript"] })),
            None,
        ),
        (
            r"<a href='javascriptFoo' />",
            Some(serde_json::json!({ "validHrefs": ["javascriptFoo"] })),
            None,
        ),
        (r"<a href={`#foo`}/>", None, None),
        (r"<a href={'foo'}/>", Some(serde_json::json!({ "validHrefs": ["foo"] })), None),
        (
            r"<a href={'javascript'}/>",
            Some(serde_json::json!({ "validHrefs": ["javascript"] })),
            None,
        ),
        (r"<a href={`#javascript`}/>", None, None),
        (r"<a href='#foo' />", Some(serde_json::json!({ "validHrefs": ["#foo"] })), None),
        (
            r"<a href='#javascript' />",
            Some(serde_json::json!({ "validHrefs": ["#javascript"] })),
            None,
        ),
        (
            r"<a href='#javascriptFoo' />",
            Some(serde_json::json!({ "validHrefs": ["#javascriptFoo"] })),
            None,
        ),
        (r"<UX.Layout>test</UX.Layout>", None, None),
        (r"<a href={this} />", None, None),
        // (r#"<Anchor {...props} />"#, Some(serde_json::json!(components))),
        // (r#"<Anchor href='foo' />"#, Some(serde_json::json!(components))),
        // (r#"<Anchor href={foo} />"#, Some(serde_json::json!(components))),
        // (r#"<Anchor href='/foo' />"#, Some(serde_json::json!(components))),
        // (r#"<Anchor href='https://foo.bar.com' />"#, Some(serde_json::json!(components))),
        // (r#"<div href='foo' />"#, Some(serde_json::json!(components))),
        // (r#"<Anchor href={`#foo`}/>"#, Some(serde_json::json!(components))),
        // (r#"<Anchor href={'foo'}/>"#, Some(serde_json::json!(components))),
        // (r#"<Anchor href='#foo' />"#, Some(serde_json::json!(components))),
        // (r#"<Link {...props} />"#, Some(serde_json::json!(components))),
        // (r#"<Link href='foo' />"#, Some(serde_json::json!(components))),
        // (r#"<Link href={foo} />"#, Some(serde_json::json!(components))),
        // (r#"<Link href='/foo' />"#, Some(serde_json::json!(components))),
        // (r#"<Link href='https://foo.bar.com' />"#, Some(serde_json::json!(components))),
        // (r#"<div href='foo' />"#, Some(serde_json::json!(components))),
        // (r#"<Link href={`#foo`}/>"#, Some(serde_json::json!(components))),
        // (r#"<Link href={'foo'}/>"#, Some(serde_json::json!(components))),
        // (r#"<Link href='#foo' />"#, Some(serde_json::json!(components))),
        (
            r"<Link href='#foo' />",
            Some(serde_json::json!({ "validHrefs": ["#foo"] })),
            Some(
                serde_json::json!({ "settings": { "jsx-a11y": { "components": { "Anchor": "a", "Link": "a" } } } }),
            ),
        ),
        // (r#"<a {...props} />"#, Some(serde_json::json!(specialLink))),
        // (r#"<a hrefLeft='foo' />"#, Some(serde_json::json!(specialLink))),
        // (r#"<a hrefLeft={foo} />"#, Some(serde_json::json!(specialLink))),
        // (r#"<a hrefLeft='/foo' />"#, Some(serde_json::json!(specialLink))),
        // (r#"<a hrefLeft='https://foo.bar.com' />"#, Some(serde_json::json!(specialLink))),
        // (r#"<div hrefLeft='foo' />"#, Some(serde_json::json!(specialLink))),
        // (r#"<a hrefLeft={`#foo`}/>"#, Some(serde_json::json!(specialLink))),
        // (r#"<a hrefLeft={'foo'}/>"#, Some(serde_json::json!(specialLink))),
        // (r#"<a hrefLeft='#foo' />"#, Some(serde_json::json!(specialLink))),
        // (r#"<UX.Layout>test</UX.Layout>"#, Some(serde_json::json!(specialLink))),
        // (r#"<a hrefRight={this} />"#, Some(serde_json::json!(specialLink))),
        // (r#"<a {...props} />"#, Some(serde_json::json!(specialLink))),
        // (r#"<a hrefRight='foo' />"#, Some(serde_json::json!(specialLink))),
        // (r#"<a hrefRight={foo} />"#, Some(serde_json::json!(specialLink))),
        // (r#"<a hrefRight='/foo' />"#, Some(serde_json::json!(specialLink))),
        // (r#"<a hrefRight='https://foo.bar.com' />"#, Some(serde_json::json!(specialLink))),
        // (r#"<div hrefRight='foo' />"#, Some(serde_json::json!(specialLink))),
        // (r#"<a hrefRight={`#foo`}/>"#, Some(serde_json::json!(specialLink))),
        // (r#"<a hrefRight={'foo'}/>"#, Some(serde_json::json!(specialLink))),
        // (r#"<a hrefRight='#foo' />"#, Some(serde_json::json!(specialLink))),
        // (r#"<UX.Layout>test</UX.Layout>"#, Some(serde_json::json!(specialLink))),
        // (r#"<a hrefRight={this} />"#, Some(serde_json::json!(specialLink))),
        // (r#"<Anchor {...props} />"#, Some(serde_json::json!(componentsAndSpecialLink))),
        // (r#"<Anchor hrefLeft='foo' />"#, Some(serde_json::json!(componentsAndSpecialLink))),
        // (r#"<Anchor hrefLeft={foo} />"#, Some(serde_json::json!(componentsAndSpecialLink))),
        // (r#"<Anchor hrefLeft='/foo' />"#, Some(serde_json::json!(componentsAndSpecialLink))),
        // (
        // r#"<Anchor hrefLeft='https://foo.bar.com' />"#,
        // Some(serde_json::json!(componentsAndSpecialLink)),
        // ),
        // (r#"<div hrefLeft='foo' />"#, Some(serde_json::json!(componentsAndSpecialLink))),
        // (r#"<Anchor hrefLeft={`#foo`}/>"#, Some(serde_json::json!(componentsAndSpecialLink))),
        // (r#"<Anchor hrefLeft={'foo'}/>"#, Some(serde_json::json!(componentsAndSpecialLink))),
        // (r#"<Anchor hrefLeft='#foo' />"#, Some(serde_json::json!(componentsAndSpecialLink))),
        // (r#"<UX.Layout>test</UX.Layout>"#, Some(serde_json::json!(componentsAndSpecialLink))),
        (r"<a {...props} onClick={() => void 0} />", None, None),
        (
            r"<a href='foo' onClick={() => void 0} />",
            Some(serde_json::json!({ "validHrefs": ["foo"] })),
            None,
        ),
        (r"<a href={foo} onClick={() => void 0} />", None, None),
        (
            r"<a href='/foo' onClick={() => void 0} />",
            Some(serde_json::json!({ "validHrefs": ["/foo"] })),
            None,
        ),
        (
            r"<a href='https://foo.bar.com' onClick={() => void 0} />",
            Some(serde_json::json!({ "validHrefs": ["https://foo.bar.com"] })),
            None,
        ),
        (r"<div href='foo' onClick={() => void 0} />", None, None),
        (r"<a href={`#foo`} onClick={() => void 0} />", None, None),
        (
            r"<a href={'foo'} onClick={() => void 0} />",
            Some(serde_json::json!({ "validHrefs": ["foo"] })),
            None,
        ),
        (
            r"<a href='#foo' onClick={() => void 0} />",
            Some(serde_json::json!({ "validHrefs": ["#foo"] })),
            None,
        ),
        (r"<a href={this} onClick={() => void 0} />", None, None),
        (r"<a href='/some/valid/uri'>valid</a>", None, None),
        (r"<a href={'/some/valid/uri'}>valid</a>", None, None),
        (r"<a href={`/some/valid/uri`}>valid</a>", None, None),
        (r"<a href='#top'>Navigate to internal page location</a>", None, None),
        (r"<a href={'#top'}>Navigate to internal page location</a>", None, None),
        (r"<a href={`#top`}>Navigate to internal page location</a>", None, None),
        (r"<a href='https://github.com'>github</a>", None, None),
        (r"<a href={'https://github.com'}>github</a>", None, None),
        (r"<a href={`https://github.com`}>github</a>", None, None),
        (r"<a href='#section'>section</a>", None, None),
        (r"<a href={'#section'}>section</a>", None, None),
        (r"<a href={`#section`}>section</a>", None, None),
        (r"<a href={`${foo}`}>valid</a>", None, None),
        (r"<a href={`#${foo}`}>valid</a>", None, None),
        (r"<a href={`#${foo}/bar`}>valid</a>", None, None),
        (r"<a href={foo + bar}>valid</a>", None, None),
        // (r#"<Anchor {...props} onClick={() => void 0} />"#, Some(serde_json::json!(components))),
        // (r#"<Anchor href='foo' onClick={() => void 0} />"#, Some(serde_json::json!(components))),
        // (r#"<Anchor href={foo} onClick={() => void 0} />"#, Some(serde_json::json!(components))),
        // (r#"<Anchor href='/foo' onClick={() => void 0} />"#, Some(serde_json::json!(components))),
        // (
        // r#"<Anchor href='https://foo.bar.com' onClick={() => void 0} />"#,
        // Some(serde_json::json!(components)),
        // ),
        // (r#"<Anchor href={`#foo`} onClick={() => void 0} />"#, Some(serde_json::json!(components))),
        // (
        // r#"<Anchor href={'foo'} onClick={() => void 0} />"#,
        // Some(serde_json::json!(components)),
        // ),
        // (r#"<Anchor href='#foo' onClick={() => void 0} />"#, Some(serde_json::json!(components))),
        // (r#"<Link {...props} onClick={() => void 0} />"#, Some(serde_json::json!(components))),
        // (r#"<Link href='foo' onClick={() => void 0} />"#, Some(serde_json::json!(components))),
        // (r#"<Link href={foo} onClick={() => void 0} />"#, Some(serde_json::json!(components))),
        // (r#"<Link href='/foo' onClick={() => void 0} />"#, Some(serde_json::json!(components))),
        // (
        // r#"<Link href='https://foo.bar.com' onClick={() => void 0} />"#,
        // Some(serde_json::json!(components)),
        // ),
        // (r#"<div href='foo' onClick={() => void 0} />"#, Some(serde_json::json!(components))),
        // (r#"<Link href={`#foo`} onClick={() => void 0} />"#, Some(serde_json::json!(components))),
        // (r#"<Link href={'foo'} onClick={() => void 0} />"#, Some(serde_json::json!(components))),
        // (r#"<Link href='#foo' onClick={() => void 0} />"#, Some(serde_json::json!(components))),
        // (r#"<a {...props} onClick={() => void 0} />"#, Some(serde_json::json!(specialLink))),
        // (r#"<a hrefLeft='foo' onClick={() => void 0} />"#, Some(serde_json::json!(specialLink))),
        // (r#"<a hrefLeft={foo} onClick={() => void 0} />"#, Some(serde_json::json!(specialLink))),
        // (r#"<a hrefLeft='/foo' onClick={() => void 0} />"#, Some(serde_json::json!(specialLink))),
        // (
        // r#"<a hrefLeft href='https://foo.bar.com' onClick={() => void 0} />"#,
        // Some(serde_json::json!(specialLink)),
        // ),
        // (
        // r#"<div hrefLeft='foo' onClick={() => void 0} />"#,
        // Some(serde_json::json!(specialLink)),
        // ),
        // (r#"<a hrefLeft={`#foo`} onClick={() => void 0} />"#, Some(serde_json::json!(specialLink))),
        // (
        // r#"<a hrefLeft={'foo'} onClick={() => void 0} />"#,
        // Some(serde_json::json!(specialLink)),
        // ),
        // (r#"<a hrefLeft='#foo' onClick={() => void 0} />"#, Some(serde_json::json!(specialLink))),
        // (r#"<a hrefRight={this} onClick={() => void 0} />"#, Some(serde_json::json!(specialLink))),
        // (r#"<a {...props} onClick={() => void 0} />"#, Some(serde_json::json!(specialLink))),
        // (r#"<a hrefRight='foo' onClick={() => void 0} />"#, Some(serde_json::json!(specialLink))),
        // (r#"<a hrefRight={foo} onClick={() => void 0} />"#, Some(serde_json::json!(specialLink))),
        // (
        // r#"<a hrefRight='/foo' onClick={() => void 0} />"#,
        // Some(serde_json::json!(specialLink)),
        // ),
        // (
        // r#"<a hrefRight href='https://foo.bar.com' onClick={() => void 0} />"#,
        // Some(serde_json::json!(specialLink)),
        // ),
        // (
        // r#"<div hrefRight='foo' onClick={() => void 0} />"#,
        // Some(serde_json::json!(specialLink)),
        // ),
        // (
        // r#"<a hrefRight={`#foo`} onClick={() => void 0} />"#,
        // Some(serde_json::json!(specialLink)),
        // ),
        // (
        // r#"<a hrefRight={'foo'} onClick={() => void 0} />"#,
        // Some(serde_json::json!(specialLink)),
        // ),
        // (r#"<a hrefRight='#foo' onClick={() => void 0} />"#, Some(serde_json::json!(specialLink))),
        // (r#"<a hrefRight={this} onClick={() => void 0} />"#, Some(serde_json::json!(specialLink))),
        // (
        // r#"<Anchor {...props} onClick={() => void 0} />"#,
        // Some(serde_json::json!(componentsAndSpecialLink)),
        // ),
        // (
        // r#"<Anchor hrefLeft='foo' onClick={() => void 0} />"#,
        // Some(serde_json::json!(componentsAndSpecialLink)),
        // ),
        // (
        // r#"<Anchor hrefLeft={foo} onClick={() => void 0} />"#,
        // Some(serde_json::json!(componentsAndSpecialLink)),
        // ),
        // (
        // r#"<Anchor hrefLeft='/foo' onClick={() => void 0} />"#,
        // Some(serde_json::json!(componentsAndSpecialLink)),
        // ),
        // (
        // r#"<Anchor hrefLeft href='https://foo.bar.com' onClick={() => void 0} />"#,
        // Some(serde_json::json!(componentsAndSpecialLink)),
        // ),
        // (
        // r#"<Anchor hrefLeft={`#foo`} onClick={() => void 0} />"#,
        // Some(serde_json::json!(componentsAndSpecialLink)),
        // ),
        // (
        // r#"<Anchor hrefLeft={'foo'} onClick={() => void 0} />"#,
        // Some(serde_json::json!(componentsAndSpecialLink)),
        // ),
        // (
        // r#"<Anchor hrefLeft='#foo'` onClick={() => void 0} />"#,
        // Some(serde_json::json!(componentsAndSpecialLink)),
        // ),
        // (r#"<a />"#, Some(serde_json::json!(invalidHrefAspect))),
        // (r#"<a href={undefined} />"#, Some(serde_json::json!(invalidHrefAspect))),
        // (r#"<a href={null} />"#, Some(serde_json::json!(invalidHrefAspect))),
        // (r#"<a />"#, Some(serde_json::json!(preferButtonAspect))),
        // (r#"<a href={undefined} />"#, Some(serde_json::json!(preferButtonAspect))),
        // (r#"<a href={null} />"#, Some(serde_json::json!(preferButtonAspect))),
        // (r#"<a />"#, Some(serde_json::json!(preferButtonInvalidHrefAspect))),
        // (r#"<a href={undefined} />"#, Some(serde_json::json!(preferButtonInvalidHrefAspect))),
        // (r#"<a href={null} />"#, Some(serde_json::json!(preferButtonInvalidHrefAspect))),
        // (r#"<a href=' />;"#, Some(serde_json::json!(preferButtonAspect))),
        // (r#"<a href='#' />"#, Some(serde_json::json!(preferButtonAspect))),
        // (r#"<a href={'#'} />"#, Some(serde_json::json!(preferButtonAspect))),
        // (r#"<a href='javascript:void(0)' />"#, Some(serde_json::json!(preferButtonAspect))),
        // (r#"<a href={'javascript:void(0)'} />"#, Some(serde_json::json!(preferButtonAspect))),
        // (r#"<a href=' />;"#, Some(serde_json::json!(noHrefAspect))),
        // (r#"<a href='#' />"#, Some(serde_json::json!(noHrefAspect))),
        // (r#"<a href={'#'} />"#, Some(serde_json::json!(noHrefAspect))),
        // (r#"<a href='javascript:void(0)' />"#, Some(serde_json::json!(noHrefAspect))),
        // (r#"<a href={'javascript:void(0)'} />"#, Some(serde_json::json!(noHrefAspect))),
        // (r#"<a href=' />;"#, Some(serde_json::json!(noHrefPreferButtonAspect))),
        // (r#"<a href='#' />"#, Some(serde_json::json!(noHrefPreferButtonAspect))),
        // (r#"<a href={'#'} />"#, Some(serde_json::json!(noHrefPreferButtonAspect))),
        // (r#"<a href='javascript:void(0)' />"#, Some(serde_json::json!(noHrefPreferButtonAspect))),
        // (
        // r#"<a href={'javascript:void(0)'} />"#,
        // Some(serde_json::json!(noHrefPreferButtonAspect)),
        // ),
        // (r#"<a onClick={() => void 0} />"#, Some(serde_json::json!(invalidHrefAspect))),
        // (r#"<a href='#' onClick={() => void 0} />"#, Some(serde_json::json!(noHrefAspect))),
        // (
        // r#"<a href='javascript:void(0)' onClick={() => void 0} />"#,
        // Some(serde_json::json!(noHrefAspect)),
        // ),
        // (
        // r#"<a href={'javascript:void(0)'} onClick={() => void 0} />"#,
        // Some(serde_json::json!(noHrefAspect)),
        // ),
        // (
        // r#"<Anchor hrefLeft={undefined} />"#,
        // Some(serde_json::json!(componentsAndSpecialLinkAndInvalidHrefAspect)),
        // ),
        // (
        // r#"<Anchor hrefLeft={null} />"#,
        // Some(serde_json::json!(componentsAndSpecialLinkAndInvalidHrefAspect)),
        // ),
        // (
        // r#"<Anchor hrefLeft={undefined} />"#,
        // Some(serde_json::json!(componentsAndSpecialLinkAndInvalidHrefAspect)),
        // ),
        // (
        // r#"<Anchor hrefLeft={null} />"#,
        // Some(serde_json::json!(componentsAndSpecialLinkAndInvalidHrefAspect)),
        // ),
        // (
        // r#"<Anchor hrefLeft={undefined} />"#,
        // Some(serde_json::json!(componentsAndSpecialLinkAndInvalidHrefAspect)),
        // ),
        // (
        // r#"<Anchor hrefLeft={null} />"#,
        // Some(serde_json::json!(componentsAndSpecialLinkAndInvalidHrefAspect)),
        // ),
    ];

    let fail = vec![
        (r"<a />", None, None),
        (r"<a href={undefined} />", None, None),
        (r"<a href={null} />", None, None),
        (r"<a href=' />;", None, None),
        (r"<a href='#' />", None, None),
        (r"<a href={'#'} />", None, None),
        (r"<a href={`#`} />", None, None),
        (r"<a href='javascript:void(0)' />", None, None),
        (r"<a href={'javascript:void(0)'} />", None, None),
        (r"<a onClick={() => void 0} />", None, None),
        (r"<a href='#' onClick={() => void 0} />", None, None),
        (r"<a href='javascript:void(0)' onClick={() => void 0} />", None, None),
        (r"<a href={'javascript:void(0)'} onClick={() => void 0} />", None, None),
        // (r#"<Link />"#, Some(serde_json::json!(components))),
        // (r#"<Link href={undefined} />"#, Some(serde_json::json!(components))),
        // (r#"<Link href={null} />"#, Some(serde_json::json!(components))),
        // (r#"<Link href=' />"#, Some(serde_json::json!(components))),
        // (r#"<Link href='#' />"#, Some(serde_json::json!(components))),
        // (r#"<Link href={'#'} />"#, Some(serde_json::json!(components))),
        // (r#"<Link href='javascript:void(0)' />"#, Some(serde_json::json!(components))),
        // (r#"<Link href={'javascript:void(0)'} />"#, Some(serde_json::json!(components))),
        // (r#"<Anchor href=' />"#, Some(serde_json::json!(components))),
        // (r#"<Anchor href='#' />"#, Some(serde_json::json!(components))),
        // (r#"<Anchor href={'#'} />"#, Some(serde_json::json!(components))),
        // (r#"<Anchor href='javascript:void(0)' />"#, Some(serde_json::json!(components))),
        // (r#"<Anchor href={'javascript:void(0)'} />"#, Some(serde_json::json!(components))),
        // (r#"<Link onClick={() => void 0} />"#, Some(serde_json::json!(components))),
        // (r#"<Link href='#' onClick={() => void 0} />"#, Some(serde_json::json!(components))),
        // (
        // r#"<Link href='javascript:void(0)' onClick={() => void 0} />"#,
        // Some(serde_json::json!(components)),
        // ),
        // (
        // r#"<Link href={'javascript:void(0)'} onClick={() => void 0} />"#,
        // Some(serde_json::json!(components)),
        // ),
        // (r#"<Anchor onClick={() => void 0} />"#, Some(serde_json::json!(components))),
        // (r#"<Anchor href='#' onClick={() => void 0} />"#, Some(serde_json::json!(components))),
        // (
        // r#"<Anchor href='javascript:void(0)' onClick={() => void 0} />"#,
        // Some(serde_json::json!(components)),
        // ),
        // (
        // r#"<Anchor href={'javascript:void(0)'} onClick={() => void 0} />"#,
        // Some(serde_json::json!(components)),
        // ),
        (
            r"<Link href='#' onClick={() => void 0} />",
            None,
            Some(
                serde_json::json!({ "settings": { "jsx-a11y": { "components": { "Anchor": "a", "Link": "a" } } } }),
            ),
        ),
        // (r#"<a hrefLeft={undefined} />"#, Some(serde_json::json!(specialLink))),
        // (r#"<a hrefLeft={null} />"#, Some(serde_json::json!(specialLink))),
        // (r#"<a hrefLeft=' />;"#, Some(serde_json::json!(specialLink))),
        // (r#"<a hrefLeft='#' />"#, Some(serde_json::json!(specialLink))),
        // (r#"<a hrefLeft={'#'} />"#, Some(serde_json::json!(specialLink))),
        // (r#"<a hrefLeft='javascript:void(0)' />"#, Some(serde_json::json!(specialLink))),
        // (r#"<a hrefLeft={'javascript:void(0)'} />"#, Some(serde_json::json!(specialLink))),
        // (r#"<a hrefLeft='#' onClick={() => void 0} />"#, Some(serde_json::json!(specialLink))),
        // (
        // r#"<a hrefLeft='javascript:void(0)' onClick={() => void 0} />"#,
        // Some(serde_json::json!(specialLink)),
        // ),
        // (
        // r#"<a hrefLeft={'javascript:void(0)'} onClick={() => void 0} />"#,
        // Some(serde_json::json!(specialLink)),
        // ),
        // (r#"<Anchor Anchor={undefined} />"#, Some(serde_json::json!(componentsAndSpecialLink))),
        // (r#"<Anchor hrefLeft={null} />"#, Some(serde_json::json!(componentsAndSpecialLink))),
        // (r#"<Anchor hrefLeft=' />;"#, Some(serde_json::json!(componentsAndSpecialLink))),
        // (r#"<Anchor hrefLeft='#' />"#, Some(serde_json::json!(componentsAndSpecialLink))),
        // (r#"<Anchor hrefLeft={'#'} />"#, Some(serde_json::json!(componentsAndSpecialLink))),
        // (
        // r#"<Anchor hrefLeft='javascript:void(0)' />"#,
        // Some(serde_json::json!(componentsAndSpecialLink)),
        // ),
        // (
        // r#"<Anchor hrefLeft={'javascript:void(0)'} />"#,
        // Some(serde_json::json!(componentsAndSpecialLink)),
        // ),
        // (
        // r#"<Anchor hrefLeft='#' onClick={() => void 0} />"#,
        // Some(serde_json::json!(componentsAndSpecialLink)),
        // ),
        // (
        // r#"<Anchor hrefLeft='javascript:void(0)' onClick={() => void 0} />"#,
        // Some(serde_json::json!(componentsAndSpecialLink)),
        // ),
        // (
        // r#"<Anchor hrefLeft={'javascript:void(0)'} onClick={() => void 0} />"#,
        // Some(serde_json::json!(componentsAndSpecialLink)),
        // ),
        // (r#"<a />"#, Some(serde_json::json!(noHrefAspect))),
        // (r#"<a />"#, Some(serde_json::json!(noHrefPreferButtonAspect))),
        // (r#"<a />"#, Some(serde_json::json!(noHrefInvalidHrefAspect))),
        // (r#"<a href={undefined} />"#, Some(serde_json::json!(noHrefAspect))),
        // (r#"<a href={undefined} />"#, Some(serde_json::json!(noHrefPreferButtonAspect))),
        // (r#"<a href={undefined} />"#, Some(serde_json::json!(noHrefInvalidHrefAspect))),
        // (r#"<a href={null} />"#, Some(serde_json::json!(noHrefAspect))),
        // (r#"<a href={null} />"#, Some(serde_json::json!(noHrefPreferButtonAspect))),
        // (r#"<a href={null} />"#, Some(serde_json::json!(noHrefInvalidHrefAspect))),
        // (r#"<a href=' />;"#, Some(serde_json::json!(invalidHrefAspect))),
        // (r#"<a href=' />;"#, Some(serde_json::json!(noHrefInvalidHrefAspect))),
        // (r#"<a href=' />;"#, Some(serde_json::json!(preferButtonInvalidHrefAspect))),
        // (r#"<a href='#' />;"#, Some(serde_json::json!(invalidHrefAspect))),
        // (r#"<a href='#' />;"#, Some(serde_json::json!(noHrefInvalidHrefAspect))),
        // (r#"<a href='#' />;"#, Some(serde_json::json!(preferButtonInvalidHrefAspect))),
        // (r#"<a href={'#'} />;"#, Some(serde_json::json!(invalidHrefAspect))),
        // (r#"<a href={'#'} />;"#, Some(serde_json::json!(noHrefInvalidHrefAspect))),
        // (r#"<a href={'#'} />;"#, Some(serde_json::json!(preferButtonInvalidHrefAspect))),
        // (r#"<a href='javascript:void(0)' />;"#, Some(serde_json::json!(invalidHrefAspect))),
        // (r#"<a href='javascript:void(0)' />;"#, Some(serde_json::json!(noHrefInvalidHrefAspect))),
        // (
        // r#"<a href='javascript:void(0)' />;"#,
        // Some(serde_json::json!(preferButtonInvalidHrefAspect)),
        // ),
        // (r#"<a href={'javascript:void(0)'} />;"#, Some(serde_json::json!(invalidHrefAspect))),
        // (
        // r#"<a href={'javascript:void(0)'} />;"#,
        // Some(serde_json::json!(noHrefInvalidHrefAspect)),
        // ),
        // (
        // r#"<a href={'javascript:void(0)'} />;"#,
        // Some(serde_json::json!(preferButtonInvalidHrefAspect)),
        // ),
        // (r#"<a onClick={() => void 0} />"#, Some(serde_json::json!(preferButtonAspect))),
        // (r#"<a onClick={() => void 0} />"#, Some(serde_json::json!(preferButtonInvalidHrefAspect))),
        // (r#"<a onClick={() => void 0} />"#, Some(serde_json::json!(noHrefPreferButtonAspect))),
        // (r#"<a onClick={() => void 0} />"#, Some(serde_json::json!(noHrefAspect))),
        // (r#"<a onClick={() => void 0} />"#, Some(serde_json::json!(noHrefInvalidHrefAspect))),
        // (r#"<a href='#' onClick={() => void 0} />"#, Some(serde_json::json!(preferButtonAspect))),
        // (
        // r#"<a href='#' onClick={() => void 0} />"#,
        // Some(serde_json::json!(noHrefPreferButtonAspect)),
        // ),
        // (
        // r#"<a href='#' onClick={() => void 0} />"#,
        // Some(serde_json::json!(preferButtonInvalidHrefAspect)),
        // ),
        // (r#"<a href='#' onClick={() => void 0} />"#, Some(serde_json::json!(invalidHrefAspect))),
        // (
        // r#"<a href='#' onClick={() => void 0} />"#,
        // Some(serde_json::json!(noHrefInvalidHrefAspect)),
        // ),
        // (
        // r#"<a href='javascript:void(0)' onClick={() => void 0} />"#,
        // Some(serde_json::json!(preferButtonAspect)),
        // ),
        // (
        // r#"<a href='javascript:void(0)' onClick={() => void 0} />"#,
        // Some(serde_json::json!(noHrefPreferButtonAspect)),
        // ),
        // (
        // r#"<a href='javascript:void(0)' onClick={() => void 0} />"#,
        // Some(serde_json::json!(preferButtonInvalidHrefAspect)),
        // ),
        // (
        // r#"<a href='javascript:void(0)' onClick={() => void 0} />"#,
        // Some(serde_json::json!(invalidHrefAspect)),
        // ),
        // (
        // r#"<a href='javascript:void(0)' onClick={() => void 0} />"#,
        // Some(serde_json::json!(noHrefInvalidHrefAspect)),
        // ),
        // (
        // r#"<a href={'javascript:void(0)'} onClick={() => void 0} />"#,
        // Some(serde_json::json!(preferButtonAspect)),
        // ),
        // (
        // r#"<a href={'javascript:void(0)'} onClick={() => void 0} />"#,
        // Some(serde_json::json!(noHrefPreferButtonAspect)),
        // ),
        // (
        // r#"<a href={'javascript:void(0)'} onClick={() => void 0} />"#,
        // Some(serde_json::json!(preferButtonInvalidHrefAspect)),
        // ),
        // (
        // r#"<a href={'javascript:void(0)'} onClick={() => void 0} />"#,
        // Some(serde_json::json!(invalidHrefAspect)),
        // ),
        // (
        // r#"<a href={'javascript:void(0)'} onClick={() => void 0} />"#,
        // Some(serde_json::json!(noHrefInvalidHrefAspect)),
        // ),
        // (
        // r#"<Anchor hrefLeft={undefined} />"#,
        // Some(serde_json::json!(componentsAndSpecialLinkAndNoHrefAspect)),
        // ),
        // (
        // r#"<Anchor hrefLeft={null} />"#,
        // Some(serde_json::json!(componentsAndSpecialLinkAndNoHrefAspect)),
        // ),
        // (
        // r#"<Anchor hrefLeft={undefined} />"#,
        // Some(serde_json::json!(componentsAndSpecialLinkAndNoHrefAspect)),
        // ),
        // (
        // r#"<Anchor hrefLeft={null} />"#,
        // Some(serde_json::json!(componentsAndSpecialLinkAndNoHrefAspect)),
        // ),
        // (
        // r#"<Anchor hrefLeft={undefined} />"#,
        // Some(serde_json::json!(componentsAndSpecialLinkAndNoHrefAspect)),
        // ),
        // (
        // r#"<Anchor hrefLeft={null} />"#,
        // Some(serde_json::json!(componentsAndSpecialLinkAndNoHrefAspect)),
        // ),
    ];

    Tester::new(AnchorIsValid::NAME, AnchorIsValid::PLUGIN, pass, fail).test_and_snapshot();
}
