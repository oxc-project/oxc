use crate::{context::LintContext, rule::Rule, utils::has_jsx_prop_lowercase, AstNode};
use oxc_ast::{
    ast::{JSXAttributeItem, JSXAttributeValue, JSXElementName},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{self, Error},
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use phf::{phf_map, phf_set};
use std::collections::HashMap;

#[derive(Debug, Error, Diagnostic)]
#[error(
    "eslint-plugin-jsx-a11y(no-redundant-roles): The element `{element}` has an implicit role of `{role}`. Defining this explicitly is redundant and should be avoided."
)]
#[diagnostic(
    severity(warning),
    help("Remove the redundant role `{role}` from the element `{element}`.")
)]
struct NoRedundantRolesDiagnostic {
    #[label]
    pub span: Span,
    pub element: String,
    pub role: String,
}

#[derive(Debug, Default, Clone)]
pub struct NoRedundantRoles;

declare_oxc_lint!(
    /// ### What it does
    /// Enforces that the explicit role property is not the same as implicit/default role property on element.
    ///
    /// ### Why is this bad?
    /// Redundant roles can lead to confusion and verbosity in the codebase.
    ///
    /// ### Example
    /// ```javascript
    /// // Bad
    /// <nav role="navigation" />
    ///
    /// // Good
    /// <nav />
    /// ```
    NoRedundantRoles,
    correctness
);

static DEFAULT_ROLE_EXCEPTIONS: phf::Map<&'static str, phf::Set<&'static str>> = phf_map! {
    "nav" => phf_set!{"navigation"},
    "button" => phf_set!{"button"},
    "body" => phf_set!{"document"},
};

static EMPTY_SET: phf::Set<&'static str> = phf_set! {};

impl Rule for NoRedundantRoles {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let components: HashMap<String, String> = ctx
            .settings()
            .jsx_a11y
            .components
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect();

        if let AstKind::JSXOpeningElement(jsx_el) = node.kind() {
            let element = match &jsx_el.name {
                JSXElementName::Identifier(identifier) => identifier.name.to_string(),
                JSXElementName::NamespacedName(namespaced_name) => {
                    namespaced_name.property.name.to_string()
                }
                JSXElementName::MemberExpression(_) => return,
            };

            let component = components.get(&element).unwrap_or(&element);

            if let Some(JSXAttributeItem::Attribute(attr)) = has_jsx_prop_lowercase(jsx_el, "role")
            {
                if let Some(JSXAttributeValue::StringLiteral(role_values)) = &attr.value {
                    let roles: Vec<String> = role_values
                        .value
                        .split_whitespace()
                        .map(std::string::ToString::to_string)
                        .collect();
                    for role in &roles {
                        let exceptions =
                            DEFAULT_ROLE_EXCEPTIONS.get(component).unwrap_or(&EMPTY_SET);
                        if exceptions.contains(role) {
                            ctx.diagnostic(NoRedundantRolesDiagnostic {
                                span: attr.span,
                                element: component.clone(),
                                role: role.to_string(),
                            });
                        }
                    }
                }
            }
        }
    }
}
#[test]
fn test() {
    use crate::rules::NoRedundantRoles;
    use crate::tester::Tester;

    fn settings() -> serde_json::Value {
        serde_json::json!({
            "jsx-a11y": {
                "components": {
                    "Button": "button",
                }
            }
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

    Tester::new_with_settings(NoRedundantRoles::NAME, pass, fail).test_and_snapshot();
}
