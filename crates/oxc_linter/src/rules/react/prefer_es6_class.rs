use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    rule::{DefaultRuleConfig, Rule},
    utils::{is_es5_component, is_es6_component},
};

fn unexpected_es6_class_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Components should use `createReactClass` instead of an ES2015 class.")
        .with_label(span)
}

fn expected_es6_class_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Components should use an ES2015 class instead of `createReactClass`.")
        .with_label(span)
}

#[derive(Debug, Default, Clone, JsonSchema, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum PreferES6ClassOptionType {
    /// Always prefer ES2015 class-style components.
    #[default]
    Always,
    /// Do not allow ES2015 class-style, prefer `createReactClass`.
    Never,
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct PreferEs6Class(PreferES6ClassOptionType);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// React offers you two ways to create traditional components: using the
    /// `create-react-class` package or the newer ES2015 class system.
    ///
    /// Note that function components are preferred over class components in modern React,
    /// and it is _especially_ discouraged to use `createReactClass` in modern React.
    ///
    /// ### Why is this bad?
    ///
    /// This rule enforces a consistent React class style.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule by default:
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
    config = PreferES6ClassOptionType,
);

impl Rule for PreferEs6Class {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::CallExpression(call_expr)
                if matches!(self.0, PreferES6ClassOptionType::Always) && is_es5_component(node) =>
            {
                ctx.diagnostic(expected_es6_class_diagnostic(call_expr.callee.span()));
            }
            AstKind::Class(class_expr)
                if !matches!(self.0, PreferES6ClassOptionType::Always)
                    && is_es6_component(node) =>
            {
                ctx.diagnostic(unexpected_es6_class_diagnostic(
                    class_expr.id.as_ref().map_or(class_expr.span, |id| id.span),
                ));
            }
            _ => {}
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
