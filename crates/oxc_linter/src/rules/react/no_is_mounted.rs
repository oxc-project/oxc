use oxc_ast::{ast::Expression, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_is_mounted_diagnostic(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("eslint(no-is-mounted): Do not use isMounted")
        .with_help("isMounted is on its way to being officially deprecated. You can use a _isMounted property to track the mounted status yourself.")
        .with_label(span0)
}

#[derive(Debug, Default, Clone)]
pub struct NoIsMounted;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule prevents using isMounted in ES6 classes
    ///
    /// ### Why is this bad?
    ///
    /// isMounted is an anti-pattern, is not available when using ES6 classes,
    /// and it is on its way to being officially deprecated.///
    ///
    /// ### Example
    /// ```javascript
    /// class Hello extends React.Component {
    ///     someMethod() {
    ///         if (!this.isMounted()) {
    ///             return;
    ///         }
    ///     }
    ///     render() {
    ///         return <div onClick={this.someMethod.bind(this)}>Hello</div>;
    ///     }
    /// };
    /// ```
    NoIsMounted,
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
            || !member_expr.static_property_name().is_some_and(|str| str == "isMounted")
        {
            return;
        }

        for ancestor in ctx.nodes().ancestors(node.id()).skip(1) {
            if matches!(
                ctx.nodes().kind(ancestor),
                AstKind::ObjectProperty(_) | AstKind::MethodDefinition(_)
            ) {
                ctx.diagnostic(no_is_mounted_diagnostic(call_expr.span));
                break;
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

    Tester::new(NoIsMounted::NAME, pass, fail).test_and_snapshot();
}
