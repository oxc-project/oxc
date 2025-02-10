use oxc_ast::{
    ast::{Expression, MemberExpression},
    AstKind,
};
use oxc_macros::declare_oxc_lint;
use oxc_semantic::SymbolId;
use oxc_span::{GetSpan, Span};

use crate::utils::{is_constructor_matching_name, ReactPerfRule};

#[derive(Debug, Default, Clone)]
pub struct JsxNoNewFunctionAsProp;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prevent Functions that are local to the current method from being used
    /// as values of JSX props.
    ///
    /// ### Why is this bad?
    ///
    /// Using locally defined Functions as values for props can lead to unintentional
    /// re-renders and performance issues. Every time the parent component renders,
    /// a new instance of the Function is created, causing unnecessary re-renders
    /// of child components. This also leads to harder-to-maintain code as the
    /// component's props are not passed consistently.
    ///
    /// ### Example
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
    perf
);

impl ReactPerfRule for JsxNoNewFunctionAsProp {
    const MESSAGE: &'static str =
        "JSX attribute values should not contain functions created in the same scope.";

    fn check_for_violation_on_expr(&self, expr: &Expression<'_>) -> Option<Span> {
        check_expression(expr)
    }

    fn check_for_violation_on_ast_kind(
        &self,
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
            if property_name == Some("bind") {
                Some(expr.span)
            } else {
                None
            }
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
    use crate::tester::Tester;

    let pass = vec![
        r"const Foo = () => <Item callback={this.props.callback} />",
        r"const Foo = () => (<Item promise={new Promise()} />)",
        r"const Foo = () => (<Item onClick={bind(foo)} />)",
        r"const Foo = () => (<Item prop={0} />)",
        r"const Foo = () => { var a; return <Item prop={a} /> }",
        r"const Foo = () => { var a;a = 1; return <Item prop={a} /> }",
        r"const Foo = () => { var a;<Item prop={a} /> }",
        r"function foo ({prop1 = function(){}, prop2}) {
            return <Comp prop={prop2} />
          }",
        r"function foo ({prop1, prop2 = function(){}}) {
            return <Comp prop={prop1} />
          }",
        r"<Item prop={function(){return true}} />",
        r"<Item prop={() => true} />",
        r"<Item prop={new Function('a', 'alert(a)')}/>",
        r"<Item prop={Function()}/>",
        r"<Item onClick={this.clickHandler.bind(this)} />",
        r"<Item callback={this.props.callback || function() {}} />",
        r"<Item callback={this.props.callback ? this.props.callback : function() {}} />",
        r"<Item prop={this.props.callback || this.props.callback ? this.props.callback : function(){}} />",
        r"<Item prop={this.props.callback || (this.props.cb ? this.props.cb : function(){})} />",
        r"
        import { FC, useCallback } from 'react';
        export const Foo: FC = props => {
            const onClick = useCallback(
                e => { props.onClick?.(e) },
                [props.onClick]
            );
            return <button onClick={onClick} />
        }",
        r"
        import React from 'react'
        function onClick(e: React.MouseEvent) {
            window.location.navigate(e.target.href)
        }
        export default function Foo() {
            return <a onClick={onClick} />
        }
        ",
    ];

    let fail = vec![
        r"const Foo = () => (<Item prop={function(){return true}} />)",
        r"const Foo = () => (<Item prop={() => true} />)",
        r"const Foo = () => (<Item prop={new Function('a', 'alert(a)')}/>)",
        r"const Foo = () => (<Item prop={Function()}/>)",
        r"const Foo = () => (<Item onClick={this.clickHandler.bind(this)} />)",
        r"const Foo = () => (<Item callback={this.props.callback || function() {}} />)",
        r"const Foo = () => (<Item callback={this.props.callback ? this.props.callback : function() {}} />)",
        r"const Foo = () => (<Item prop={this.props.callback || this.props.callback ? this.props.callback : function(){}} />)",
        r"const Foo = () => (<Item prop={this.props.callback || (this.props.cb ? this.props.cb : function(){})} />)",
        r"
        const Foo = ({ onClick }) => {
            const _onClick = onClick.bind(this)
            return <button onClick={_onClick} />
        }",
        r"
        const Foo = () => {
            function onClick(e) {
                window.location.navigate(e.target.href)
            }
            return <a onClick={onClick} />
        }
        ",
        r"
        const Foo = () => {
            const onClick = (e) => {
                window.location.navigate(e.target.href)
            }
            return <a onClick={onClick} />
        }
        ",
        r"
        const Foo = () => {
            const onClick = function (e) {
                window.location.navigate(e.target.href)
            }
            return <a onClick={onClick} />
        }
        ",
    ];

    Tester::new(JsxNoNewFunctionAsProp::NAME, JsxNoNewFunctionAsProp::PLUGIN, pass, fail)
        .with_react_perf_plugin(true)
        .test_and_snapshot();
}
