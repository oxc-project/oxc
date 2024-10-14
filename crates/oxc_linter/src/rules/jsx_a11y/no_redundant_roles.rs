use oxc_ast::{
    ast::{JSXAttributeItem, JSXAttributeValue},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use phf::phf_map;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{get_element_type, has_jsx_prop_ignore_case},
    AstNode,
};

fn no_redundant_roles_diagnostic(span: Span, element: &str, role: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "The element `{element}` has an implicit role of `{role}`. Defining this explicitly is redundant and should be avoided."
    ))
    .with_help(format!("Remove the redundant role `{role}` from the element `{element}`."))
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoRedundantRoles;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces that the explicit `role` property is not the same as
    /// implicit/default role property on element.
    ///
    /// ### Why is this bad?
    /// Redundant roles can lead to confusion and verbosity in the codebase.
    ///
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <nav role="navigation" />
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <nav />
    /// ```
    NoRedundantRoles,
    correctness,
    fix
);

static DEFAULT_ROLE_EXCEPTIONS: phf::Map<&'static str, &'static str> = phf_map! {
    "nav" => "navigation",
    "button" => "button",
    "body" => "document",
};

impl Rule for NoRedundantRoles {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(jsx_el) = node.kind() else {
            return;
        };
        let Some(component) = get_element_type(ctx, jsx_el) else {
            return;
        };

        if let Some(JSXAttributeItem::Attribute(attr)) = has_jsx_prop_ignore_case(jsx_el, "role") {
            if let Some(JSXAttributeValue::StringLiteral(role_values)) = &attr.value {
                let roles = role_values.value.split_whitespace().collect::<Vec<_>>();
                for role in &roles {
                    let exceptions = DEFAULT_ROLE_EXCEPTIONS.get(&component);
                    if exceptions.map_or(false, |set| set.contains(role)) {
                        ctx.diagnostic_with_fix(
                            no_redundant_roles_diagnostic(attr.span, &component, role),
                            |fixer| fixer.delete_range(attr.span),
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn test() {
    use crate::{rules::NoRedundantRoles, tester::Tester};

    fn settings() -> serde_json::Value {
        serde_json::json!({
            "settings": { "jsx-a11y": {
                "components": {
                    "Button": "button",
                }
            } }
        })
    }

    let pass = vec![
        ("<div />", None, None, None),
        ("<button role='main' />", None, None, None),
        ("<MyComponent role='button' />", None, None, None),
        ("<button role={`${foo}button`} />", None, None, None),
        ("<Button role={`${foo}button`} />", None, Some(settings()), None),
    ];

    let fail = vec![
        ("<button role='button' />", None, None, None),
        ("<body role='document' />", None, None, None),
        ("<Button role='button' />", None, Some(settings()), None),
    ];

    let fix = vec![
        ("<button role='button' />", "<button  />"),
        ("<body role='document' />", "<body  />"),
    ];

    Tester::new(NoRedundantRoles::NAME, pass, fail).expect_fix(fix).test_and_snapshot();
}
