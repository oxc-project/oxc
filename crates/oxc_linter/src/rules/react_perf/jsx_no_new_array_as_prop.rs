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
pub struct JsxNoNewArrayAsProp(Box<ReactPerfConfig>);

impl std::ops::Deref for JsxNoNewArrayAsProp {
    type Target = ReactPerfConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prevent arrays that are local to the current method from being used
    /// as values of JSX props.
    ///
    /// ### Why is this bad?
    ///
    /// Using locally defined arrays as values for props can lead to unintentional
    /// re-renders and performance issues. Every time the parent component renders,
    /// a new instance of the Array is created, causing unnecessary re-renders of
    /// child components. This also leads to harder-to-maintain code as the
    /// component's props are not passed consistently.
    ///
    /// ### Examples
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
    perf,
    config = ReactPerfConfig,
    version = "0.2.3",
    short_description = "Prevent arrays that are local to the current method from being used as values of JSX props.",
);

impl JsxNoNewArrayAsProp {
    const MESSAGE: &'static str =
        "JSX attribute values should not contain Arrays created in the same scope.";

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

impl Rule for JsxNoNewArrayAsProp {
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
    use serde_json::json;

    use crate::tester::Tester;

    let pass = vec![
        (r"<Item list={this.props.list} />", None),
        (r"<Item list={[]} />", None),
        (r"<Item list={new Array()} />", None),
        (r"<Item list={Array()} />", None),
        (r"<Item list={this.props.list || []} />", None),
        (r"<Item list={this.props.list ? this.props.list : []} />", None),
        (r"<Item list={this.props.list || (this.props.arr ? this.props.arr : [])} />", None),
        (r"const Foo = () => <Item list={this.props.list} />", None),
        (r"const x = []; const Foo = () => <Item list={x} />", None),
        (r"const DEFAULT_X = []; const Foo = ({ x = DEFAULT_X }) => <Item list={x} />", None),
        (r"const Foo = () => <div list={[]} />", Some(json!([{ "nativeAllowList": "all" }]))),
        (r"const Foo = () => <div list={[]} />", Some(json!([{ "nativeAllowList": ["list"] }]))),
    ];

    let fail = vec![
        (r"const Foo = () => (<Item list={[]} />)", None),
        (r"const Foo = () => (<Item list={new Array()} />)", None),
        (r"const Foo = () => (<Item list={Array()} />)", None),
        (r"const Foo = () => (<Item list={arr1.concat(arr2)} />)", None),
        (r"const Foo = () => (<Item list={arr1.filter(x => x > 0)} />)", None),
        (r"const Foo = () => (<Item list={arr1.map(x => x * x)} />)", None),
        (r"const Foo = () => (<Item list={this.props.list || []} />)", None),
        (r"const Foo = () => (<Item list={this.props.list ? this.props.list : []} />)", None),
        (
            r"const Foo = () => (<Item list={this.props.list || (this.props.arr ? this.props.arr : [])} />)",
            None,
        ),
        (r"const Foo = () => { let x = []; return <Item list={x} /> }", None),
        (r"const Foo = ({ x = [] }) => <Item list={x} />", None),
        (r"const Foo = () => <div list={[]} />", Some(json!([{ "nativeAllowList": ["style"] }]))),
        // components are never exempt, even with `"all"`
        (r"const Foo = () => <Item list={[]} />", Some(json!([{ "nativeAllowList": "all" }]))),
    ];

    Tester::new(JsxNoNewArrayAsProp::NAME, JsxNoNewArrayAsProp::PLUGIN, pass, fail)
        .test_and_snapshot();
}
