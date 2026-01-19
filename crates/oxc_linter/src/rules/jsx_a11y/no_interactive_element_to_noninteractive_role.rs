use oxc_ast::AstKind;
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
        NON_INTERACTIVE_ROLES,
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
    /// <button role="img" onClick={() => {}} />
    /// <a href="#" role="listitem" />
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <button onClick={() => {}} />
    /// <div role="img" onClick={() => {}} />
    /// <a href="#" role="button" />
    /// ```
    NoInteractiveElementToNoninteractiveRole,
    jsx_a11y,
    correctness
);

impl crate::rule::RuleRunner for NoInteractiveElementToNoninteractiveRole {
    const NODE_TYPES: Option<&'static oxc_semantic::AstTypesBitset> = None;
}

impl Rule for NoInteractiveElementToNoninteractiveRole {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(jsx_opening_el) = node.kind() else {
            return;
        };

        let element_type = get_element_type(ctx, jsx_opening_el);

        if !HTML_TAG.contains(element_type.as_ref()) {
            return;
        }

        if is_hidden_from_screen_reader(ctx, jsx_opening_el) {
            return;
        }

        let is_natively_interactive = is_interactive_element(&element_type, jsx_opening_el);

        if !is_natively_interactive {
            return;
        }

        // Report when a natively interactive element is given a non-interactive or presentation role.
        let role = has_jsx_prop(jsx_opening_el, "role");
        if let Some(role_attr) = role {
            if let Some(role_val) = crate::utils::get_string_literal_prop_value(role_attr) {
                 if NON_INTERACTIVE_ROLES.contains(&role_val)
                    || role_val == "presentation" 
                    || role_val == "none" 
                 {
                     ctx.diagnostic(no_interactive_element_to_noninteractive_role_diagnostic(jsx_opening_el.span));
                 }
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"<button onClick={() => {}} />"#,
        r#"<div role="button" onClick={() => {}} />"#,
        r#"<div role="img" onClick={() => {}} />"#,
        r#"<a href="#" role="button" />"#,
        r#"<a href="#" role="link" />"#,
        r#"<input type="text" />"#,
    ];

    let fail = vec![
        r#"<button role="img" onClick={() => {}} />"#,
        r#"<button role="presentation" />"#,
        r#"<button role="none" />"#,
        r#"<a href="#" role="listitem" />"#,
        r#"<a href="#" role="img" />"#,
        r#"<input type="text" role="note" />"#,
    ];

    Tester::new(NoInteractiveElementToNoninteractiveRole::NAME, NoInteractiveElementToNoninteractiveRole::PLUGIN, pass, fail).test_and_snapshot();
}
