use oxc_ast::AstKind;
use oxc_ast::ast::{JSXAttributeItem, JSXAttributeValue, JSXElementName};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule, utils::has_jsx_prop_ignore_case};

fn use_focusable_interactive_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Interactive elements must be focusable.")
        .with_help("Add a `tabIndex` attribute to make this interactive element focusable, or use a natively focusable element.")
        .with_label(span)
}

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
    "textbox",
    "tree",
    "treegrid",
    "treeitem",
];

const NATIVELY_FOCUSABLE: &[&str] =
    &["a", "button", "input", "select", "textarea", "details", "summary"];

#[derive(Debug, Default, Clone)]
pub struct UseFocusableInteractive;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Ensures that elements with interactive ARIA roles are focusable.
    ///
    /// ### Why is this bad?
    ///
    /// Elements with interactive roles must be focusable for keyboard
    /// navigation. If a non-focusable element has an interactive role,
    /// keyboard users cannot interact with it.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <div role="button" />
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <div role="button" tabIndex={0} />
    /// <button />
    /// ```
    UseFocusableInteractive,
    jsx_a11y,
    correctness,
    pending
);

impl Rule for UseFocusableInteractive {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(jsx_el) = node.kind() else {
            return;
        };

        let element_name = match &jsx_el.name {
            JSXElementName::Identifier(id) => id.name.as_str(),
            JSXElementName::IdentifierReference(id) => id.name.as_str(),
            _ => return,
        };

        // Natively focusable elements are fine
        if NATIVELY_FOCUSABLE.contains(&element_name) {
            return;
        }

        // Check if it has an interactive role
        let Some(JSXAttributeItem::Attribute(attr)) = has_jsx_prop_ignore_case(jsx_el, "role")
        else {
            return;
        };

        let Some(JSXAttributeValue::StringLiteral(role_value)) = &attr.value else {
            return;
        };

        let role = role_value.value.as_str().to_lowercase();
        if !INTERACTIVE_ROLES.contains(&role.as_str()) {
            return;
        }

        // Check if tabIndex is set
        if has_jsx_prop_ignore_case(jsx_el, "tabIndex").is_some()
            || has_jsx_prop_ignore_case(jsx_el, "tabindex").is_some()
        {
            return;
        }

        ctx.diagnostic(use_focusable_interactive_diagnostic(jsx_el.span));
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "<div role=\"button\" tabIndex={0} />",
        "<button />",
        "<div />",
        "<div role=\"article\" />",
        "<a href=\"#\" role=\"button\" />",
    ];

    let fail = vec!["<div role=\"button\" />", "<span role=\"link\" />"];

    Tester::new(UseFocusableInteractive::NAME, UseFocusableInteractive::PLUGIN, pass, fail)
        .test_and_snapshot();
}
