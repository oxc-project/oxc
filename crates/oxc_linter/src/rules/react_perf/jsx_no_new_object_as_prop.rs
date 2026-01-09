use oxc_ast::{AstKind, ast::BindingPattern, ast::Expression};
use oxc_macros::declare_oxc_lint;
use oxc_semantic::SymbolId;
use oxc_span::{GetSpan, Span};

use crate::{
    ast_util::is_method_call,
    utils::{ReactPerfRule, find_initialized_binding, is_constructor_matching_name},
};

#[derive(Debug, Default, Clone)]
pub struct JsxNoNewObjectAsProp;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prevent Objects that are local to the current method from being used
    /// as values of JSX props.
    ///
    /// ### Why is this bad?
    ///
    /// Using locally defined Objects as values for props can lead to unintentional
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
    perf
);

impl ReactPerfRule for JsxNoNewObjectAsProp {
    const MESSAGE: &'static str =
        "JSX attribute values should not contain objects created in the same scope.";

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
                match &decl.id {
                    BindingPattern::ObjectPattern(pattern) => {
                        // Check for default values in object pattern properties
                        // e.g., `const { a = {} } = value ?? {};`
                        pattern
                            .properties
                            .iter()
                            .find_map(|prop| process_pattern(&prop.value, symbol_id))
                    }
                    BindingPattern::ArrayPattern(pattern) => {
                        // Check for default values in array pattern elements
                        // e.g., `const [a = {}] = value ?? {};`
                        pattern
                            .elements
                            .iter()
                            .flatten()
                            .find_map(|ele| process_pattern(ele, symbol_id))
                    }
                    _ => {
                        // For non-destructuring assignments, check the initialization expression
                        if let Some(init_span) = decl.init.as_ref().and_then(check_expression) {
                            return Some((decl.id.span(), Some(init_span)));
                        }
                        None
                    }
                }
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

fn process_pattern<'a>(
    pattern: &'a BindingPattern<'a>,
    symbol_id: SymbolId,
) -> Option<(Span, Option<Span>)> {
    find_initialized_binding(pattern, symbol_id).and_then(|(binding_id, default_expr)| {
        check_default_value(default_expr).map(|span| (binding_id.span(), Some(span)))
    })
}

fn check_default_value(expr: &Expression) -> Option<Span> {
    let inner = expr.get_inner_expression();

    if let Expression::ObjectExpression(expr) = inner {
        return Some(expr.span);
    }

    let (callee, span) = match inner {
        Expression::CallExpression(e) => (Some(&e.callee), e.span),
        Expression::NewExpression(e) => (Some(&e.callee), e.span),
        _ => (None, Span::default()),
    };

    callee.filter(|c| is_constructor_matching_name(c, "Object")).map(|_| span)
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
    use crate::tester::Tester;

    let pass = vec![
        r"<Item config={staticConfig} />",
        r"
            const a = ({ value }) => {
                const { min } = value ?? {}
                return <InputNumber value={min} />
            }
        ",
        r"
            const a = ({ value }) => {
                const { min = [] } = value ?? {}
                return <InputNumber value={min} />
            }
        ",
        r"
            const a = ({ value }) => {
                const { min = getDefault() } = value ?? {}
                return <InputNumber value={min} />
            }
        ",
        r"<Item config={'foo'} />",
        r"const Foo = () => <Item config={staticConfig} />",
        r"const Foo = (props) => <Item {...props} />",
        r"const Foo = (props) => <Item x={props.x} />",
        r"const Foo = ({ x = 5 }) => <Item x={x} />",
        r"const x = {}; const Foo = () => <Bar x={x} />",
        r"const DEFAULT_X = {}; const Foo = ({ x = DEFAULT_X }) => <Bar x={x} />",
        r"
        import { FC, useMemo } from 'react';
        import { Bar } from './bar';
        export const Foo: FC = () => {
            const x = useMemo(() => ({ foo: 'bar' }), []);
            return <Bar prop={x} />
        }
        ",
        r"
        import { FC, useMemo } from 'react';
        import { Bar } from './bar';
        export const Foo: FC = () => {
            const x = useMemo(() => ({ foo: 'bar' }), []);
            const y = x;
            return <Bar prop={y} />
        }
        ",
        // new arr, not an obj
        r"const Foo = () => <Item arr={[]} />",
    ];

    let fail = vec![
        r"
            const a = ({ value }) => {
                const { min = {} } = value;
                return <InputNumber value={min} />
            }
        ",
        r"
            const a = ({ value }) => {
                const { min = Object() } = value;
                return <InputNumber value={min} />
            }
        ",
        r"
            const a = ({ value }) => {
                const { min = new Object() } = value;
                return <InputNumber value={min} />
            }
        ",
        r"const Foo = () => <Item config={{}} />",
        r"const Foo = () => <Item config={Object.create(null)} />",
        r"const Foo = ({ x }) => <Item config={Object.assign({}, x)} />",
        r"const Foo = () => (<Item config={new Object()} />)",
        r"const Foo = () => (<Item config={Object()} />)",
        r"const Foo = () => (<div style={{display: 'none'}} />)",
        r"const Foo = () => (<Item config={this.props.config || {}} />)",
        r"const Foo = () => (<Item config={this.props.config ? this.props.config : {}} />)",
        r"const Foo = () => (<Item config={this.props.config || (this.props.default ? this.props.default : {})} />)",
        r"const Foo = () => { const x = {}; return <Bar x={x} /> }",
        r"const Foo = ({ x = {} }) => <Item x={x} />",
        r"const Foo = () => { const x: Foo = {}; return <Bar x={x} /> }",
        r"const Foo = () => { const x: Foo = {} as Foo; return <Bar x={x} /> }",
        r"const Foo = () => { const x: Foo = {} satisfies Foo; return <Bar x={x} /> }",
        r"const Foo = () => { const x: Foo = {} as const; return <Bar x={x} /> }",
    ];

    Tester::new(JsxNoNewObjectAsProp::NAME, JsxNoNewObjectAsProp::PLUGIN, pass, fail)
        .test_and_snapshot();
}
