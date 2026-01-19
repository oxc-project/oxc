use oxc_ast::{
    AstKind,
    ast::{Expression, JSXAttributeItem, JSXAttributeValue},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    context::LintContext,
    globals::HTML_TAG,
    rule::Rule,
    utils::{
        get_element_type, get_string_literal_prop_value, has_jsx_prop,
        is_hidden_from_screen_reader, is_interactive_element, is_presentation_role,
    },
};

fn interactive_supports_focus_diagnostic(span: Span, role: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Elements with the '{role}' interactive role must be focusable."))
        .with_help("Interactive elements must be able to receive focus. In JSX, add a valid tabIndex prop.")
        .with_label(span)
}

fn interactive_supports_focus_non_interactive_diagnostic(span: Span, element: &str, role: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("The '{element}' element with the '{role}' interactive role must be focusable."))
        .with_help("Interactive elements must be able to receive focus. In JSX, add a valid tabIndex prop.")
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
    correctness,
    fix = pending
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

        let has_interactive_props = INTERACTIVE_PROPS
            .iter()
            .any(|prop| has_jsx_prop(jsx_opening_el, prop).is_some());

        if !has_interactive_props {
            return;
        }

        if has_jsx_prop(jsx_opening_el, "disabled").is_some()
            || has_jsx_prop(jsx_opening_el, "aria-disabled").is_some_and(|attr| {
                get_string_literal_prop_value(attr).is_some_and(|val| val == "true")
            })
        {
            return;
        }

        if is_interactive_element(&element_type, jsx_opening_el) {
            return;
        }

        let role = has_jsx_prop(jsx_opening_el, "role");
        let Some(role_attr) = role else {
            return;
        };
        
        let role_val = get_string_literal_prop_value(role_attr);
        let is_interactive = if let Some(val) = role_val {
             crate::utils::is_interactive_role(val)
        } else {
             false
        };

        if !is_interactive {
            return;
        }

        // Check for `tabIndex`.
        match has_jsx_prop(jsx_opening_el, "tabIndex") {
            Some(JSXAttributeItem::Attribute(attr)) => {
                 match &attr.value {
                     Some(JSXAttributeValue::StringLiteral(s)) => {
                         if s.value.parse::<i32>().is_err() {
                              if element_type == "div" || element_type == "span" {
                                  ctx.diagnostic(interactive_supports_focus_diagnostic(role_attr.span(), role_val.unwrap()));
                              } else {
                                  ctx.diagnostic(interactive_supports_focus_non_interactive_diagnostic(role_attr.span(), &element_type, role_val.unwrap()));
                              }
                         }
                     }
                     Some(JSXAttributeValue::ExpressionContainer(container)) => {
                         if let Some(expr) = container.expression.as_expression() {
                                 match expr {
                                     Expression::NumericLiteral(_) => {}
                                     Expression::UnaryExpression(unary) => {
                                         if let Expression::NumericLiteral(_) = &unary.argument {
                                              // Valid
                                         } else {
                                              // Unknown, assume valid
                                         }
                                     }
                                      Expression::Identifier(id) if id.name == "undefined" => {
                                          if element_type == "div" || element_type == "span" {
                                              ctx.diagnostic(interactive_supports_focus_diagnostic(role_attr.span(), role_val.unwrap()));
                                          } else {
                                              ctx.diagnostic(interactive_supports_focus_non_interactive_diagnostic(role_attr.span(), &element_type, role_val.unwrap()));
                                          }
                                      }
                                     _ => {}
                                 }
                         }
                     }
                     _ => {} 
                 }
            }
            Some(JSXAttributeItem::SpreadAttribute(_)) => {}
            None => {
                if element_type == "div" || element_type == "span" {
                    ctx.diagnostic(interactive_supports_focus_diagnostic(role_attr.span(), role_val.unwrap()));
                } else {
                    ctx.diagnostic(interactive_supports_focus_non_interactive_diagnostic(role_attr.span(), &element_type, role_val.unwrap()));
                }
            }
        }
    }
}

const INTERACTIVE_PROPS: [&str; 6] = [
    "onClick",
    "onMouseDown",
    "onMouseUp",
    "onKeyPress",
    "onKeyDown",
    "onKeyUp",
];

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"<div role="button" onClick={() => {}} tabIndex="0" />"#,
        r#"<div role="checkbox" onClick={() => {}} tabIndex="-1" />"#,
        r#"<button onClick={() => {}} />"#,
        r#"<input type="text" onClick={() => {}} />"#,
        r#"<a href="foo" onClick={() => {}} />"#,
        r#"<div />"#,
        r#"<div role="presentation" />"#,
        r#"<div role="button" onClick={() => {}} tabIndex={0} />"#,
        r#"<MyButton onClick={() => {}} />"#,
        r#"<div role="button" />"#, // Valid because no handler
        r#"<div role="button" onClick={() => {}} aria-disabled="true" />"#, // Valid becuse disabled
        r#"<div role="button" onClick={() => {}} disabled />"#, // Valid becuse disabled check
    ];

    let fail = vec![
        r#"<div role="button" onClick={() => {}} />"#,
        r#"<div role="checkbox" onClick={() => {}} />"#,
        r#"<div role="link" onClick={() => {}} />"#,
        r#"<span role="slider" onClick={() => {}} />"#,
        r#"<div role="button" onClick={() => {}} tabIndex={undefined} />"#,
        r#"<section role="button" onClick={() => {}} />"#,
        r#"<main role="button" onClick={() => {}} />"#,
        r#"<article role="button" onClick={() => {}} />"#,
        r#"<header role="button" onClick={() => {}} />"#,
        r#"<footer role="button" onClick={() => {}} />"#,
    ];

    Tester::new(InteractiveSupportsFocus::NAME, InteractiveSupportsFocus::PLUGIN, pass, fail).test_and_snapshot();
}
