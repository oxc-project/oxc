use oxc_ast::{
    ast::{
        Expression, JSXAttributeItem, JSXAttributeValue, JSXChild, JSXElement, JSXElementName,
        JSXExpression,
    },
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use oxc_allocator::Vec;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{get_element_type, get_prop_value, has_jsx_prop_lowercase},
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
                // check self attr
                if has_jsx_prop_lowercase(&jsx_el.opening_element, "aria-hidden").is_some() {
                    ctx.diagnostic(AnchorHasContentDiagnostic::RemoveAriaHidden(jsx_el.span));
                    return;
                }

                // check if self attr has title/aria-label
                if (has_jsx_prop_lowercase(&jsx_el.opening_element, "title").is_some()
                    || has_jsx_prop_lowercase(&jsx_el.opening_element, "aria-label").is_some()
                    || has_jsx_prop_lowercase(&jsx_el.opening_element, "children").is_some()
                    || has_jsx_prop_lowercase(&jsx_el.opening_element, "dangerouslysetinnerhtml")
                        .is_some())
                    && match_valid_prop(&jsx_el.opening_element.attributes)
                {
                    // pass
                    return;
                }

                // check content accessible
                check_has_accessible_child(jsx_el, ctx);
            }
        }

        // custom component
    }
}

fn match_valid_prop(attr_items: &Vec<JSXAttributeItem>) -> bool {
    attr_items
        .into_iter()
        .any(|attr| matches!(get_prop_value(attr), Some(JSXAttributeValue::ExpressionContainer(_))))
}

fn check_has_accessible_child(jsx: &JSXElement, ctx: &LintContext) {
    let children = &jsx.children;
    if children.len() == 0 {
        if let JSXElementName::Identifier(ident) = &jsx.opening_element.name {
            ctx.diagnostic(AnchorHasContentDiagnostic::MissingContent(ident.span));
            return;
        }
    }

    // If each child is inaccessible, an error is reported
    let mut diagnostic = AnchorHasContentDiagnostic::MissingContent(jsx.span);
    let all_not_has_content = children.into_iter().all(|child| match child {
        JSXChild::Text(text) => {
            if text.value.trim() == "" {
                return true;
            }
            false
        }
        JSXChild::ExpressionContainer(exp) => {
            if let JSXExpression::Expression(jsexp) = &exp.expression {
                if let Expression::Identifier(ident) = jsexp {
                    if ident.name == "undefined" {
                        return true;
                    }
                } else if let Expression::NullLiteral(_) = jsexp {
                    return true;
                }
            };
            false
        }
        JSXChild::Element(ele) => {
            let is_hidden = has_jsx_prop_lowercase(&ele.opening_element, "aria-hidden").is_some();
            if is_hidden {
                diagnostic = AnchorHasContentDiagnostic::RemoveAriaHidden(jsx.span);
                return true;
            }
            false
        }
        _ => false,
    });

    if all_not_has_content {
        ctx.diagnostic(diagnostic);
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
        (
            r"<Link>foo</Link>",
            None,
            Some(serde_json::json!({ "jsx-a11y": { "components": { "Link": "a" } } })),
        ),
        (r"<a title={title} />", None, None),
        (r"<a aria-label={ariaLabel} />", None, None),
        (r"<a title={title} aria-label={ariaLabel} />", None, None),
        (r"<a><Bar aria-hidden />Foo</a>", None, None),
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
        (r"<a aria-hidden ></a>", None, None),
        (r"<a>{null}</a>", None, None),
        (r"<a title />", None, None),
    ];

    Tester::new(AnchorHasContent::NAME, pass, fail).test_and_snapshot();
}
