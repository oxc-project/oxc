use serde::Deserialize;

use oxc_ast::{
    AstKind,
    ast::{Expression, MemberExpression},
};
use oxc_macros::declare_oxc_lint;
use oxc_semantic::SymbolId;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode, LintContext,
    context::ContextHost,
    rule::Rule,
    utils::{
        NativeAllowList, ReactPerfConfig, is_constructor_matching_name,
        react_perf_from_configuration, run_react_perf_rule, should_run_react_perf,
    },
};

#[derive(Debug, Default, Clone, Deserialize)]
pub struct JsxNoNewFunctionAsProp(Box<ReactPerfConfig>);

impl std::ops::Deref for JsxNoNewFunctionAsProp {
    type Target = ReactPerfConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prevent functions that are local to the current method from being used
    /// as values of JSX props.
    ///
    /// ### Why is this bad?
    ///
    /// Using locally defined functions as values for props can lead to unintentional
    /// re-renders and performance issues. Every time the parent component renders,
    /// a new instance of the function is created, causing unnecessary re-renders
    /// of child components. This also leads to harder-to-maintain code as the
    /// component's props are not passed consistently.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <Item callback={new Function(...)} />
    /// <Item callback={this.props.callback || function() {}} />
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <Item callback={this.props.callback} />
    /// ```
    JsxNoNewFunctionAsProp,
    react_perf,
    perf,
    config = ReactPerfConfig,
    version = "0.2.3",
    short_description = "Prevent functions that are local to the current method from being used as values of JSX props.",
);

impl JsxNoNewFunctionAsProp {
    const MESSAGE: &'static str =
        "JSX attribute values should not contain functions created in the same scope.";

    fn native_allow_list(&self) -> &NativeAllowList {
        self.0.native_allow_list()
    }

    fn check_for_violation_on_expr(expr: &Expression<'_>) -> Option<Span> {
        check_expression(expr)
    }

    fn check_for_violation_on_ast_kind(
        kind: &AstKind<'_>,
        _symbol_id: SymbolId,
    ) -> Option<(/* decl */ Span, /* init */ Option<Span>)> {
        match kind {
            AstKind::VariableDeclarator(decl)
                if decl.init.as_ref().and_then(check_expression).is_some() =>
            {
                // don't report init span, b/c thats usually an arrow
                // function expression which gets quite large. It also
                // doesn't add any value.
                Some((decl.id.span(), None))
            }
            AstKind::Function(f) => Some((f.id.as_ref().map_or(f.span, GetSpan::span), None)),
            _ => None,
        }
    }
}

impl Rule for JsxNoNewFunctionAsProp {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        react_perf_from_configuration(value)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXAttribute(attr) = node.kind() else {
            return;
        };

        run_react_perf_rule(
            attr,
            node.scope_id(),
            ctx,
            Self::MESSAGE,
            self.native_allow_list(),
            Self::check_for_violation_on_expr,
            Self::check_for_violation_on_ast_kind,
        );
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        should_run_react_perf(ctx)
    }
}

fn check_expression(expr: &Expression) -> Option<Span> {
    match expr.without_parentheses() {
        Expression::ArrowFunctionExpression(expr) => Some(expr.span),
        Expression::FunctionExpression(expr) => Some(expr.span),
        Expression::CallExpression(expr) => {
            if is_constructor_matching_name(&expr.callee, "Function") {
                return Some(expr.span);
            }

            let member_expr = expr.callee.as_member_expression()?;
            let property_name = MemberExpression::static_property_name(member_expr);
            if property_name == Some("bind") { Some(expr.span) } else { None }
        }
        Expression::NewExpression(expr) => {
            if is_constructor_matching_name(&expr.callee, "Function") {
                Some(expr.span)
            } else {
                None
            }
        }
        Expression::LogicalExpression(expr) => {
            check_expression(&expr.left).or_else(|| check_expression(&expr.right))
        }
        Expression::ConditionalExpression(expr) => {
            check_expression(&expr.consequent).or_else(|| check_expression(&expr.alternate))
        }
        _ => None,
    }
}

#[test]
fn test() {
    use serde_json::json;

    use crate::tester::Tester;

    let pass = vec![
        (r"const Foo = () => <Item callback={this.props.callback} />", None),
        (r"const Foo = () => (<Item promise={new Promise()} />)", None),
        (r"const Foo = () => (<Item onClick={bind(foo)} />)", None),
        (r"const Foo = () => (<Item prop={0} />)", None),
        (r"const Foo = () => { var a; return <Item prop={a} /> }", None),
        (r"const Foo = () => { var a;a = 1; return <Item prop={a} /> }", None),
        (r"const Foo = () => { var a;<Item prop={a} /> }", None),
        (
            r"function foo ({prop1 = function(){}, prop2}) {
            return <Comp prop={prop2} />
          }",
            None,
        ),
        (
            r"function foo ({prop1, prop2 = function(){}}) {
            return <Comp prop={prop1} />
          }",
            None,
        ),
        (r"<Item prop={function(){return true}} />", None),
        (r"<Item prop={() => true} />", None),
        (r"<Item prop={new Function('a', 'alert(a)')}/>", None),
        (r"<Item prop={Function()}/>", None),
        (r"<Item onClick={this.clickHandler.bind(this)} />", None),
        (r"<Item callback={this.props.callback || function() {}} />", None),
        (r"<Item callback={this.props.callback ? this.props.callback : function() {}} />", None),
        (
            r"<Item prop={this.props.callback || this.props.callback ? this.props.callback : function(){}} />",
            None,
        ),
        (
            r"<Item prop={this.props.callback || (this.props.cb ? this.props.cb : function(){})} />",
            None,
        ),
        (
            r"
        import { FC, useCallback } from 'react';
        export const Foo: FC = props => {
            const onClick = useCallback(
                e => { props.onClick?.(e) },
                [props.onClick]
            );
            return <button onClick={onClick} />
        }",
            None,
        ),
        (
            r"
        import React from 'react'
        function onClick(e: React.MouseEvent) {
            window.location.navigate(e.target.href)
        }
        export default function Foo() {
            return <a onClick={onClick} />
        }
        ",
            None,
        ),
        (
            r"const Foo = () => <button onClick={() => true} />",
            Some(json!([{ "nativeAllowList": "all" }])),
        ),
        (
            r"const Foo = () => <button onClick={() => true} />",
            Some(json!([{ "nativeAllowList": ["onClick"] }])),
        ),
    ];

    let fail = vec![
        (r"const Foo = () => (<Item prop={function(){return true}} />)", None),
        (r"const Foo = () => (<Item prop={() => true} />)", None),
        (r"const Foo = () => (<Item prop={new Function('a', 'alert(a)')}/>)", None),
        (r"const Foo = () => (<Item prop={Function()}/>)", None),
        (r"const Foo = () => (<Item onClick={this.clickHandler.bind(this)} />)", None),
        (r"const Foo = () => (<Item callback={this.props.callback || function() {}} />)", None),
        (
            r"const Foo = () => (<Item callback={this.props.callback ? this.props.callback : function() {}} />)",
            None,
        ),
        (
            r"const Foo = () => (<Item prop={this.props.callback || this.props.callback ? this.props.callback : function(){}} />)",
            None,
        ),
        (
            r"const Foo = () => (<Item prop={this.props.callback || (this.props.cb ? this.props.cb : function(){})} />)",
            None,
        ),
        (
            r"
        const Foo = ({ onClick }) => {
            const _onClick = onClick.bind(this)
            return <button onClick={_onClick} />
        }",
            None,
        ),
        (
            r"
        const Foo = () => {
            function onClick(e) {
                window.location.navigate(e.target.href)
            }
            return <a onClick={onClick} />
        }
        ",
            None,
        ),
        (
            r"
        const Foo = () => {
            const onClick = (e) => {
                window.location.navigate(e.target.href)
            }
            return <a onClick={onClick} />
        }
        ",
            None,
        ),
        (
            r"
        const Foo = () => {
            const onClick = function (e) {
                window.location.navigate(e.target.href)
            }
            return <a onClick={onClick} />
        }
        ",
            None,
        ),
        (
            r"const Foo = () => <button onClick={() => true} />",
            Some(json!([{ "nativeAllowList": ["onChange"] }])),
        ),
        // components are never exempt, even with `"all"`
        (
            r"const Foo = () => <Item onClick={() => true} />",
            Some(json!([{ "nativeAllowList": "all" }])),
        ),
    ];

    Tester::new(JsxNoNewFunctionAsProp::NAME, JsxNoNewFunctionAsProp::PLUGIN, pass, fail)
        .test_and_snapshot();
}
