use crate::utils::ReactPerfRule;
use oxc_ast::{ast::Expression, AstKind};
use oxc_macros::declare_oxc_lint;
use oxc_semantic::SymbolId;
use oxc_span::{GetSpan, Span};

#[derive(Debug, Default, Clone)]
pub struct JsxNoJsxAsProp;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prevent JSX that are local to the current method from being used as values of JSX props
    ///
    /// ### Example
    /// ```jsx
    /// // Bad
    /// <Item jsx={<SubItem />} />
    /// <Item jsx={this.props.jsx || <SubItem />} />
    /// <Item jsx={this.props.jsx ? this.props.jsx : <SubItem />} />
    ///
    /// // Good
    /// <Item callback={this.props.jsx} />
    /// ```
    JsxNoJsxAsProp,
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
        let AstKind::VariableDeclarator(decl) = kind else {
            return None;
        };
        let init_span = decl.init.as_ref().and_then(check_expression)?;
        Some((decl.id.span(), Some(init_span)))
    }
}

fn check_expression(expr: &Expression) -> Option<Span> {
    match expr.without_parenthesized() {
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

    Tester::new(JsxNoJsxAsProp::NAME, pass, fail).with_react_perf_plugin(true).test_and_snapshot();
}
