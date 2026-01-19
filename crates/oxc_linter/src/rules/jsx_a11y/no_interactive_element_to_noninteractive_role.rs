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

impl Rule for NoInteractiveElementToNoninteractiveRole {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(jsx_opening_el) = node.kind() else {
            return;
        };

        let element_type = get_element_type(ctx, jsx_opening_el);

        if !HTML_TAG.contains(element_type.as_ref()) {
            return;
        }

        if is_hidden_from_screen_reader(ctx, jsx_opening_el) || is_presentation_role(jsx_opening_el) {
            // If it's presentation role, it's effectively non-interactive, but checking if it WAS interactive...
            // If <button role="presentation">, that IS exactly what we want to catch?
            // "Interactive elements should not be assigned non-interactive roles."
            // 'presentation' IS a non-interactive role.
            // So if `is_presentation_role` is true, we might actually want to FLAG it if the element *was* interactive.
            
            // However, `is_presentation_role` checks if the role attribute IS 'presentation' or 'none'.
            // If we have <button role="presentation">, `is_interactive_element` might return false because the role overrides?
            // `is_interactive_element` checks semantics.
            // Let's check `is_interactive_element` details. 
            // It usually checks the tag name and attributes.
            
            // If <button role="presentation">, does `is_interactive_element` return true?
            // Usually `is_interactive_element` checks *native* interactivity or *role* interactivity.
            // If it's <button>, it IS natively interactive.
        }
        
        // Wait, `is_presentation_role` returning true means the user explicitly set role="presentation".
        // If the element *would have been* interactive (like <button>), then setting role="presentation" is bad (usually).
        // Actually, sometimes you DO want to remove semantics from a button? No, that's usually bad for accessibility if it's still clickable.
        // But if it's just visual... why use a button?
        
        // The rule says: "Interactive elements should not be assigned non-interactive roles."
        // Presentation is non-interactive.
        // So we should NOT return early here if we want to catch <button role="presentation">.
        
        // Let's look at `is_interactive_element`. It takes `element_type` and `jsx_opening_el`.
        // If `jsx_opening_el` has `role="presentation"`, does `is_interactive_element` account for that?
        // Likely yes. If role is presentation, it might be considered non-interactive.
        
        // We need to check if the *element itself* (tag) is natively interactive, regardless of the role.
        // We can force `is_interactive_element` to check only tag? 
        // Or we can just assume common interactive tags: button, input, select, textarea, option, a[href], area[href].
        
        // Let's use `is_interactive_element` but understand we might need to be careful.
        
        // Actually, we want to know if the element *is inherently interactive*.
        let is_natively_interactive = is_interactive_element(&element_type, jsx_opening_el);
        
        if !is_natively_interactive {
            return;
        }

        // It IS interactive. Now check if it has a non-interactive role.
        let role = has_jsx_prop(jsx_opening_el, "role");
        if let Some(role_attr) = role {
            if let Some(role_val) = crate::utils::get_string_literal_prop_value(role_attr) {
                 if crate::rules::jsx_a11y::no_static_element_interactions::NON_INTERACTIVE_ROLES.contains(&role_val) {
                     // Wait, `is_presentation_role` checks for "presentation" or "none".
                     // "presentation" is in NON_INTERACTIVE_ROLES?
                     // Let's check `no_static_element_interactions.rs` again.
                     // The list in step 1669 (lines 114-158) does NOT seem to include "presentation" or "none".
                     // It has "article", "img", "list", etc.
                     
                     // We should ALSO check for "presentation" and "none" explicitly if they aren't in that list.
                     // Or check `is_presentation_role`.
                     
                     // If I use `is_presentation_role(jsx_opening_el)` and it's true, AND it's a native interactive element, that's a violation.
                     
                     ctx.diagnostic(no_interactive_element_to_noninteractive_role_diagnostic(jsx_opening_el.span));
                 } else if role_val == "presentation" || role_val == "none" {
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
