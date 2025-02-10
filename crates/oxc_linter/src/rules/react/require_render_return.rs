use oxc_ast::{ast::Expression, AstKind};
use oxc_cfg::{
    graph::visit::neighbors_filtered_by_edge_weight, EdgeType, Instruction, InstructionKind,
    ReturnInstructionKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    utils::{is_es5_component, is_es6_component},
    AstNode,
};

fn require_render_return_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Your render method should have a return statement")
        .with_help("When writing the `render` method in a component it is easy to forget to return the JSX content. This rule will warn if the return statement is missing.")
        .with_label(span)
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
    /// ```jsx
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
    react,
    nursery
);

impl Rule for RequireRenderReturn {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if !matches!(node.kind(), AstKind::ArrowFunctionExpression(_) | AstKind::Function(_)) {
            return;
        }
        let Some(parent) = ctx.nodes().parent_node(node.id()) else {
            return;
        };
        if !is_render_fn(parent) {
            return;
        }
        if !is_in_es6_component(parent, ctx) && !is_in_es5_component(parent, ctx) {
            return;
        }

        if !contains_return_statement(node, ctx) {
            match parent.kind() {
                AstKind::MethodDefinition(method) => {
                    ctx.diagnostic(require_render_return_diagnostic(method.key.span()));
                }
                AstKind::PropertyDefinition(property) => {
                    ctx.diagnostic(require_render_return_diagnostic(property.key.span()));
                }
                AstKind::ObjectProperty(property) => {
                    ctx.diagnostic(require_render_return_diagnostic(property.key.span()));
                }
                _ => {}
            };
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_jsx()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
enum FoundReturn {
    #[default]
    No,
    Yes,
}

const KEEP_WALKING_ON_THIS_PATH: bool = true;
const STOP_WALKING_ON_THIS_PATH: bool = false;

fn contains_return_statement(node: &AstNode, ctx: &LintContext) -> bool {
    let cfg = ctx.cfg();
    let state = neighbors_filtered_by_edge_weight(
        cfg.graph(),
        node.cfg_id(),
        &|edge| match edge {
            // We only care about normal edges having a return statement.
            EdgeType::Jump | EdgeType::Normal => None,
            // For these two type, we flag it as not found.
            EdgeType::Unreachable
            | EdgeType::Error(_)
            | EdgeType::Finalize
            | EdgeType::Join
            | EdgeType::NewFunction
            | EdgeType::Backedge => Some(FoundReturn::No),
        },
        &mut |basic_block_id, _state_going_into_this_rule| {
            // If its an arrow function with an expression, marked as founded and stop walking.
            if let AstKind::ArrowFunctionExpression(arrow_expr) = node.kind() {
                if arrow_expr.expression {
                    return (FoundReturn::Yes, STOP_WALKING_ON_THIS_PATH);
                }
            }

            for Instruction { kind, .. } in cfg.basic_block(*basic_block_id).instructions() {
                match kind {
                    InstructionKind::Return(ReturnInstructionKind::NotImplicitUndefined) => {
                        return (FoundReturn::Yes, STOP_WALKING_ON_THIS_PATH);
                    }
                    InstructionKind::Unreachable | InstructionKind::Throw => {
                        return (FoundReturn::No, STOP_WALKING_ON_THIS_PATH);
                    }
                    InstructionKind::Return(ReturnInstructionKind::ImplicitUndefined)
                    | InstructionKind::ImplicitReturn
                    | InstructionKind::Break(_)
                    | InstructionKind::Continue(_)
                    | InstructionKind::Iteration(_)
                    | InstructionKind::Condition
                    | InstructionKind::Statement => {}
                }
            }

            (FoundReturn::No, KEEP_WALKING_ON_THIS_PATH)
        },
    );

    state.iter().any(|&state| state == FoundReturn::Yes)
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

    // let too_many_if_else = (1..10)
    //     .map(|i| {
    //         "
    //         if (a > i) {
    //             foo1()
    //         } else {
    //             foo2()
    //         }
    //     "
    //     })
    //     .collect::<String>();

    // let too_many_if_else_case = format!(
    //     "
    //     class Hello extends React.Component {{
    //         render() {{
    //             {too_many_if_else}
    //             return 'div'
    //         }}
    //     }}
    //     ",
    // );

    let pass = vec![
        // &too_many_if_else_case,
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
        r"
           class Foo extends Component {
             render = () => {
               if (true) {
                  return <div>Hello</div>;
                }
              }
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
        r"
            class Hello extends React.Component {
              render() {
                function foo() {
                        return <div>Hello {this.props.name}</div>;
                    }
                }
            }
         ",
        r"
            class Hello extends React.Component {
              render() {
                return
              }
            }
         ",
    ];

    Tester::new(RequireRenderReturn::NAME, RequireRenderReturn::PLUGIN, pass, fail)
        .test_and_snapshot();
}
