use serde::Deserialize;

use oxc_ast::{AstKind, ast::Expression};
use oxc_macros::declare_oxc_lint;
use oxc_semantic::SymbolId;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode, LintContext,
    context::ContextHost,
    rule::Rule,
    utils::{
        NativeAllowList, ReactPerfConfig, react_perf_from_configuration, run_react_perf_rule,
        should_run_react_perf,
    },
};

#[derive(Debug, Default, Clone, Deserialize)]
pub struct JsxNoJsxAsProp(Box<ReactPerfConfig>);

impl std::ops::Deref for JsxNoJsxAsProp {
    type Target = ReactPerfConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prevent JSX elements that are local to the current method from being
    /// used as values of JSX props.
    ///
    /// ### Why is this bad?
    ///
    /// Using locally defined JSX elements as values for props can lead to
    /// unintentional re-renders and performance issues. Every time the parent
    /// renders, a new instance of the JSX element is created, causing unnecessary
    /// re-renders of child components. This also leads to harder-to-maintain code
    /// as the component's props are not passed consistently.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <Item jsx={<SubItem />} />
    /// <Item jsx={this.props.jsx || <SubItem />} />
    /// <Item jsx={this.props.jsx ? this.props.jsx : <SubItem />} />
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <Item callback={this.props.jsx} />
    /// ```
    JsxNoJsxAsProp,
    react_perf,
    perf,
    config = ReactPerfConfig,
    version = "0.2.3",
    short_description = "Prevent JSX elements that are local to the current method from being used as values of JSX props.",
);

impl JsxNoJsxAsProp {
    const MESSAGE: &'static str = "JSX attribute values should not contain other JSX.";

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
        let decl = kind.as_variable_declarator()?;
        let init_span = decl.init.as_ref().and_then(check_expression)?;
        Some((decl.id.span(), Some(init_span)))
    }
}

impl Rule for JsxNoJsxAsProp {
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
        Expression::JSXElement(expr) => Some(expr.span),
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
        (r"<Item callback={this.props.jsx} />", None),
        (r"const Foo = () => <Item callback={this.props.jsx} />", None),
        (r"<Item jsx={<SubItem />} />", None),
        (r"<Item jsx={this.props.jsx || <SubItem />} />", None),
        (r"<Item jsx={this.props.jsx ? this.props.jsx : <SubItem />} />", None),
        (
            r"<Item jsx={this.props.jsx || (this.props.component ? this.props.component : <SubItem />)} />",
            None,
        ),
        (r"const Icon = <svg />; const Foo = () => (<IconButton icon={Icon} />)", None),
        (
            r"const Foo = () => <div jsx={<SubItem />} />",
            Some(json!([{ "nativeAllowList": "all" }])),
        ),
        (
            r"const Foo = () => <div jsx={<SubItem />} />",
            Some(json!([{ "nativeAllowList": ["jsx"] }])),
        ),
    ];

    let fail = vec![
        (r"const Foo = () => (<Item jsx={<SubItem />} />)", None),
        (r"const Foo = () => (<Item jsx={this.props.jsx || <SubItem />} />)", None),
        (r"const Foo = () => (<Item jsx={this.props.jsx ? this.props.jsx : <SubItem />} />)", None),
        (
            r"const Foo = () => (<Item jsx={this.props.jsx || (this.props.component ? this.props.component : <SubItem />)} />)",
            None,
        ),
        (r"const Foo = () => { const Icon = <svg />; return (<IconButton icon={Icon} />) }", None),
        (
            r"const Foo = () => <div jsx={<SubItem />} />",
            Some(json!([{ "nativeAllowList": ["icon"] }])),
        ),
        // components are never exempt, even with `"all"`
        (
            r"const Foo = () => <Item jsx={<SubItem />} />",
            Some(json!([{ "nativeAllowList": "all" }])),
        ),
    ];

    Tester::new(JsxNoJsxAsProp::NAME, JsxNoJsxAsProp::PLUGIN, pass, fail).test_and_snapshot();
}
