use oxc_ast::{
    AstKind,
    ast::{Class, Expression, ObjectExpression, ObjectPropertyKind},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, Span};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
    utils::is_es5_component,
};

fn require_optimization_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Component is not optimized. Please add a shouldComponentUpdate method.")
        .with_label(span)
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename_all = "camelCase")]
pub struct RequireOptimizationConfig {
    /// Sets the allowed names of decorators. If the variable is present in the chain of decorators, it validates
    ///
    /// Examples of correct code for this rule:
    /// ```jsx
    ///  // ['pureRender']
    /// @pureRender
    /// class Hello extends React.Component {}
    /// ```
    #[serde(rename = "allowDecorators")]
    allow_decorators: Vec<CompactStr>,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
pub struct RequireOptimization(RequireOptimizationConfig);

impl std::ops::Deref for RequireOptimization {
    type Target = RequireOptimizationConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce React components to have a shouldComponentUpdate method
    ///
    /// ### Why is this bad?
    ///
    /// Without `shouldComponentUpdate`, React components will re-render whenever their parent re-renders,
    /// even if their props haven't changed. This can lead to unnecessary re-renders and performance issues.
    /// Implementing `shouldComponentUpdate` or using `PureComponent`/`React.memo` helps optimize rendering performance.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// class YourComponent extends React.Component {
    /// }
    ///
    /// createReactClass({
    /// });
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// class YourComponent extends React.Component {
    ///   shouldComponentUpdate () {
    ///     return false;
    ///   }
    /// }
    ///
    /// createReactClass({
    ///   shouldComponentUpdate: function () {
    ///     return false;
    ///   }
    /// });
    ///
    /// createReactClass({
    ///  mixins: [PureRenderMixin]
    /// });
    ///
    ///  @reactMixin.decorate(PureRenderMixin)
    /// createReactClass({
    ///
    /// });
    /// ```
    RequireOptimization,
    react,
    perf,
    config = RequireOptimizationConfig,
);

impl Rule for RequireOptimization {
    fn from_configuration(value: serde_json::Value) -> Self {
        serde_json::from_value::<DefaultRuleConfig<RequireOptimization>>(value)
            .unwrap_or_default()
            .into_inner()
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::Class(class_node) => {
                if !is_react_class_component(node) {
                    return;
                }

                if has_es6_scu(class_node) {
                    return;
                }

                if has_allowed_decorator(class_node, &self.allow_decorators) {
                    return;
                }

                ctx.diagnostic(require_optimization_diagnostic(class_node.span));
            }
            AstKind::CallExpression(call_expr) => {
                if !is_es5_component(node) {
                    return;
                }

                let Some(arg) = call_expr.arguments.first() else {
                    return;
                };

                let Some(Expression::ObjectExpression(object_expr)) = arg.as_expression() else {
                    return;
                };

                if has_es5_scu(object_expr) {
                    return;
                }

                if has_allowed_mixin(object_expr, &self.allow_decorators) {
                    return;
                }

                ctx.diagnostic(require_optimization_diagnostic(call_expr.span));
            }
            _ => (),
        }
    }
}

/// checks if `React.Component` has `shouldComponentUpdate`
fn has_es6_scu(class: &Class) -> bool {
    class.body.body.iter().any(|prop| {
        prop.property_key().is_some_and(|key| {
            key.static_name().is_some_and(|name| name == "shouldComponentUpdate")
        })
    })
}

/// checks if `createReactClass()` argument has `shouldComponentUpdate`
fn has_es5_scu(obj_expr: &ObjectExpression) -> bool {
    obj_expr.properties.iter().any(|prop| {
        let Some(prop) = prop.as_property() else {
            return false;
        };

        prop.key.name().is_some_and(|name| name == "shouldComponentUpdate")
    })
}

/// checks if `React.Component` has allowed decorator
fn has_allowed_decorator(class: &Class, allowed: &[CompactStr]) -> bool {
    class.decorators.iter().any(|decorator| match decorator.expression.get_inner_expression() {
        Expression::Identifier(ident) => is_allowed_name(ident.name.as_str(), allowed),
        Expression::CallExpression(call_expr) => {
            // @reactMixin.decorate(PureRenderMixin) pattern
            if let Expression::StaticMemberExpression(member) = &call_expr.callee {
                if let Expression::Identifier(obj) = member.object.get_inner_expression() {
                    if obj.name == "reactMixin" && member.property.name == "decorate" {
                        return call_expr
                            .arguments
                            .first()
                            .and_then(|arg| arg.as_expression())
                            .and_then(Expression::get_identifier_reference)
                            .is_some_and(|ident| is_allowed_name(ident.name.as_str(), allowed));
                    }
                }
            }

            let callee_ok = match call_expr.callee.get_inner_expression() {
                Expression::Identifier(ident) => is_allowed_name(ident.name.as_str(), allowed),
                Expression::StaticMemberExpression(member) => {
                    let name = member.static_property_info().1;
                    is_allowed_name(name, allowed)
                }
                _ => false,
            };
            let args_ok = call_expr.arguments.iter().any(|arg| {
                arg.as_expression()
                    .and_then(Expression::get_identifier_reference)
                    .is_some_and(|ident| is_allowed_name(ident.name.as_str(), allowed))
            });
            callee_ok || args_ok
        }
        Expression::StaticMemberExpression(member) => {
            let name = member.static_property_info().1;
            is_allowed_name(name, allowed)
        }
        _ => false,
    })
}

/// checks if `createReactClass` has allowed mixin
fn has_allowed_mixin(obj: &ObjectExpression, allowed: &[CompactStr]) -> bool {
    obj.properties.iter().any(|prop| {
        let ObjectPropertyKind::ObjectProperty(prop) = prop else { return false };
        if prop.key.name().is_none_or(|name| name != "mixins") {
            return false;
        }
        let Expression::ArrayExpression(mixins) = &prop.value else {
            return false;
        };
        mixins.elements.iter().any(|el| {
            el.as_expression()
                .and_then(Expression::get_identifier_reference)
                .is_some_and(|ident| is_allowed_name(ident.name.as_str(), allowed))
        })
    })
}

const BUILTIN_DECORATORS: &[&str] = &["pureRender", "renderPure", "PureRenderMixin"];

fn is_allowed_name(name: &str, allowed: &[CompactStr]) -> bool {
    BUILTIN_DECORATORS.contains(&name) || allowed.iter().any(|s| s == name)
}

/// checks if class is React.Component
fn is_react_class_component(node: &AstNode) -> bool {
    let AstKind::Class(class_expr) = node.kind() else {
        return false;
    };
    if let Some(super_class) = &class_expr.super_class {
        if let Some(member_expr) = super_class.as_member_expression()
            && let Expression::Identifier(ident) = member_expr.object()
        {
            return ident.name == "React"
                && member_expr.static_property_name().is_some_and(|name| name == "Component");
        }

        if let Some(ident_reference) = super_class.get_identifier_reference() {
            return ident_reference.name == "Component";
        }
    }

    false
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "
			        class A {}
			      ",
            None,
        ),
        (
            r#"
			        import React from "react";
			        class YourComponent extends React.Component {
			          shouldComponentUpdate () {}
			        }
			      "#,
            None,
        ),
        (
            r#"
			        import React, {Component} from "react";
			        class YourComponent extends Component {
			          shouldComponentUpdate () {}
			        }
			      "#,
            None,
        ),
        (
            r#"
			        import React, {Component} from "react";
			        @reactMixin.decorate(PureRenderMixin)
			        class YourComponent extends Component {
			          componentDidMount () {}
			          render() {}
			        }
			      "#,
            None,
        ),
        (
            r#"
			        import React from "react";
			        createReactClass({
			          shouldComponentUpdate: function () {}
			        })
			      "#,
            None,
        ),
        (
            r#"
			        import React from "react";
			        createReactClass({
			          mixins: [PureRenderMixin]
			        })
			      "#,
            None,
        ),
        (
            "
			        @reactMixin.decorate(PureRenderMixin)
			        class DecoratedComponent extends Component {}
			      ",
            None,
        ),
        (
            "
			        const FunctionalComponent = function (props) {
			          return <div />;
			        }
			      ",
            None,
        ),
        (
            "
			        function FunctionalComponent(props) {
			          return <div />;
			        }
			      ",
            None,
        ),
        (
            "
			        const FunctionalComponent = (props) => {
			          return <div />;
			        }
			      ",
            None,
        ),
        (
            "
			        @bar
			        @pureRender
			        @foo
			        class DecoratedComponent extends Component {}
			      ",
            Some(serde_json::json!([{ "allowDecorators": ["renderPure", "pureRender"] }])),
        ),
        (
            r#"
			        import React from "react";
			        class YourComponent extends React.PureComponent {}
			      "#,
            Some(serde_json::json!([{ "allowDecorators": ["renderPure", "pureRender"] }])),
        ),
        (
            r#"
			        import React, {PureComponent} from "react";
			        class YourComponent extends PureComponent {}
			      "#,
            Some(serde_json::json!([{ "allowDecorators": ["renderPure", "pureRender"] }])),
        ),
        (
            "
			        const obj = { prop: [,,,,,] }
			      ",
            None,
        ),
        (
            r#"
			        import React from "react";
			        class YourComponent extends React.Component {
			          handleClick = () => {}
			          shouldComponentUpdate(){
			            return true;
			          }
			          render() {
			            return <div onClick={this.handleClick}>123</div>
			          }
			        }
			      "#,
            None,
        ),
        (
            "
			        @bar
			        @pure
			        @randomDecorator
			        class DecoratedComponent extends Component {}
			      ",
            Some(serde_json::json!([{ "allowDecorators": ["randomDecorator"] }])),
        ),
    ];

    let fail = vec![
        (
            r#"
			        import React from "react";
			        class YourComponent extends React.Component {}
			      "#,
            None,
        ),
        (
            r#"
			        import React from "react";
			        class YourComponent extends React.Component {
			          handleClick() {}
			          render() {
			            return <div onClick={this.handleClick}>123</div>
			          }
			        }
			      "#,
            None,
        ),
        (
            r#"
			        import React from "react";
			        class YourComponent extends React.Component {
			          handleClick = () => {}
			          render() {
			            return <div onClick={this.handleClick}>123</div>
			          }
			        }
			      "#,
            None,
        ),
        (
            r#"
			        import React, {Component} from "react";
			        class YourComponent extends Component {}
			      "#,
            None,
        ),
        (
            r#"
			        import React from "react";
			        createReactClass({})
			      "#,
            None,
        ),
        (
            r#"
			        import React from "react";
			        createReactClass({
			          mixins: [RandomMixin]
			        })
			      "#,
            None,
        ),
        (
            "
			        @reactMixin.decorate(SomeOtherMixin)
			        class DecoratedComponent extends Component {}
			      ",
            None,
        ),
        (
            "
			        @bar
			        @pure
			        @foo
			        class DecoratedComponent extends Component {}
			      ",
            Some(serde_json::json!([{ "allowDecorators": ["renderPure", "pureRender"] }])),
        ),
    ];

    Tester::new(RequireOptimization::NAME, RequireOptimization::PLUGIN, pass, fail)
        .test_and_snapshot();
}
