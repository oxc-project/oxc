use serde::Deserialize;

use oxc_ast::{AstKind, ast::Expression};
use oxc_macros::declare_oxc_lint;
use oxc_semantic::SymbolId;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode, LintContext,
    ast_util::is_method_call,
    context::ContextHost,
    rule::Rule,
    utils::{
        NativeAllowList, ReactPerfConfig, find_initialized_binding, is_constructor_matching_name,
        react_perf_from_configuration, run_react_perf_rule, should_run_react_perf,
    },
};

#[derive(Debug, Default, Clone, Deserialize)]
pub struct JsxNoNewObjectAsProp(Box<ReactPerfConfig>);

impl std::ops::Deref for JsxNoNewObjectAsProp {
    type Target = ReactPerfConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prevent objects that are local to the current method from being used
    /// as values of JSX props.
    ///
    /// ### Why is this bad?
    ///
    /// Using locally defined objects as values for props can lead to unintentional
    /// re-renders and performance issues. Every time the parent component renders,
    /// a new instance of the Object is created, causing unnecessary re-renders of
    /// child components. This also leads to harder-to-maintain code as the
    /// component's props are not passed consistently.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <Item config={{}} />
    /// <Item config={new Object()} />
    /// <Item config={Object()} />
    /// <Item config={this.props.config || {}} />
    /// <Item config={this.props.config ? this.props.config : {}} />
    /// <div style={{display: 'none'}} />
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <Item config={staticConfig} />
    /// ```
    JsxNoNewObjectAsProp,
    react_perf,
    perf,
    config = ReactPerfConfig,
    version = "0.2.3",
    short_description = "Prevent objects that are local to the current method from being used as values of JSX props.",
);

impl JsxNoNewObjectAsProp {
    const MESSAGE: &'static str =
        "JSX attribute values should not contain objects created in the same scope.";

    fn native_allow_list(&self) -> &NativeAllowList {
        self.0.native_allow_list()
    }

    fn check_for_violation_on_expr(expr: &Expression<'_>) -> Option<Span> {
        check_expression(expr)
    }

    fn check_for_violation_on_ast_kind(
        kind: &AstKind<'_>,
        symbol_id: SymbolId,
    ) -> Option<(/* decl */ Span, /* init */ Option<Span>)> {
        match kind {
            AstKind::VariableDeclarator(decl) => {
                if let Some(init_span) = decl.init.as_ref().and_then(check_expression) {
                    return Some((decl.id.span(), Some(init_span)));
                }
                None
            }
            AstKind::FormalParameter(param) => {
                let (id, init) = find_initialized_binding(&param.pattern, symbol_id)?;
                let init_span = check_expression(init)?;
                Some((id.span(), Some(init_span)))
            }
            _ => None,
        }
    }
}

impl Rule for JsxNoNewObjectAsProp {
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
    match expr.get_inner_expression() {
        Expression::ObjectExpression(expr) => Some(expr.span),
        Expression::CallExpression(expr) => {
            if is_constructor_matching_name(&expr.callee, "Object")
                || is_method_call(
                    expr.as_ref(),
                    Some(&["Object"]),
                    Some(&["assign", "create"]),
                    None,
                    None,
                )
            {
                Some(expr.span)
            } else {
                None
            }
        }
        Expression::NewExpression(expr) => {
            if is_constructor_matching_name(&expr.callee, "Object") {
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
        (r"<Item config={staticConfig} />", None),
        (r"<Item config={{}} />", None),
        (r"<Item config={'foo'} />", None),
        (r"const Foo = () => <Item config={staticConfig} />", None),
        (r"const Foo = (props) => <Item {...props} />", None),
        (r"const Foo = (props) => <Item x={props.x} />", None),
        (r"const Foo = ({ x = 5 }) => <Item x={x} />", None),
        (r"const x = {}; const Foo = () => <Bar x={x} />", None),
        (r"const DEFAULT_X = {}; const Foo = ({ x = DEFAULT_X }) => <Bar x={x} />", None),
        (
            r"
        import { FC, useMemo } from 'react';
        import { Bar } from './bar';
        export const Foo: FC = () => {
            const x = useMemo(() => ({ foo: 'bar' }), []);
            return <Bar prop={x} />
        }
        ",
            None,
        ),
        (
            r"
        import { FC, useMemo } from 'react';
        import { Bar } from './bar';
        export const Foo: FC = () => {
            const x = useMemo(() => ({ foo: 'bar' }), []);
            const y = x;
            return <Bar prop={y} />
        }
        ",
            None,
        ),
        // new arr, not an obj
        (r"const Foo = () => <Item arr={[]} />", None),
        (r"const Foo = () => <div style={{}} />", Some(json!([{ "nativeAllowList": "all" }]))),
        (r"const Foo = () => <div style={{}} />", Some(json!([{ "nativeAllowList": ["style"] }]))),
    ];

    let fail = vec![
        (r"const Foo = () => <Item config={{}} />", None),
        (r"const Foo = () => <Item config={Object.create(null)} />", None),
        (r"const Foo = ({ x }) => <Item config={Object.assign({}, x)} />", None),
        (r"const Foo = () => (<Item config={new Object()} />)", None),
        (r"const Foo = () => (<Item config={Object()} />)", None),
        (r"const Foo = () => (<div style={{display: 'none'}} />)", None),
        (r"const Foo = () => (<Item config={this.props.config || {}} />)", None),
        (r"const Foo = () => (<Item config={this.props.config ? this.props.config : {}} />)", None),
        (
            r"const Foo = () => (<Item config={this.props.config || (this.props.default ? this.props.default : {})} />)",
            None,
        ),
        (r"const Foo = () => { const x = {}; return <Bar x={x} /> }", None),
        (r"const Foo = ({ x = {} }) => <Item x={x} />", None),
        (r"const Foo = () => { const x: Foo = {}; return <Bar x={x} /> }", None),
        (r"const Foo = () => { const x: Foo = {} as Foo; return <Bar x={x} /> }", None),
        (r"const Foo = () => { const x: Foo = {} satisfies Foo; return <Bar x={x} /> }", None),
        (r"const Foo = () => { const x: Foo = {} as const; return <Bar x={x} /> }", None),
        (r"const Foo = () => <div config={{}} />", Some(json!([{ "nativeAllowList": ["style"] }]))),
        // components are never exempt, even with `"all"`
        (r"const Foo = () => <Item style={{}} />", Some(json!([{ "nativeAllowList": "all" }]))),
    ];

    Tester::new(JsxNoNewObjectAsProp::NAME, JsxNoNewObjectAsProp::PLUGIN, pass, fail)
        .test_and_snapshot();
}
