use oxc_ast::{
    ast::{JSXAttributeValue, JSXExpression},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{get_element_type, get_prop_value, has_jsx_prop_ignore_case},
    AstNode,
};

fn iframe_has_title_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Missing `title` attribute for the `iframe` element.")
        .with_help("Provide title property for iframe element.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct IframeHasTitle;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce iframe elements have a title attribute.
    ///
    /// ### Why is this bad?
    ///
    /// Screen reader users rely on a iframe title to describe the contents of the iframe.
    /// Navigating through iframe and iframe elements quickly becomes difficult and confusing for users of this technology if the markup does not contain a title attribute.
    ///
    /// ### What it checks
    ///
    /// This rule checks for title property on iframe element.
    ///
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <iframe />
    /// <iframe {...props} />
    /// <iframe title="" />
    /// <iframe title={''} />
    /// <iframe title={``} />
    /// <iframe title={undefined} />
    /// <iframe title={false} />
    /// <iframe title={true} />
    /// <iframe title={42} />
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <iframe title="This is a unique title" />
    /// <iframe title={uniqueTitle} />
    /// ```
    IframeHasTitle,
    jsx_a11y,
    correctness
);

impl Rule for IframeHasTitle {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(jsx_el) = node.kind() else {
            return;
        };

        let name = get_element_type(ctx, jsx_el);

        if name != "iframe" {
            return;
        }
        let Some(alt_prop) = has_jsx_prop_ignore_case(jsx_el, "title") else {
            ctx.diagnostic(iframe_has_title_diagnostic(jsx_el.name.span()));
            return;
        };

        match get_prop_value(alt_prop) {
            Some(JSXAttributeValue::StringLiteral(str)) => {
                if !str.value.as_str().is_empty() {
                    return;
                }
            }
            Some(JSXAttributeValue::ExpressionContainer(container)) => {
                match &container.expression {
                    JSXExpression::StringLiteral(str) => {
                        if !str.value.is_empty() {
                            return;
                        }
                    }
                    JSXExpression::TemplateLiteral(tmpl) => {
                        if !tmpl.quasis.is_empty()
                            & !tmpl.expressions.is_empty()
                            & tmpl.quasis.iter().any(|q| !q.value.raw.as_str().is_empty())
                        {
                            return;
                        }
                    }
                    JSXExpression::CallExpression(_) => {
                        return;
                    }
                    expr @ JSXExpression::Identifier(_) => {
                        if !expr.is_undefined() {
                            return;
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }

        ctx.diagnostic(iframe_has_title_diagnostic(jsx_el.name.span()));
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // DEFAULT ELEMENT TESTS
        (r"<div />;", None, None),
        (r"<iframe title='Unique title' />", None, None),
        (r"<iframe title={foo} />", None, None),
        (r"<FooComponent />", None, None),
        (r"<iframe title={titleGenerator('hello')} />", None, None),
        // CUSTOM ELEMENT TESTS FOR COMPONENTS SETTINGS
        (
            r"<FooComponent title='Unique title' />",
            None,
            Some(serde_json::json!({
              "settings": { "jsx-a11y": {
                "components": {
                  "FooComponent": "iframe",
                },
              }, }
            })),
        ),
    ];

    let fail = vec![
        // DEFAULT ELEMENT TESTS
        (r"<iframe />", None, None),
        (r"<iframe {...props} />", None, None),
        (r"<iframe title={undefined} />", None, None),
        (r"<iframe title='' />", None, None),
        (r"<iframe title={false} />", None, None),
        (r"<iframe title={true} />", None, None),
        (r"<iframe title={''} />", None, None),
        (r"<iframe title={``} />", None, None),
        (r"<iframe title={42} />", None, None),
        // CUSTOM ELEMENT TESTS FOR COMPONENTS SETTINGS
        (
            r"<FooComponent />",
            None,
            Some(serde_json::json!({
              "settings": { "jsx-a11y": {
                "components": {
                  "FooComponent": "iframe",
                },
              }, }
            })),
        ),
    ];

    Tester::new(IframeHasTitle::NAME, IframeHasTitle::PLUGIN, pass, fail)
        .with_jsx_a11y_plugin(true)
        .test_and_snapshot();
}
