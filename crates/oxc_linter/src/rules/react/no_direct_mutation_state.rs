use oxc_ast::{
    ast::{
        AssignmentTarget, Expression, MemberExpression, MethodDefinitionKind,
        SimpleAssignmentTarget, StaticMemberExpression,
    },
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{is_es5_component, is_es6_component},
    AstNode,
};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-react(no-direct-mutation-state): never mutate this.state directly.")]
#[diagnostic(
    severity(warning),
    help("calling setState() afterwards may replace the mutation you made.")
)]
struct NoDirectMutationStateDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoDirectMutationState;

// code: https://github.com/jsx-eslint/eslint-plugin-react/blob/master/lib/rules/no-direct-mutation-state.js
// doc: https://github.com/jsx-eslint/eslint-plugin-react/blob/master/docs/rules/no-direct-mutation-state.md
// test: https://github.com/jsx-eslint/eslint-plugin-react/blob/master/tests/lib/rules/no-direct-mutation-state.js

declare_oxc_lint!(
    /// ### What it does
    /// The restriction coder cannot directly change the value of this.state
    ///
    /// ### Why is this bad?
    /// calling setState() afterwards may replace the mutation you made
    ///
    /// ### Example
    /// ```javascript
    ///  // error
    ///  var Hello = createReactClass({
    ///    componentDidMount: function() {
    ///      this.state.name = this.props.name.toUpperCase();
    ///    },
    ///    render: function() {
    ///      return <div>Hello {this.state.name}</div>;
    ///    }
    ///  });
    ///
    ///  class Hello extends React.Component {
    ///    constructor(props) {
    ///      super(props)
    ///
    ///      doSomethingAsync(() => {
    ///        this.state = 'bad';
    ///      });
    ///    }
    ///  }
    ///
    ///  // success
    ///  var Hello = createReactClass({
    ///    componentDidMount: function() {
    ///      this.setState({
    ///        name: this.props.name.toUpperCase();
    ///      });
    ///    },
    ///    render: function() {
    ///      return <div>Hello {this.state.name}</div>;
    ///    }
    ///  });
    ///
    ///  class Hello extends React.Component {
    ///    constructor(props) {
    ///      super(props)
    ///
    ///      this.state = {
    ///        foo: 'bar',
    ///      }
    ///    }
    ///  }
    /// ```
    NoDirectMutationState,
    correctness
);

// check current node is this.state.xx
fn is_state_member_expression(expression: &StaticMemberExpression<'_>) -> bool {
    if let Expression::ThisExpression(_) = &expression.object {
        return expression.property.name == "state";
    }

    false
}

// get the top iterator
// example: this.state.a.b.c.d => this.state
fn get_outer_member_expression<'a, 'b>(
    assignment: &'b SimpleAssignmentTarget<'a>,
) -> Option<&'b StaticMemberExpression<'a>> {
    if let SimpleAssignmentTarget::MemberAssignmentTarget(member_expr) = assignment {
        match &member_expr.0 {
            MemberExpression::StaticMemberExpression(expr) => {
                let mut node = expr;
                loop {
                    if node.object.is_null() {
                        return Some(node);
                    }

                    if let Some(object) = get_static_member_expression_obj(&node.object) {
                        if !object.property.name.is_empty() {
                            node = object;

                            continue;
                        }
                    }

                    return Some(node);
                }
            }
            MemberExpression::PrivateFieldExpression(_)
            | MemberExpression::ComputedMemberExpression(_) => {}
        }
    }

    None
}

// Because node.object is of type &Expression<'_>
// We need a function to get static_member_expression
fn get_static_member_expression_obj<'a, 'b>(
    expression: &'b Expression<'a>,
) -> Option<&'b StaticMemberExpression<'a>> {
    match expression {
        Expression::MemberExpression(member_expr) => match &member_expr.0 {
            MemberExpression::StaticMemberExpression(expr) => Some(expr),
            _ => None,
        },
        _ => None,
    }
}

fn should_ignore_component<'a, 'b>(node: &'b AstNode<'a>, ctx: &'b LintContext<'a>) -> bool {
    let mut is_constructor: bool = false;
    let mut is_call_expression_node: bool = false;
    let mut is_component: bool = false;

    for parent in ctx.nodes().iter_parents(node.id()) {
        if let AstKind::MethodDefinition(method_def) = parent.kind() {
            if method_def.kind == MethodDefinitionKind::Constructor {
                is_constructor = true;
            }
        }

        if let AstKind::CallExpression(_) = parent.kind() {
            is_call_expression_node = true;
        }

        if is_es6_component(parent) || is_es5_component(parent) {
            is_component = true;
        }
    }

    is_constructor && !is_call_expression_node || !is_component
}

impl Rule for NoDirectMutationState {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::AssignmentExpression(assignment_expr) => {
                if should_ignore_component(node, ctx) {
                    return;
                }

                if let AssignmentTarget::SimpleAssignmentTarget(assignment) = &assignment_expr.left
                {
                    if let Some(outer_member_expression) = get_outer_member_expression(assignment) {
                        if is_state_member_expression(outer_member_expression) {
                            ctx.diagnostic(NoDirectMutationStateDiagnostic(
                                assignment_expr.left.span(),
                            ));
                        }
                    }
                }
            }

            AstKind::UpdateExpression(update_expr) => {
                if should_ignore_component(node, ctx) {
                    return;
                }

                if let Some(outer_member_expression) =
                    get_outer_member_expression(&update_expr.argument)
                {
                    if is_state_member_expression(outer_member_expression) {
                        ctx.diagnostic(NoDirectMutationStateDiagnostic(update_expr.span));
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
        (
            "var Hello = createReactClass({
          render: function() {
            return <div>Hello {this.props.name}</div>;
          }
        });",
            None,
        ),
        (
            "
          var Hello = createReactClass({
            render: function() {
              var obj = {state: {}};
              obj.state.name = 'foo';
              return <div>Hello {obj.state.name}</div>;
            }
          });
        ",
            None,
        ),
        (
            "
           var Hello = 'foo';
           module.exports = {};
         ",
            None,
        ),
        (
            "
           class Hello {
             getFoo() {
               this.state.foo = 'bar'
               return this.state.foo;
             }
           }
         ",
            None,
        ),
        (
            "
           class Hello extends React.Component {
             constructor() {
               this.state.foo = 'bar'
             }
           }
         ",
            None,
        ),
        (
            "
        class Hello extends React.Component {
          constructor() {
            this.state.foo = 1;
          }
        }
      ",
            None,
        ),
        (
            "
       class OneComponent extends Component {
         constructor() {
           super();
           class AnotherComponent extends Component {
             constructor() {
               super();
             }
           }
           this.state = {};
         }
       }
     ",
            None,
        ),
    ];

    let fail = vec![
        (
            r#"
                  var Hello = createReactClass({

                    componentWillMount() {
                      this.state.foo = "Chicken, you're so beautiful"
                    },

                          render: function() {
                            this.state.foo = "Chicken, you're so beautiful"
                            return <div>Hello{this.props.name} <Hello2/></div>;
                          }
                        });

                  var Hello2 = createReactClass({
                          render: () => {
                             this.state.foo = "Chicken, you're so beautiful"
                            return <div>Hello {this.props.name}</div>;
                          }
                        });
          "#,
            None,
        ),
        (
            "
                 var Hello = createReactClass({
                   render: function() {
                     this.state.foo++;
                     return <div>Hello {this.props.name}</div>;
                   }
                 });
               ",
            None,
        ),
        (
            r#"
        var Hello = createReactClass({
          render: function() {
            this.state.person.name= "bar"
            return <div>Hello {this.props.name}</div>;
          }
        });
      "#,
            None,
        ),
        (
            r#"
          var Hello = createReactClass({
            render: function() {
              this.state.person.name.first = "bar"
              return <div>Hello</div>;
            }
          });
        "#,
            None,
        ),
        (
            r#"
          var Hello = createReactClass({
            render: function() {
              this.state.person.name.first = "bar"
              this.state.person.name.last = "baz"
              return <div>Hello</div>;
            }
          });
        "#,
            None,
        ),
        (
            r#"
          class Hello extends React.Component {
            constructor() {
              someFn()
            }
            someFn() {
              this.state.foo = "bar"
            }
          }
        "#,
            None,
        ),
        (
            r#"
          class Hello extends React.Component {
            constructor(props) {
              super(props)
              doSomethingAsync(() => {
                this.state = "bad";
              });
            }
          }
        "#,
            None,
        ),
        (
            r#"
          class Hello extends React.Component {
            componentWillMount() {
              this.state.foo = "bar"
            }
          }
        "#,
            None,
        ),
        (
            r#"
          class Hello extends React.Component {
            componentDidMount() {
              this.state.foo = "bar"
            }
          }
        "#,
            None,
        ),
        (
            r#"
          class Hello extends React.Component {
            componentWillReceiveProps() {
              this.state.foo = "bar"
            }
          }
        "#,
            None,
        ),
        (
            r#"
          class Hello extends React.Component {
            shouldComponentUpdate() {
              this.state.foo = "bar"
            }
          }
        "#,
            None,
        ),
        (
            r#"
          class Hello extends React.Component {
            componentWillUpdate() {
              this.state.foo = "bar"
            }
          }
        "#,
            None,
        ),
        (
            r#"
          class Hello extends React.Component {
            componentDidUpdate() {
              this.state.foo = "bar"
            }
          }
        "#,
            None,
        ),
        (
            r#"
          class Hello extends React.Component {
            componentWillUnmount() {
              this.state.foo = "bar"
            }
          }
        "#,
            None,
        ),
    ];

    Tester::new(NoDirectMutationState::NAME, pass, fail).test_and_snapshot();
}
