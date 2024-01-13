use oxc_ast::{
    ast::{Argument, ClassElement, Expression, FunctionBody, ObjectPropertyKind},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    rule::Rule,
    rules::eslint::array_callback_return::return_checker::{
        check_statement, StatementReturnStatus,
    },
    utils::{is_es5_component, is_es6_component},
    AstNode,
};

#[derive(Debug, Error, Diagnostic)]
#[error(
    "eslint-plugin-react(require-render-return): Your render method should have a return statement"
)]
#[diagnostic(severity(warning), help("When writing the `render` method in a component it is easy to forget to return the JSX content. This rule will warn if the return statement is missing."))]
struct RequireRenderReturnDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct RequireRenderReturn;

declare_oxc_lint!(
    /// ### What it does
    /// Enforce ES5 or ES6 class for returning value in render function
    ///
    /// ### Why is this bad?
    /// When writing the `render` method in a component it is easy to forget to return the JSX content. This rule will warn if the return statement is missing.
    ///
    /// ### Example
    /// ```javascript
    /// var Hello = createReactClass({
    ///   render() {
    ///     <div>Hello</div>;
    ///   }
    /// });
    ///
    /// class Hello extends React.Component {
    ///   render() {
    ///     <div>Hello</div>;
    ///   }
    /// }
    /// ```
    RequireRenderReturn,
    correctness
);

impl Rule for RequireRenderReturn {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if !is_es5_component(node) && !is_es6_component(node) {
            return;
        }

        match node.kind() {
            AstKind::Class(cl) => {
                if let Some((fn_body, is_arrow_function_expression)) =
                    cl.body.body.iter().find_map(|ce| match ce {
                        ClassElement::MethodDefinition(md)
                            if md.key.is_specific_static_name("render") =>
                        {
                            md.value.body.as_ref().map(|v| (v, false))
                        }
                        ClassElement::PropertyDefinition(pd)
                            if pd.key.is_specific_static_name("render") =>
                        {
                            if let Some(Expression::ArrowExpression(ref ae)) = pd.value {
                                Some((&ae.body, ae.expression))
                            } else {
                                None
                            }
                        }
                        _ => None,
                    })
                {
                    if is_arrow_function_expression || has_return_in_fn_body(fn_body) {
                        return;
                    }

                    ctx.diagnostic(RequireRenderReturnDiagnostic(fn_body.span));
                }
            }
            AstKind::CallExpression(ce) => {
                if let Some(Argument::Expression(Expression::ObjectExpression(obj_expr))) =
                    ce.arguments.first()
                {
                    if let Some(fn_body) = obj_expr
                        .properties
                        .iter()
                        .filter_map(|prop| match prop {
                            ObjectPropertyKind::ObjectProperty(prop)
                                if prop.key.is_specific_static_name("render") =>
                            {
                                if let Expression::FunctionExpression(ae) = &prop.value {
                                    ae.body.as_ref()
                                } else {
                                    None
                                }
                            }
                            _ => None,
                        })
                        .find(|fn_body| !has_return_in_fn_body(fn_body))
                    {
                        ctx.diagnostic(RequireRenderReturnDiagnostic(fn_body.span));
                    }
                }
            }
            _ => {}
        }
    }
}

fn has_return_in_fn_body<'a>(fn_body: &oxc_allocator::Box<'a, FunctionBody<'a>>) -> bool {
    fn_body.statements.iter().any(|stmt| check_statement(stmt) != StatementReturnStatus::NotReturn)
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"
			        class Hello extends React.Component {
			          render() {
			            return <div>Hello {this.props.name}</div>;
			          }
			        }
			      ",
        r"
			        class Hello extends React.Component {
			          render = () => {
			            return <div>Hello {this.props.name}</div>;
			          }
			        }
			      ",
        r"
			        class Hello extends React.Component {
			          render = () => (
			            <div>Hello {this.props.name}</div>
			          )
			        }
			      ",
        r"
			        var Hello = createReactClass({
			          displayName: 'Hello',
			          render: function() {
			            return <div></div>
			          }
			        });
			      ",
        r"
			        function Hello() {
			          return <div></div>;
			        }
			      ",
        r"
			        var Hello = () => (
			          <div></div>
			        );
			      ",
        r"
			        var Hello = createReactClass({
			          render: function() {
			            switch (this.props.name) {
			              case 'Foo':
			                return <div>Hello Foo</div>;
			              default:
			                return <div>Hello {this.props.name}</div>;
			            }
			          }
			        });
			      ",
        r"
			        var Hello = createReactClass({
			          render: function() {
			            if (this.props.name === 'Foo') {
			              return <div>Hello Foo</div>;
			            } else {
			              return <div>Hello {this.props.name}</div>;
			            }
			          }
			        });
			      ",
        r"
			        class Hello {
			          render() {}
			        }
			      ",
        r"class Hello extends React.Component {}",
        r"var Hello = createReactClass({});",
        r"
			        var render = require('./render');
			        var Hello = createReactClass({
			          render
			        });
			      ",
        r"
			        class Foo extends Component {
			          render
			        }
			      ",
    ];

    let fail = vec![
        r"
        	        var Hello = createReactClass({
        	          displayName: 'Hello',
        	          render: function() {}
        	        });
        	      ",
        r"
        	        class Hello extends React.Component {
        	          render() {}
        	        }
        	      ",
        r"
        	        class Hello extends React.Component {
        	          render() {
        	            const names = this.props.names.map(function(name) {
        	              return <div>{name}</div>
        	            });
        	          }
        	        }
        	      ",
        r"
        	        class Hello extends React.Component {
        	          render = () => {
        	            <div>Hello {this.props.name}</div>
        	          }
        	        }
        	      ",
    ];

    Tester::new(RequireRenderReturn::NAME, pass, fail).test_and_snapshot();
}
