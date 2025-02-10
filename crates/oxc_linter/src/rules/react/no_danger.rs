use oxc_ast::{
    ast::{Argument, JSXAttributeItem, ObjectPropertyKind},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    utils::{has_jsx_prop, is_create_element_call},
    AstNode,
};

fn no_danger_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not use `dangerouslySetInnerHTML` prop")
        .with_help("`dangerouslySetInnerHTML` is a way to inject HTML into your React component. This is dangerous because it can easily lead to XSS vulnerabilities.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoDanger;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule prevents the use of `dangerouslySetInnerHTML` prop.
    ///
    /// ### Why is this bad?
    ///
    /// `dangerouslySetInnerHTML` is a way to inject HTML into your React
    /// component. This is dangerous because it can easily lead to XSS
    /// vulnerabilities.
    ///
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule:
    ///
    /// ```jsx
    /// import React from "react";
    ///
    /// const Hello = <div dangerouslySetInnerHTML={{ __html: "Hello World" }}></div>;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    ///
    /// ```jsx
    /// import React from "react";
    ///
    /// const Hello = <div>Hello World</div>;
    /// ```
    NoDanger,
    react,
    restriction
);

impl Rule for NoDanger {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::JSXElement(jsx_elem) => {
                if let Some(JSXAttributeItem::Attribute(prop)) =
                    has_jsx_prop(&jsx_elem.opening_element, "dangerouslySetInnerHTML")
                {
                    ctx.diagnostic(no_danger_diagnostic(prop.name.span()));
                }
            }
            AstKind::CallExpression(call_expr) => {
                if !is_create_element_call(call_expr) {
                    return;
                }

                let Some(props) = call_expr.arguments.get(1) else {
                    return;
                };

                let Argument::ObjectExpression(obj_expr) = props else {
                    return;
                };

                for prop in &obj_expr.properties {
                    if let ObjectPropertyKind::ObjectProperty(obj_prop) = prop {
                        if let Some(prop_name) = obj_prop.key.static_name() {
                            if prop_name == "dangerouslySetInnerHTML" {
                                ctx.diagnostic(no_danger_diagnostic(obj_prop.key.span()));
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_jsx()
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("<App />;", None),
        ("<div className=\"bar\"></div>;", None),
        ("React.createElement(\"div\", { className: \"bar\" });", None),
    ];

    let fail = vec![
        ("<div dangerouslySetInnerHTML={{ __html: \"\" }}></div>;", None),
        ("<button dangerouslySetInnerHTML={{ __html: \"baz\" }}>Foo</button>;", None),
        ("React.createElement(\"div\", { dangerouslySetInnerHTML: { __html: \"\" } });", None),
        (
            "React.createElement(\"button\", { dangerouslySetInnerHTML: { __html: \"baz\" } }, \"Foo\");",
            None,
        ),
    ];

    Tester::new(NoDanger::NAME, NoDanger::PLUGIN, pass, fail).test_and_snapshot();
}
