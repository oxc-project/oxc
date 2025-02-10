use oxc_ast::{ast::Expression, AstKind};
use oxc_macros::declare_oxc_lint;
use oxc_semantic::SymbolId;
use oxc_span::{GetSpan, Span};

use crate::{
    ast_util::is_method_call,
    utils::{find_initialized_binding, is_constructor_matching_name, ReactPerfRule},
};

#[derive(Debug, Default, Clone)]
pub struct JsxNoNewArrayAsProp;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prevent Arrays that are local to the current method from being used
    /// as values of JSX props.
    ///
    /// ### Why is this bad?
    ///
    /// Using locally defined Arrays as values for props can lead to unintentional
    /// re-renders and performance issues. Every time the parent component renders,
    /// a new instance of the Array is created, causing unnecessary re-renders of
    /// child components. This also leads to harder-to-maintain code as the
    /// component's props are not passed consistently.
    ///
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <Item list={[]} />
    /// <Item list={new Array()} />
    /// <Item list={Array()} />
    /// <Item list={this.props.list || []} />
    /// <Item list={this.props.list ? this.props.list : []} />
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <Item list={this.props.list} />
    /// ```
    JsxNoNewArrayAsProp,
    react_perf,
    perf
);

impl ReactPerfRule for JsxNoNewArrayAsProp {
    const MESSAGE: &'static str =
        "JSX attribute values should not contain Arrays created in the same scope.";

    fn check_for_violation_on_expr(&self, expr: &Expression<'_>) -> Option<Span> {
        check_expression(expr)
    }

    fn check_for_violation_on_ast_kind(
        &self,
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

fn check_expression(expr: &Expression) -> Option<Span> {
    match expr.without_parentheses() {
        Expression::ArrayExpression(expr) => Some(expr.span),
        Expression::CallExpression(expr) => {
            if is_constructor_matching_name(&expr.callee, "Array")
                || is_method_call(
                    expr.as_ref(),
                    None,
                    Some(&["concat", "map", "filter"]),
                    Some(1),
                    Some(1),
                )
            {
                Some(expr.span)
            } else {
                None
            }
        }
        Expression::NewExpression(expr) => {
            if is_constructor_matching_name(&expr.callee, "Array") {
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
        r"<Item list={this.props.list} />",
        r"<Item list={[]} />",
        r"<Item list={new Array()} />",
        r"<Item list={Array()} />",
        r"<Item list={this.props.list || []} />",
        r"<Item list={this.props.list ? this.props.list : []} />",
        r"<Item list={this.props.list || (this.props.arr ? this.props.arr : [])} />",
        r"const Foo = () => <Item list={this.props.list} />",
        r"const x = []; const Foo = () => <Item list={x} />",
        r"const DEFAULT_X = []; const Foo = ({ x = DEFAULT_X }) => <Item list={x} />",
    ];

    let fail = vec![
        r"const Foo = () => (<Item list={[]} />)",
        r"const Foo = () => (<Item list={new Array()} />)",
        r"const Foo = () => (<Item list={Array()} />)",
        r"const Foo = () => (<Item list={arr1.concat(arr2)} />)",
        r"const Foo = () => (<Item list={arr1.filter(x => x > 0)} />)",
        r"const Foo = () => (<Item list={arr1.map(x => x * x)} />)",
        r"const Foo = () => (<Item list={this.props.list || []} />)",
        r"const Foo = () => (<Item list={this.props.list ? this.props.list : []} />)",
        r"const Foo = () => (<Item list={this.props.list || (this.props.arr ? this.props.arr : [])} />)",
        r"const Foo = () => { let x = []; return <Item list={x} /> }",
        r"const Foo = ({ x = [] }) => <Item list={x} />",
    ];

    Tester::new(JsxNoNewArrayAsProp::NAME, JsxNoNewArrayAsProp::PLUGIN, pass, fail)
        .with_react_perf_plugin(true)
        .test_and_snapshot();
}
