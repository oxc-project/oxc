use oxc_ast::AstKind;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{
        get_element_type, has_jsx_prop_lowercase, is_hidden_from_screen_reader,
        object_has_accessible_child,
    },
    AstNode,
};

#[derive(Debug, Error, Diagnostic)]
enum AnchorHasContentDiagnostic {
    #[error("eslint-plugin-jsx-a11y(anchor-has-content): Missing accessible content when using `a` elements.")]
    #[diagnostic(
        severity(warning),
        help("Provide screen reader accessible content when using `a` elements.")
    )]
    MissingContent(#[label] Span),

    #[error("eslint-plugin-jsx-a11y(anchor-has-content): Missing accessible content when using `a` elements.")]
    #[diagnostic(severity(warning), help("Remove the `aria-hidden` attribute to allow the anchor element and its content visible to assistive technologies."))]
    RemoveAriaHidden(#[label] Span),
}

#[derive(Debug, Default, Clone)]
pub struct AnchorHasContent;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce that anchors have content and that the content is accessible to screen readers.
    /// Accessible means that it is not hidden using the `aria-hidden` prop.
    ///
    /// Alternatively, you may use the `title` prop or the `aria-label` prop.
    ///
    /// ### Why is this bad?
    ///
    ///
    /// ### Example
    ///
    /// #### good
    ///
    /// ```
    /// <a>Anchor Content!</a>
    ///  <a><TextWrapper /></a>
    ///  <a dangerouslySetInnerHTML={{ __html: 'foo' }} />
    ///  <a title='foo' />
    ///  <a aria-label='foo' />
    /// ```
    ///
    /// #### bad
    ///
    /// ```
    /// <a />
    /// <a><TextWrapper aria-hidden /></a>
    /// ```
    ///
    AnchorHasContent,
    correctness
);

impl Rule for AnchorHasContent {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::JSXElement(jsx_el) = node.kind() {
            let Some(name) = &get_element_type(ctx, &jsx_el.opening_element) else { return };
            if name == "a" {
                if is_hidden_from_screen_reader(ctx, &jsx_el.opening_element) {
                    ctx.diagnostic(AnchorHasContentDiagnostic::RemoveAriaHidden(jsx_el.span));
                    return;
                }

                if object_has_accessible_child(ctx, jsx_el) {
                    return;
                }

                for attr in ["title", "aria-label"] {
                    if has_jsx_prop_lowercase(&jsx_el.opening_element, attr).is_some() {
                        return;
                    };
                }

                ctx.diagnostic(AnchorHasContentDiagnostic::MissingContent(jsx_el.span));
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    // https://raw.githubusercontent.com/jsx-eslint/eslint-plugin-jsx-a11y/main/__tests__/src/rules/anchor-has-content-test.js
    let pass = vec![
        (r"<div />;", None, None),
        (r"<a>Foo</a>", None, None),
        (r"<a><Bar /></a>", None, None),
        (r"<a>{foo}</a>", None, None),
        (r"<a>{foo.bar}</a>", None, None),
        (r#"<a dangerouslySetInnerHTML={{ __html: "foo" }} />"#, None, None),
        (r"<a children={children} />", None, None),
        (r"<Link />", None, None),
        (
            r"<Link>foo</Link>",
            None,
            Some(serde_json::json!({ "jsx-a11y": { "components": { "Link": "a" } } })),
        ),
        (r"<a title={title} />", None, None),
        (r"<a aria-label={ariaLabel} />", None, None),
        (r"<a title={title} aria-label={ariaLabel} />", None, None),
    ];

    let fail = vec![
        (r"<a />", None, None),
        (r"<a><Bar aria-hidden /></a>", None, None),
        (r"<a>{undefined}</a>", None, None),
        (
            r"<Link />",
            None,
            Some(serde_json::json!({ "jsx-a11y": { "components": { "Link": "a" } } })),
        ),
    ];

    Tester::new(AnchorHasContent::NAME, pass, fail).test_and_snapshot();
}
