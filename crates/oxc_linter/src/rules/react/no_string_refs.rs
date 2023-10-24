use oxc_ast::ast::{JSXAttributeItem, JSXAttributeName, CallExpression};
use oxc_ast::{
    ast::{Expression, JSXAttributeValue, JSXExpression, JSXExpressionContainer, MemberExpression},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-react(no-string-refs):")]
#[diagnostic(
    severity(warning),
    help("Using string literals in ref attributes is deprecated. Use a callback instead.")
)]
struct NoStringRefsDiagnostic(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-react(no-string-refs):")]
#[diagnostic(severity(warning), help("Using this.refs is deprecated."))]
struct NoThisRefsDiagnositc(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoStringRefs {
    /// When set to `true`, it will give a warning when using template literals for refs.
    pub no_template_literals: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule disallows using string references in JSX `ref` attributes.
    ///
    /// ### Why is this bad?
    ///
    /// String refs are considered legacy in the React documentation. Callback refs are preferred.
    ///
    /// ### Example
    /// ```javascript
    /// // Bad
    /// var Hello = createReactClass({
    ///   componentDidMount: function() {
    ///     var component = this.refs.hello;
    ///     // ...do something with component
    ///   },
    ///   render: function() {
    ///     return <div ref="hello">Hello, world.</div>;
    ///   },
    /// });
    ///
    /// // Good
    /// var Hello = createReactClass({
    ///   componentDidMount: function() {
    ///     var component = this.hello;
    ///     // ...do something with component
    ///   },
    ///   render: function() {
    ///     return <div ref={c => this.hello = c}>Hello, world.</div>;
    ///   },
    /// });
    /// ```
    NoStringRefs,
    correctness
);

impl Rule for NoStringRefs {
    fn from_configuration(value: serde_json::Value) -> Self {
        Self {
            no_template_literals: value.get(0).and_then(|x| x.get("noTemplateLiterals")).map_or(
                false,
                |x| match x {
                    serde_json::Value::Bool(b) => *b,
                    _ => false,
                },
            ),
        }
    }
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::JSXAttributeItem(JSXAttributeItem::Attribute(attribute)) = node.kind() {
            let Some(value) = &attribute.0.value else {
                return;
            };
            if let JSXAttributeName::Identifier(iden) = &attribute.0.name {
                if iden.name.as_str() != "ref" {
                    return;
                };
            }
            match value {
                JSXAttributeValue::StringLiteral(s) => {
                    ctx.diagnostic(NoStringRefsDiagnostic(s.span));
                }
                JSXAttributeValue::ExpressionContainer(JSXExpressionContainer {
                    expression: JSXExpression::Expression(expr),
                    ..
                }) => match expr {
                    Expression::TemplateLiteral(s) if self.no_template_literals => {
                        ctx.diagnostic(NoStringRefsDiagnostic(s.span));
                    }
                    Expression::StringLiteral(s) => {
                        ctx.diagnostic(NoStringRefsDiagnostic(s.span));
                    }
                    _ => { return },
                },
                _ => { return },
            }
        }

        if let AstKind::MemberExpression(MemberExpression::StaticMemberExpression(expr)) = node.kind() {
            if let (&Expression::ThisExpression(_), "refs") =
                (&expr.object, expr.property.name.as_str())
            {
                for node_id in ctx.nodes().ancestors(node.id()).skip(1) {
                    let parent = ctx.nodes().get_node(node_id);
                    if let AstKind::CallExpression(CallExpression { callee: Expression::Identifier(iden), .. }) = parent.kind() {
                        if iden.name.as_str() == "createReactClass" {
                            ctx.diagnostic(NoThisRefsDiagnositc(expr.span));
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "
			        var Hello = blah({
			          componentDidMount: function() {
			            var component = this.refs.hello;
			          },
			          render: function() {
			            return <div>Hello {this.props.name}</div>;
			          }
			        });
			      ",
            None,
        ),
        (
            "
			        var Hello = createReactClass({
			          componentDidMount: function() {
			            var component = this.hello;
			          },
			          render: function() {
			            return <div ref={c => this.hello = c}>Hello {this.props.name}</div>;
			          }
			        });
			      ",
            None,
        ),
        (
            "
			        var Hello = createReactClass({
			          render: function() {
			            return <div ref={`hello`}>Hello {this.props.name}</div>;
			          }
			        });
			      ",
            None,
        ),
        (
            "
			        var Hello = createReactClass({
			          render: function() {
			            return <div ref={`hello${index}`}>Hello {this.props.name}</div>;
			          }
			        });
			      ",
            None,
        ),
    ];

    let fail = vec![
        (
            "
			        var Hello = createReactClass({
			          componentDidMount: function() {
			            var component = this.refs.hello;
			          },
			          render: function() {
			            return <div>Hello {this.props.name}</div>;
			          }
			        });
			      ",
            None,
        ),
        (
            "
			        var Hello = createReactClass({
			          render: function() {
			            return <div ref=\"hello\">Hello {this.props.name}</div>;
			          }
			        });
			      ",
            None,
        ),
        (
            "
			        var Hello = createReactClass({
			          render: function() {
			            return <div ref={'hello'}>Hello {this.props.name}</div>;
			          }
			        });
			      ",
            None,
        ),
        (
            "
			        var Hello = createReactClass({
			          componentDidMount: function() {
			            var component = this.refs.hello;
			          },
			          render: function() {
			            return <div ref=\"hello\">Hello {this.props.name}</div>;
			          }
			        });
			      ",
            None,
        ),
        (
            "
			        var Hello = createReactClass({
			          componentDidMount: function() {
			            var component = this.refs.hello;
			          },
			          render: function() {
			            return <div ref={`hello`}>Hello {this.props.name}</div>;
			          }
			        });
			      ",
            Some(serde_json::json!([{ "noTemplateLiterals": true }])),
        ),
        (
            "
			        var Hello = createReactClass({
			          componentDidMount: function() {
			            var component = this.refs.hello;
			          },
			          render: function() {
			            return <div ref={`hello${index}`}>Hello {this.props.name}</div>;
			          }
			        });
			      ",
            Some(serde_json::json!([{ "noTemplateLiterals": true }])),
        ),
    ];

    Tester::new(NoStringRefs::NAME, pass, fail).test_and_snapshot();
}
