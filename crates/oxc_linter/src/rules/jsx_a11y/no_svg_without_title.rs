use oxc_ast::AstKind;
use oxc_ast::ast::{JSXChild, JSXElementName};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule, utils::has_jsx_prop_ignore_case};

fn no_svg_without_title_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("SVG elements must have a `<title>` element or an accessible label.")
        .with_help(
            "Add a `<title>` child element, or add an `aria-label` or `aria-labelledby` attribute.",
        )
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoSvgWithoutTitle;

declare_oxc_lint!(
    /// ### What it does
    /// Enforces that `<svg>` elements have an accessible title.
    ///
    /// ### Why is this bad
    /// SVG images without accessible text are invisible to screen readers.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <svg><circle /></svg>
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <svg><title>Icon</title><circle /></svg>
    /// <svg aria-label="Icon"><circle /></svg>
    /// ```
    NoSvgWithoutTitle,
    jsx_a11y,
    correctness,
    pending
);

fn get_element_name<'a>(name: &'a JSXElementName<'a>) -> Option<&'a str> {
    match name {
        JSXElementName::Identifier(id) => Some(id.name.as_str()),
        JSXElementName::IdentifierReference(id) => Some(id.name.as_str()),
        _ => None,
    }
}

impl Rule for NoSvgWithoutTitle {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(jsx_el) = node.kind() else {
            return;
        };

        if get_element_name(&jsx_el.name) != Some("svg") {
            return;
        }

        if has_jsx_prop_ignore_case(jsx_el, "aria-label").is_some()
            || has_jsx_prop_ignore_case(jsx_el, "aria-labelledby").is_some()
            || has_jsx_prop_ignore_case(jsx_el, "role").is_some()
        {
            return;
        }

        let parent = ctx.nodes().parent_node(node.id());
        if let AstKind::JSXElement(element) = parent.kind() {
            for child in &element.children {
                if let JSXChild::Element(child_el) = child {
                    if get_element_name(&child_el.opening_element.name) == Some("title") {
                        return;
                    }
                }
            }
        }

        ctx.diagnostic(no_svg_without_title_diagnostic(jsx_el.span));
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "<svg><title>Icon</title><circle /></svg>",
        r#"<svg aria-label="Icon"><circle /></svg>"#,
        r#"<svg aria-labelledby="title-id"><circle /></svg>"#,
        r#"<svg role="img"><circle /></svg>"#,
    ];

    let fail = vec!["<svg><circle /></svg>"];

    Tester::new(NoSvgWithoutTitle::NAME, NoSvgWithoutTitle::PLUGIN, pass, fail).test_and_snapshot();
}
