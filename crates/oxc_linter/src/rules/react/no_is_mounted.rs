use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    rule::Rule,
};

fn no_is_mounted_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not use `isMounted`.")
        .with_help("`isMounted` is not supported in modern React, and does not work in class or function components.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoIsMounted;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule prevents using `isMounted` in class components.
    ///
    /// ### Why is this bad?
    ///
    /// `isMounted` is an anti-pattern, and is not available
    /// when using classes or function components.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    ///
    /// ```jsx
    /// class Hello extends React.Component {
    ///   someMethod() {
    ///     if (!this.isMounted()) {
    ///       return;
    ///     }
    ///   }
    ///   render() {
    ///     return <div onClick={this.someMethod.bind(this)}>Hello</div>;
    ///   }
    /// };
    /// ```
    NoIsMounted,
    react,
    correctness
);

impl Rule for NoIsMounted {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };
        let Some(member_expr) = call_expr.callee.as_member_expression() else {
            return;
        };

        if !matches!(member_expr.object(), Expression::ThisExpression(_))
            || member_expr.static_property_name().is_none_or(|str| str != "isMounted")
        {
            return;
        }

        for ancestor_kind in ctx.nodes().ancestor_kinds(node.id()) {
            if matches!(ancestor_kind, AstKind::ObjectProperty(_) | AstKind::MethodDefinition(_)) {
                ctx.diagnostic(no_is_mounted_diagnostic(call_expr.span));
                break;
            }
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_jsx()
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "
            var Hello = function() {
            };
        ",
            None,
        ),
        (
            "
            var Hello = createReactClass({
                componentDidUpdate: function() {
                    someNonMemberFunction(arg);
                    this.someFunc = this.isMounted;
                },
                render: function() {
                    return <div>Hello</div>;
                }
            });
            ",
            None,
        ),
        (
            "
            class Hello extends React.Component {
                notIsMounted() {}
                render() {
                    this.notIsMounted();
                    return <div>Hello</div>;
                }
            };
            ",
            None,
        ),
    ];

    let fail = vec![
        (
            "
            var Hello = createReactClass({
                componentDidUpdate: function() {
                  if (!this.isMounted()) {
                    return;
                  }
                },
                render: function() {
                  return <div>Hello</div>;
                }
            });
            ",
            None,
        ),
        (
            "
            var Hello = createReactClass({
                someMethod: function() {
                  if (!this.isMounted()) {
                    return;
                  }
                },
                render: function() {
                  return <div onClick={this.someMethod.bind(this)}>Hello</div>;
                }
            });
            ",
            None,
        ),
        (
            "
            class Hello extends React.Component {
                someMethod() {
                  if (!this.isMounted()) {
                    return;
                  }
                }
                render() {
                  return <div onClick={this.someMethod.bind(this)}>Hello</div>;
                }
            };
            ",
            None,
        ),
    ];

    Tester::new(NoIsMounted::NAME, NoIsMounted::PLUGIN, pass, fail).test_and_snapshot();
}
