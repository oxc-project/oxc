use oxc_ast::{
    ast::{
        Argument, ArrayExpressionElement, BindingPatternKind, CallExpression, Expression, Statement,
    },
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    ast_util::is_method_call,
    context::LintContext,
    rule::Rule,
    utils::{
        get_first_parameter_name, get_return_identifier_name, is_empty_array_expression,
        is_prototype_property,
    },
    AstNode,
};

fn prefer_array_flat_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer Array#flat() over legacy techniques to flatten arrays.")
        .with_help(r"Call `.flat()` on the array instead.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferArrayFlat;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefers `Array#flat()` over legacy techniques to flatten arrays.
    ///
    /// ### Why is this bad?
    ///
    /// ES2019 introduced a new method [`Array#flat()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/flat) that flatten arrays.
    ///
    /// This rule aims to standardize the use of `Array#flat()` over legacy techniques to flatten arrays.
    ///
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// const foo = array.flatMap(x => x);
    /// const foo = array.reduce((a, b) => a.concat(b), []);
    /// const foo = array.reduce((a, b) => [...a, ...b], []);
    /// const foo = [].concat(maybeArray);
    /// const foo = [].concat(...array);
    /// const foo = [].concat.apply([], array);
    /// const foo = Array.prototype.concat.apply([], array);
    /// const foo = Array.prototype.concat.call([], maybeArray);
    /// const foo = Array.prototype.concat.call([], ...array);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// const foo = array.flat();
    /// const foo = [maybeArray].flat();
    /// ```
    PreferArrayFlat,
    unicorn,
    pedantic,
    conditional_fix
);

impl Rule for PreferArrayFlat {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        check_array_flat_map_case(call_expr, ctx);
        check_array_reduce_case(call_expr, ctx);
        check_array_concat_case(call_expr, ctx);
        check_array_prototype_concat_case(call_expr, ctx);
    }
}

// `array.flatMap(x => x)`
fn check_array_flat_map_case<'a>(call_expr: &CallExpression<'a>, ctx: &LintContext<'a>) {
    if !is_method_call(call_expr, None, Some(&["flatMap"]), Some(1), Some(1)) {
        return;
    }

    let Argument::ArrowFunctionExpression(first_argument) = &call_expr.arguments[0] else {
        return;
    };

    if first_argument.r#async || first_argument.params.parameters_count() != 1 {
        return;
    }

    let Some(first_param_name) = get_first_parameter_name(&first_argument.params) else {
        return;
    };

    let Some(return_param_name) = get_return_identifier_name(&first_argument.body) else {
        return;
    };

    if first_param_name != return_param_name {
        return;
    }

    let target_fix_span = call_expr
        .callee
        .as_member_expression()
        .and_then(oxc_ast::ast::MemberExpression::static_property_info)
        .map(|v| Span::new(v.0.start, call_expr.span.end));

    if let Some(span) = target_fix_span {
        ctx.diagnostic_with_fix(prefer_array_flat_diagnostic(call_expr.span), |fixer| {
            fixer.replace(span, "flat()")
        });
    } else {
        ctx.diagnostic(prefer_array_flat_diagnostic(call_expr.span));
    }
}

// `array.reduce((a, b) => a.concat(b), [])`
// `array.reduce((a, b) => [...a, ...b], [])`
fn check_array_reduce_case<'a>(call_expr: &CallExpression<'a>, ctx: &LintContext<'a>) {
    if !is_method_call(call_expr, None, Some(&["reduce"]), Some(2), Some(2)) {
        return;
    }
    let Argument::ArrowFunctionExpression(first_argument) = &call_expr.arguments[0] else {
        return;
    };
    let Some(second_argument) = call_expr.arguments[1].as_expression() else {
        return;
    };

    if first_argument.r#async
        || first_argument.params.parameters_count() != 2
        || !is_empty_array_expression(second_argument)
    {
        return;
    }

    let Some((first_parameter, second_parameter)) = ({
        match (
            &first_argument.params.items[0].pattern.kind,
            &first_argument.params.items[1].pattern.kind,
        ) {
            (
                BindingPatternKind::BindingIdentifier(first_param),
                BindingPatternKind::BindingIdentifier(second_param),
            ) => Some((&first_param.name, &second_param.name)),

            _ => None,
        }
    }) else {
        return;
    };

    let Some(Statement::ExpressionStatement(expr_stmt)) = first_argument.body.statements.first()
    else {
        return;
    };

    // `array.reduce((a, b) => a.concat(b), [])`
    if let Expression::CallExpression(concat_call_expr) = &expr_stmt.expression {
        if is_method_call(concat_call_expr, None, Some(&["concat"]), Some(1), Some(1)) {
            if let Argument::Identifier(first_argument_ident) = &concat_call_expr.arguments[0] {
                if first_argument_ident.name != second_parameter {
                    return;
                }

                let Expression::Identifier(second_argument_ident) =
                    concat_call_expr.callee.get_member_expr().unwrap().object()
                else {
                    return;
                };

                if second_argument_ident.name != first_parameter {
                    return;
                }

                ctx.diagnostic_with_fix(prefer_array_flat_diagnostic(call_expr.span), |fixer| {
                    let target_fix_span = call_expr
                        .callee
                        .as_member_expression()
                        .and_then(oxc_ast::ast::MemberExpression::static_property_info)
                        .map(|v| Span::new(v.0.start, call_expr.span.end));

                    debug_assert!(target_fix_span.is_some());

                    if let Some(span) = target_fix_span {
                        fixer.replace(span, "flat()")
                    } else {
                        fixer.noop()
                    }
                });
            }
        }
    }

    // `array.reduce((a, b) => [...a, ...b], [])`
    if let Expression::ArrayExpression(array_expr) = &expr_stmt.expression {
        if array_expr.elements.len() != 2 {
            return;
        }

        let Some((first_element, second_element)) = ({
            match (&array_expr.elements[0], &array_expr.elements[1]) {
                (
                    ArrayExpressionElement::SpreadElement(first_element),
                    ArrayExpressionElement::SpreadElement(second_element),
                ) => match (&first_element.argument, &second_element.argument) {
                    (
                        Expression::Identifier(first_element),
                        Expression::Identifier(second_element),
                    ) => Some((first_element, second_element)),
                    _ => None,
                },
                _ => None,
            }
        }) else {
            return;
        };

        if first_element.name != first_parameter || second_element.name != second_parameter {
            return;
        }

        ctx.diagnostic_with_fix(prefer_array_flat_diagnostic(call_expr.span), |fixer| {
            let target_fix_span = call_expr
                .callee
                .as_member_expression()
                .and_then(oxc_ast::ast::MemberExpression::static_property_info)
                .map(|v| Span::new(v.0.start, call_expr.span.end));

            debug_assert!(target_fix_span.is_some());

            if let Some(target_fix_span) = target_fix_span {
                fixer.replace(target_fix_span, "flat()")
            } else {
                fixer.noop()
            }
        });
    };
}

// `[].concat(maybeArray)`
// `[].concat(...array)`
fn check_array_concat_case<'a>(call_expr: &CallExpression<'a>, ctx: &LintContext<'a>) {
    if is_method_call(call_expr, None, Some(&["concat"]), Some(1), Some(1)) {
        // `array.concat(maybeArray)`
        if let Expression::ArrayExpression(array_expr) =
            call_expr.callee.get_member_expr().unwrap().object()
        {
            if !array_expr.elements.is_empty() {
                return;
            }
            ctx.diagnostic(prefer_array_flat_diagnostic(call_expr.span));
        }
    }
}

// - `[].concat.apply([], array)` and `Array.prototype.concat.apply([], array)`
// - `[].concat.call([], maybeArray)` and `Array.prototype.concat.call([], maybeArray)`
// - `[].concat.call([], ...array)` and `Array.prototype.concat.call([], ...array)`
fn check_array_prototype_concat_case<'a>(call_expr: &CallExpression<'a>, ctx: &LintContext<'a>) {
    let Some(member_expr) = call_expr.callee.get_member_expr() else {
        return;
    };

    if let Some(member_expr_obj) = member_expr.object().as_member_expression() {
        let is_call_call = is_method_call(call_expr, None, Some(&["call"]), Some(2), Some(2));

        if (is_call_call || is_method_call(call_expr, None, Some(&["apply"]), Some(2), Some(2)))
            && is_prototype_property(member_expr_obj, "concat", Some("Array"))
        {
            if let Some(first_argument) = call_expr.arguments[0].as_expression() {
                if is_empty_array_expression(first_argument)
                    && (is_call_call
                        || !matches!(call_expr.arguments.get(1), Some(Argument::SpreadElement(_))))
                {
                    ctx.diagnostic(prefer_array_flat_diagnostic(call_expr.span));
                }
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"array.flatMap",
        r"new array.flatMap(x => x)",
        r"flatMap(x => x)",
        r"array.notFlatMap(x => x)",
        r"array[flatMap](x => x)",
        r"array.flatMap(x => x, thisArgument)",
        r"array.flatMap(...[x => x])",
        r"array.flatMap(function (x) { return x; })",
        r"array.flatMap(async x => x)",
        r"array.flatMap(function * (x) { return x;})",
        r"array.flatMap(() => x)",
        r"array.flatMap((x, y) => x)",
        r"array.flatMap(x => y)",
        r"(array?.flatMap)?.(x => y)",
        r"new array.reduce((a, b) => a.concat(b), [])",
        r"array.reduce",
        r"reduce((a, b) => a.concat(b), [])",
        r"array[reduce]((a, b) => a.concat(b), [])",
        r"array.notReduce((a, b) => a.concat(b), [])",
        r"array.reduce((a, b) => a.concat(b), [], EXTRA_ARGUMENT)",
        r"array.reduce((a, b) => a.concat(b), NOT_EMPTY_ARRAY)",
        r"array.reduce((a, b, extraParameter) => a.concat(b), [])",
        r"array.reduce((a,) => a.concat(b), [])",
        r"array.reduce(() => a.concat(b), [])",
        r"array.reduce((a, b) => {return a.concat(b); }, [])",
        r"array.reduce(function (a, b) { return a.concat(b); }, [])",
        r"array.reduce((a, b) => b.concat(b), [])",
        r"array.reduce((a, b) => a.concat(a), [])",
        r"array.reduce((a, b) => b.concat(a), [])",
        r"array.reduce((a, b) => a.notConcat(b), [])",
        r"array.reduce((a, b) => a.concat, [])",
        r"new array.reduce((a, b) => [...a, ...b], [])",
        r"array[reduce]((a, b) => [...a, ...b], [])",
        r"reduce((a, b) => [...a, ...b], [])",
        r"array.notReduce((a, b) => [...a, ...b], [])",
        r"array.reduce((a, b) => [...a, ...b], [], EXTRA_ARGUMENT)",
        r"array.reduce((a, b) => [...a, ...b], NOT_EMPTY_ARRAY)",
        r"array.reduce((a, b, extraParameter) => [...a, ...b], [])",
        r"array.reduce((a,) => [...a, ...b], [])",
        r"array.reduce(() => [...a, ...b], [])",
        r"array.reduce((a, b) => {return [...a, ...b]; }, [])",
        r"array.reduce(function (a, b) { return [...a, ...b]; }, [])",
        r"array.reduce((a, b) => [...b, ...b], [])",
        r"array.reduce((a, b) => [...a, ...a], [])",
        r"array.reduce((a, b) => [...b, ...a], [])",
        r"array.reduce((a, b) => [a, ...b], [])",
        r"array.reduce((a, b) => [...a, b], [])",
        r"array.reduce((a, b) => [a, b], [])",
        r"array.reduce((a, b) => [...a, ...b, c], [])",
        r"array.reduce((a, b) => [...a, ...b,,], [])",
        r"array.reduce((a, b) => [,...a, ...b], [])",
        r"array.reduce((a, b) => [, ], [])",
        r"array.reduce((a, b) => [, ,], [])",
        r"[].concat",
        r"new [].concat(array)",
        r"[][concat](array)",
        r"[].notConcat(array)",
        r"[,].concat(array)",
        r"({}).concat(array)",
        r"[].concat()",
        r"[].concat(array, EXTRA_ARGUMENT)",
        r"new [].concat(...array)",
        r"[][concat](...array)",
        r"[].notConcat(...array)",
        r"[,].concat(...array)",
        r"({}).concat(...array)",
        r"[].concat()",
        r"[].concat(...array, EXTRA_ARGUMENT)",
        r"new [].concat.apply([], array)",
        r"[].concat.apply",
        r"[].concat.apply([], ...array)",
        r"[].concat.apply([], array, EXTRA_ARGUMENT)",
        r"[].concat.apply([])",
        r"[].concat.apply(NOT_EMPTY_ARRAY, array)",
        r"[].concat.apply([,], array)",
        r"[,].concat.apply([], array)",
        r"[].concat[apply]([], array)",
        r"[][concat].apply([], array)",
        r"[].concat.notApply([], array)",
        r"[].notConcat.apply([], array)",
        r"new Array.prototype.concat.apply([], array)",
        r"Array.prototype.concat.apply",
        r"Array.prototype.concat.apply([], ...array)",
        r"Array.prototype.concat.apply([], array, EXTRA_ARGUMENT)",
        r"Array.prototype.concat.apply([])",
        r"Array.prototype.concat.apply(NOT_EMPTY_ARRAY, array)",
        r"Array.prototype.concat.apply([,], array)",
        r"Array.prototype.concat[apply]([], array)",
        r"Array.prototype[concat].apply([], array)",
        r"Array[prototype].concat.apply([], array)",
        r"Array.prototype.concat.notApply([], array)",
        r"Array.prototype.notConcat.apply([], array)",
        r"Array.notPrototype.concat.apply([], array)",
        r"NotArray.prototype.concat.apply([], array)",
        r"Array.prototype?.concat.apply([], array)",
        r"object.Array.prototype.concat.apply([], array)",
    ];

    let fail = vec![
        r"array.flatMap(x => x)",
        r"(array?.flatMap)?.(x => x)",
        r"function foo(){return[].flatMap(x => x)}",
        r"foo.flatMap(x => x) instanceof Array",
        r"array.reduce((a, b) => a.concat(b), [])",
        r"function foo(){return[].reduce((a, b) => a.concat(b), [])}",
        r"array.reduce((a, b) => [...a, ...b], [])",
        r"array.reduce((a, b) => [...a, ...b,], [])",
        r"function foo(){return[].reduce((a, b) => [...a, ...b,], [])}",
        r"[].concat(maybeArray)",
        r"[].concat( ((0, maybeArray)) )",
        r"[].concat( ((maybeArray)) )",
        r"[].concat( [foo] )",
        r"[].concat( [[foo]] )",
        r"function foo(){return[].concat(maybeArray)}",
        r"[].concat(...array)",
        r"[].concat(...(( array )))",
        r"[].concat(...(( [foo] )))",
        r"[].concat(...(( [[foo]] )))",
        r"function foo(){return[].concat(...array)}",
        r"class A extends[].concat(...array){}",
        r"const A = class extends[].concat(...array){}",
        r"[].concat.apply([], array)",
        r"[].concat.apply([], ((0, array)))",
        r"[].concat.apply([], ((array)))",
        r"[].concat.apply([], [foo])",
        r"[].concat.apply([], [[foo]])",
        r"[].concat.call([], maybeArray)",
        r"[].concat.call([], ((0, maybeArray)))",
        r"[].concat.call([], ((maybeArray)))",
        r"[].concat.call([], [foo])",
        r"[].concat.call([], [[foo]])",
        r"[].concat.call([], ...array)",
        r"[].concat.call([], ...((0, array)))",
        r"[].concat.call([], ...((array)))",
        r"[].concat.call([], ...[foo])",
        r"[].concat.call([], ...[[foo]])",
        r"function foo(){return[].concat.call([], ...array)}",
        r"Array.prototype.concat.apply([], array)",
        r"Array.prototype.concat.apply([], ((0, array)))",
        r"Array.prototype.concat.apply([], ((array)))",
        r"Array.prototype.concat.apply([], [foo])",
        r"Array.prototype.concat.apply([], [[foo]])",
        r"Array.prototype.concat.call([], maybeArray)",
        r"Array.prototype.concat.call([], ((0, maybeArray)))",
        r"Array.prototype.concat.call([], ((maybeArray)))",
        r"Array.prototype.concat.call([], [foo])",
        r"Array.prototype.concat.call([], [[foo]])",
        r"Array.prototype.concat.call([], ...array)",
        r"Array.prototype.concat.call([], ...((0, array)))",
        r"Array.prototype.concat.call([], ...((array)))",
        r"Array.prototype.concat.call([], ...[foo])",
        r"Array.prototype.concat.call([], ...[[foo]])",
        r"[].concat.apply([], array)",
        r"Array.prototype.concat.apply([], array)",
        r"Array.prototype.concat.apply([], (0, array))",
        r"Array.prototype.concat.call([], (0, array))",
        r"async function a() { return [].concat(await getArray()); }",
        r"[].concat(some./**/array)",
        r"[/**/].concat(some./**/array)",
        r"[/**/].concat(some.array)",
    ];

    let fix = vec![
        ("array.flatMap(x => x)", "array.flat()"),
        ("array.reduce((a, b) => a.concat(b), [])", "array.flat()"),
        ("array.reduce((a, b) => [...a, ...b], [])", "array.flat()"),
    ];

    Tester::new(PreferArrayFlat::NAME, PreferArrayFlat::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
