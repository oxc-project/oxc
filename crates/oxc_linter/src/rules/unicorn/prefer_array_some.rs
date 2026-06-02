use oxc_ast::{
    AstKind,
    ast::{Argument, BinaryExpression, Expression, UnaryOperator},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::BinaryOperator;

use crate::{
    AstNode,
    ast_util::{call_expr_method_callee_info, is_method_call, outermost_paren_parent},
    context::LintContext,
    rule::Rule,
    utils::is_boolean_node,
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
    /// Prefers using [`Array#some()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/some) over [`Array#find()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/find), [`Array#findLast()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/findLast) with comparing to `undefined`,
    /// or [`Array#findIndex()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/findIndex), [`Array#findLastIndex()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/findLastIndex)
    /// and a non-zero length check on the result of [`Array#filter()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/filter)
    ///
    /// ### Why is this bad?
    ///
    /// Using `.some()` is more idiomatic and easier to read.
    ///
    /// ### Examples
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
    suggestion,
    version = "0.0.18",
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

                let nullish_comparison = find_nullish_comparison_parent(node, ctx);

                if nullish_comparison.is_none() && !is_boolean_node(node, ctx) {
                    return;
                }

                ctx.diagnostic_with_suggestion(
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

                        if let (Some(target_span), Some((bin_expr, should_negate))) =
                            (target_span, nullish_comparison)
                        {
                            let mut replacement =
                                bin_expr.left.span().source_text(ctx.source_text()).to_string();
                            let replacement_start =
                                (target_span.start - bin_expr.left.span().start) as usize;
                            let replacement_end =
                                (target_span.end - bin_expr.left.span().start) as usize;
                            replacement.replace_range(replacement_start..replacement_end, "some");

                            if should_negate {
                                replacement.insert(0, '!');
                            }

                            fixer.replace(bin_expr.span, replacement)
                        } else if let Some(target_span) = target_span {
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

                if with_negative_one
                    && let Expression::UnaryExpression(right_unary_expr) =
                        &bin_expr.right.without_parentheses()
                    && matches!(right_unary_expr.operator, UnaryOperator::UnaryNegation)
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

                ctx.diagnostic_with_suggestion(
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

                        let Some(target_span) = target_span else {
                            return fixer.noop();
                        };

                        // Replace `filter` with `some` and delete `.length > 0` or `.length !== 0`
                        let multi_fixer = fixer.for_multifix();
                        let mut multi_fix = multi_fixer.new_fix_with_capacity(2);
                        multi_fix.push(multi_fixer.replace(target_span, "some"));
                        multi_fix.push(
                            multi_fixer.delete_range(Span::new(
                                left_call_expr.span.end,
                                bin_expr.span.end,
                            )),
                        );
                        multi_fix.with_message("Replace `.filter(…).length` with `.some(…)`")
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

fn find_nullish_comparison_parent<'a, 'b>(
    node: &'b AstNode<'a>,
    ctx: &'b LintContext<'a>,
) -> Option<(&'b BinaryExpression<'a>, bool)> {
    let parent = outermost_paren_parent(node, ctx)?;

    let AstKind::BinaryExpression(bin_expr) = parent.kind() else {
        return None;
    };

    let right_without_paren = bin_expr.right.without_parentheses();

    if right_without_paren.without_parentheses().is_undefined() {
        return match bin_expr.operator {
            BinaryOperator::Equality | BinaryOperator::StrictEquality => Some((bin_expr, true)),
            BinaryOperator::Inequality | BinaryOperator::StrictInequality => {
                Some((bin_expr, false))
            }
            _ => None,
        };
    }

    if right_without_paren.is_null() {
        return match bin_expr.operator {
            BinaryOperator::Equality => Some((bin_expr, true)),
            BinaryOperator::Inequality => Some((bin_expr, false)),
            _ => None,
        };
    }

    None
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // Not `boolean`
        "const bar = foo.find(fn)",
        "const bar = foo.find(fn) || baz",
        "if (foo.find(fn) ?? bar) {}",
        // Not matched `CallExpression` — find
        "if (new foo.find(fn)) {}",
        "if (find(fn)) {}",
        // TODO: Get these passing.
        // r#"if (foo["find"](fn)) {}"#,
        r#"if (foo["fi" + "nd"](fn) /* find */) {}"#, // spellchecker:disable-line
        // TODO: Get these passing.
        // "if (foo[`find`](fn)) {}",
        "if (foo[find](fn)) {}",
        "if (foo.notFind(fn) /* find */) {}",
        "if (foo.find()) {}",
        "if (foo.find(fn, thisArgument, extraArgument)) {}",
        // TODO: Get these passing.
        // "if (foo.find(...argumentsArray)) {}",
        // Not matched `CallExpression` — findLast
        "if (new foo.findLast(fn)) {}",
        "if (findLast(fn)) {}",
        // TODO: Get these passing.
        // r#"if (foo["findLast"](fn)) {}"#,
        r#"if (foo["fi" + "nd"](fn) /* findLast */) {}"#, // spellchecker:disable-line
        // TODO: Get these passing.
        // "if (foo[`findLast`](fn)) {}",
        "if (foo[findLast](fn)) {}",
        "if (foo.notFind(fn) /* findLast */) {}",
        "if (foo.findLast()) {}",
        "if (foo.findLast(fn, thisArgument, extraArgument)) {}",
        // TODO: Get these passing.
        // "if (foo.findLast(...argumentsArray)) {}",
        // .filter(…).length > 0
        "array.filter(fn).length > 0.",
        "array.filter(fn).length > .0",
        "array.filter(fn).length > 0.0",
        "array.filter(fn).length > 0x00",
        "array.filter(fn).length < 0",
        "array.filter(fn).length >= 0",
        "0 > array.filter(fn).length",
        // .filter(…).length !== 0
        "array.filter(fn).length !== 0.",
        "array.filter(fn).length !== .0",
        "array.filter(fn).length !== 0.0",
        "array.filter(fn).length !== 0x00",
        "array.filter(fn).length != 0",
        "array.filter(fn).length === 0",
        "array.filter(fn).length == 0",
        "array.filter(fn).length = 0",
        "0 !== array.filter(fn).length",
        // .filter(…).length >= 1
        "array.filter(fn).length >= 1",
        "array.filter(fn).length >= 1.",
        "array.filter(fn).length >= 1.0",
        "array.filter(fn).length >= 0x1",
        "array.filter(fn).length > 1",
        "array.filter(fn).length < 1",
        "array.filter(fn).length = 1",
        "array.filter(fn).length += 1",
        "1 >= array.filter(fn).length",
        // .length
        "array.filter(fn)?.length > 0",
        "array.filter(fn)[length] > 0",
        "array.filter(fn).notLength > 0",
        "array.filter(fn).length() > 0",
        "+array.filter(fn).length >= 1",
        // .filter
        "array.filter?.(fn).length > 0",
        "array?.filter(fn).length > 0",
        "array.notFilter(fn).length > 0",
        "array.filter.length > 0",
        // jQuery#filter
        r#"$element.filter(":visible").length > 0"#,
        // Compare with `undefined`
        "foo.find(fn) == 0",
        r#"foo.find(fn) != """#,
        "foo.find(fn) === null",
        r#"foo.find(fn) !== "null""#,
        "foo.find(fn) >= undefined",
        "foo.find(fn) instanceof undefined",
        r#"typeof foo.find(fn) === "undefined""#,
        // findIndex: negative one
        "foo.notMatchedMethod(bar) !== -1",
        "new foo.findIndex(bar) !== -1",
        "foo.findIndex(bar, extraArgument) !== -1",
        "foo.findIndex(bar) instanceof -1",
        "foo.findIndex(...bar) !== -1",
        // findLastIndex: negative one
        "new foo.findLastIndex(bar) !== -1",
        "foo.findLastIndex(bar, extraArgument) !== -1",
        "foo.findLastIndex(bar) instanceof -1",
        "foo.findLastIndex(...bar) !== -1",
        // lodash/underscore findIndex
        "_.findIndex(bar)",
        "_.findIndex(foo, bar)",
    ];

    let fail = vec![
        // find — boolean contexts
        "const bar = !foo.find(fn)",
        // TODO: Get this working. Boolean() is not recognized as a boolean context by Oxlint
        // "const bar = Boolean(foo.find(fn))",
        "if (foo.find(fn)) {}",
        "const bar = foo.find(fn) ? 1 : 2",
        "while (foo.find(fn)) foo.shift();",
        "do {foo.shift();} while (foo.find(fn));",
        "for (; foo.find(fn); ) foo.shift();",
        // findLast — boolean contexts
        "const bar = !foo.findLast(fn)",
        // TODO: Get this working. Boolean() is not recognized as a boolean context by Oxlint
        // "const bar = Boolean(foo.findLast(fn))",
        "if (foo.findLast(fn)) {}",
        "const bar = foo.findLast(fn) ? 1 : 2",
        "while (foo.findLast(fn)) foo.shift();",
        "do {foo.shift();} while (foo.findLast(fn));",
        "for (; foo.findLast(fn); ) foo.shift();",
        // Comments
        "console.log(foo /* comment 1 */ . /* comment 2 */ find /* comment 3 */ (fn) ? a : b)",
        // jQuery.find — always truthy, should not be used as boolean
        r#"if (jQuery.find(".outer > div")) {}"#,
        // Actual messages
        "if (bar.find(fn)) {}",
        "if (bar.findLast(fn)) {}",
        // Snapshot cases
        r#"if (array.find(element => element === "🦄")) {}"#,
        r#"const foo = array.find(element => element === "🦄") ? bar : baz;"#,
        // Chained find — only the outer .find should report
        "
        if (
            array
                .find(element => Array.isArray(element))
            // ^^^^ This should NOT report
                .find(x => x === 0)
            // ^^^^ This should report
        ) {
        }
        ",
        // .filter(…).length
        "array.filter(fn).length > 0",
        "array.filter(fn).length !== 0",
        // TODO: Get this working.
        // "
        // if (
        //     ((
        //         ((
        //             ((
        //                 ((
        //                     array
        //                 ))
        //                     .filter(what_ever_here)
        //             ))
        //                 .length
        //         ))
        //         >
        //         (( 0 ))
        //     ))
        // );
        // ",
        // Compare with `undefined`
        "foo.find(fn) == null",
        "foo.find(fn) == undefined",
        "foo.find(fn) === undefined",
        "foo.find(fn) != null",
        "foo.find(fn) != undefined",
        "foo.find(fn) !== undefined",
        "foo.findLast(fn) == null",
        "foo.findLast(fn) == undefined",
        "foo.findLast(fn) === undefined",
        "foo.findLast(fn) != null",
        "foo.findLast(fn) != undefined",
        "foo.findLast(fn) !== undefined",
        r#"a = (( ((foo.find(fn))) == ((null)) )) ? "no" : "yes";"#,
        // findIndex: negative one || ( >= || < ) 0
        "foo.findIndex(bar) !== -1",
        "foo.findIndex(bar) != -1",
        "foo.findIndex(bar) > - 1",
        "foo.findIndex(bar) === -1",
        "foo.findIndex(bar) == - 1",
        "foo.findIndex(bar) >= 0",
        "foo.findIndex(bar) < 0",
        // findLastIndex: negative one || ( >= || < ) 0
        "foo.findLastIndex(bar) !== -1",
        "foo.findLastIndex(bar) != -1",
        "foo.findLastIndex(bar) > - 1",
        "foo.findLastIndex(bar) === -1",
        "foo.findLastIndex(bar) == - 1",
        "foo.findLastIndex(bar) >= 0",
        "foo.findLastIndex(bar) < 0",
        "foo.findIndex(bar) !== (( - 1 ))",
        "foo.findIndex(element => element.bar === 1) !== (( - 1 ))",
        "foo.findLastIndex(bar) !== (( - 1 ))",
        "foo.findLastIndex(element => element.bar === 1) !== (( - 1 ))",
    ];

    let fix = vec![
        // find — boolean contexts
        ("const bar = !foo.find(fn)", "const bar = !foo.some(fn)"),
        ("if (foo.find(fn)) {}", "if (foo.some(fn)) {}"),
        ("const bar = foo.find(fn) ? 1 : 2", "const bar = foo.some(fn) ? 1 : 2"),
        ("while (foo.find(fn)) foo.shift();", "while (foo.some(fn)) foo.shift();"),
        ("do {foo.shift();} while (foo.find(fn));", "do {foo.shift();} while (foo.some(fn));"),
        ("for (; foo.find(fn); ) foo.shift();", "for (; foo.some(fn); ) foo.shift();"),
        // findLast — boolean contexts
        ("const bar = !foo.findLast(fn)", "const bar = !foo.some(fn)"),
        ("if (foo.findLast(fn)) {}", "if (foo.some(fn)) {}"),
        ("const bar = foo.findLast(fn) ? 1 : 2", "const bar = foo.some(fn) ? 1 : 2"),
        ("while (foo.findLast(fn)) foo.shift();", "while (foo.some(fn)) foo.shift();"),
        ("do {foo.shift();} while (foo.findLast(fn));", "do {foo.shift();} while (foo.some(fn));"),
        ("for (; foo.findLast(fn); ) foo.shift();", "for (; foo.some(fn); ) foo.shift();"),
        // Comments
        (
            "console.log(foo /* comment 1 */ . /* comment 2 */ find /* comment 3 */ (fn) ? a : b)",
            "console.log(foo /* comment 1 */ . /* comment 2 */ some /* comment 3 */ (fn) ? a : b)",
        ),
        // jQuery
        (r#"if (jQuery.find(".outer > div")) {}"#, r#"if (jQuery.some(".outer > div")) {}"#),
        // Actual messages
        ("if (bar.find(fn)) {}", "if (bar.some(fn)) {}"),
        ("if (bar.findLast(fn)) {}", "if (bar.some(fn)) {}"),
        // Snapshot cases
        (
            r#"if (array.find(element => element === "🦄")) {}"#,
            r#"if (array.some(element => element === "🦄")) {}"#,
        ),
        (
            r#"const foo = array.find(element => element === "🦄") ? bar : baz;"#,
            r#"const foo = array.some(element => element === "🦄") ? bar : baz;"#,
        ),
        // .filter(…).length
        ("array.filter(fn).length > 0", "array.some(fn)"),
        ("array.filter(fn).length !== 0", "array.some(fn)"),
        // Compare with `undefined`
        ("foo.find(fn) == null", "!foo.some(fn)"),
        ("foo.find(fn) == undefined", "!foo.some(fn)"),
        ("foo.find(fn) === undefined", "!foo.some(fn)"),
        ("foo.find(fn) != null", "foo.some(fn)"),
        ("foo.find(fn) != undefined", "foo.some(fn)"),
        ("foo.find(fn) !== undefined", "foo.some(fn)"),
        ("foo.findLast(fn) == null", "!foo.some(fn)"),
        ("foo.findLast(fn) == undefined", "!foo.some(fn)"),
        ("foo.findLast(fn) === undefined", "!foo.some(fn)"),
        ("foo.findLast(fn) != null", "foo.some(fn)"),
        ("foo.findLast(fn) != undefined", "foo.some(fn)"),
        ("foo.findLast(fn) !== undefined", "foo.some(fn)"),
        (
            r#"a = (( ((foo.find(fn))) == ((null)) )) ? "no" : "yes";"#,
            r#"a = (( !((foo.some(fn))) )) ? "no" : "yes";"#,
        ),
    ];

    Tester::new(PreferArraySome::NAME, PreferArraySome::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
