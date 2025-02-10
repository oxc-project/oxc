use oxc_ast::{
    ast::{Expression, MethodDefinitionKind, SimpleAssignmentTarget, StaticMemberExpression},
    AstKind,
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

fn no_direct_mutation_state_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("never mutate this.state directly.")
        .with_help("calling setState() afterwards may replace the mutation you made.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoDirectMutationState;

// code: https://github.com/jsx-eslint/eslint-plugin-react/blob/v7.37.2/lib/rules/no-direct-mutation-state.js
// doc: https://github.com/jsx-eslint/eslint-plugin-react/blob/v7.37.2/docs/rules/no-direct-mutation-state.md
// test: https://github.com/jsx-eslint/eslint-plugin-react/blob/v7.37.2/tests/lib/rules/no-direct-mutation-state.js

declare_oxc_lint!(
    /// ### What it does
    /// The restriction coder cannot directly change the value of this.state
    ///
    /// ### Why is this bad?
    /// calling setState() afterwards may replace the mutation you made
    ///
    /// ### Example
    /// ```jsx
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
    react,
    correctness
);

impl Rule for NoDirectMutationState {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::AssignmentExpression(assignment_expr) => {
                if should_ignore_component(node, ctx) {
                    return;
                }

                if let Some(assignment) = assignment_expr.left.as_simple_assignment_target() {
                    if let Some(outer_member_expression) = get_outer_member_expression(assignment) {
                        if is_state_member_expression(outer_member_expression) {
                            ctx.diagnostic(no_direct_mutation_state_diagnostic(
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
                        ctx.diagnostic(no_direct_mutation_state_diagnostic(update_expr.span));
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
    match assignment {
        SimpleAssignmentTarget::StaticMemberExpression(expr) => {
            let mut node = &**expr;
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
        _ => None,
    }
}

// Because node.object is of type &Expression<'_>
// We need a function to get static_member_expression
fn get_static_member_expression_obj<'a, 'b>(
    expression: &'b Expression<'a>,
) -> Option<&'b StaticMemberExpression<'a>> {
    match expression {
        Expression::StaticMemberExpression(expr) => Some(expr),
        _ => None,
    }
}

fn should_ignore_component<'a, 'b>(node: &'b AstNode<'a>, ctx: &'b LintContext<'a>) -> bool {
    let mut is_constructor = false;
    let mut is_call_expression = false;
    let mut is_component = false;

    for parent in ctx.nodes().ancestors(node.id()) {
        if let AstKind::MethodDefinition(method_def) = parent.kind() {
            if method_def.kind == MethodDefinitionKind::Constructor {
                is_constructor = true;
            }
        }

        if matches!(parent.kind(), AstKind::CallExpression(_)) {
            is_call_expression = true;
        }

        if is_es6_component(parent) || is_es5_component(parent) {
            is_component = true;
        }

        if matches!(parent.kind(), AstKind::Class(_)) {
            break;
        }
    }

    (is_constructor && !is_call_expression) || !is_component
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "var Hello = createReactClass({
          render: function() {
            return <div>Hello {this.props.name}</div>;
          }
        });",
        "
          var Hello = createReactClass({
            render: function() {
              var obj = {state: {}};
              obj.state.name = 'foo';
              return <div>Hello {obj.state.name}</div>;
            }
          });
        ",
        "
           var Hello = 'foo';
           module.exports = {};
         ",
        "
           class Hello {
             getFoo() {
               this.state.foo = 'bar'
               return this.state.foo;
             }
           }
         ",
        "
           class Hello extends React.Component {
             constructor() {
               this.state.foo = 'bar'
             }
           }
         ",
        "
        class Hello extends React.Component {
          constructor() {
            this.state.foo = 1;
          }
        }
      ",
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
        "
     describe('Component spec', () => {
        it('should apply default props on rerender', () => {
          class Outer extends Component {
            constructor() {
              super();
              this.state = { i: 1 };
            }
          }
        });
     });
",
    ];

    let fail = vec![
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
        "
                 var Hello = createReactClass({
                   render: function() {
                     this.state.foo++;
                     return <div>Hello {this.props.name}</div>;
                   }
                 });
               ",
        r#"
        var Hello = createReactClass({
          render: function() {
            this.state.person.name= "bar"
            return <div>Hello {this.props.name}</div>;
          }
        });
      "#,
        r#"
          var Hello = createReactClass({
            render: function() {
              this.state.person.name.first = "bar"
              return <div>Hello</div>;
            }
          });
        "#,
        r#"
          var Hello = createReactClass({
            render: function() {
              this.state.person.name.first = "bar"
              this.state.person.name.last = "baz"
              return <div>Hello</div>;
            }
          });
        "#,
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
        r#"
          class Hello extends React.Component {
            componentWillMount() {
              this.state.foo = "bar"
            }
          }
        "#,
        r#"
          class Hello extends React.Component {
            componentDidMount() {
              this.state.foo = "bar"
            }
          }
        "#,
        r#"
          class Hello extends React.Component {
            componentWillReceiveProps() {
              this.state.foo = "bar"
            }
          }
        "#,
        r#"
          class Hello extends React.Component {
            shouldComponentUpdate() {
              this.state.foo = "bar"
            }
          }
        "#,
        r#"
          class Hello extends React.Component {
            componentWillUpdate() {
              this.state.foo = "bar"
            }
          }
        "#,
        r#"
          class Hello extends React.Component {
            componentDidUpdate() {
              this.state.foo = "bar"
            }
          }
        "#,
        r#"
          class Hello extends React.Component {
            componentWillUnmount() {
              this.state.foo = "bar"
            }
          }
        "#,
    ];

    Tester::new(NoDirectMutationState::NAME, NoDirectMutationState::PLUGIN, pass, fail)
        .test_and_snapshot();
}
