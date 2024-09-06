use oxc_ast::{
    ast::{Argument, CallExpression, Expression},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::operator::BinaryOperator;

use crate::{
    ast_util::{call_expr_method_callee_info, is_method_call, outermost_paren_parent},
    context::LintContext,
    rule::Rule,
    utils::is_boolean_node,
    AstNode,
};

fn over_method(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer `.some(…)` over `.find(…)`or `.findLast(…)`.").with_label(span)
}

fn non_zero_filter(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer `.some(…)` over non-zero length check from `.filter(…)`.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferArraySome;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefers using [`Array#some`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/some) over [`Array#find()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/find), [`Array#findLast()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/findLast) and a non-zero length check on the result of [`Array#filter()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/filter)
    ///
    /// ### Why is this bad?
    ///
    /// Using `.some()` is more idiomatic and easier to read.
    ///
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// const foo = array.find(fn) ? bar : baz;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// const foo = array.some(fn) ? bar : baz;
    /// ```
    PreferArraySome,
    pedantic,
    fix
);

impl Rule for PreferArraySome {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::CallExpression(call_expr) => {
                if !is_method_call(call_expr, None, Some(&["find", "findLast"]), Some(1), Some(2)) {
                    return;
                }

                let is_compare = is_checking_undefined(node, call_expr, ctx);
                if !is_compare && !is_boolean_node(node, ctx) {
                    return;
                }

                ctx.diagnostic_with_fix(
                    over_method(
                        // SAFETY: `call_expr_method_callee_info` returns `Some` if `is_method_call` returns `true`.
                        call_expr_method_callee_info(call_expr).unwrap().0,
                    ),
                    |fixer| {
                        let target_span = call_expr
                            .callee
                            .as_member_expression()
                            .and_then(|v| v.static_property_info().map(|(span, _)| span));

                        debug_assert!(target_span.is_some());

                        if let Some(target_span) = target_span {
                            fixer.replace(target_span, "some")
                        } else {
                            fixer.noop()
                        }
                    },
                );
            }
            AstKind::BinaryExpression(bin_expr) => {
                if !matches!(
                    bin_expr.operator,
                    BinaryOperator::GreaterThan | BinaryOperator::StrictInequality
                ) {
                    return;
                }

                let Expression::NumericLiteral(right_num_lit) = &bin_expr.right else {
                    return;
                };

                if right_num_lit.raw != "0" {
                    return;
                }

                let Some(left_member_expr) =
                    bin_expr.left.without_parentheses().as_member_expression()
                else {
                    return;
                };

                let Some(static_property_name) = left_member_expr.static_property_name() else {
                    return;
                };

                if !matches!(static_property_name, "length") {
                    return;
                }

                let Expression::CallExpression(left_call_expr) =
                    &left_member_expr.object().without_parentheses()
                else {
                    return;
                };

                if !is_method_call(left_call_expr, None, Some(&["filter"]), None, None) {
                    return;
                }

                let Some(first_filter_call_arg) =
                    left_call_expr.arguments.first().and_then(Argument::as_expression)
                else {
                    return;
                };

                if is_node_value_not_function(first_filter_call_arg) {
                    return;
                }

                ctx.diagnostic_with_fix(
                    non_zero_filter(
                        // SAFETY: `call_expr_method_callee_info` returns `Some` if `is_method_call` returns `true`.
                        call_expr_method_callee_info(left_call_expr).unwrap().0,
                    ),
                    |fixer| {
                        let target_span = left_call_expr
                            .callee
                            .as_member_expression()
                            .and_then(|v| v.static_property_info().map(|(span, _)| span));

                        debug_assert!(target_span.is_some());

                        if let Some(target_span) = target_span {
                            fixer.replace(target_span, "some")
                        } else {
                            fixer.noop()
                        }
                    },
                );
            }
            _ => {}
        }
    }
}

fn is_node_value_not_function(expr: &Expression) -> bool {
    if matches!(
        expr,
        Expression::ArrayExpression(_)
            | Expression::BinaryExpression(_)
            | Expression::ClassExpression(_)
            | Expression::ObjectExpression(_)
            | Expression::TemplateLiteral(_)
            | Expression::UnaryExpression(_)
            | Expression::UpdateExpression(_)
    ) {
        return true;
    }
    if expr.is_literal() {
        return true;
    }
    if matches!(
        expr,
        Expression::AssignmentExpression(_)
            | Expression::AwaitExpression(_)
            | Expression::LogicalExpression(_)
            | Expression::NewExpression(_)
            | Expression::TaggedTemplateExpression(_)
            | Expression::ThisExpression(_)
    ) {
        return true;
    }
    if expr.is_undefined() {
        return true;
    }

    false
}

fn is_checking_undefined<'a, 'b>(
    node: &'b AstNode<'a>,
    _call_expr: &'b CallExpression<'a>,
    ctx: &'b LintContext<'a>,
) -> bool {
    let Some(parent) = outermost_paren_parent(node, ctx) else {
        return false;
    };

    let AstKind::BinaryExpression(bin_expr) = parent.kind() else {
        return false;
    };

    let right_without_paren = bin_expr.right.without_parentheses();

    if matches!(
        bin_expr.operator,
        BinaryOperator::Inequality
            | BinaryOperator::Equality
            | BinaryOperator::StrictInequality
            | BinaryOperator::StrictEquality
    ) && right_without_paren.without_parentheses().is_undefined()
    {
        return true;
    }

    if matches!(bin_expr.operator, BinaryOperator::Inequality | BinaryOperator::Equality)
        && right_without_paren.is_null()
    {
        return true;
    }

    false
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"const bar = foo.find(fn)",
        r"const bar = foo.find(fn) || baz",
        r"if (foo.find(fn) ?? bar) {}",
        r"array.filter(fn).length > 0.",
        r"array.filter(fn).length > .0",
        r"array.filter(fn).length > 0.0",
        r"array.filter(fn).length > 0x00",
        r"array.filter(fn).length < 0",
        r"array.filter(fn).length >= 0",
        r"0 > array.filter(fn).length",
        r"array.filter(fn).length !== 0.",
        r"array.filter(fn).length !== .0",
        r"array.filter(fn).length !== 0.0",
        r"array.filter(fn).length !== 0x00",
        r"array.filter(fn).length != 0",
        r"array.filter(fn).length === 0",
        r"array.filter(fn).length == 0",
        r"array.filter(fn).length = 0",
        r"0 !== array.filter(fn).length",
        r"array.filter(fn).length >= 1",
        r"array.filter(fn).length >= 1.",
        r"array.filter(fn).length >= 1.0",
        r"array.filter(fn).length >= 0x1",
        r"array.filter(fn).length > 1",
        r"array.filter(fn).length < 1",
        r"array.filter(fn).length = 1",
        r"array.filter(fn).length += 1",
        r"1 >= array.filter(fn).length",
        r"array.filter(fn)?.length > 0",
        r"array.filter(fn)[length] > 0",
        r"array.filter(fn).notLength > 0",
        r"array.filter(fn).length() > 0",
        r"+array.filter(fn).length >= 1",
        r"array.filter?.(fn).length > 0",
        r"array?.filter(fn).length > 0",
        r"array.notFilter(fn).length > 0",
        r"array.filter.length > 0",
        r#"$element.filter(":visible").length > 0"#,
        r"foo.find(fn) == 0",
        r#"foo.find(fn) != """#,
        r"foo.find(fn) === null",
        r#"foo.find(fn) !== "null""#,
        r"foo.find(fn) >= undefined",
        r"foo.find(fn) instanceof undefined",
        r#"typeof foo.find(fn) === "undefined""#,
    ];

    let fail = vec![
        r"if (foo.find(fn)) {}",
        r"if (foo.findLast(fn)) {}",
        r#"if (array.find(element => element === "🦄")) {}"#,
        r#"const foo = array.find(element => element === "🦄") ? bar : baz;"#,
        r"array.filter(fn).length > 0",
        r"array.filter(fn).length !== 0",
        r"foo.find(fn) == null",
        r"foo.find(fn) == undefined",
        r"foo.find(fn) === undefined",
        r"foo.find(fn) != null",
        r"foo.find(fn) != undefined",
        r"foo.find(fn) !== undefined",
        r#"a = (( ((foo.find(fn))) == ((null)) )) ? "no" : "yes";"#,
    ];

    let fix = vec![
        (r"if (foo.find(fn)) {}", r"if (foo.some(fn)) {}"),
        (r"if (foo.findLast(fn)) {}", r"if (foo.some(fn)) {}"),
        (
            r#"if (array.find(element => element === "🦄")) {}"#,
            r#"if (array.some(element => element === "🦄")) {}"#,
        ),
        (
            r#"const foo = array.find(element => element === "🦄") ? bar : baz;"#,
            r#"const foo = array.some(element => element === "🦄") ? bar : baz;"#,
        ),
        (r"array.filter(fn).length > 0", r"array.some(fn).length > 0"),
        (r"array.filter(fn).length !== 0", r"array.some(fn).length !== 0"),
        (r"foo.find(fn) == null", r"foo.some(fn) == null"),
        (r"foo.find(fn) == undefined", r"foo.some(fn) == undefined"),
        (r"foo.find(fn) === undefined", r"foo.some(fn) === undefined"),
        (r"foo.find(fn) != null", r"foo.some(fn) != null"),
        (r"foo.find(fn) != undefined", r"foo.some(fn) != undefined"),
        (r"foo.find(fn) !== undefined", r"foo.some(fn) !== undefined"),
        (
            r#"a = (( ((foo.find(fn))) == ((null)) )) ? "no" : "yes";"#,
            r#"a = (( ((foo.some(fn))) == ((null)) )) ? "no" : "yes";"#,
        ),
    ];

    Tester::new(PreferArraySome::NAME, pass, fail).expect_fix(fix).test_and_snapshot();
}
