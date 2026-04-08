use oxc_ast::AstKind;
use oxc_ast::ast::JSXElementName;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule, utils::has_jsx_prop_ignore_case};

fn diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Non-interactive element should not have event handlers without a role.")
        .with_help(
            "Add an appropriate ARIA role to the element, or use an interactive element instead.",
        )
        .with_label(span)
}

const NON_INTERACTIVE_ELEMENTS: &[&str] = &[
    "article",
    "aside",
    "blockquote",
    "dd",
    "div",
    "dl",
    "dt",
    "fieldset",
    "figcaption",
    "figure",
    "footer",
    "form",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "header",
    "hr",
    "li",
    "main",
    "nav",
    "ol",
    "p",
    "pre",
    "section",
    "span",
    "table",
    "tbody",
    "td",
    "tfoot",
    "th",
    "thead",
    "tr",
    "ul",
];

const INTERACTIVE_HANDLERS: &[&str] =
    &["onClick", "onKeyDown", "onKeyUp", "onKeyPress", "onMouseDown", "onMouseUp"];

#[derive(Debug, Default, Clone)]
pub struct NoNoninteractiveElementInteractions;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows event handlers on non-interactive HTML elements without an
    /// appropriate ARIA role.
    ///
    /// ### Why is this bad?
    ///
    /// Non-interactive HTML elements like `<div>` and `<span>` do not have
    /// built-in interaction behaviors. Adding event handlers without proper
    /// ARIA roles makes the element inaccessible to keyboard and screen
    /// reader users.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <div onClick={() => {}} />
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <button onClick={() => {}} />
    /// <div role="button" onClick={() => {}} />
    /// ```
    NoNoninteractiveElementInteractions,
    jsx_a11y,
    correctness,
    pending
);

impl Rule for NoNoninteractiveElementInteractions {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(jsx_el) = node.kind() else {
            return;
        };

        let element_name = match &jsx_el.name {
            JSXElementName::Identifier(id) => id.name.as_str(),
            JSXElementName::IdentifierReference(id) => id.name.as_str(),
            _ => return,
        };

        if !NON_INTERACTIVE_ELEMENTS.contains(&element_name) {
            return;
        }

        // If element has a role, it's fine
        if has_jsx_prop_ignore_case(jsx_el, "role").is_some() {
            return;
        }

        // Check for interactive event handlers
        for handler in INTERACTIVE_HANDLERS {
            if let Some(prop) = has_jsx_prop_ignore_case(jsx_el, handler) {
                if let oxc_ast::ast::JSXAttributeItem::Attribute(attr) = prop {
                    ctx.diagnostic(diagnostic(attr.span));
                    return;
                }
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "<button onClick={() => {}} />",
        "<div role=\"button\" onClick={() => {}} />",
        "<div />",
        "<input onClick={() => {}} />",
    ];

    let fail = vec!["<div onClick={() => {}} />", "<span onClick={() => {}} />"];

    Tester::new(
        NoNoninteractiveElementInteractions::NAME,
        NoNoninteractiveElementInteractions::PLUGIN,
        pass,
        fail,
    )
    .test_and_snapshot();
}
