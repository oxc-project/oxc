use oxc_ast::{
    AstKind,
    ast::{Argument, JSXElementName},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule, utils::is_create_element_call};

fn no_namespace_diagnostic(span: Span, component_name: &str) -> OxcDiagnostic {
    let message = format!(
        r"React component {component_name} must not be in a namespace, as React does not support them."
    );

    OxcDiagnostic::warn(message).with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoNamespace;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce that namespaces are not used in React elements.
    ///
    /// ### Why is this bad?
    ///
    /// Namespaces in React elements, such as svg:circle, are not supported by React.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <ns:TestComponent />
    /// <Ns:TestComponent />
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <TestComponent />
    /// <testComponent />
    /// ```
    NoNamespace,
    react,
    suspicious,
);

impl Rule for NoNamespace {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::JSXOpeningElement(element) => {
                if let JSXElementName::NamespacedName(namespaced_name) = &element.name {
                    let component_name_with_ns =
                        format!("{}:{}", namespaced_name.namespace.name, namespaced_name.name.name);

                    ctx.diagnostic(no_namespace_diagnostic(
                        namespaced_name.span,
                        &component_name_with_ns,
                    ));
                }
            }
            AstKind::CallExpression(call_expr) => {
                if is_create_element_call(call_expr) {
                    let Some(Argument::StringLiteral(str_lit)) = call_expr.arguments.first() else {
                        return;
                    };

                    if str_lit.value.contains(':') {
                        ctx.diagnostic(no_namespace_diagnostic(str_lit.span, &str_lit.value));
                    }
                }
            }
            _ => {}
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "<testcomponent />",
        r#"React.createElement("testcomponent")"#,
        "<testComponent />",
        r#"React.createElement("testComponent")"#,
        "<test_component />",
        r#"React.createElement("test_component")"#,
        "<TestComponent />",
        r#"React.createElement("TestComponent")"#,
        "<object.testcomponent />",
        r#"React.createElement("object.testcomponent")"#,
        "<object.testComponent />",
        r#"React.createElement("object.testComponent")"#,
        "<object.test_component />",
        r#"React.createElement("object.test_component")"#,
        "<object.TestComponent />",
        r#"React.createElement("object.TestComponent")"#,
        "<Object.testcomponent />",
        r#"React.createElement("Object.testcomponent")"#,
        "<Object.testComponent />",
        r#"React.createElement("Object.testComponent")"#,
        "<Object.test_component />",
        r#"React.createElement("Object.test_component")"#,
        "<Object.TestComponent />",
        r#"React.createElement("Object.TestComponent")"#,
        "React.createElement(null)",
        "React.createElement(true)",
        "React.createElement({})",
    ];

    let fail = vec![
        "<ns:testcomponent />",
        r#"React.createElement("ns:testcomponent")"#,
        "<ns:testComponent />",
        r#"React.createElement("ns:testComponent")"#,
        "<ns:test_component />",
        r#"React.createElement("ns:test_component")"#,
        "<ns:TestComponent />",
        r#"React.createElement("ns:TestComponent")"#,
        "<Ns:testcomponent />",
        r#"React.createElement("Ns:testcomponent")"#,
        "<Ns:testComponent />",
        r#"React.createElement("Ns:testComponent")"#,
        "<Ns:test_component />",
        r#"React.createElement("Ns:test_component")"#,
        "<Ns:TestComponent />",
        r#"React.createElement("Ns:TestComponent")"#,
    ];

    Tester::new(NoNamespace::NAME, NoNamespace::PLUGIN, pass, fail).test_and_snapshot();
}
