use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{self, Error},
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-jsx-a11y(anchor-is-valid):")]
#[diagnostic(severity(warning), help(""))]
struct AnchorIsValidDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct AnchorIsValid;

declare_oxc_lint!(
    /// ### What it does
    ///
    ///
    /// ### Why is this bad?
    ///
    ///
    /// ### Example
    /// ```javascript
    /// ```
    AnchorIsValid,
    correctness
);

impl Rule for AnchorIsValid {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {}
}

#[test]
fn test() {
    use crate::tester::Tester;

    let components = vec![1];
    let specialLink = vec![1];
    let componentsAndSpecialLink = vec![1];
    let invalidHrefAspect = vec![1];
    let preferButtonAspect = vec![1];
    let preferButtonInvalidHrefAspect = vec![1];
    let noHrefAspect = vec![1];
    let noHrefPreferButtonAspect = vec![1];
    let componentsAndSpecialLinkAndInvalidHrefAspect = vec![1];
    let noHrefInvalidHrefAspect = vec![1];
    let componentsAndSpecialLinkAndNoHrefAspect = vec![1];

    // https://raw.githubusercontent.com/jsx-eslint/eslint-plugin-jsx-a11y/main/__tests__/src/rules/anchor-is-valid-test.js
    let pass = vec![
        (r#"<Anchor />"#, None),
        (r#"<a {...props} />"#, None),
        (r#"<a href=\"foo\" />"#, None),
        (r#"<a href={foo} />"#, None),
        (r#"<a href=\"/foo\" />"#, None),
        (r#"<a href=\"https://foo.bar.com\" />"#, None),
        (r#"<div href=\"foo\" />"#, None),
        (r#"<a href=\"javascript\" />"#, None),
        (r#"<a href=\"javascriptFoo\" />"#, None),
        (r#"<a href={`#foo`}/>"#, None),
        (r#"<a href={\"foo\"}/>"#, None),
        (r#"<a href={\"javascript\"}/>"#, None),
        (r#"<a href={`#javascript`}/>"#, None),
        (r#"<a href='#foo' />"#, None),
        (r#"<a href='#javascript' />"#, None),
        (r#"<a href='#javascriptFoo' />"#, None),
        (r#"<UX.Layout>test</UX.Layout>"#, None),
        (r#"<a href={this} />"#, None),
        (r#"<Anchor {...props} />"#, Some(serde_json::json!(components))),
        (r#"<Anchor href=\"foo\" />"#, Some(serde_json::json!(components))),
        (r#"<Anchor href={foo} />"#, Some(serde_json::json!(components))),
        (r#"<Anchor href=\"/foo\" />"#, Some(serde_json::json!(components))),
        (r#"<Anchor href=\"https://foo.bar.com\" />"#, Some(serde_json::json!(components))),
        (r#"<div href=\"foo\" />"#, Some(serde_json::json!(components))),
        (r#"<Anchor href={`#foo`}/>"#, Some(serde_json::json!(components))),
        (r#"<Anchor href={\"foo\"}/>"#, Some(serde_json::json!(components))),
        (r#"<Anchor href='#foo' />"#, Some(serde_json::json!(components))),
        (r#"<Link {...props} />"#, Some(serde_json::json!(components))),
        (r#"<Link href=\"foo\" />"#, Some(serde_json::json!(components))),
        (r#"<Link href={foo} />"#, Some(serde_json::json!(components))),
        (r#"<Link href=\"/foo\" />"#, Some(serde_json::json!(components))),
        (r#"<Link href=\"https://foo.bar.com\" />"#, Some(serde_json::json!(components))),
        (r#"<div href=\"foo\" />"#, Some(serde_json::json!(components))),
        (r#"<Link href={`#foo`}/>"#, Some(serde_json::json!(components))),
        (r#"<Link href={\"foo\"}/>"#, Some(serde_json::json!(components))),
        (r#"<Link href='#foo' />"#, Some(serde_json::json!(components))),
        (r#"<Link href='#foo' />"#, None),
        (r#"<a {...props} />"#, Some(serde_json::json!(specialLink))),
        (r#"<a hrefLeft=\"foo\" />"#, Some(serde_json::json!(specialLink))),
        (r#"<a hrefLeft={foo} />"#, Some(serde_json::json!(specialLink))),
        (r#"<a hrefLeft='/foo' />"#, Some(serde_json::json!(specialLink))),
        (r#"<a hrefLeft=\"https://foo.bar.com\" />"#, Some(serde_json::json!(specialLink))),
        (r#"<div hrefLeft=\"foo\" />"#, Some(serde_json::json!(specialLink))),
        (r#"<a hrefLeft={`#foo`}/>"#, Some(serde_json::json!(specialLink))),
        (r#"<a hrefLeft={\"foo\"}/>"#, Some(serde_json::json!(specialLink))),
        (r#"<a hrefLeft='#foo' />"#, Some(serde_json::json!(specialLink))),
        (r#"<UX.Layout>test</UX.Layout>"#, Some(serde_json::json!(specialLink))),
        (r#"<a hrefRight={this} />"#, Some(serde_json::json!(specialLink))),
        (r#"<a {...props} />"#, Some(serde_json::json!(specialLink))),
        (r#"<a hrefRight=\"foo\" />"#, Some(serde_json::json!(specialLink))),
        (r#"<a hrefRight={foo} />"#, Some(serde_json::json!(specialLink))),
        (r#"<a hrefRight=\"/foo\" />"#, Some(serde_json::json!(specialLink))),
        (r#"<a hrefRight=\"https://foo.bar.com\" />"#, Some(serde_json::json!(specialLink))),
        (r#"<div hrefRight=\"foo\" />"#, Some(serde_json::json!(specialLink))),
        (r#"<a hrefRight={`#foo`}/>"#, Some(serde_json::json!(specialLink))),
        (r#"<a hrefRight={\"foo\"}/>"#, Some(serde_json::json!(specialLink))),
        (r#"<a hrefRight='#foo' />"#, Some(serde_json::json!(specialLink))),
        (r#"<UX.Layout>test</UX.Layout>"#, Some(serde_json::json!(specialLink))),
        (r#"<a hrefRight={this} />"#, Some(serde_json::json!(specialLink))),
        (r#"<Anchor {...props} />"#, Some(serde_json::json!(componentsAndSpecialLink))),
        (r#"<Anchor hrefLeft=\"foo\" />"#, Some(serde_json::json!(componentsAndSpecialLink))),
        (r#"<Anchor hrefLeft={foo} />"#, Some(serde_json::json!(componentsAndSpecialLink))),
        (r#"<Anchor hrefLeft=\"/foo\" />"#, Some(serde_json::json!(componentsAndSpecialLink))),
        (
            r#"<Anchor hrefLeft=\"https://foo.bar.com\" />"#,
            Some(serde_json::json!(componentsAndSpecialLink)),
        ),
        (r#"<div hrefLeft=\"foo\" />"#, Some(serde_json::json!(componentsAndSpecialLink))),
        (r#"<Anchor hrefLeft={`#foo`}/>"#, Some(serde_json::json!(componentsAndSpecialLink))),
        (r#"<Anchor hrefLeft={\"foo\"}/>"#, Some(serde_json::json!(componentsAndSpecialLink))),
        (r#"<Anchor hrefLeft='#foo' />"#, Some(serde_json::json!(componentsAndSpecialLink))),
        (r#"<UX.Layout>test</UX.Layout>"#, Some(serde_json::json!(componentsAndSpecialLink))),
        (r#"<a {...props} onClick={() => void 0} />"#, None),
        (r#"<a href=\"foo\" onClick={() => void 0} />"#, None),
        (r#"<a href={foo} onClick={() => void 0} />"#, None),
        (r#"<a href=\"/foo\" onClick={() => void 0} />"#, None),
        (r#"<a href=\"https://foo.bar.com\" onClick={() => void 0} />"#, None),
        (r#"<div href=\"foo\" onClick={() => void 0} />"#, None),
        (r#"<a href={`#foo`} onClick={() => void 0} />"#, None),
        (r#"<a href={\"foo\"} onClick={() => void 0} />"#, None),
        (r#"<a href='#foo' onClick={() => void 0} />"#, None),
        (r#"<a href={this} onClick={() => void 0} />"#, None),
        (r#"<Anchor {...props} onClick={() => void 0} />"#, Some(serde_json::json!(components))),
        (r#"<Anchor href=\"foo\" onClick={() => void 0} />"#, Some(serde_json::json!(components))),
        (r#"<Anchor href={foo} onClick={() => void 0} />"#, Some(serde_json::json!(components))),
        (r#"<Anchor href=\"/foo\" onClick={() => void 0} />"#, Some(serde_json::json!(components))),
        (
            r#"<Anchor href=\"https://foo.bar.com\" onClick={() => void 0} />"#,
            Some(serde_json::json!(components)),
        ),
        (r#"<Anchor href={`#foo`} onClick={() => void 0} />"#, Some(serde_json::json!(components))),
        (
            r#"<Anchor href={\"foo\"} onClick={() => void 0} />"#,
            Some(serde_json::json!(components)),
        ),
        (r#"<Anchor href='#foo' onClick={() => void 0} />"#, Some(serde_json::json!(components))),
        (r#"<Link {...props} onClick={() => void 0} />"#, Some(serde_json::json!(components))),
        (r#"<Link href=\"foo\" onClick={() => void 0} />"#, Some(serde_json::json!(components))),
        (r#"<Link href={foo} onClick={() => void 0} />"#, Some(serde_json::json!(components))),
        (r#"<Link href=\"/foo\" onClick={() => void 0} />"#, Some(serde_json::json!(components))),
        (
            r#"<Link href=\"https://foo.bar.com\" onClick={() => void 0} />"#,
            Some(serde_json::json!(components)),
        ),
        (r#"<div href=\"foo\" onClick={() => void 0} />"#, Some(serde_json::json!(components))),
        (r#"<Link href={`#foo`} onClick={() => void 0} />"#, Some(serde_json::json!(components))),
        (r#"<Link href={\"foo\"} onClick={() => void 0} />"#, Some(serde_json::json!(components))),
        (r#"<Link href='#foo' onClick={() => void 0} />"#, Some(serde_json::json!(components))),
        (r#"<a {...props} onClick={() => void 0} />"#, Some(serde_json::json!(specialLink))),
        (r#"<a hrefLeft=\"foo\" onClick={() => void 0} />"#, Some(serde_json::json!(specialLink))),
        (r#"<a hrefLeft={foo} onClick={() => void 0} />"#, Some(serde_json::json!(specialLink))),
        (r#"<a hrefLeft=\"/foo\" onClick={() => void 0} />"#, Some(serde_json::json!(specialLink))),
        (
            r#"<a hrefLeft href=\"https://foo.bar.com\" onClick={() => void 0} />"#,
            Some(serde_json::json!(specialLink)),
        ),
        (
            r#"<div hrefLeft=\"foo\" onClick={() => void 0} />"#,
            Some(serde_json::json!(specialLink)),
        ),
        (r#"<a hrefLeft={`#foo`} onClick={() => void 0} />"#, Some(serde_json::json!(specialLink))),
        (
            r#"<a hrefLeft={\"foo\"} onClick={() => void 0} />"#,
            Some(serde_json::json!(specialLink)),
        ),
        (r#"<a hrefLeft='#foo' onClick={() => void 0} />"#, Some(serde_json::json!(specialLink))),
        (r#"<a hrefRight={this} onClick={() => void 0} />"#, Some(serde_json::json!(specialLink))),
        (r#"<a {...props} onClick={() => void 0} />"#, Some(serde_json::json!(specialLink))),
        (r#"<a hrefRight=\"foo\" onClick={() => void 0} />"#, Some(serde_json::json!(specialLink))),
        (r#"<a hrefRight={foo} onClick={() => void 0} />"#, Some(serde_json::json!(specialLink))),
        (
            r#"<a hrefRight=\"/foo\" onClick={() => void 0} />"#,
            Some(serde_json::json!(specialLink)),
        ),
        (
            r#"<a hrefRight href=\"https://foo.bar.com\" onClick={() => void 0} />"#,
            Some(serde_json::json!(specialLink)),
        ),
        (
            r#"<div hrefRight=\"foo\" onClick={() => void 0} />"#,
            Some(serde_json::json!(specialLink)),
        ),
        (
            r#"<a hrefRight={`#foo`} onClick={() => void 0} />"#,
            Some(serde_json::json!(specialLink)),
        ),
        (
            r#"<a hrefRight={\"foo\"} onClick={() => void 0} />"#,
            Some(serde_json::json!(specialLink)),
        ),
        (r#"<a hrefRight='#foo' onClick={() => void 0} />"#, Some(serde_json::json!(specialLink))),
        (r#"<a hrefRight={this} onClick={() => void 0} />"#, Some(serde_json::json!(specialLink))),
        (
            r#"<Anchor {...props} onClick={() => void 0} />"#,
            Some(serde_json::json!(componentsAndSpecialLink)),
        ),
        (
            r#"<Anchor hrefLeft=\"foo\" onClick={() => void 0} />"#,
            Some(serde_json::json!(componentsAndSpecialLink)),
        ),
        (
            r#"<Anchor hrefLeft={foo} onClick={() => void 0} />"#,
            Some(serde_json::json!(componentsAndSpecialLink)),
        ),
        (
            r#"<Anchor hrefLeft=\"/foo\" onClick={() => void 0} />"#,
            Some(serde_json::json!(componentsAndSpecialLink)),
        ),
        (
            r#"<Anchor hrefLeft href=\"https://foo.bar.com\" onClick={() => void 0} />"#,
            Some(serde_json::json!(componentsAndSpecialLink)),
        ),
        (
            r#"<Anchor hrefLeft={`#foo`} onClick={() => void 0} />"#,
            Some(serde_json::json!(componentsAndSpecialLink)),
        ),
        (
            r#"<Anchor hrefLeft={\"foo\"} onClick={() => void 0} />"#,
            Some(serde_json::json!(componentsAndSpecialLink)),
        ),
        (
            r#"<Anchor hrefLeft='#foo'` onClick={() => void 0} />"#,
            Some(serde_json::json!(componentsAndSpecialLink)),
        ),
        (r#"<a />"#, Some(serde_json::json!(invalidHrefAspect))),
        (r#"<a href={undefined} />"#, Some(serde_json::json!(invalidHrefAspect))),
        (r#"<a href={null} />"#, Some(serde_json::json!(invalidHrefAspect))),
        (r#"<a />"#, Some(serde_json::json!(preferButtonAspect))),
        (r#"<a href={undefined} />"#, Some(serde_json::json!(preferButtonAspect))),
        (r#"<a href={null} />"#, Some(serde_json::json!(preferButtonAspect))),
        (r#"<a />"#, Some(serde_json::json!(preferButtonInvalidHrefAspect))),
        (r#"<a href={undefined} />"#, Some(serde_json::json!(preferButtonInvalidHrefAspect))),
        (r#"<a href={null} />"#, Some(serde_json::json!(preferButtonInvalidHrefAspect))),
        (r#"<a href=\"\" />;"#, Some(serde_json::json!(preferButtonAspect))),
        (r#"<a href='#' />"#, Some(serde_json::json!(preferButtonAspect))),
        (r#"<a href={'#'} />"#, Some(serde_json::json!(preferButtonAspect))),
        (r#"<a href=\"javascript:void(0)\" />"#, Some(serde_json::json!(preferButtonAspect))),
        (r#"<a href={\"javascript:void(0)\"} />"#, Some(serde_json::json!(preferButtonAspect))),
        (r#"<a href=\"\" />;"#, Some(serde_json::json!(noHrefAspect))),
        (r#"<a href='#' />"#, Some(serde_json::json!(noHrefAspect))),
        (r#"<a href={'#'} />"#, Some(serde_json::json!(noHrefAspect))),
        (r#"<a href=\"javascript:void(0)\" />"#, Some(serde_json::json!(noHrefAspect))),
        (r#"<a href={\"javascript:void(0)\"} />"#, Some(serde_json::json!(noHrefAspect))),
        (r#"<a href=\"\" />;"#, Some(serde_json::json!(noHrefPreferButtonAspect))),
        (r#"<a href='#' />"#, Some(serde_json::json!(noHrefPreferButtonAspect))),
        (r#"<a href={'#'} />"#, Some(serde_json::json!(noHrefPreferButtonAspect))),
        (r#"<a href=\"javascript:void(0)\" />"#, Some(serde_json::json!(noHrefPreferButtonAspect))),
        (
            r#"<a href={\"javascript:void(0)\"} />"#,
            Some(serde_json::json!(noHrefPreferButtonAspect)),
        ),
        (r#"<a onClick={() => void 0} />"#, Some(serde_json::json!(invalidHrefAspect))),
        (r#"<a href='#' onClick={() => void 0} />"#, Some(serde_json::json!(noHrefAspect))),
        (
            r#"<a href=\"javascript:void(0)\" onClick={() => void 0} />"#,
            Some(serde_json::json!(noHrefAspect)),
        ),
        (
            r#"<a href={\"javascript:void(0)\"} onClick={() => void 0} />"#,
            Some(serde_json::json!(noHrefAspect)),
        ),
        (
            r#"<Anchor hrefLeft={undefined} />"#,
            Some(serde_json::json!(componentsAndSpecialLinkAndInvalidHrefAspect)),
        ),
        (
            r#"<Anchor hrefLeft={null} />"#,
            Some(serde_json::json!(componentsAndSpecialLinkAndInvalidHrefAspect)),
        ),
        (
            r#"<Anchor hrefLeft={undefined} />"#,
            Some(serde_json::json!(componentsAndSpecialLinkAndInvalidHrefAspect)),
        ),
        (
            r#"<Anchor hrefLeft={null} />"#,
            Some(serde_json::json!(componentsAndSpecialLinkAndInvalidHrefAspect)),
        ),
        (
            r#"<Anchor hrefLeft={undefined} />"#,
            Some(serde_json::json!(componentsAndSpecialLinkAndInvalidHrefAspect)),
        ),
        (
            r#"<Anchor hrefLeft={null} />"#,
            Some(serde_json::json!(componentsAndSpecialLinkAndInvalidHrefAspect)),
        ),
    ];

    let fail = vec![
        (r#"<a />"#, None),
        (r#"<a href={undefined} />"#, None),
        (r#"<a href={null} />"#, None),
        (r#"<a href=\"\" />;"#, None),
        (r#"<a href='#' />"#, None),
        (r#"<a href={'#'} />"#, None),
        (r#"<a href=\"javascript:void(0)\" />"#, None),
        (r#"<a href={\"javascript:void(0)\"} />"#, None),
        (r#"<a onClick={() => void 0} />"#, None),
        (r#"<a href='#' onClick={() => void 0} />"#, None),
        (r#"<a href=\"javascript:void(0)\" onClick={() => void 0} />"#, None),
        (r#"<a href={\"javascript:void(0)\"} onClick={() => void 0} />"#, None),
        (r#"<Link />"#, Some(serde_json::json!(components))),
        (r#"<Link href={undefined} />"#, Some(serde_json::json!(components))),
        (r#"<Link href={null} />"#, Some(serde_json::json!(components))),
        (r#"<Link href=\"\" />"#, Some(serde_json::json!(components))),
        (r#"<Link href='#' />"#, Some(serde_json::json!(components))),
        (r#"<Link href={'#'} />"#, Some(serde_json::json!(components))),
        (r#"<Link href=\"javascript:void(0)\" />"#, Some(serde_json::json!(components))),
        (r#"<Link href={\"javascript:void(0)\"} />"#, Some(serde_json::json!(components))),
        (r#"<Anchor href=\"\" />"#, Some(serde_json::json!(components))),
        (r#"<Anchor href='#' />"#, Some(serde_json::json!(components))),
        (r#"<Anchor href={'#'} />"#, Some(serde_json::json!(components))),
        (r#"<Anchor href=\"javascript:void(0)\" />"#, Some(serde_json::json!(components))),
        (r#"<Anchor href={\"javascript:void(0)\"} />"#, Some(serde_json::json!(components))),
        (r#"<Link onClick={() => void 0} />"#, Some(serde_json::json!(components))),
        (r#"<Link href='#' onClick={() => void 0} />"#, Some(serde_json::json!(components))),
        (
            r#"<Link href=\"javascript:void(0)\" onClick={() => void 0} />"#,
            Some(serde_json::json!(components)),
        ),
        (
            r#"<Link href={\"javascript:void(0)\"} onClick={() => void 0} />"#,
            Some(serde_json::json!(components)),
        ),
        (r#"<Anchor onClick={() => void 0} />"#, Some(serde_json::json!(components))),
        (r#"<Anchor href='#' onClick={() => void 0} />"#, Some(serde_json::json!(components))),
        (
            r#"<Anchor href=\"javascript:void(0)\" onClick={() => void 0} />"#,
            Some(serde_json::json!(components)),
        ),
        (
            r#"<Anchor href={\"javascript:void(0)\"} onClick={() => void 0} />"#,
            Some(serde_json::json!(components)),
        ),
        (r#"<Link href='#' onClick={() => void 0} />"#, None),
        (r#"<a hrefLeft={undefined} />"#, Some(serde_json::json!(specialLink))),
        (r#"<a hrefLeft={null} />"#, Some(serde_json::json!(specialLink))),
        (r#"<a hrefLeft=\"\" />;"#, Some(serde_json::json!(specialLink))),
        (r#"<a hrefLeft='#' />"#, Some(serde_json::json!(specialLink))),
        (r#"<a hrefLeft={'#'} />"#, Some(serde_json::json!(specialLink))),
        (r#"<a hrefLeft=\"javascript:void(0)\" />"#, Some(serde_json::json!(specialLink))),
        (r#"<a hrefLeft={\"javascript:void(0)\"} />"#, Some(serde_json::json!(specialLink))),
        (r#"<a hrefLeft='#' onClick={() => void 0} />"#, Some(serde_json::json!(specialLink))),
        (
            r#"<a hrefLeft=\"javascript:void(0)\" onClick={() => void 0} />"#,
            Some(serde_json::json!(specialLink)),
        ),
        (
            r#"<a hrefLeft={\"javascript:void(0)\"} onClick={() => void 0} />"#,
            Some(serde_json::json!(specialLink)),
        ),
        (r#"<Anchor Anchor={undefined} />"#, Some(serde_json::json!(componentsAndSpecialLink))),
        (r#"<Anchor hrefLeft={null} />"#, Some(serde_json::json!(componentsAndSpecialLink))),
        (r#"<Anchor hrefLeft=\"\" />;"#, Some(serde_json::json!(componentsAndSpecialLink))),
        (r#"<Anchor hrefLeft='#' />"#, Some(serde_json::json!(componentsAndSpecialLink))),
        (r#"<Anchor hrefLeft={'#'} />"#, Some(serde_json::json!(componentsAndSpecialLink))),
        (
            r#"<Anchor hrefLeft=\"javascript:void(0)\" />"#,
            Some(serde_json::json!(componentsAndSpecialLink)),
        ),
        (
            r#"<Anchor hrefLeft={\"javascript:void(0)\"} />"#,
            Some(serde_json::json!(componentsAndSpecialLink)),
        ),
        (
            r#"<Anchor hrefLeft='#' onClick={() => void 0} />"#,
            Some(serde_json::json!(componentsAndSpecialLink)),
        ),
        (
            r#"<Anchor hrefLeft=\"javascript:void(0)\" onClick={() => void 0} />"#,
            Some(serde_json::json!(componentsAndSpecialLink)),
        ),
        (
            r#"<Anchor hrefLeft={\"javascript:void(0)\"} onClick={() => void 0} />"#,
            Some(serde_json::json!(componentsAndSpecialLink)),
        ),
        (r#"<a />"#, Some(serde_json::json!(noHrefAspect))),
        (r#"<a />"#, Some(serde_json::json!(noHrefPreferButtonAspect))),
        (r#"<a />"#, Some(serde_json::json!(noHrefInvalidHrefAspect))),
        (r#"<a href={undefined} />"#, Some(serde_json::json!(noHrefAspect))),
        (r#"<a href={undefined} />"#, Some(serde_json::json!(noHrefPreferButtonAspect))),
        (r#"<a href={undefined} />"#, Some(serde_json::json!(noHrefInvalidHrefAspect))),
        (r#"<a href={null} />"#, Some(serde_json::json!(noHrefAspect))),
        (r#"<a href={null} />"#, Some(serde_json::json!(noHrefPreferButtonAspect))),
        (r#"<a href={null} />"#, Some(serde_json::json!(noHrefInvalidHrefAspect))),
        (r#"<a href=\"\" />;"#, Some(serde_json::json!(invalidHrefAspect))),
        (r#"<a href=\"\" />;"#, Some(serde_json::json!(noHrefInvalidHrefAspect))),
        (r#"<a href=\"\" />;"#, Some(serde_json::json!(preferButtonInvalidHrefAspect))),
        (r#"<a href='#' />;"#, Some(serde_json::json!(invalidHrefAspect))),
        (r#"<a href='#' />;"#, Some(serde_json::json!(noHrefInvalidHrefAspect))),
        (r#"<a href='#' />;"#, Some(serde_json::json!(preferButtonInvalidHrefAspect))),
        (r#"<a href={'#'} />;"#, Some(serde_json::json!(invalidHrefAspect))),
        (r#"<a href={'#'} />;"#, Some(serde_json::json!(noHrefInvalidHrefAspect))),
        (r#"<a href={'#'} />;"#, Some(serde_json::json!(preferButtonInvalidHrefAspect))),
        (r#"<a href=\"javascript:void(0)\" />;"#, Some(serde_json::json!(invalidHrefAspect))),
        (r#"<a href=\"javascript:void(0)\" />;"#, Some(serde_json::json!(noHrefInvalidHrefAspect))),
        (
            r#"<a href=\"javascript:void(0)\" />;"#,
            Some(serde_json::json!(preferButtonInvalidHrefAspect)),
        ),
        (r#"<a href={\"javascript:void(0)\"} />;"#, Some(serde_json::json!(invalidHrefAspect))),
        (
            r#"<a href={\"javascript:void(0)\"} />;"#,
            Some(serde_json::json!(noHrefInvalidHrefAspect)),
        ),
        (
            r#"<a href={\"javascript:void(0)\"} />;"#,
            Some(serde_json::json!(preferButtonInvalidHrefAspect)),
        ),
        (r#"<a onClick={() => void 0} />"#, Some(serde_json::json!(preferButtonAspect))),
        (r#"<a onClick={() => void 0} />"#, Some(serde_json::json!(preferButtonInvalidHrefAspect))),
        (r#"<a onClick={() => void 0} />"#, Some(serde_json::json!(noHrefPreferButtonAspect))),
        (r#"<a onClick={() => void 0} />"#, Some(serde_json::json!(noHrefAspect))),
        (r#"<a onClick={() => void 0} />"#, Some(serde_json::json!(noHrefInvalidHrefAspect))),
        (r#"<a href='#' onClick={() => void 0} />"#, Some(serde_json::json!(preferButtonAspect))),
        (
            r#"<a href='#' onClick={() => void 0} />"#,
            Some(serde_json::json!(noHrefPreferButtonAspect)),
        ),
        (
            r#"<a href='#' onClick={() => void 0} />"#,
            Some(serde_json::json!(preferButtonInvalidHrefAspect)),
        ),
        (r#"<a href='#' onClick={() => void 0} />"#, Some(serde_json::json!(invalidHrefAspect))),
        (
            r#"<a href='#' onClick={() => void 0} />"#,
            Some(serde_json::json!(noHrefInvalidHrefAspect)),
        ),
        (
            r#"<a href=\"javascript:void(0)\" onClick={() => void 0} />"#,
            Some(serde_json::json!(preferButtonAspect)),
        ),
        (
            r#"<a href=\"javascript:void(0)\" onClick={() => void 0} />"#,
            Some(serde_json::json!(noHrefPreferButtonAspect)),
        ),
        (
            r#"<a href=\"javascript:void(0)\" onClick={() => void 0} />"#,
            Some(serde_json::json!(preferButtonInvalidHrefAspect)),
        ),
        (
            r#"<a href=\"javascript:void(0)\" onClick={() => void 0} />"#,
            Some(serde_json::json!(invalidHrefAspect)),
        ),
        (
            r#"<a href=\"javascript:void(0)\" onClick={() => void 0} />"#,
            Some(serde_json::json!(noHrefInvalidHrefAspect)),
        ),
        (
            r#"<a href={\"javascript:void(0)\"} onClick={() => void 0} />"#,
            Some(serde_json::json!(preferButtonAspect)),
        ),
        (
            r#"<a href={\"javascript:void(0)\"} onClick={() => void 0} />"#,
            Some(serde_json::json!(noHrefPreferButtonAspect)),
        ),
        (
            r#"<a href={\"javascript:void(0)\"} onClick={() => void 0} />"#,
            Some(serde_json::json!(preferButtonInvalidHrefAspect)),
        ),
        (
            r#"<a href={\"javascript:void(0)\"} onClick={() => void 0} />"#,
            Some(serde_json::json!(invalidHrefAspect)),
        ),
        (
            r#"<a href={\"javascript:void(0)\"} onClick={() => void 0} />"#,
            Some(serde_json::json!(noHrefInvalidHrefAspect)),
        ),
        (
            r#"<Anchor hrefLeft={undefined} />"#,
            Some(serde_json::json!(componentsAndSpecialLinkAndNoHrefAspect)),
        ),
        (
            r#"<Anchor hrefLeft={null} />"#,
            Some(serde_json::json!(componentsAndSpecialLinkAndNoHrefAspect)),
        ),
        (
            r#"<Anchor hrefLeft={undefined} />"#,
            Some(serde_json::json!(componentsAndSpecialLinkAndNoHrefAspect)),
        ),
        (
            r#"<Anchor hrefLeft={null} />"#,
            Some(serde_json::json!(componentsAndSpecialLinkAndNoHrefAspect)),
        ),
        (
            r#"<Anchor hrefLeft={undefined} />"#,
            Some(serde_json::json!(componentsAndSpecialLinkAndNoHrefAspect)),
        ),
        (
            r#"<Anchor hrefLeft={null} />"#,
            Some(serde_json::json!(componentsAndSpecialLinkAndNoHrefAspect)),
        ),
    ];

    Tester::new(AnchorIsValid::NAME, pass, fail).test_and_snapshot();
}
