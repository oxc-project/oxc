use oxc_ast::{AstKind, ast::JSXAttributeItem};
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

fn interactive_supports_focus_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Elements with interactive roles must be focusable.")
        .with_help("Interactive elements must be able to receive focus. Add a valid `tabIndex` attribute.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct InteractiveSupportsFocus;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces that elements with interactive roles are focusable.
    ///
    /// ### Why is this bad?
    ///
    /// Interactive elements that are not focusable cannot be accessed by keyboard users,
    /// making them inaccessible to users with disabilities who rely on keyboard navigation.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <div role="button" onClick={() => {}} />
    /// <span role="checkbox" aria-checked="false" onClick={() => {}} />
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <div role="button" onClick={() => {}} tabIndex="0" />
    /// <button onClick={() => {}} />
    /// <input type="text" />
    /// ```
    InteractiveSupportsFocus,
    jsx_a11y,
    correctness
);

impl Rule for InteractiveSupportsFocus {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(jsx_opening_el) = node.kind() else {
            return;
        };

        let element_type = get_element_type(ctx, jsx_opening_el);

        // Logic adapted from eslint-plugin-jsx-a11y/interactive-supports-focus
        
        // 1. If it's not a DOM element, return
        if !HTML_TAG.contains(element_type.as_ref()) {
            return;
        }

        // 2. If it's hidden or presentation, return
        if is_hidden_from_screen_reader(ctx, jsx_opening_el) || is_presentation_role(jsx_opening_el) {
            return;
        }

        // 3. Check if it is interactive
        // The rule only cares if the element is interactive.
        // If it IS interactive, it MUST be focusable.
        // Wait, `is_interactive_element` checks if the *semantics* are interactive (e.g. role=button, or <button>).
        // ESLint logic:
        // - Get roles (implicit + explicit)
        // - If explicit role is interactive, check focusable.
        // - If semantic element is interactive, check focusable.
        
        // OxC's `is_interactive_element` seems to cover native elements + some roles?
        // Let's check `is_interactive_element` implementation again. 
        // It checks: button, details, embed, iframe, label, select, textarea, input(!hidden), a(href), audio(controls), video(controls), img(usemap).
        // It does NOT seem to check `role="button"` on a `div`.
        // We need to check roles manually.

        let is_native_interactive = is_interactive_element(&element_type, jsx_opening_el);
        
        // Note: We need to handle `role="button"` etc. logic similar to ESLint.
        // For now, let's look at `no_static_element_interactions` which specifically handles roles.
        // But `interactive-supports-focus` logic is:
        // "Elements with interactive roles must be focusable."
        
        // If the element is natively interactive (like <button>), it IS focusable by default (unless disabled, etc, but typically yes).
        // So this rule mostly catches:
        // - Non-interactive elements (div, span) given an interactive role.
        // - Native interactive elements that are somehow made non-focusable (maybe, but less common).
        
        // Let's check if the element has an interactive role.
        let role = has_jsx_prop(jsx_opening_el, "role");
        let has_interactive_role = if let Some(role_attr) = role {
            if let Some(role_val) = crate::utils::get_string_literal_prop_value(role_attr) {
                 // List from no_static_element_interactions.rs
                 crate::rules::jsx_a11y::no_static_element_interactions::INTERACTIVE_ROLES.contains(&role_val)
            } else {
                false
            }
        } else {
            false
        };

        // If it's not interactive (neither native nor role), we don't care.
        if !is_native_interactive && !has_interactive_role {
            return;
        }

        // 4. Check if focusable.
        // An element is focusable if:
        // - It is natively focusable (<button>, <input>, <a href>, etc.)
        // - OR it has a tabIndex (>= -1 ? No, usually just having the attribute is enough to be focusable programmatically, but "tabbable" usually means >= 0. The rule says "associating a key handler... need to be focusable". 
        // ESLint says: "Elements with interactive roles must be focusable."
        // Focusable means `tabIndex` is present (even -1 makes it focusable programmatically, 0 makes it tabbable).
        
        // If it is natively interactive, it is focusable.
        // Exception: <a without href>.
        if is_native_interactive {
             // Are there cases where a native interactive element is NOT focusable?
             // Maybe disabled? But usually they are still compatible with the rule.
             // The main target is div role="button".
             return;
        }

        // If we are here, it has an interactive role, but is NOT a native interactive element.
        // E.g. <div role="button">
        
        // We must ensure it has a tabIndex.
        // Check for `tabIndex`.
        if has_jsx_prop(jsx_opening_el, "tabIndex").is_none() {
             ctx.diagnostic(interactive_supports_focus_diagnostic(jsx_opening_el.span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"<div role="button" tabIndex="0" />"#,
        r#"<div role="checkbox" tabIndex="-1" />"#,
        r#"<button />"#,
        r#"<input type="text" />"#,
        r#"<a href="foo" />"#,
        r#"<div />"#,
        r#"<div role="presentation" />"#,
         // Custom components ignored
        r#"<MyButton />"#,
    ];

    let fail = vec![
        r#"<div role="button" />"#,
        r#"<div role="checkbox" />"#,
        r#"<div role="link" />"#,
        r#"<span role="slider" />"#,
        r#"<div role="button" tabIndex={undefined} />"#, // effectively missing if undefined
    ];

    Tester::new(InteractiveSupportsFocus::NAME, InteractiveSupportsFocus::PLUGIN, pass, fail).test_and_snapshot();
}
