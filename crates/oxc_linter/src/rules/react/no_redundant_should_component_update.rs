use oxc_ast::{
    AstKind,
    ast::{Class, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_redundant_should_component_update_diagnostic(
    span: Span,
    component_name: &str,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "{component_name} does not need `shouldComponentUpdate` when extending `React.PureComponent`."
    ))
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoRedundantShouldComponentUpdate;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow usage of `shouldComponentUpdate` when extending `React.PureComponent`.
    ///
    /// ### Why is this bad?
    ///
    /// `React.PureComponent` automatically implements `shouldComponentUpdate` with a shallow prop and state comparison.
    /// Defining `shouldComponentUpdate` in a class that extends `React.PureComponent` is redundant and defeats the purpose
    /// of using `React.PureComponent`. If you need custom comparison logic, extend `React.Component` instead.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// class Foo extends React.PureComponent {
    ///   shouldComponentUpdate() {
    ///     // do check
    ///   }
    ///
    ///   render() {
    ///     return <div>Radical!</div>
    ///   }
    /// }
    ///
    /// function Bar() {
    ///   return class Baz extends React.PureComponent {
    ///     shouldComponentUpdate() {
    ///       // do check
    ///     }
    ///
    ///     render() {
    ///       return <div>Groovy!</div>
    ///     }
    ///   }
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// class Foo extends React.Component {
    ///   shouldComponentUpdate() {
    ///     // do check
    ///   }
    ///
    ///   render() {
    ///     return <div>Radical!</div>
    ///   }
    /// }
    ///
    /// function Bar() {
    ///   return class Baz extends React.Component {
    ///     shouldComponentUpdate() {
    ///       // do check
    ///     }
    ///
    ///     render() {
    ///       return <div>Groovy!</div>
    ///     }
    ///   }
    /// }
    ///
    /// class Qux extends React.PureComponent {
    ///   render() {
    ///     return <div>Tubular!</div>
    ///   }
    /// }
    /// ```
    NoRedundantShouldComponentUpdate,
    react,
    style,
);

impl Rule for NoRedundantShouldComponentUpdate {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::Class(class_expr) = node.kind() else { return };
        if !is_react_pure_component(class_expr) {
            return;
        }

        let Some(component_should_update_method_span) = get_should_component_update(class_expr)
        else {
            return;
        };

        let component_name = class_expr
            .name()
            .or_else(|| {
                // e.g. var Foo = class extends PureComponent
                let parent = ctx.nodes().parent_node(node.id());
                if let AstKind::VariableDeclarator(declarator) = parent.kind() {
                    declarator.id.get_identifier_name()
                } else {
                    None
                }
            })
            .map_or("", |name| name.as_str());

        ctx.diagnostic(no_redundant_should_component_update_diagnostic(
            component_should_update_method_span,
            component_name,
        ));
    }
}

fn get_should_component_update(class: &Class<'_>) -> Option<Span> {
    class.body.body.iter().find_map(|prop| {
        let key = prop.property_key()?;
        (key.static_name()? == "shouldComponentUpdate").then_some(key.span())
    })
}

/// Checks if class is React.PureComponent and returns this class if true
fn is_react_pure_component<'a>(class: &'a Class<'a>) -> bool {
    if let Some(super_class) = &class.super_class {
        if let Some(member_expr) = super_class.as_member_expression()
            && let Expression::Identifier(ident) = member_expr.object()
        {
            return ident.name == "React"
                && member_expr.static_property_name().is_some_and(|name| name == "PureComponent");
        }

        if let Some(ident_reference) = super_class.get_identifier_reference() {
            return ident_reference.name == "PureComponent";
        }
    }

    false
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "
			        class Foo extends React.Component {
			          shouldComponentUpdate() {
			            return true;
			          }
			        }
			      ",
        "
			        class Foo extends React.Component {
			          shouldComponentUpdate = () => {
			            return true;
			          }
			        }
			      ",
        "
			        function Foo() {
			          return class Bar extends React.Component {
			            shouldComponentUpdate() {
			              return true;
			            }
			          };
			        }
			      ",
    ];

    let fail = vec![
        "
			        class Foo extends React.PureComponent {
			          shouldComponentUpdate() {
			            return true;
			          }
			        }
			      ",
        "
			        class Foo extends PureComponent {
			          shouldComponentUpdate() {
			            return true;
			          }
			        }
			      ",
        "
			        class Foo extends React.PureComponent {
			          shouldComponentUpdate = () => {
			            return true;
			          }
			        }
			      ",
        "
			        function Foo() {
			          return class Bar extends React.PureComponent {
			            shouldComponentUpdate() {
			              return true;
			            }
			          };
			        }
			      ",
        "
			        function Foo() {
			          return class Bar extends PureComponent {
			            shouldComponentUpdate() {
			              return true;
			            }
			          };
			        }
			      ",
        "
			        var Foo = class extends PureComponent {
			          shouldComponentUpdate() {
			            return true;
			          }
			        }
			      ",
    ];

    Tester::new(
        NoRedundantShouldComponentUpdate::NAME,
        NoRedundantShouldComponentUpdate::PLUGIN,
        pass,
        fail,
    )
    .test_and_snapshot();
}
