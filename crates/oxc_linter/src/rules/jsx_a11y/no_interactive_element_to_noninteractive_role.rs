use oxc_ast::{
    AstKind,
    ast::{JSXAttributeValue, JSXElementName},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    context::LintContext,
    globals::HTML_TAG,
    rule::Rule,
    utils::{
        get_element_type, has_jsx_prop, is_hidden_from_screen_reader, is_interactive_element,
        is_presentation_role,
    },
};

fn no_interactive_element_to_noninteractive_role_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Interactive elements should not be assigned non-interactive roles.")
        .with_help("Interactive elements like <button> or <a href> should not use roles like 'img', 'listitem', or 'presentation' that negate their interactive semantics. Remove the role or use a non-interactive element (like <div>).")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoInteractiveElementToNoninteractiveRole;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces that interactive elements are not assigned non-interactive roles.
    ///
    /// ### Why is this bad?
    ///
    /// Interactive elements indicate controls in the user interface. Assigning them non-interactive roles
    /// (like `img`, `listitem`, or `presentation`) causes assistive technologies to treat them as static content,
    /// making them inaccessible to users who rely on keyboard navigation or screen readers.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <button role="img" />
    /// <a href="https://google.com" role="listitem" />
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <button role="menuitem" />
    /// <a href="https://google.com" role="button" />
    /// <div role="img" />
    /// ```
    NoInteractiveElementToNoninteractiveRole,
    jsx_a11y,
    correctness
);

impl Rule for NoInteractiveElementToNoninteractiveRole {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(jsx_el) = node.kind() else {
            return;
        };

        let element_type = get_element_type(ctx, jsx_el);

        if !HTML_TAG.contains(element_type.as_ref()) {
            return;
        }

        if is_hidden_from_screen_reader(ctx, jsx_el) || is_presentation_role(jsx_el) {
            return;
        }

        if !is_interactive_element(&element_type, jsx_el) {
            return;
        }

        let Some(role_prop) = has_jsx_prop(jsx_el, "role") else {
            return;
        };

        let Some(role_value) = role_prop.as_attribute().and_then(|attr| {
            match &attr.value {
                Some(JSXAttributeValue::StringLiteral(lit)) => Some(lit.value.as_str()),
                _ => None,
            }
        }) else {
            return;
        };

        if crate::utils::NON_INTERACTIVE_ROLES.contains(&role_value) {
            let span = match &jsx_el.name {
                JSXElementName::Identifier(ident) => ident.span,
                JSXElementName::IdentifierReference(ident) => ident.span,
                JSXElementName::NamespacedName(ns) => ns.span,
                JSXElementName::MemberExpression(expr) => expr.span,
                JSXElementName::ThisExpression(expr) => expr.span,
            };
            ctx.diagnostic(no_interactive_element_to_noninteractive_role_diagnostic(span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (r"<button role='button' />", None),
        (r"<button role='menuitem' />", None),
        (r"<a href='foo' role='button' />", None),
        (r"<a href='foo' role='link' />", None),
        (r"<input type='text' role='textbox' />", None),
        (r"<div role='img' />", None),
        (r"<div role='listitem' />", None),
        (r"<div />", None),
        (r"<button />", None),
        (r"<a href='foo' />", None),
        (r"<input type='text' />", None),
        // presentation role is allowed as it is handled by is_presentation_role early return or similar
        (r"<button role='presentation' />", None),
        (r"<button role='none' />", None),
    ];

    let fail = vec![
        (r"<button role='img' />", None),
        (r"<button role='listitem' />", None),
        (r"<button role='article' />", None),
        (r"<a href='foo' role='img' />", None),
        (r"<a href='foo' role='listitem' />", None),
        (r"<input type='text' role='img' />", None),
        (r"<input type='checkbox' role='listitem' />", None),
    ];

    Tester::new(NoInteractiveElementToNoninteractiveRole::NAME, NoInteractiveElementToNoninteractiveRole::PLUGIN, pass, fail).test_and_snapshot();
}
