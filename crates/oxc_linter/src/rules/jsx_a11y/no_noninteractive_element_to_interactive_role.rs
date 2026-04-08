use oxc_ast::AstKind;
use oxc_ast::ast::{JSXAttributeItem, JSXAttributeValue, JSXElementName};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule, utils::has_jsx_prop_ignore_case};

fn diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Non-interactive element should not be assigned an interactive role.")
        .with_help("Use an interactive HTML element instead of adding an interactive role to a non-interactive element.")
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

const INTERACTIVE_ROLES: &[&str] = &[
    "button",
    "checkbox",
    "combobox",
    "grid",
    "gridcell",
    "link",
    "listbox",
    "menu",
    "menubar",
    "menuitem",
    "menuitemcheckbox",
    "menuitemradio",
    "option",
    "progressbar",
    "radio",
    "radiogroup",
    "scrollbar",
    "searchbox",
    "slider",
    "spinbutton",
    "switch",
    "tab",
    "tablist",
    "tabpanel",
    "textbox",
    "tree",
    "treegrid",
    "treeitem",
];

#[derive(Debug, Default, Clone)]
pub struct NoNoninteractiveElementToInteractiveRole;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows assigning interactive ARIA roles to non-interactive HTML elements.
    ///
    /// ### Why is this bad?
    ///
    /// Non-interactive HTML elements (like `<div>`) with interactive roles
    /// (like `button`) lack the built-in accessibility features of native
    /// interactive elements. Use `<button>` instead of `<div role="button">`.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <div role="button" />
    /// <span role="link" />
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <button />
    /// <a href="#" />
    /// <div role="article" />
    /// ```
    NoNoninteractiveElementToInteractiveRole,
    jsx_a11y,
    correctness,
    pending
);

impl Rule for NoNoninteractiveElementToInteractiveRole {
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

        let Some(JSXAttributeItem::Attribute(attr)) = has_jsx_prop_ignore_case(jsx_el, "role")
        else {
            return;
        };

        let Some(JSXAttributeValue::StringLiteral(role_value)) = &attr.value else {
            return;
        };

        let role = role_value.value.as_str().to_lowercase();
        if INTERACTIVE_ROLES.contains(&role.as_str()) {
            ctx.diagnostic(diagnostic(attr.span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec!["<button />", "<a href=\"#\" />", "<div role=\"article\" />", "<div />"];

    let fail = vec!["<div role=\"button\" />", "<span role=\"link\" />", "<p role=\"checkbox\" />"];

    Tester::new(
        NoNoninteractiveElementToInteractiveRole::NAME,
        NoNoninteractiveElementToInteractiveRole::PLUGIN,
        pass,
        fail,
    )
    .test_and_snapshot();
}
