use oxc_ast::{
    ast::{Argument, CallExpression, Expression, UnaryOperator},
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
    OxcDiagnostic::warn("Prefer `.some(…)` over `.find(…)` or `.findLast(…)`.").with_label(span)
}

fn non_zero_filter(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer `.some(…)` over non-zero length check from `.filter(…)`.")
        .with_label(span)
}

fn negative_one_or_zero_filter(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer `.some(…)` over `.findIndex(…)` or `.findLastIndex(…)`.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferArraySome;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefers using [`Array#some()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/some) over [`Array#find()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/find), [`Array#findLast()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/findLast) with comparing to undefined,
    /// or [`Array#findIndex()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/findIndex), [`Array#findLastIndex()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/findLastIndex)
    /// and a non-zero length check on the result of [`Array#filter()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/filter)
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
    /// const foo = array.findLast(elem => hasRole(elem)) !== null;
    /// foo.findIndex(bar) < 0;
    /// foo.findIndex(element => element.bar === 1) !== -1;
    /// foo.findLastIndex(element => element.bar === 1) !== -1;
    /// array.filter(fn).length === 0;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// const foo = array.some(fn) ? bar : baz;
    /// foo.some(element => element.bar === 1);
    /// !array.some(fn);
    /// ```
    PreferArraySome,
    unicorn,
    pedantic,
    fix
);

/// <https://github.com/sindresorhus/eslint-plugin-unicorn/blob/v56.0.1/docs/rules/prefer-array-some.md>
impl Rule for PreferArraySome {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            // `.find(…)`
            // `.findLast(…)`
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
                // `.{findIndex,findLastIndex}(…) !== -1`
                // `.{findIndex,findLastIndex}(…) != -1`
                // `.{findIndex,findLastIndex}(…) > -1`
                // `.{findIndex,findLastIndex}(…) === -1`
                // `.{findIndex,findLastIndex}(…) == -1`
                // `.{findIndex,findLastIndex}(…) >= 0`
                // `.{findIndex,findLastIndex}(…) < 0`
                let with_negative_one = matches!(
                    bin_expr.operator,
                    BinaryOperator::StrictInequality
                        | BinaryOperator::Inequality
                        | BinaryOperator::GreaterThan
                        | BinaryOperator::StrictEquality
                        | BinaryOperator::Equality
                ) && matches!(
                    bin_expr.right.without_parentheses(),
                    Expression::UnaryExpression(_)
                );

                let matches_against_zero = matches!(
                    bin_expr.operator,
                    BinaryOperator::GreaterEqualThan | BinaryOperator::LessThan
                );

                if with_negative_one {
                    if let Expression::UnaryExpression(right_unary_expr) =
                        &bin_expr.right.without_parentheses()
                    {
                        if matches!(right_unary_expr.operator, UnaryOperator::UnaryNegation)
                            && right_unary_expr.argument.is_number_literal()
                            && right_unary_expr.argument.is_number_value(1_f64)
                        {
                            let Expression::CallExpression(left_call_expr) =
                                &bin_expr.left.without_parentheses()
                            else {
                                return;
                            };

                            let Some(argument) = left_call_expr.arguments.first() else {
                                return;
                            };

                            if matches!(argument, Argument::SpreadElement(_)) {
                                return;
                            }

                            if is_method_call(
                                left_call_expr,
                                None,
                                Some(&["findIndex", "findLastIndex"]),
                                None,
                                Some(1),
                            ) {
                                // TODO: fixer
                                ctx.diagnostic(negative_one_or_zero_filter(
                                    call_expr_method_callee_info(left_call_expr).unwrap().0,
                                ));
                            }
                        }
                    }
                }

                if matches_against_zero {
                    let Expression::NumericLiteral(right_num_lit) = &bin_expr.right else {
                        return;
                    };

                    let Expression::CallExpression(left_call_expr) =
                        &bin_expr.left.without_parentheses()
                    else {
                        return;
                    };

                    if right_num_lit.raw.as_ref().unwrap().as_str() == "0"
                        && is_method_call(
                            left_call_expr,
                            None,
                            Some(&["findIndex", "findLastIndex"]),
                            None,
                            Some(1),
                        )
                    {
                        // TODO: fixer
                        ctx.diagnostic(negative_one_or_zero_filter(
                            call_expr_method_callee_info(left_call_expr).unwrap().0,
                        ));
                    }
                }

                // `.filter(…).length > 0`
                // `.filter(…).length !== 0`
                if !matches!(
                    bin_expr.operator,
                    BinaryOperator::GreaterThan | BinaryOperator::StrictInequality
                ) {
                    return;
                }

                let Expression::NumericLiteral(right_num_lit) = &bin_expr.right else {
                    return;
                };

                if right_num_lit.raw.as_ref().unwrap() != "0" {
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
        // findIndex: negative one
        r"foo.notMatchedMethod(bar) !== -1",
        r"new foo.findIndex(bar) !== -1",
        r"foo.findIndex(bar, extraArgument) !== -1",
        r"foo.findIndex(bar) instanceof -1",
        r"foo.findIndex(...bar) !== -1",
        // findLastIndex: negative one
        r"new foo.findLastIndex(bar) !== -1",
        r"foo.findLastIndex(bar, extraArgument) !== -1",
        r"foo.findLastIndex(bar) instanceof -1",
        r"foo.findLastIndex(...bar) !== -1",
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
        // findIndex: negative one || ( >= || < ) 0
        r"foo.findIndex(bar) !== -1",
        r"foo.findIndex(bar) != -1",
        r"foo.findIndex(bar) > - 1",
        r"foo.findIndex(bar) === -1",
        r"foo.findIndex(bar) == - 1",
        r"foo.findIndex(bar) >= 0",
        r"foo.findIndex(bar) < 0",
        r"foo.findIndex(bar) !== (( - 1 ))",
        r"foo.findIndex(element => element.bar === 1) !== (( - 1 ))",
        // findLastIndex: negative one || ( >= || < ) 0
        r"foo.findLastIndex(bar) !== -1",
        r"foo.findLastIndex(bar) != -1",
        r"foo.findLastIndex(bar) > - 1",
        r"foo.findLastIndex(bar) === -1",
        r"foo.findLastIndex(bar) == - 1",
        r"foo.findLastIndex(bar) >= 0",
        r"foo.findLastIndex(bar) < 0",
        r"foo.findLastIndex(bar) !== (( - 1 ))",
        r"foo.findLastIndex(element => element.bar === 1) !== (( - 1 ))",
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

    Tester::new(PreferArraySome::NAME, PreferArraySome::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
