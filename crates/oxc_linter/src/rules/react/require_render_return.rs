use oxc_ast::{ast::Expression, AstKind};
use oxc_diagnostics::OxcDiagnostic;

use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNodeId;
use oxc_span::{GetSpan, Span};
use rustc_hash::FxHashSet;

use crate::{
    ast_util::get_enclosing_function,
    context::LintContext,
    rule::Rule,
    utils::{is_es5_component, is_es6_component},
    AstNode,
};

fn require_render_return_diagnostic(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("eslint-plugin-react(require-render-return): Your render method should have a return statement")
        .with_help("When writing the `render` method in a component it is easy to forget to return the JSX content. This rule will warn if the return statement is missing.")
        .with_labels([span0.into()])
}

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
    fn run_once(&self, ctx: &LintContext) {
        let mut render_fn_set: FxHashSet<AstNodeId> = FxHashSet::default();

        for node in ctx.nodes().iter() {
            if is_render_fn(node) {
                render_fn_set.insert(node.id());
            }

            let is_return_detected = match node.kind() {
                AstKind::ReturnStatement(_) => true,
                AstKind::ArrowFunctionExpression(arrow) => arrow.expression,
                _ => false,
            };

            if is_return_detected {
                let Some(function) = get_enclosing_function(node, ctx) else {
                    continue;
                };
                let Some(fn_parent_node) = ctx.nodes().parent_node(function.id()) else {
                    continue;
                };

                if matches!(
                    fn_parent_node.kind(),
                    AstKind::MethodDefinition(_)
                        | AstKind::PropertyDefinition(_)
                        | AstKind::ObjectProperty(_)
                ) {
                    render_fn_set.remove(&fn_parent_node.id());
                }
            }
        }

        for node_id in render_fn_set {
            let render_fn_node = ctx.nodes().get_node(node_id);
            if is_in_es6_component(render_fn_node, ctx) {
                diagnostic_on_render_fn(node_id, ctx);
                return;
            } else if is_in_es5_component(render_fn_node, ctx) {
                diagnostic_on_render_fn(node_id, ctx);
            }
        }
    }
}

const RENDER_METHOD_NAME: &str = "render";

fn is_render_fn(node: &AstNode) -> bool {
    match node.kind() {
        AstKind::MethodDefinition(method) => {
            if method.key.is_specific_static_name(RENDER_METHOD_NAME) {
                return true;
            }
        }
        AstKind::PropertyDefinition(property) => {
            if property.key.is_specific_static_name(RENDER_METHOD_NAME)
                && property.value.as_ref().is_some_and(Expression::is_function)
            {
                return true;
            }
        }
        AstKind::ObjectProperty(property) => {
            if property.key.is_specific_static_name(RENDER_METHOD_NAME)
                && property.value.is_function()
            {
                return true;
            }
        }
        _ => {}
    }
    false
}

fn diagnostic_on_render_fn(node_id: AstNodeId, ctx: &LintContext) {
    let span = match ctx.nodes().get_node(node_id).kind() {
        AstKind::MethodDefinition(method) => Some(method.key.span()),
        AstKind::PropertyDefinition(property) => Some(property.key.span()),
        AstKind::ObjectProperty(property) => Some(property.key.span()),
        _ => None,
    };

    if let Some(span) = span {
        ctx.diagnostic(require_render_return_diagnostic(span));
    }
}

fn is_in_es5_component<'a, 'b>(node: &'b AstNode<'a>, ctx: &'b LintContext<'a>) -> bool {
    let Some(ancestors_0) = ctx.nodes().parent_node(node.id()) else { return false };
    if !matches!(ancestors_0.kind(), AstKind::ObjectExpression(_)) {
        return false;
    }

    let Some(ancestors_1) = ctx.nodes().parent_node(ancestors_0.id()) else { return false };
    if !matches!(ancestors_1.kind(), AstKind::Argument(_)) {
        return false;
    }

    let Some(ancestors_2) = ctx.nodes().parent_node(ancestors_1.id()) else { return false };

    is_es5_component(ancestors_2)
}

fn is_in_es6_component<'a, 'b>(node: &'b AstNode<'a>, ctx: &'b LintContext<'a>) -> bool {
    let Some(parent) = ctx.nodes().parent_node(node.id()) else { return false };
    if !matches!(parent.kind(), AstKind::ClassBody(_)) {
        return false;
    }

    let Some(grandparent) = ctx.nodes().parent_node(parent.id()) else { return false };
    is_es6_component(grandparent)
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
