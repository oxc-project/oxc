use oxc_ast::{
    ast::{
        Expression, JSXAttribute, JSXAttributeItem, JSXAttributeName, JSXAttributeValue,
        JSXExpression, JSXExpressionContainer,
    },
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{get_parent_es5_component, get_parent_es6_component},
    AstNode,
};

fn this_refs_deprecated(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Using this.refs is deprecated.")
        .with_help("Using this.xxx instead of this.refs.xxx")
        .with_label(span0)
}

fn string_in_ref_deprecated(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Using string literals in ref attributes is deprecated.")
        .with_help("Using reference callback instead")
        .with_label(span0)
}

#[derive(Debug, Default, Clone)]
pub struct NoStringRefs {
    no_template_literals: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule prevents using string literals in ref attributes.
    ///
    /// ### Example
    /// ```javascript
    /// // Bad
    /// var Hello = createReactClass({
    ///   render: function() {
    ///     return <div ref="hello">Hello, world.</div>;
    ///   }
    /// });
    ///
    /// var Hello = createReactClass({
    ///   componentDidMount: function() {
    ///     var component = this.refs.hello;
    ///     // ...do something with component
    ///   },
    ///   render: function() {
    ///     return <div ref="hello">Hello, world.</div>;
    ///   }
    /// });
    ///
    /// // Good
    /// var Hello = createReactClass({
    ///   componentDidMount: function() {
    ///     var component = this.hello;
    ///     // ...do something with component
    ///   },
    ///   render() {
    ///     return <div ref={(c) => { this.hello = c; }}>Hello, world.</div>;
    ///   }
    /// });
    /// ```
    NoStringRefs,
    correctness
);

fn contains_string_literal(
    expr_container: &JSXExpressionContainer,
    no_template_literals: bool,
) -> bool {
    let expr = &expr_container.expression;
    matches!(expr, JSXExpression::StringLiteral(_))
        || (no_template_literals && matches!(expr, JSXExpression::TemplateLiteral(_)))
}

fn is_literal_ref_attribute(attr: &JSXAttribute, no_template_literals: bool) -> bool {
    let JSXAttributeName::Identifier(attr_ident) = &attr.name else {
        return false;
    };
    if attr_ident.name == "ref" {
        if let Some(attr_value) = &attr.value {
            return match attr_value {
                JSXAttributeValue::ExpressionContainer(expr_container) => {
                    contains_string_literal(expr_container, no_template_literals)
                }
                JSXAttributeValue::StringLiteral(_) => true,
                _ => false,
            };
        }
    }

    false
}

impl Rule for NoStringRefs {
    fn from_configuration(value: serde_json::Value) -> Self {
        let no_template_literals =
            value.get("noTemplateLiterals").and_then(serde_json::Value::as_bool).unwrap_or(false);

        Self { no_template_literals }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::JSXAttributeItem(JSXAttributeItem::Attribute(attr)) => {
                if is_literal_ref_attribute(attr, self.no_template_literals) {
                    ctx.diagnostic(string_in_ref_deprecated(attr.span));
                }
            }
            AstKind::MemberExpression(member_expr) => {
                if matches!(member_expr.object(), Expression::ThisExpression(_))
                    && member_expr.static_property_name() == Some("refs")
                    && (get_parent_es5_component(node, ctx).is_some()
                        || get_parent_es6_component(ctx).is_some())
                {
                    ctx.diagnostic(this_refs_deprecated(member_expr.span()));
                }
            }
            _ => {}
        }
    }

    fn should_run(&self, ctx: &LintContext) -> bool {
        ctx.source_type().is_jsx()
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "
                    var Hello = React.createReactClass({
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
            Some(serde_json::json!({ "noTemplateLiterals": true })),
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
            Some(serde_json::json!({ "noTemplateLiterals": true })),
        ),
        (
            "
              var Hello = createReactClass({
                render: function() {
                  return <div ref={`hello${index}`}>Hello {this.props.name}</div>;
                }
              });
            ",
            Some(serde_json::json!({ "noTemplateLiterals": true })),
        ),
        (
            "
              class Hello extends React.Component {
                componentDidMount() {
                  var component = this.refs.hello;
                }
              }
            ",
            None,
        ),
        (
            "
              class Hello extends React.Component {
                componentDidMount() {
                  var component = this.refs.hello;
                }
              }
            ",
            None,
        ),
        (
            "
              class Hello extends React.PureComponent {
                componentDidMount() {
                  var component = this.refs.hello;
                }
                render() {
                  return <div ref={`hello${index}`}>Hello {this.props.name}</div>;
                }
              }
            ",
            Some(serde_json::json!({ "noTemplateLiterals": true })),
        ),
    ];

    Tester::new(NoStringRefs::NAME, pass, fail).test_and_snapshot();
}
