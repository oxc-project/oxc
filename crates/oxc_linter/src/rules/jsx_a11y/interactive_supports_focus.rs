use oxc_ast::{
    AstKind,
    ast::{Expression, JSXAttributeItem, JSXAttributeValue, JSXExpression},
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

fn interactive_supports_focus_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Elements with interactive roles must be focusable.")
        .with_help("Interactive elements must be able to receive focus. In JSX, add a valid `tabIndex` prop.")
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

        if !HTML_TAG.contains(element_type.as_ref()) {
            return;
        }

        if is_hidden_from_screen_reader(ctx, jsx_opening_el) || is_presentation_role(jsx_opening_el) {
            return;
        }

        let is_native_interactive = is_interactive_element(&element_type, jsx_opening_el);
        
        // Check if the element has an interactive role.
        let role = has_jsx_prop(jsx_opening_el, "role");
        let has_interactive_role = if let Some(role_attr) = role {
            if let Some(role_val) = crate::utils::get_string_literal_prop_value(role_attr) {
                 crate::rules::jsx_a11y::no_static_element_interactions::INTERACTIVE_ROLES.contains(&role_val)
            } else {
                false
            }
        } else {
            false
        };

        if !is_native_interactive && !has_interactive_role {
            return;
        }

        if is_native_interactive {
             return;
        }

        // Check for `tabIndex`.
        match has_jsx_prop(jsx_opening_el, "tabIndex") {
            Some(JSXAttributeItem::Attribute(attr)) => {
                 if let Some(JSXAttributeValue::ExpressionContainer(container)) = &attr.value {
                     if let JSXExpression::Expression(expr) = &container.expression {
                         if let Expression::Identifier(id) = expr {
                             if id.name == "undefined" {
                                 ctx.diagnostic(interactive_supports_focus_diagnostic(jsx_opening_el.span));
                             }
                         }
                     }
                 }
            }
            Some(JSXAttributeItem::SpreadAttribute(_)) => {}
            None => {
                 ctx.diagnostic(interactive_supports_focus_diagnostic(jsx_opening_el.span));
            }
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
        r#"<div role="button" tabIndex={0} />"#,
        r#"<MyButton />"#,
    ];

    let fail = vec![
        r#"<div role="button" />"#,
        r#"<div role="checkbox" />"#,
        r#"<div role="link" />"#,
        r#"<span role="slider" />"#,
        r#"<div role="button" tabIndex={undefined} />"#,
    ];

    Tester::new(InteractiveSupportsFocus::NAME, InteractiveSupportsFocus::PLUGIN, pass, fail).test_and_snapshot();
}
