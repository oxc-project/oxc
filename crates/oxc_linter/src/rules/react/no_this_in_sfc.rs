use oxc_ast::{AstKind, ast::BindingPattern};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    context::LintContext,
    rule::Rule,
    utils::{is_es5_component, is_es6_component, is_react_component_name},
};

fn no_this_in_sfc_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Stateless functional components should not use `this`")
        .with_help("Use props and context directly as function parameters instead of accessing them through `this`")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoThisInSfc;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prevents using `this` in stateless functional components.
    ///
    /// ### Why is this bad?
    ///
    /// In React, stateless functional components (SFCs) receive props and context as function parameters,
    /// not through `this`. Using `this` in an SFC typically indicates a mistake when converting from
    /// class components or unfamiliarity with the two component styles.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// function Foo(props) {
    ///   return <div>{this.props.bar}</div>;
    /// }
    ///
    /// function Foo(props) {
    ///   const { bar } = this.props;
    ///   return <div>{bar}</div>;
    /// }
    ///
    /// const Foo = (props) => this.props.foo ? <span>{props.bar}</span> : null;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// function Foo(props) {
    ///   return <div>{props.bar}</div>;
    /// }
    ///
    /// function Foo({ bar }) {
    ///   return <div>{bar}</div>;
    /// }
    ///
    /// class Foo extends React.Component {
    ///   render() {
    ///     return <div>{this.props.bar}</div>;
    ///   }
    /// }
    /// ```
    NoThisInSfc,
    react,
    correctness
);

impl Rule for NoThisInSfc {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ThisExpression(this_expr) = node.kind() else { return };

        let Some(component_node) = get_parent_function(node, ctx) else { return };

        if ctx
            .nodes()
            .ancestors(component_node.id())
            .any(|ancestor| is_es6_component(ancestor) || is_es5_component(ancestor))
        {
            return;
        }

        if is_in_nested_this_context(node, component_node, ctx) {
            return;
        }

        if !is_potential_react_component(component_node, ctx) {
            return;
        }

        ctx.diagnostic(no_this_in_sfc_diagnostic(this_expr.span));
    }

    fn should_run(&self, ctx: &crate::context::ContextHost) -> bool {
        ctx.source_type().is_jsx()
    }
}

fn get_parent_function<'a, 'b>(
    node: &'b AstNode<'a>,
    ctx: &'b LintContext<'a>,
) -> Option<&'b AstNode<'a>> {
    ctx.nodes().ancestors(node.id()).find(|ancestor| {
        matches!(ancestor.kind(), AstKind::Function(_) | AstKind::ArrowFunctionExpression(_))
    })
}

fn is_in_nested_this_context<'a>(
    this_node: &AstNode<'a>,
    component_node: &AstNode<'a>,
    ctx: &LintContext<'a>,
) -> bool {
    ctx.nodes()
        .ancestors(this_node.id())
        .take_while(|ancestor| ancestor.id() != component_node.id())
        .any(|ancestor| match ancestor.kind() {
            AstKind::Function(_)
            | AstKind::MethodDefinition(_)
            | AstKind::PropertyDefinition(_) => true,
            AstKind::ObjectProperty(_) => {
                matches!(ctx.nodes().parent_kind(ancestor.id()), AstKind::ObjectExpression(_))
            }

            _ => false,
        })
}

fn is_potential_react_component<'a>(function_node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
    let function_name = get_function_name(function_node, ctx);

    if let Some(name) = function_name
        && is_react_component_name(&name)
    {
        return true;
    }

    false
}

fn get_function_name<'a>(function_node: &AstNode<'a>, ctx: &LintContext<'a>) -> Option<String> {
    match function_node.kind() {
        AstKind::Function(func) => func.id.as_ref().map(|id| id.name.to_string()),
        AstKind::ArrowFunctionExpression(_) => {
            let parent = ctx.nodes().parent_node(function_node.id());
            if let AstKind::VariableDeclarator(declarator) = parent.kind()
                && let BindingPattern::BindingIdentifier(ident) = &declarator.id
            {
                return Some(ident.name.to_string());
            }
            None
        }
        _ => None,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "
			        function Foo(props) {
			          const { foo } = props;
			          return <div bar={foo} />;
			        }
			      ",
            None,
            None,
        ),
        (
            "
			        function Foo({ foo }) {
			          return <div bar={foo} />;
			        }
			      ",
            None,
            None,
        ),
        (
            "
			        class Foo extends React.Component {
			          render() {
			            const { foo } = this.props;
			            return <div bar={foo} />;
			          }
			        }
			      ",
            None,
            None,
        ),
        (
            "
			        const Foo = createReactClass({
			          render: function() {
			            return <div>{this.props.foo}</div>;
			          }
			        });
			      ",
            None,
            None,
        ),
        (
            "
			        const Foo = React.createClass({
			          render: function() {
			            return <div>{this.props.foo}</div>;
			          }
			        });
			      ",
            None,
            Some(serde_json::json!({ "settings": { "react": { "createClass": "createClass" } } })),
        ),
        (
            "
			        function foo(bar) {
			          this.bar = bar;
			          this.props = 'baz';
			          this.getFoo = function() {
			            return this.bar + this.props;
			          }
			        }
			      ",
            None,
            None,
        ),
        (
            "
			        function Foo(props) {
			          return props.foo ? <span>{props.bar}</span> : null;
			        }
			      ",
            None,
            None,
        ),
        (
            "
			        function Foo(props) {
			          if (props.foo) {
			            return <div>{props.bar}</div>;
			          }
			          return null;
			        }
			      ",
            None,
            None,
        ),
        (
            "
			        function Foo(props) {
			          if (props.foo) {
			            something();
			          }
			          return null;
			        }
			      ",
            None,
            None,
        ),
        ("const Foo = (props) => <span>{props.foo}</span>", None, None),
        ("const Foo = ({ foo }) => <span>{foo}</span>", None, None),
        ("const Foo = (props) => props.foo ? <span>{props.bar}</span> : null;", None, None),
        ("const Foo = ({ foo, bar }) => foo ? <span>{bar}</span> : null;", None, None),
        (
            "
			        class Foo {
			          bar() {
			            () => {
			              this.something();
			              return null;
			            };
			          }
			        }
			      ",
            None,
            None,
        ),
        (
            "
			        class Foo {
			          bar = () => {
			            this.something();
			            return null;
			          };
			        }
			      ",
            None,
            None,
        ),
        (
            "
			        export const Example = ({ prop }) => {
			          return {
			            handleClick: () => {},
			            renderNode() {
			              return <div onClick={this.handleClick} />;
			            },
			          };
			        };
			      ",
            None,
            None,
        ),
        (
            r#"
			        export const prepareLogin = new ValidatedMethod({
			          name: "user.prepare",
			          validate: new SimpleSchema({
			          }).validator(),
			          run({ remember }) {
			              if (Meteor.isServer) {
			                  const connectionId = this.connection.id; // react/no-this-in-sfc
			                  return Methods.prepareLogin(connectionId, remember);
			              }
			              return null;
			          },
			        });
			      "#,
            None,
            None,
        ),
        (
            "
			        obj.notAComponent = function () {
			          return this.a || null;
			        };
			      ",
            None,
            None,
        ),
        (
            "
			        $.fn.getValueAsStringWeak = function (): string | null {
			          const val = this.length === 1 ? this.val() : null;

			          return typeof val === 'string' ? val : null;
			        };
			      ",
            None,
            None,
        ),
    ];

    let fail = vec![
        (
            "
			        function Foo(props) {
			          const { foo } = this.props;
			          return <div>{foo}</div>;
			        }
			      ",
            None,
            None,
        ),
        (
            "
			        function Foo(props) {
			          return <div>{this.props.foo}</div>;
			        }
			      ",
            None,
            None,
        ),
        (
            "
			        function Foo(props) {
			          return <div>{this.state.foo}</div>;
			        }
			      ",
            None,
            None,
        ),
        (
            "
			        function Foo(props) {
			          const { foo } = this.state;
			          return <div>{foo}</div>;
			        }
			      ",
            None,
            None,
        ),
        (
            "
			        function Foo(props) {
			          return props.foo ? <div>{this.props.bar}</div> : null;
			        }
			      ",
            None,
            None,
        ),
        (
            "
			        function Foo(props) {
			          if (props.foo) {
			            return <div>{this.props.bar}</div>;
			          }
			          return null;
			        }
			      ",
            None,
            None,
        ),
        (
            "
			        function Foo(props) {
			          if (this.props.foo) {
			            something();
			          }
			          return null;
			        }
			      ",
            None,
            None,
        ),
        ("const Foo = (props) => <span>{this.props.foo}</span>", None, None),
        ("const Foo = (props) => this.props.foo ? <span>{props.bar}</span> : null;", None, None),
        (
            "
			        function Foo(props) {
			          function onClick(bar) {
			            this.props.onClick();
			          }
			          return <div onClick={onClick}>{this.props.foo}</div>;
			        }
			      ",
            None,
            None,
        ),
    ];

    Tester::new(NoThisInSfc::NAME, NoThisInSfc::PLUGIN, pass, fail).test_and_snapshot();
}
