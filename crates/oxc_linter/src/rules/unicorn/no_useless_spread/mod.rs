mod const_eval;

use const_eval::{is_array_from, is_new_typed_array, ConstEval};
use oxc_ast::{
    ast::{
        ArrayExpression, ArrayExpressionElement, CallExpression, Expression, NewExpression,
        ObjectExpression, ObjectPropertyKind, SpreadElement,
    },
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    ast_util::{
        get_new_expr_ident_name, is_method_call, is_new_expression, outermost_paren_parent,
    },
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::Rule,
    AstNode,
};

fn spread_in_list(span: Span, arr_or_obj: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Using a spread operator here creates a new {arr_or_obj} unnecessarily."
    ))
    .with_help("Consider removing the spread operator.")
    .with_label(span)
}

fn spread_in_arguments(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Using a spread operator here creates a new array unnecessarily.").with_help("This function accepts a rest parameter, it's unnecessary to create a new array and then spread it. Instead, supply the arguments directly.\nFor example, replace `foo(...[1, 2, 3])` with `foo(1, 2, 3)`.").with_label(span)
}

fn iterable_to_array(span: Span, ctor_name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "`{ctor_name}` accepts an iterable, so it's unnecessary to convert the iterable to an array."
    ))
    .with_help("Consider removing the spread operator.")
    .with_label(span)
}

fn iterable_to_array_in_for_of(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Using a spread operator here creates a new array unnecessarily.")
        .with_help("`for…of` can iterate over iterable, it's unnecessary to convert to an array.")
        .with_label(span)
}

fn iterable_to_array_in_yield_star(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Using a spread operator here creates a new array unnecessarily.")
        .with_help("`yield*` can delegate to another iterable, so it's unnecessary to convert the iterable to an array.")
        .with_label(span)
}

fn clone(span: Span, is_array: bool, method_name: Option<&str>) -> OxcDiagnostic {
    let noun = if is_array { "array" } else { "object" };
    OxcDiagnostic::warn(format!("Using a spread operator here creates a new {noun} unnecessarily."))
        .with_help(
            if let Some(method_name) = method_name {
                format!("`{method_name}` returns a new {noun}. Spreading it into an {noun} expression to create a new {noun} is redundant.")
            } else {

                format!("This expression returns a new {noun}. Spreading it into an {noun} expression to create a new {noun} is redundant.")
            }).with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoUselessSpread;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows using spread syntax in following, unnecessary cases:
    ///
    ///   - Spread an array literal as elements of an array literal
    ///   - Spread an array literal as arguments of a call or a `new` call
    ///   - Spread an object literal as properties of an object literal
    ///   - Use spread syntax to clone an array created inline
    ///
    /// ### Why is this bad?
    ///
    /// - The following builtins accept an iterable, so it's unnecessary to convert the iterable to an array:
    ///
    ///   - `Map` constructor
    ///   - `WeakMap` constructor
    ///   - `Set` constructor
    ///   - `WeakSet` constructor
    ///   - `TypedArray` constructor
    ///   - `Array.from(…)`
    ///   - `TypedArray.from(…)`
    ///   - `Promise.{all,allSettled,any,race}(…)`
    ///   - `Object.fromEntries(…)`
    ///
    /// - `for…of` loop can iterate over any iterable object not just array, so it's unnecessary to convert the iterable to an array.
    ///
    /// - `yield*` can delegate to another iterable, so it's unnecessary to convert the iterable to an array.
    ///
    /// ### Example
    /// ```javascript
    ///
    /// const array = [firstElement, ...[secondElement], thirdElement];
    /// const object = {firstProperty, ...{secondProperty}, thirdProperty};
    /// foo(firstArgument, ...[secondArgument], thirdArgument);
    /// const object = new Foo(firstArgument, ...[secondArgument], thirdArgument);
    /// const set = new Set([...iterable]);
    /// async function foo() {
    ///     const results = await Promise.all([...iterable]);
    /// }
    /// for (const foo of [...set]);
    /// function * foo() {
    /// 	yield * [...anotherGenerator()];
    /// }
    /// function foo(bar) {
    /// 	return [
    /// 		...bar.map(x => x * 2),
    /// 	];
    /// }
    ///
    /// // Pass
    ///
    /// const array = [firstElement, secondElement, thirdElement];
    /// const object = {firstProperty, secondProperty, thirdProperty};
    /// foo(firstArgument, secondArgument, thirdArgument);
    /// const object = new Foo(firstArgument, secondArgument, thirdArgument);
    /// const array = [...foo, bar];
    /// const object = {...foo, bar};
    /// foo(foo, ...bar);
    /// const object = new Foo(...foo, bar);
    /// const set = new Set(iterable);
    /// async function foo() {
    ///    const results = await Promise.all(iterable);
    /// }
    /// for (const foo of set);
    /// function * foo() {
    /// 	yield * anotherGenerator();
    /// }
    /// function foo(bar) {
    /// 	return bar.map(x => x * 2);
    /// }
    ///
    /// ```
    NoUselessSpread,
    correctness,
    conditional_fix
);

impl Rule for NoUselessSpread {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if !matches!(node.kind(), AstKind::ArrayExpression(_) | AstKind::ObjectExpression(_)) {
            return;
        }

        if check_useless_spread_in_list(node, ctx) {
            return;
        }

        match node.kind() {
            AstKind::ArrayExpression(array_expr) => {
                let Some(spread_elem) = as_single_array_spread(array_expr) else {
                    return;
                };

                if check_useless_iterable_to_array(node, array_expr, spread_elem, ctx) {
                    return;
                }

                check_useless_clone(array_expr.span, true, spread_elem, ctx);
            }
            AstKind::ObjectExpression(obj_expr) => {
                let Some(spread_elem) = as_single_obj_spread(obj_expr) else {
                    return;
                };
                check_useless_clone(obj_expr.span, false, spread_elem, ctx);
            }
            _ => unreachable!(),
        }
    }
}

fn check_useless_spread_in_list<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
    let Some(parent) = outermost_paren_parent(node, ctx) else {
        return false;
    };

    // we're in ...[]
    let AstKind::SpreadElement(spread_elem) = parent.kind() else {
        return false;
    };
    let Some(parent_parent) = outermost_paren_parent(parent, ctx) else {
        return false;
    };

    let span = Span::new(spread_elem.span.start, spread_elem.span.start + 3);

    match node.kind() {
        AstKind::ObjectExpression(_) => {
            // { ...{ } }
            if matches!(parent_parent.kind(), AstKind::ObjectExpression(_)) {
                ctx.diagnostic_with_fix(spread_in_list(span, "object"), |fixer| {
                    fix_by_removing_object_spread(fixer, spread_elem)
                });
                true
            } else {
                false
            }
        }
        AstKind::ArrayExpression(array_expr) => match parent_parent.kind() {
            // ...[ ...[] ]
            AstKind::ArrayExpressionElement(_) => {
                let diagnostic = spread_in_list(span, "array");
                if let Some(outer_array) = ctx.nodes().parent_kind(parent_parent.id()) {
                    diagnose_array_in_array_spread(ctx, diagnostic, &outer_array, array_expr);
                } else {
                    ctx.diagnostic(diagnostic);
                }
                true
            }
            // foo(...[ ])
            AstKind::Argument(_) => {
                ctx.diagnostic_with_fix(spread_in_arguments(span), |fixer| {
                    fix_by_removing_array_spread(fixer, array_expr, spread_elem)
                });
                true
            }
            _ => false,
        },
        _ => {
            unreachable!()
        }
    }
}

/// `...[ ...[] ]`. May contain multiple spread elements.
fn diagnose_array_in_array_spread<'a>(
    ctx: &LintContext<'a>,
    diagnostic: OxcDiagnostic,
    outer_array: &AstKind<'a>,
    inner_array: &ArrayExpression<'a>,
) {
    let AstKind::ArrayExpression(outer_array) = outer_array else {
        ctx.diagnostic(diagnostic);
        return;
    };
    match outer_array.elements.len() {
        0 => unreachable!(),
        1 => {
            ctx.diagnostic_with_fix(diagnostic, |fixer| {
                fixer.replace_with(&outer_array.span, inner_array)
            });
        }
        _ => {
            // If all elements are array spreads, we can merge them all together
            let mut spreads: Vec<&'a ArrayExpression> = vec![];
            for el in &outer_array.elements {
                let ArrayExpressionElement::SpreadElement(spread) = el else {
                    ctx.diagnostic(diagnostic);
                    return;
                };
                let Expression::ArrayExpression(arr) = &spread.argument else {
                    ctx.diagnostic(diagnostic);
                    return;
                };
                spreads.push(arr.as_ref());
            }

            // [ ...[a, b, c], ...[d, e, f] ] -> [a, b, c, d, e, f]
            ctx.diagnostic_with_fix(diagnostic, |fixer| {
                let mut codegen = fixer.codegen();
                codegen.print_ascii_byte(b'[');
                let elements =
                    spreads.iter().flat_map(|arr| arr.elements.iter()).collect::<Vec<_>>();
                let n = elements.len();
                for (i, el) in elements.into_iter().enumerate() {
                    codegen.print_expression(el.to_expression());
                    if i < n - 1 {
                        codegen.print_ascii_byte(b',');
                        codegen.print_ascii_byte(b' ');
                    }
                }
                codegen.print_ascii_byte(b']');
                fixer.replace(outer_array.span, codegen)
            });
        }
    }
}

fn check_useless_iterable_to_array<'a>(
    node: &AstNode<'a>,
    array_expr: &ArrayExpression<'a>,
    spread_elem: &SpreadElement<'a>,
    ctx: &LintContext<'a>,
) -> bool {
    let Some(parent) = outermost_paren_parent(node, ctx) else {
        return false;
    };

    let span = Span::new(spread_elem.span.start, spread_elem.span.start + 3);

    let parent = if let AstKind::Argument(_) = parent.kind() {
        let Some(parent) = outermost_paren_parent(parent, ctx) else {
            return false;
        };
        parent
    } else {
        parent
    };

    match parent.kind() {
        AstKind::ForOfStatement(for_of_stmt) => {
            if for_of_stmt.right.without_parentheses().span() == array_expr.span {
                ctx.diagnostic(iterable_to_array_in_for_of(span));
                return true;
            }
            false
        }
        AstKind::YieldExpression(yield_expr) => {
            if yield_expr.delegate
                && yield_expr.argument.as_ref().is_some_and(|arg| arg.span() == array_expr.span)
            {
                ctx.diagnostic(iterable_to_array_in_yield_star(span));
                return true;
            }
            false
        }

        AstKind::NewExpression(new_expr) => {
            if !((is_new_map_or_set_with_iterable(new_expr) || is_new_typed_array(new_expr))
                && new_expr.arguments[0].span().contains_inclusive(array_expr.span))
            {
                return false;
            }
            ctx.diagnostic_with_fix(
                iterable_to_array(span, get_new_expr_ident_name(new_expr).unwrap_or("unknown")),
                |fixer| fix_by_removing_array_spread(fixer, &new_expr.arguments[0], spread_elem),
            );
            true
        }
        AstKind::CallExpression(call_expr) => {
            if !((is_method_call(
                call_expr,
                Some(&["Promise"]),
                Some(&["all", "allSettled", "any", "race"]),
                Some(1),
                Some(1),
            ) || is_array_from(call_expr)
                || is_method_call(
                    call_expr,
                    Some(&["Object"]),
                    Some(&["fromEntries"]),
                    Some(1),
                    Some(1),
                ))
                && call_expr.arguments[0].span().contains_inclusive(array_expr.span))
            {
                return false;
            }

            ctx.diagnostic_with_fix(
                iterable_to_array(
                    span,
                    &get_method_name(call_expr).unwrap_or_else(|| "unknown".into()),
                ),
                |fixer| fix_by_removing_array_spread(fixer, array_expr, spread_elem),
            );
            true
        }
        _ => false,
    }
}

/// Matches `new {Set,WeakSet,Map,WeakMap}(iterable)`
pub fn is_new_map_or_set_with_iterable(new_expr: &NewExpression) -> bool {
    is_new_expression(new_expr, &["Map", "WeakMap", "Set", "WeakSet"], Some(1), Some(1))
}

fn check_useless_clone<'a>(
    array_or_obj_span: Span,
    is_array: bool,
    spread_elem: &SpreadElement<'a>,
    ctx: &LintContext<'a>,
) -> bool {
    let span = Span::new(spread_elem.span.start, spread_elem.span.start + 3);
    let target = spread_elem.argument.get_inner_expression();

    // already diagnosed by first check
    if matches!(target, Expression::ArrayExpression(_) | Expression::ObjectExpression(_)) {
        return false;
    }

    let hint = target.const_eval();
    let hint_matches_expr = if is_array { hint.is_array() } else { hint.is_object() };
    if hint_matches_expr {
        let name = diagnostic_name(ctx, target);

        ctx.diagnostic_with_fix(clone(span, is_array, name), |fixer| {
            fix_by_removing_array_spread(fixer, &array_or_obj_span, spread_elem)
        });
        return true;
    }
    false
}

fn diagnostic_name<'a>(ctx: &LintContext<'a>, expr: &Expression<'a>) -> Option<&'a str> {
    fn pretty_snippet(snippet: &str) -> Option<&str> {
        // unweildy snippets don't get included in diagnostic messages
        if snippet.len() > 50 || snippet.contains('\n') {
            None
        } else {
            Some(snippet)
        }
    }

    match expr {
        Expression::CallExpression(call) => diagnostic_name(ctx, &call.callee),
        Expression::AwaitExpression(expr) => diagnostic_name(ctx, &expr.argument),
        Expression::SequenceExpression(expr) => {
            let span_with_parens = expr.span().expand(1);
            pretty_snippet(ctx.source_range(span_with_parens))
        }
        _ => pretty_snippet(ctx.source_range(expr.span())),
    }
}

/// Creates a fix that replaces `[...spread]` with `spread`
fn fix_by_removing_array_spread<'a, S: GetSpan>(
    fixer: RuleFixer<'_, 'a>,
    iterable: &S,
    spread: &SpreadElement<'a>,
) -> RuleFix<'a> {
    fixer.replace(iterable.span(), fixer.source_range(spread.argument.span()))
}

/// Creates a fix that replaces `{...spread}` with `spread`, when `spread` is an
/// object literal
///
/// ## Examples
/// - `{...{ a, b, }}` -> `{ a, b }`
fn fix_by_removing_object_spread<'a>(
    fixer: RuleFixer<'_, 'a>,
    spread: &SpreadElement<'a>,
) -> RuleFix<'a> {
    // get contents inside object brackets
    // e.g. `...{ a, b, }` -> ` a, b, `
    let replacement_span = &spread.argument.span().shrink(1);

    // remove trailing commas to avoid syntax errors if this spread is followed
    // by another property
    // e.g. ` a, b, ` -> `a, b`
    let mut end_shrink_amount = 0;
    for c in fixer.source_range(*replacement_span).chars().rev() {
        if c.is_whitespace() || c == ',' {
            end_shrink_amount += 1;
        } else {
            break;
        }
    }
    let replacement_span = replacement_span.shrink_right(end_shrink_amount);

    fixer.replace_with(&spread.span, &replacement_span)
}

/// Checks if `node` is `[...(expr)]`
fn as_single_array_spread<'a, 's>(node: &'s ArrayExpression<'a>) -> Option<&'s SpreadElement<'a>> {
    if node.elements.len() != 1 {
        return None;
    }
    match &node.elements[0] {
        ArrayExpressionElement::SpreadElement(spread) => Some(spread.as_ref()),
        _ => None,
    }
}

fn as_single_obj_spread<'a, 's>(node: &'s ObjectExpression<'a>) -> Option<&'s SpreadElement<'a>> {
    if node.properties.len() != 1 {
        return None;
    }
    match &node.properties[0] {
        ObjectPropertyKind::SpreadProperty(spread) => Some(spread.as_ref()),
        ObjectPropertyKind::ObjectProperty(_) => None,
    }
}

fn get_method_name(call_expr: &CallExpression) -> Option<String> {
    let callee = call_expr.callee.get_member_expr()?;

    let object_name = if let Expression::Identifier(ident) = &callee.object() {
        ident.name.as_str()
    } else {
        "unknown"
    };

    Some(format!("{}.{}", object_name, callee.static_property_name().unwrap()))
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"const array = [[]]",
        r"const array = [{}]",
        r"const object = ({...[]})",
        r"foo([])",
        r"foo({})",
        r"new Foo([])",
        r"new Foo({})",
        r"const array = [...a]",
        r"const object = {...a}",
        r"const [first, ...rest] = []",
        r"const {foo, ...rest} = {}",
        r"function a(foo, ...rest) {}",
        r"new NotMatchedConstructor([...iterable])",
        r"new foo.Map([...iterable])",
        r"new Map([...iterable], extraArgument)",
        r"new Map()",
        r"new Map(...iterable)",
        r"new Map([,...iterable])",
        r"new Map([...iterable, extraElement])",
        r"new Map({...iterable})",
        r"new Uint8Array(...iterable)",
        r"new Uint8Array(before, [...iterable])",
        r"Promise.notMatchedMethod([...iterable])",
        r"NotPromise.all([...iterable])",
        r"foo.Promise.all([...iterable])",
        r"Promise[all]([...iterable])",
        r"Promise.all()",
        r"Promise.all([...iterable], extraArgument)",
        r"Promise.all(...iterable)",
        r"Promise.all([,...iterable])",
        r"Promise.all([...iterable, extraElement])",
        r"Promise.all({...iterable})",
        r"Object.notFromEntries([...iterable])",
        r"NotObject.fromEntries([...iterable])",
        r"Object[fromEntries]([...iterable])",
        r"Object.fromEntries()",
        r"Object.fromEntries([...iterable], extraArgument)",
        r"Object.fromEntries(...iterable)",
        r"Object.fromEntries({...iterable})",
        r"Uint8Array.notFrom([...iterable])",
        r"NotTypedArray.from([...iterable])",
        r"Uint8Array.from([...iterable], extraArgument)",
        r"Uint8Array.from(...iterable)",
        r"Uint8Array.from({...iterable})",
        r"for (const [...iterable] of foo);",
        r"for (const foo of bar) [...iterable];",
        r"for (const foo of [,...iterable]);",
        r"for (const foo of [...iterable, extraElement]);",
        r"for (const foo of {...iterable});",
        r"for (const foo in [...iterable]);",
        r"function * fn() {yield [...iterable];}",
        r"function * fn() {yield* [...iterable, extraElement];}",
        r"function * fn() {yield* {...iterable};}",
        r"[...not.array]",
        r"[...not.array()]",
        r"[...array.unknown()]",
        r"const arr = [1, 2, 3]; const unique = [...arr];", // valid method to shallow-clone an array
        r"[...Object.notReturningArray(foo)]",
        r"[...NotObject.keys(foo)]",
        // NOTE (@DonIsaac) these are pathological, should really not be done,
        // and supporting them would add a lot of complexity to the rule's
        // implementation.
        // r"[...Int8Array.from(foo)]",
        // r"[...Int8Array.of()]",
        // r"[...new Int8Array(3)]",
        r"[...new Set(iter)]",
        r"const set = new Set([1, 2, 3]); const unique = [...set];",
        r"[...Promise.all(foo)]",
        r"[...Promise.allSettled(foo)]",
        r"[...await Promise.all(foo, extraArgument)]",
        // Complex object spreads
        r"const obj = { ...obj, ...(addFoo ? { foo: 'foo' } : {}) }",
        r"<Button {...(isLoading ? { data: undefined } : { data: dataFromApi })} />",
        r"const obj = { ...(foo ? getObjectInOpaqueManner() : { a: 2 }) }",
        "[...arr.reduce((set, b) => set.add(b), new Set())]",
        "[...arr.reduce((set, b) => set.add(b), new Set(iter))]",
        // NOTE: we may want to consider this a violation in the future
        "[...(foo ? new Set() : [])]",
    ];

    let fail = vec![
        r"const array = [...[a]]",
        r"const object = {...{a}}",
        r"foo(...[a])",
        r"new Foo(...[a])",
        r"const array = [...[a,]]",
        r"const object = {...{a,}}",
        r"foo(...[a,])",
        r"new Foo(...[a,])",
        r"const array = [...[a,],]",
        r"const object = {...{a,},}",
        r"foo(...[a,],)",
        r"new Foo(...[a,],)",
        r"const array = [...(( [a] ))]",
        r"const object = {...(( {a} ))}",
        r"foo(...(( [a] )))",
        r"new Foo(...(( [a] )))",
        r"const array = [...[]]",
        r"const object = {...{}}",
        r"foo(...[])",
        r"new Foo(...[])",
        r"const array = [...[,]]",
        r"foo(...[,])",
        r"new Foo(...[,])",
        r"const array = [...[,,]]",
        r"foo(...[,,])",
        r"new Foo(...[,,])",
        r"const array = [...[a, , b,]]",
        r"foo(...[a, , b,])",
        r"new Foo(...[a, , b,])",
        r"const array = [...[a, , b,],]",
        r"foo(...[a, , b,],)",
        r"new Foo(...[a, , b,],)",
        r"foo(...[,, ,(( a )), ,,(0, b), ,,])",
        r"const array = [a, ...[a, b]]",
        r"const object = {a, ...{a, b}}",
        r"foo(a, ...[a, b])",
        r"new Foo(a, ...[a, b])",
        r"const array = [...[a, b], b,]",
        r"const object = {...{a, b}, b,}",
        r"foo(...[a, b], b,)",
        r"new Foo(...[a, b], b,)",
        r"const array = [a, ...[a, b], b,]",
        r"const object = {a, ...{a, b}, b,}",
        r"foo(a, ...[a, b], b,)",
        r"new Foo(a, ...[a, b], b,)",
        r"({a:1, ...{a: 2}})",
        r"({...{a:1}, ...{a: 2}})",
        r"({[a]:1, ...{[a]: 2}})",
        r"Promise.all(...[...iterable])",
        r"new Map(...[...iterable])",
        r"const map = new Map([...iterable])",
        r"const weakMap = new WeakMap([...iterable])",
        r"const set = new Set([...iterable])",
        r"const weakSet = new WeakSet([...iterable])",
        r"const typed = new BigUint64Array([...iterable], byteOffset, length)",
        r"const typed = new BigUint64Array([...iterable], ...args)",
        r"const promise = Promise.all([...iterable])",
        r"const promise = Promise.allSettled([...iterable])",
        r"const promise = Promise.any([...iterable])",
        r"const promise = Promise.race([...iterable])",
        r"const array = Array.from([...iterable])",
        r"const typed = BigUint64Array.from([...iterable])",
        r"const object = Object.fromEntries([...iterable])",
        r"for (const foo of [...iterable]);",
        r"async () => {for await (const foo of [...iterable]);}",
        r"const map = new Map([...iterable,])",
        r"for (const foo of [...iterable]);",
        r"const map = new Map([...iterable,],)",
        r"const map = new Map([...(( iterable ))])",
        r"for (const foo of [...(( iterable ))]);",
        r"const map = new Map((( [...(( iterable ))] )))",
        r"for (const foo of (( [...(( iterable ))] )));",
        r"for (const foo of [...iterable]);",
        r"[...foo.concat(bar)]",
        r"[...foo.copyWithin(-2)]",
        r"[...foo.filter(bar)]",
        r"[...foo.flat()]",
        r"[...foo.flatMap(bar)]",
        r"[...foo.map(bar)]",
        r"[...foo.slice(1)]",
        r"[...foo.splice(1)]",
        r"[...foo.toReversed()]",
        r"[...foo.toSorted()]",
        r"[...foo.toSpliced(0, 1)]",
        r"[...foo.with(0, bar)]",
        r#"[...foo.split("|")]"#,
        r"[...new Array(3)]",
        r"[...Object.keys(foo)]",
        r"[...Object.values(foo)]",
        r"[...Array.from(foo)]",
        r"[...Array.of()]",
        r"[...await Promise.all(foo)]",
        r"[...await Promise.allSettled(foo)]",
        r"for (const foo of[...iterable]);",
        r"for (const foo of[...iterable2]);",
        // https://github.com/getsentry/sentry/blob/9e4359030e7ec088aa3f47582f1afbad539a6377/static/app/views/performance/database/useAvailableDurationAggregates.tsx#L15-L17
        r"
        if (organization.features?.includes('performance-database-view-percentiles')) {
            availableAggregates.push(...['p50', 'p75', 'p95', 'p99']);
        }
        ",
        // useless array clones with complex expressions
        "[...(foo ? [1] : [2])]",
        "[...(foo
            ? [1]
            : bar
                ? [2]
                : [2]
                )]",
        "[...(foo ? x.map(x => x) : await Promise.all(foo))]",
        "[...((0, []))]",
        "[...arr.reduce((a, b) => a.push(b), [])]",
        // wait on non-promise value `v` produces `v` itself
        "[...arr.reduce((a, b) => a.push(b), await [])]",
        "[...arr.reduce((a, b) => a.push(b), new Array())]",
        "[...arr.reduce((a, b) => a.push(b), new Array(1, 2, 3))]",
        "[...arr.reduce((a, b) => a.push(b), Array.from(iter))]",
        "[...arr.reduce((a, b) => a.push(b), foo.map(x => x))]",
        "[...arr.reduce((a, b) => a.push(b), await Promise.all(promises))]",
        // useless object clones with complex expressions
        r"const obj = { ...(foo ? { a: 1 } : { a: 2 }) }",
        r"const obj = { ...(foo ? Object.entries(obj).reduce(fn, {}) : { a: 2 }) }",
    ];

    let fix = vec![
        // array literals
        ("[...[1,2,3]]", "[1,2,3]"),
        ("[...[1,2,3], ...[4,5,6]]", "[1, 2, 3, 4, 5, 6]"),
        ("[...[1,2,3], ...x]", "[...[1,2,3], ...x]"),
        ("[...[...[1,2,3]]]", "[...[1,2,3]]"),
        ("const array = [...[a, , b,]]", "const array = [a, , b,]"),
        // object literals
        ("const obj = { a, ...{ b, c } }", "const obj = { a,  b, c }"),
        ("const obj = { a, ...{ b, c, } }", "const obj = { a,  b, c }"),
        ("const obj = {a, ...{b,c}}", "const obj = {a, b,c}"),
        ("const obj = {a, ...{b,c,}}", "const obj = {a, b,c}"),
        ("const obj = { a, ...{ b, c }, ...{ d } }", "const obj = { a,  b, c,  d }"),
        // iterable spread
        (r"const promise = Promise.any([...iterable])", r"const promise = Promise.any(iterable)"),
        (r"new Map([...iterable])", r"new Map(iterable)"),
        (r"new Map([ ...((iterable)) ])", r"new Map(((iterable)))"),
        // (r"new Map(...[...iterable])", r"new Map(iterable)"),
        // useless clones - simple arrays
        ("[...foo.map(x => !!x)]", "foo.map(x => !!x)"),
        ("[...new Array()]", "new Array()"),
        ("[...new Array(1, 2, 3)]", "new Array(1, 2, 3)"),
        // useless clones - complex
        (r"[...await Promise.all(foo)]", r"await Promise.all(foo)"),
        (r"[...Array.from(iterable)]", r"Array.from(iterable)"),
        ("[...((0, []))]", "((0, []))"),
        ("[...arr.reduce((a, b) => a.push(b), [])]", "arr.reduce((a, b) => a.push(b), [])"),
        ("[...arr.reduce((a, b) => a.push(b), [])]", "arr.reduce((a, b) => a.push(b), [])"),
    ];
    Tester::new(NoUselessSpread::NAME, pass, fail).expect_fix(fix).test_and_snapshot();
}
