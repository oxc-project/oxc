use oxc_ast::{
    ast::{Expression, JSXAttributeItem, JSXAttributeValue, JSXElementName, JSXExpression},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, utils::has_jsx_prop_lowercase, AstNode};

#[derive(Debug, Error, Diagnostic)]
enum AnchorIsValidDiagnostic {
    #[error(
        "eslint-plugin-jsx-a11y(anchor-is-valid): Missing `href` attribute for the `a` element."
    )]
    #[diagnostic(severity(warning), help("Provide an href for the `a` element."))]
    MissingHrefAttribute(#[label] Span),

    #[error("eslint-plugin-jsx-a11y(anchor-is-valid): Use an incorrect href for the 'a' element.")]
    #[diagnostic(severity(warning), help("Provide a correct href for the `a` element."))]
    IncorrectHref(#[label] Span),

    #[error("eslint-plugin-jsx-a11y(anchor-is-valid):  The a element has `href` and `onClick`.")]
    #[diagnostic(severity(warning), help("Use a `button` element instead of an `a` element."))]
    CantBeAnchor(#[label] Span),
}
#[derive(Debug, Default, Clone)]
pub struct AnchorIsValid;

declare_oxc_lint!(
    /// ### What it does
    /// The HTML <a> element, with a valid href attribute, is formally defined as representing a **hyperlink**.
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
    /// ```javascript
    /// <a href="javascript:void(0)" onClick={foo}>Perform action</a>
    /// <a href="#" onClick={foo}>Perform action</a>
    /// <a onClick={foo}>Perform action</a>
    /// ````
    ///
    /// All these anchor implementations indicate that the element is only used to execute JavaScript code. All the above should be replaced with:
    ///
    /// ```javascript
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
    /// ```javascript
    /// <a href={`https://www.javascript.com`}>navigate here</a>
    /// ```
    ///
    /// ```javascript
    /// <a href={somewhere}>navigate here</a>
    /// ```
    ///
    /// ```javascript
    /// <a {...spread}>navigate here</a>
    /// ```
    ///
    /// #### Invalid
    ///
    /// ```javascript
    /// <a href={null}>navigate here</a>
    /// ```
    /// ```javascript
    /// <a href={undefined}>navigate here</a>
    /// ```
    /// ```javascript
    /// <a href>navigate here</a>
    /// ```
    /// ```javascript
    /// <a href="javascript:void(0)">navigate here</a>
    /// ```
    /// ```javascript
    /// <a href="https://example.com" onClick={something}>navigate here</a>
    /// ```
    ///
    /// ### Reference
    ///
    /// - [WCAG 2.1.1](https://www.w3.org/WAI/WCAG21/Understanding/keyboard)
    AnchorIsValid,
    correctness
);

fn check_value_is_empty(value: &JSXAttributeValue) -> bool {
    match value {
        JSXAttributeValue::Element(_) => false,
        JSXAttributeValue::StringLiteral(str_lit) => {
            str_lit.value.is_empty()
                || str_lit.value == "#"
                || str_lit.value == "javascript:void(0)"
        }
        JSXAttributeValue::ExpressionContainer(exp) => {
            if let JSXExpression::Expression(jsexp) = &exp.expression {
                if let Expression::Identifier(ident) = jsexp {
                    if ident.name == "undefined" {
                        return true;
                    }
                } else if let Expression::NullLiteral(_) = jsexp {
                    return true;
                } else if let Expression::StringLiteral(str_lit) = jsexp {
                    return str_lit.value.is_empty()
                        || str_lit.value == "#"
                        || str_lit.value == "javascript:void(0)";
                }
            };
            false
        }
        JSXAttributeValue::Fragment(_) => true,
    }
}

impl Rule for AnchorIsValid {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::JSXElement(jsx_el) = node.kind() {
            let JSXElementName::Identifier(ident) = &jsx_el.opening_element.name else { return };
            let name = ident.name.as_str();
            if name == "a" {
                if let Option::Some(herf_attr) =
                    has_jsx_prop_lowercase(&jsx_el.opening_element, "href")
                {
                    // Check if the 'a' element has a correct href attribute
                    match herf_attr {
                        JSXAttributeItem::Attribute(attr) => match &attr.value {
                            Some(value) => {
                                let is_empty = check_value_is_empty(value);
                                if is_empty {
                                    if has_jsx_prop_lowercase(&jsx_el.opening_element, "onclick")
                                        .is_some()
                                    {
                                        ctx.diagnostic(AnchorIsValidDiagnostic::CantBeAnchor(
                                            ident.span,
                                        ));
                                        return;
                                    }
                                    ctx.diagnostic(AnchorIsValidDiagnostic::IncorrectHref(
                                        ident.span,
                                    ));
                                    return;
                                }
                            }
                            None => {
                                ctx.diagnostic(AnchorIsValidDiagnostic::IncorrectHref(ident.span));
                                return;
                            }
                        },

                        JSXAttributeItem::SpreadAttribute(_) => {
                            // pass
                            return;
                        }
                    }
                    return;
                }
                // Exclude '<a {...props} />' case
                let has_spreed_attr =
                    jsx_el.opening_element.attributes.iter().any(|attr| match attr {
                        JSXAttributeItem::SpreadAttribute(_) => true,
                        JSXAttributeItem::Attribute(_) => false,
                    });

                if has_spreed_attr {
                    return;
                }

                ctx.diagnostic(AnchorIsValidDiagnostic::MissingHrefAttribute(ident.span));
            }
        }
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
        (r"<Anchor />", None),
        (r"<a {...props} />", None),
        (r"<a href='foo' />", None),
        (r"<a href={foo} />", None),
        (r"<a href='/foo' />", None),
        (r"<a href='https://foo.bar.com' />", None),
        (r"<div href='foo' />", None),
        (r"<a href='javascript' />", None),
        (r"<a href='javascriptFoo' />", None),
        (r"<a href={`#foo`}/>", None),
        (r"<a href={'foo'}/>", None),
        (r"<a href={'javascript'}/>", None),
        (r"<a href={`#javascript`}/>", None),
        (r"<a href='#foo' />", None),
        (r"<a href='#javascript' />", None),
        (r"<a href='#javascriptFoo' />", None),
        (r"<UX.Layout>test</UX.Layout>", None),
        (r"<a href={this} />", None),
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
        (r"<Link href='#foo' />", None),
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
        (r"<a {...props} onClick={() => void 0} />", None),
        (r"<a href='foo' onClick={() => void 0} />", None),
        (r"<a href={foo} onClick={() => void 0} />", None),
        (r"<a href='/foo' onClick={() => void 0} />", None),
        (r"<a href='https://foo.bar.com' onClick={() => void 0} />", None),
        (r"<div href='foo' onClick={() => void 0} />", None),
        (r"<a href={`#foo`} onClick={() => void 0} />", None),
        (r"<a href={'foo'} onClick={() => void 0} />", None),
        (r"<a href='#foo' onClick={() => void 0} />", None),
        (r"<a href={this} onClick={() => void 0} />", None),
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
        (r"<a />", None),
        (r"<a href={undefined} />", None),
        (r"<a href={null} />", None),
        (r"<a href=' />;", None),
        (r"<a href='#' />", None),
        (r"<a href={'#'} />", None),
        (r"<a href='javascript:void(0)' />", None),
        (r"<a href={'javascript:void(0)'} />", None),
        (r"<a onClick={() => void 0} />", None),
        (r"<a href='#' onClick={() => void 0} />", None),
        (r"<a href='javascript:void(0)' onClick={() => void 0} />", None),
        (r"<a href={'javascript:void(0)'} onClick={() => void 0} />", None),
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
        // (r#"<Link href='#' onClick={() => void 0} />"#, None),
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

    Tester::new(AnchorIsValid::NAME, pass, fail).test_and_snapshot();
}
