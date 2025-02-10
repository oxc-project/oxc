use oxc_ast::{ast::Expression, AstKind};
use oxc_macros::declare_oxc_lint;
use oxc_semantic::SymbolId;
use oxc_span::{GetSpan, Span};

use crate::utils::ReactPerfRule;

#[derive(Debug, Default, Clone)]
pub struct JsxNoJsxAsProp;

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
    /// ### Example
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
    perf
);

impl ReactPerfRule for JsxNoJsxAsProp {
    const MESSAGE: &'static str = "JSX attribute values should not contain other JSX.";

    fn check_for_violation_on_expr(&self, expr: &Expression<'_>) -> Option<Span> {
        check_expression(expr)
    }

    fn check_for_violation_on_ast_kind(
        &self,
        kind: &AstKind<'_>,
        _symbol_id: SymbolId,
    ) -> Option<(/* decl */ Span, /* init */ Option<Span>)> {
        let decl = kind.as_variable_declarator()?;
        let init_span = decl.init.as_ref().and_then(check_expression)?;
        Some((decl.id.span(), Some(init_span)))
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
    use crate::tester::Tester;

    let pass = vec![
        r"<Item callback={this.props.jsx} />",
        r"const Foo = () => <Item callback={this.props.jsx} />",
        r"<Item jsx={<SubItem />} />",
        r"<Item jsx={this.props.jsx || <SubItem />} />",
        r"<Item jsx={this.props.jsx ? this.props.jsx : <SubItem />} />",
        r"<Item jsx={this.props.jsx || (this.props.component ? this.props.component : <SubItem />)} />",
        r"const Icon = <svg />; const Foo = () => (<IconButton icon={Icon} />)",
    ];

    let fail = vec![
        r"const Foo = () => (<Item jsx={<SubItem />} />)",
        r"const Foo = () => (<Item jsx={this.props.jsx || <SubItem />} />)",
        r"const Foo = () => (<Item jsx={this.props.jsx ? this.props.jsx : <SubItem />} />)",
        r"const Foo = () => (<Item jsx={this.props.jsx || (this.props.component ? this.props.component : <SubItem />)} />)",
        r"const Foo = () => { const Icon = <svg />; return (<IconButton icon={Icon} />) }",
    ];

    Tester::new(JsxNoJsxAsProp::NAME, JsxNoJsxAsProp::PLUGIN, pass, fail)
        .with_react_perf_plugin(true)
        .test_and_snapshot();
}
