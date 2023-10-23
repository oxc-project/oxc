use oxc_ast::{
    ast::{Argument, Expression, JSXAttributeItem, ObjectPropertyKind},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::GetSpan;
use oxc_span::Span;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{has_jsx_prop, is_create_element_call},
    AstNode,
};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-react(no-danger): Do not use `dangerouslySetInnerHTML` prop")]
#[diagnostic(severity(warning), help("`dangerouslySetInnerHTML` is a way to inject HTML into your React component. This is dangerous because it can easily lead to XSS vulnerabilities."))]
struct NoDangerouslySetInnerHtmlDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoDangerouslySetInnerHtml;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule prevents the use of `dangerouslySetInnerHTML` prop.
    ///
    /// ### Why is this bad?
    ///
    /// `dangerouslySetInnerHTML` is a way to inject HTML into your React component. This is dangerous because it can easily lead to XSS vulnerabilities.
    ///
    /// ### Example
    /// ```javascript
    /// ```
    NoDangerouslySetInnerHtml,
    restriction
);

impl Rule for NoDangerouslySetInnerHtml {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::JSXElement(jsx_elem) => {
                if let Some(JSXAttributeItem::Attribute(prop)) =
                    has_jsx_prop(&jsx_elem.opening_element, "dangerouslySetInnerHTML")
                {
                    ctx.diagnostic(NoDangerouslySetInnerHtmlDiagnostic(prop.name.span()));
                }
            }
            AstKind::CallExpression(call_expr) => {
                if !is_create_element_call(call_expr) {
                    return;
                }

                let Some(Argument::Expression(props)) = call_expr.arguments.get(1) else { return };

                let Expression::ObjectExpression(obj_expr) = props else { return };

                for prop in &obj_expr.properties {
                    if let ObjectPropertyKind::ObjectProperty(obj_prop) = prop {
                        if let Some(prop_name) = obj_prop.key.static_name() {
                            if prop_name.as_str() == "dangerouslySetInnerHTML" {
                                ctx.diagnostic(NoDangerouslySetInnerHtmlDiagnostic(
                                    obj_prop.key.span(),
                                ));
                            }
                        }
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
        ("<App />;", None),
        ("<div className=\"bar\"></div>;", None),
        ("React.createElement(\"div\", { className: \"bar\" });", None),
    ];

    let fail = vec![
        ("<div dangerouslySetInnerHTML={{ __html: \"\" }}></div>;", None),
        ("<button dangerouslySetInnerHTML={{ __html: \"baz\" }}>Foo</button>;", None),
        ("React.createElement(\"div\", { dangerouslySetInnerHTML: { __html: \"\" } });", None),
        ("React.createElement(\"button\", { dangerouslySetInnerHTML: { __html: \"baz\" } }, \"Foo\");", None),
    ];

    Tester::new(NoDangerouslySetInnerHtml::NAME, pass, fail).test_and_snapshot();
}
