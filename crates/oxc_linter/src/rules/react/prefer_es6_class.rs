use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    utils::{is_es5_component, is_es6_component},
    AstNode,
};

fn unexpected_es6_class_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Components should use createClass instead of ES6 class.").with_label(span)
}

fn expected_es6_class_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Components should use es6 class instead of createClass.").with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferEs6Class {
    prefer_es6_class_option: PreferES6ClassOptionType,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// React offers you two ways to create traditional components: using the ES5
    /// create-react-class module or the new ES6 class system.
    ///
    /// ### Why is this bad?
    ///
    /// This rule enforces a consistent React class style.
    ///
    /// ### Example
    /// ```jsx
    /// var Hello = createReactClass({
    ///   render: function() {
    ///     return <div>Hello {this.props.name}</div>;
    ///   }
    /// });
    /// ```
    PreferEs6Class,
    react,
    style,
);

impl Rule for PreferEs6Class {
    fn from_configuration(value: serde_json::Value) -> Self {
        let obj = value.get(0);

        Self {
            prefer_es6_class_option: obj
                .and_then(serde_json::Value::as_str)
                .map(PreferES6ClassOptionType::from)
                .unwrap_or_default(),
        }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if matches!(self.prefer_es6_class_option, PreferES6ClassOptionType::Always) {
            if is_es5_component(node) {
                let AstKind::CallExpression(call_expr) = node.kind() else {
                    return;
                };
                ctx.diagnostic(expected_es6_class_diagnostic(call_expr.callee.span()));
            }
        } else if is_es6_component(node) {
            let AstKind::Class(class_expr) = node.kind() else {
                return;
            };
            ctx.diagnostic(unexpected_es6_class_diagnostic(
                class_expr.id.as_ref().map_or(class_expr.span, |id| id.span),
            ));
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_jsx()
    }
}

#[derive(Debug, Default, Clone)]
enum PreferES6ClassOptionType {
    #[default]
    Always,
    Never,
}

impl PreferES6ClassOptionType {
    pub fn from(raw: &str) -> Self {
        match raw {
            "always" => Self::Always,
            _ => Self::Never,
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            r"
            class Hello extends React.Component {
              render() {
                return <div>Hello {this.props.name}</div>;
              }
            }
            Hello.displayName = 'Hello'
            ",
            None,
        ),
        (
            r"
            export default class Hello extends React.Component {
              render() {
                return <div>Hello {this.props.name}</div>;
              }
            }
            Hello.displayName = 'Hello'
            ",
            None,
        ),
        (
            r"
            var Hello = 'foo';
            module.exports = {};
            ",
            None,
        ),
        (
            r"
            var Hello = createReactClass({
              render: function() {
                return <div>Hello {this.props.name}</div>;
              }
            });
            ",
            Some(serde_json::json!(["never"])),
        ),
        (
            r"
            class Hello extends React.Component {
              render() {
                return <div>Hello {this.props.name}</div>;
              }
            }
            ",
            Some(serde_json::json!(["always"])),
        ),
    ];

    let fail = vec![
        (
            r"
            var Hello = createReactClass({
              displayName: 'Hello',
              render: function() {
                return <div>Hello {this.props.name}</div>;
              }
            });
            ",
            None,
        ),
        (
            r"
            var Hello = createReactClass({
              render: function() {
                return <div>Hello {this.props.name}</div>;
              }
            });
            ",
            Some(serde_json::json!(["always"])),
        ),
        (
            r"
            class Hello extends React.Component {
              render() {
                  return <div>Hello {this.props.name}</div>;
              }
            }
            ",
            Some(serde_json::json!(["never"])),
        ),
    ];

    Tester::new(PreferEs6Class::NAME, PreferEs6Class::PLUGIN, pass, fail).test_and_snapshot();
}
