use oxc_ast::{
    AstKind,
    ast::{
        CallExpression, Expression, ForOfStatement, FormalParameters, NewExpression, SpreadElement,
        YieldExpression,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    ast_util::{is_method_call, is_new_expression},
    context::LintContext,
    rule::Rule,
};

fn iterable_accepting_diagnostic(span: Span, description: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "`{description}` accepts an iterable, `.toArray()` is unnecessary."
    ))
    .with_label(span)
}

fn for_of_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("`for…of` can iterate over an iterable, `.toArray()` is unnecessary.")
        .with_label(span)
}

fn yield_star_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("`yield*` can delegate to an iterable, `.toArray()` is unnecessary.")
        .with_label(span)
}

fn spread_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Spread works on iterables, `.toArray()` is unnecessary.").with_label(span)
}

fn iterator_method_diagnostic(span: Span, method: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "`Iterator` has a `.{method}()` method, `.toArray()` is unnecessary."
    ))
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoUselessIteratorToArray;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow unnecessary `.toArray()` on iterators.
    ///
    /// ### Why is this bad?
    ///
    /// [`Iterator.prototype.toArray()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Iterator/toArray)
    /// converts an iterator to an array. However, this conversion is unnecessary in many cases:
    ///
    /// - The following builtins accept an iterable directly:
    ///   - `Map` constructor
    ///   - `WeakMap` constructor
    ///   - `Set` constructor
    ///   - `WeakSet` constructor
    ///   - `TypedArray` constructor
    ///   - `Array.from(…)`
    ///   - `TypedArray.from(…)`
    ///   - `Object.fromEntries(…)`
    /// - `for…of` can iterate over any iterable, so converting to an array first is unnecessary.
    /// - `yield*` can delegate to any iterable, so converting to an array first is unnecessary.
    /// - `Promise.{all,allSettled,any,race}(…)` accept an iterable, so `.toArray()` is unnecessary.
    ///   Removing `.toArray()` here can change a synchronous throw into an asynchronous rejection
    ///   when iteration fails, so these cases are suggestions rather than autofixes.
    /// - The spread operator (`...`) works on any iterable, so converting to an array first is unnecessary.
    /// - Some `Array` methods also exist on `Iterator`, so converting to an array to call them is unnecessary:
    ///   - `.every()`
    ///   - `.find()`
    ///   - `.forEach()`
    ///   - `.reduce()`
    ///   - `.some()`
    ///
    /// `Array` callbacks receive additional arguments (for example, the 3rd `array` argument)
    /// that `Iterator` callbacks do not. Removing `.toArray()` can change behavior if callbacks
    /// depend on those extra arguments, so those cases are reported as suggestions.
    ///
    /// This rule does not flag `.filter()`, `.map()`, or `.flatMap()` because their `Iterator`
    /// versions return iterators, not arrays, so the semantics differ.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// const set = new Set(iterator.toArray());
    ///
    /// const results = await Promise.all(iterator.toArray());
    ///
    /// for (const item of iterator.toArray());
    ///
    /// function * foo() {
    /// 	yield * iterator.toArray();
    /// }
    ///
    /// const items = [...iterator.toArray()];
    ///
    /// call(...iterator.toArray());
    ///
    /// iterator.toArray().every(fn);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const set = new Set(iterator);
    ///
    /// const results = await Promise.all(iterator);
    ///
    /// for (const item of iterator);
    ///
    /// function * foo() {
    /// 	yield * iterator;
    /// }
    ///
    /// const items = [...iterator];
    ///
    /// call(...iterator);
    ///
    /// iterator.every(fn);
    ///
    /// `.filter()` returns an array on Array but an iterator on Iterator
    /// iterator.toArray().filter(fn);
    /// ```
    NoUselessIteratorToArray,
    unicorn,
    nursery,
    pending,
);

impl Rule for NoUselessIteratorToArray {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::NewExpression(new_expr) => check_new_expr(new_expr, ctx),
            AstKind::CallExpression(call_expr) => check_call_expr(call_expr, ctx),
            AstKind::ForOfStatement(for_of_stmt) => check_for_of_statement(for_of_stmt, ctx),
            AstKind::YieldExpression(yield_expr) => check_yield_expression(yield_expr, ctx),
            AstKind::SpreadElement(spread_elem) => check_spread_element(spread_elem, ctx),
            _ => (),
        }
    }
}

/// Checks if the expression is a call to `.toArray()` on an iterator, and returns the span of the `.toArray` property if so.
fn to_array_span(expr: &Expression) -> Option<Span> {
    let Expression::CallExpression(call_expr) = expr else {
        return None;
    };

    let callee_member_expr = call_expr.callee.get_member_expr()?;

    if call_expr.optional || callee_member_expr.optional() || callee_member_expr.is_computed() {
        return None;
    }

    if !is_method_call(call_expr, None, Some(&["toArray"]), Some(0), Some(0)) {
        return None;
    }

    callee_member_expr.static_property_info().map(|(span, _)| span)
}

const TYPED_ARRAYS: &[&str] = &[
    "Int8Array",
    "Uint8Array",
    "Uint8ClampedArray",
    "Int16Array",
    "Uint16Array",
    "Int32Array",
    "Uint32Array",
    "Float16Array",
    "Float32Array",
    "Float64Array",
    "BigInt64Array",
    "BigUint64Array",
];

fn is_array_or_typed_array_from_call(call_expr: &CallExpression) -> bool {
    is_method_call(call_expr, Some(&["Array"]), Some(&["from"]), Some(1), None)
        || is_method_call(call_expr, Some(TYPED_ARRAYS), Some(&["from"]), Some(1), None)
}

// Case 1: `new Set(iterator.toArray())`, `new Map(iterator.toArray())`, etc.
fn check_new_expr(new_expr: &NewExpression, ctx: &LintContext) {
    if !is_new_expression(new_expr, &["Map", "WeakMap", "Set", "WeakSet"], Some(1), Some(1))
        && !is_new_expression(new_expr, TYPED_ARRAYS, Some(1), None)
    {
        return;
    }

    let Some(argument) = new_expr.arguments.first() else {
        return;
    };

    let Some(argument_expr) = argument.as_expression() else {
        return;
    };

    let Some(to_array_span) = to_array_span(argument_expr.get_inner_expression()) else {
        return;
    };

    let Expression::Identifier(callee_ident) = new_expr.callee.get_inner_expression() else {
        return;
    };
    let description = format!("new {}(…)", callee_ident.name);

    ctx.diagnostic(iterable_accepting_diagnostic(to_array_span, &description));
}

// Case 2: Call expressions — static methods and iterator prototype methods.
fn check_call_expr(call_expr: &CallExpression, ctx: &LintContext) {
    // Case 2a: `Array.from(iterator.toArray())`, `TypedArray.from(…)`, `Object.fromEntries(…)`
    if is_array_or_typed_array_from_call(call_expr)
        || is_method_call(call_expr, Some(&["Object"]), Some(&["fromEntries"]), Some(1), None)
    {
        let Some(callee_member_expr) = call_expr.callee.get_member_expr() else {
            return;
        };
        if call_expr.optional || callee_member_expr.optional() || callee_member_expr.is_computed() {
            return;
        }

        let Some(argument) = call_expr.arguments.first() else {
            return;
        };
        let Expression::Identifier(object_ident) =
            callee_member_expr.object().get_inner_expression()
        else {
            return;
        };
        let Some(method_name) = callee_member_expr.static_property_name() else {
            return;
        };

        let Some(arg_expr) = argument.as_expression() else {
            return;
        };

        let arg_expr = arg_expr.get_inner_expression();

        if let Some(to_array_span) = to_array_span(arg_expr) {
            let description = format!("{}.{}(…)", object_ident.name, method_name);

            ctx.diagnostic(iterable_accepting_diagnostic(to_array_span, &description));
            return;
        }
    }

    // Case 2b: `Promise.all(iterator.toArray())`, etc.
    if is_method_call(
        call_expr,
        Some(&["Promise"]),
        Some(&["all", "allSettled", "any", "race"]),
        Some(1),
        Some(1),
    ) {
        let Some(callee_member_expr) = call_expr.callee.get_member_expr() else {
            return;
        };
        if call_expr.optional || callee_member_expr.optional() || callee_member_expr.is_computed() {
            return;
        }

        let Some(argument) = call_expr.arguments.first() else {
            return;
        };
        let Expression::Identifier(object_ident) =
            callee_member_expr.object().get_inner_expression()
        else {
            return;
        };
        let Some(method_name) = callee_member_expr.static_property_name() else {
            return;
        };

        let Some(arg_expr) = argument.as_expression() else {
            return;
        };

        let arg_expr = arg_expr.get_inner_expression();

        if let Some(to_array_span) = to_array_span(arg_expr) {
            let description = format!("{}.{}(…)", object_ident.name, method_name);

            ctx.diagnostic(iterable_accepting_diagnostic(to_array_span, &description));
            return;
        }
    }

    // Case 2c: `iterator.toArray().every(fn)`, `.find(fn)`, `.forEach(fn)`, `.some(fn)`, `.reduce(fn, init)`
    if is_method_call(call_expr, None, Some(&["every", "find", "forEach", "some"]), None, Some(1))
        || is_method_call(call_expr, None, Some(&["reduce"]), Some(2), Some(2))
    {
        let Some(callee_member_expr) = call_expr.callee.get_member_expr() else {
            return;
        };

        if call_expr.optional || callee_member_expr.optional() || callee_member_expr.is_computed() {
            return;
        }

        let caller_object = callee_member_expr.object().get_inner_expression();
        let Some(to_array_span) = to_array_span(caller_object) else {
            return;
        };

        let Some(method_name) = callee_member_expr.static_property_name() else {
            return;
        };
        let is_reduce_call = method_name == "reduce";
        let array_parameter_index = if is_reduce_call { 3 } else { 2 };

        let callback_params_len = call_expr
            .arguments
            .first()
            .and_then(|argument| argument.as_expression())
            .map(Expression::get_inner_expression)
            .and_then(|callback| match callback {
                Expression::ArrowFunctionExpression(callback) => {
                    Some(formal_parameters_len(&callback.params))
                }
                Expression::FunctionExpression(callback) => {
                    Some(formal_parameters_len(&callback.params))
                }
                _ => None,
            });

        if callback_params_len.is_some_and(|len| len > array_parameter_index) {
            return;
        }

        ctx.diagnostic(iterator_method_diagnostic(to_array_span, method_name));
    }
}

fn formal_parameters_len(params: &FormalParameters) -> usize {
    params.items.len() + usize::from(params.rest.is_some())
}

// Case 3: `for (const x of iterator.toArray())`
fn check_for_of_statement(for_of_stmt: &ForOfStatement, ctx: &LintContext) {
    let Some(to_array_span) = to_array_span(for_of_stmt.right.get_inner_expression()) else {
        return;
    };

    ctx.diagnostic(for_of_diagnostic(to_array_span));
}

// Case 4: `yield* iterator.toArray()`
fn check_yield_expression(yield_expr: &YieldExpression, ctx: &LintContext) {
    if !yield_expr.delegate {
        return;
    }

    let Some(argument) = &yield_expr.argument else {
        return;
    };

    let Some(to_array_span) = to_array_span(argument.get_inner_expression()) else {
        return;
    };

    ctx.diagnostic(yield_star_diagnostic(to_array_span));
}

// Case 5: `[...iterator.toArray()]`, `call(...iterator.toArray())`
// Spread works on iterables — `.toArray()` is unnecessary.
fn check_spread_element(spread_elem: &SpreadElement, ctx: &LintContext) {
    let Some(to_array_span) = to_array_span(spread_elem.argument.get_inner_expression()) else {
        return;
    };

    let parent = ctx.nodes().parent_node(spread_elem.node_id());

    if !matches!(
        parent.kind(),
        AstKind::ArrayExpression(_) | AstKind::CallExpression(_) | AstKind::NewExpression(_)
    ) {
        return;
    }

    ctx.diagnostic(spread_diagnostic(to_array_span));
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "new Set(iterator)",
        "for (const x of iterator);",
        "iterator.every(fn)",
        "new Set(iterator.toArray(true))",
        "for (const x of iterator.toArray(true));",
        "new Set(iterator?.toArray())",
        "new Set(iterator.toArray?.())",
        "for (const x of iterator?.toArray());",
        "for (const x of iterator.toArray?.());",
        "Promise.all?.(iterator.toArray())",
        "Promise?.all(iterator.toArray())",
        "Array.from?.(iterator.toArray())",
        "Object.fromEntries?.(iterator.toArray())",
        "iterator.toArray().filter(fn)",
        "iterator.toArray().map(fn)",
        "iterator.toArray().flatMap(fn)",
        "iterator.toArray().slice(1)",
        "iterator.toArray().sort()",
        "iterator.toArray()[0]",
        "iterator.toArray().length",
        "iterator.toArray().at(0)",
        "iterator.toArray().flat()",
        "iterator.toArray().includes(1)",
        "iterator.toArray().indexOf(1)",
        r#"iterator.toArray().join(",")"#,
        "const arr = iterator.toArray()",
        "function foo() { return iterator.toArray() }",
        "foo(iterator.toArray())",
        "[...iterator]",
        "call(...iterator)",
        "[...iterator?.toArray()]",
        "[...iterator.toArray?.()]",
        "call(...iterator?.toArray())",
        "call(...iterator.toArray?.())",
        "({...iterator.toArray()})",
        r#"new Set(iterator["toArray"]())"#,
        "new Map(iterator.toArray(), extraArg)",
        "Promise.all()",
        "Promise.all(iterator.toArray(), extraArg)",
        "new foo.Set(iterator.toArray())",
        "function * foo() {
                yield iterator.toArray();
            }",
        "NotPromise.all(iterator.toArray())",
        "Promise.notAMethod(iterator.toArray())",
        "iterator.toArray()?.every(fn)",
        "iterator.toArray().every?.(fn)",
        "iterator.toArray().every(fn, thisArg)",
        "iterator.toArray().find(fn, thisArg)",
        "iterator.toArray().forEach(fn, thisArg)",
        "iterator.toArray().some(fn, thisArg)",
        "iterator.toArray().every((value, index, array) => array.length === 1)",
        "iterator.toArray().reduce((accumulator, value, index, array) => array.length, 0)",
        "iterator.toArray().reduce(fn)",
        "iterator.toArray().reduce(fn, init, extra)",
    ];

    let fail = vec![
        "new Set(iterator.toArray())",
        "new Set(...iterator.toArray())",
        "new Map(iterator.toArray())",
        "new WeakSet(iterator.toArray())",
        "new WeakMap(iterator.toArray())",
        "new Int8Array(iterator.toArray())",
        "new Uint8Array(iterator.toArray())",
        "new Float64Array(iterator.toArray())",
        "Promise.all(iterator.toArray())",
        "Promise.allSettled(iterator.toArray())",
        "Promise.any(iterator.toArray())",
        "Promise.race(iterator.toArray())",
        "Array.from(iterator.toArray())",
        "Object.fromEntries(iterator.toArray())",
        "Uint8Array.from(iterator.toArray())",
        "Array.from(iterator.toArray(), mapFn)",
        "Object.fromEntries(iterator.toArray(), extra)",
        "for (const x of iterator.toArray());",
        "for (const x of foo.bar().toArray());",
        "async () => { for await (const x of iterator.toArray()); }",
        "function * foo() {
                yield * iterator.toArray();
            }",
        "iterator.toArray().every(fn)",
        "iterator.toArray().find(fn)",
        "iterator.toArray().forEach(fn)",
        "iterator.toArray().reduce(fn, init)",
        "iterator.toArray().some(fn)",
        "const result = iterator
                .take(10)
                .toArray()
                .every(x => x > 0);",
        "[...iterator.toArray()]",
        "[a, ...iterator.toArray()]",
        "[...iterator.toArray(), b]",
        "call(...iterator.toArray())",
        "call(a, ...iterator.toArray())",
        "new Foo(...iterator.toArray())",
        "new Set((iterator.toArray()))",
        "Promise.all((iterator.toArray()))",
        "Array.from((iterator.toArray()))",
        "Object.fromEntries((iterator.toArray()))",
        "for (const x of (iterator.toArray()));",
        "[...(iterator.toArray())]",
        "call(...(iterator.toArray()))",
        "new Foo(...(iterator.toArray()))",
    ];

    Tester::new(NoUselessIteratorToArray::NAME, NoUselessIteratorToArray::PLUGIN, pass, fail)
        .test_and_snapshot();
}
