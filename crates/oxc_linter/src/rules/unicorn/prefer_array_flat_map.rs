use std::borrow::Cow;

use oxc_ast::{
    AstKind,
    ast::{Argument, BindingPattern, CallExpression, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::IsGlobalReference;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    ast_util::{get_declaration_of_variable, is_method_call, variable_declaration_kind},
    context::LintContext,
    fixer::Fix,
    rule::Rule,
};

fn prefer_array_flat_map_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("`Array.flatMap` performs `Array.map` and `Array.flat` in one step.")
        .with_help("Prefer `.flatMap(…)` over `.map(…).flat()`.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferArrayFlatMap;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefers a single `.flatMap()` over `.map().flat()` or `.filter().flatMap()`.
    ///
    /// ### Why is this bad?
    ///
    /// A single `.flatMap(…)` avoids creating an intermediate array.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// const bar = [1,2,3].map(i => [i]).flat();
    /// const result = values.filter(value => value > 0).flatMap(value => [value, value]);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// const bar = [1,2,3].flatMap(i => [i]);
    /// const result = values.flatMap(value => value > 0 ? [value, value] : []);
    /// ```
    PreferArrayFlatMap,
    unicorn,
    perf,
    fix_suggestion,
    version = "0.0.14",
    short_description = "Prefer a single `.flatMap()` over `.map().flat()` or `.filter().flatMap()`.",
);

// skip React.Children because we are only looking at `StaticMemberExpression.property` and not its object
const IGNORE_OBJECTS: [&str; 1] = [/* "React.Children", */ "Children"];

impl Rule for PreferArrayFlatMap {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(flat_call_expr) = node.kind() else {
            return;
        };

        check_filter_flat_map(flat_call_expr, ctx);

        if flat_call_expr.optional {
            return;
        }

        if flat_call_expr.arguments.len() > 1 {
            return;
        }

        if !is_method_call(flat_call_expr, None, Some(&["flat"]), None, None) {
            return;
        }
        let Some(member_expr) = flat_call_expr.callee.as_member_expression() else {
            return;
        };

        if member_expr.optional() {
            return;
        }

        let Expression::CallExpression(call_expr) = &member_expr.object().without_parentheses()
        else {
            return;
        };

        if call_expr.optional {
            return;
        }

        if !is_method_call(call_expr, None, Some(&["map"]), None, None) {
            return;
        }

        // Check for Call Expressions which should be ignored
        if is_ignored_call_expression(call_expr) {
            return;
        }

        if let Some(first_arg) = flat_call_expr.arguments.first() {
            // `Array.prototype.flat` rounds down the argument.
            // So `.flat(1.5)` is equivalent to `.flat(1)`.
            // https://tc39.es/ecma262/#sec-array.prototype.flat
            // https://tc39.es/ecma262/#sec-tointegerorinfinity
            // https://tc39.es/ecma262/#eqn-truncate
            #[expect(clippy::float_cmp)]
            if !matches!(first_arg, Argument::NumericLiteral(lit) if lit.value.floor() == 1.0) {
                return;
            }
        }

        ctx.diagnostic_with_fix(prefer_array_flat_map_diagnostic(flat_call_expr.span), |fixer| {
            let mut fix = fixer.new_fix_with_capacity(2);
            // delete flat
            let delete_start = member_expr.object().span().end;
            let delete_end = flat_call_expr.span().end;
            let delete_span = Span::new(delete_start, delete_end);
            fix.push(Fix::delete(delete_span));
            // replace map with flatMap
            let replace_end = call_expr.callee.span().end;
            let replace_start = replace_end - 3;
            let replace_span = Span::new(replace_start, replace_end);
            fix.push(Fix::new("flatMap", replace_span));
            fix.with_message("Replace `.map().flat()` with `.flatMap()`")
        });
    }
}

fn check_filter_flat_map<'a>(call: &CallExpression<'a>, ctx: &LintContext<'a>) {
    if call.optional || call.type_arguments.is_some() || call.arguments.len() != 1 {
        return;
    }
    if !call.optional
        && call.type_arguments.is_none()
        && call.arguments.len() == 1
        && let Expression::StaticMemberExpression(member) = call.callee.get_inner_expression()
        && !member.optional
        && member.property.name == "flatMap"
        && let Expression::CallExpression(filter) = member.object.get_inner_expression()
        && !filter.optional
        && filter.type_arguments.is_none()
        && filter.arguments.len() == 1
        && let Expression::StaticMemberExpression(filter_member) =
            filter.callee.get_inner_expression()
        && !filter_member.optional
        && filter_member.property.name == "filter"
        && !has_optional_chain(&filter_member.object)
        && !ctx.has_comments_between(call.span)
        && let Some((filter_name, predicate)) = simple_callback(&filter.arguments[0])
        && let Some((map_name, mapped)) = simple_callback(&call.arguments[0])
        && filter_name == map_name
        && matches!(mapped.without_parentheses(), Expression::ArrayExpression(array) if array.elements.len() > 1)
        && !is_known_non_indexed_receiver(&filter_member.object, ctx)
    {
        ctx.diagnostic_with_suggestion(
            OxcDiagnostic::warn("Prefer a single `.flatMap(…)` over `.filter(…).flatMap(…)`.")
                .with_label(member.property.span),
            |fixer| {
                let receiver = fixer.source_range(filter_member.object.span());
                let predicate_text = fixer.source_range(predicate.span());
                // These expressions need grouping when used as the test of a conditional.
                let predicate_text = if matches!(
                    predicate,
                    Expression::ArrowFunctionExpression(_)
                        | Expression::AssignmentExpression(_)
                        | Expression::ClassExpression(_)
                        | Expression::ConditionalExpression(_)
                        | Expression::FunctionExpression(_)
                        | Expression::ObjectExpression(_)
                        | Expression::SequenceExpression(_)
                        | Expression::TSAsExpression(_)
                        | Expression::TSNonNullExpression(_)
                        | Expression::TSSatisfiesExpression(_)
                        | Expression::TSTypeAssertion(_)
                        | Expression::YieldExpression(_)
                ) {
                    Cow::Owned(format!("({predicate_text})"))
                } else {
                    Cow::Borrowed(predicate_text)
                };
                let mapped = fixer.source_range(mapped.without_parentheses().span());
                fixer
                    .replace(
                        call.span,
                        format!(
                            "{receiver}.flatMap({filter_name} => {predicate_text} ? {mapped} : [])"
                        ),
                    )
                    .with_message("Replace `.filter(…).flatMap(…)` with a single `.flatMap(…)`.")
            },
        );
    }
}

// Only follow constant bindings; a reassigned receiver may be an array.
fn is_known_non_indexed_receiver<'a>(expression: &Expression<'a>, ctx: &LintContext<'a>) -> bool {
    let mut expression = expression.get_inner_expression();
    if let Expression::Identifier(identifier) = expression
        && let Some(declaration) = get_declaration_of_variable(identifier, ctx)
        && let AstKind::VariableDeclarator(declaration) = declaration.kind()
        && variable_declaration_kind(declaration, ctx).is_const()
        && declaration.id.is_binding_identifier()
        && let Some(initializer) = &declaration.init
    {
        expression = initializer.get_inner_expression();
    }

    match expression {
        Expression::ObjectExpression(_)
        | Expression::FunctionExpression(_)
        | Expression::ArrowFunctionExpression(_)
        | Expression::ClassExpression(_)
        | Expression::TemplateLiteral(_) => true,
        Expression::NewExpression(constructor) => {
            // Arrays and typed arrays are indexed collections; other constructors are not.
            !matches!(constructor.callee.get_inner_expression(), Expression::Identifier(identifier)
                if matches!(identifier.name.as_str(),
                    "Array" | "Int8Array" | "Uint8Array" | "Uint8ClampedArray"
                    | "Int16Array" | "Uint16Array" | "Int32Array" | "Uint32Array"
                    | "Float16Array" | "Float32Array" | "Float64Array"
                    | "BigInt64Array" | "BigUint64Array")
                && identifier.is_global_reference(ctx.scoping()))
        }
        _ => expression.is_literal(),
    }
}

fn simple_callback<'a>(argument: &'a Argument<'a>) -> Option<(&'a str, &'a Expression<'a>)> {
    let Expression::ArrowFunctionExpression(callback) =
        argument.as_expression()?.without_parentheses()
    else {
        return None;
    };
    if callback.r#async
        || callback.return_type.is_some()
        || callback.type_parameters.is_some()
        || callback.params.rest.is_some()
    {
        return None;
    }
    let [parameter] = callback.params.items.as_slice() else { return None };
    if parameter.optional || parameter.type_annotation.is_some() || parameter.initializer.is_some()
    {
        return None;
    }
    let BindingPattern::BindingIdentifier(identifier) = &parameter.pattern else { return None };
    Some((identifier.name.as_str(), callback.get_expression()?))
}

fn has_optional_chain(expression: &Expression) -> bool {
    match expression.get_inner_expression() {
        Expression::ChainExpression(_) => true,
        Expression::CallExpression(call) => call.optional || has_optional_chain(&call.callee),
        expression => expression
            .as_member_expression()
            .is_some_and(|member| member.optional() || has_optional_chain(member.object())),
    }
}

/// Returns true if the object of the method call is `Children` or `React.Children`.
fn is_ignored_call_expression(call_expr: &CallExpression) -> bool {
    let Some(member_expr) = call_expr.callee.get_member_expr() else {
        return false;
    };
    match member_expr.object().get_inner_expression() {
        Expression::Identifier(ident) => IGNORE_OBJECTS.contains(&ident.name.as_str()),
        Expression::StaticMemberExpression(mem) => {
            IGNORE_OBJECTS.contains(&mem.property.name.as_str())
        }
        _ => false,
    }
}

#[test]
fn test() {
    use crate::{
        fixer::FixKind,
        tester::{ExpectFixTestCase, Tester},
    };

    let pass = vec![
        "const bar = [1,2,3].map()",
        "const bar = [1,2,3].map(i => i)",
        "const bar = [1,2,3].map((i) => i)",
        "const bar = [1,2,3].map((i) => { return i; })",
        "const bar = foo.map(i => i)",
        "const bar = foo.map?.(i => [i]).flat()",
        "const bar = foo.map(i => [i])?.flat()",
        "const bar = foo.map(i => [i]).flat?.()",
        "const bar = [[1],[2],[3]].flat()",
        "const bar = [1,2,3].map(i => [i]).sort().flat()",
        "let bar = [1,2,3].map(i => [i]);
            bar = bar.flat();",
        "const bar = [[1],[2],[3]].map(i => [i]).flat(2)",
        "const bar = [[1],[2],[3]].map(i => [i]).flat(2.0)",
        // Parsed as 0.9999999999999999. Rounds down to 0.
        "const bar = [[1],[2],[3]].map(i => [i]).flat(0.99999999999999994)",
        // Parsed as 2.0.
        "const bar = [[1],[2],[3]].map(i => [i]).flat(1.99999999999999989)",
        "const bar = [[1],[2],[3]].map(i => [i]).flat(1, null)",
        "const bar = [[1],[2],[3]].map(i => [i]).flat(-1)",
        "const bar = [[1],[2],[3]].map(i => [i]).flat(Infinity)",
        "const bar = [[1],[2],[3]].map(i => [i]).flat(Number.POSITIVE_INFINITY)",
        "const bar = [[1],[2],[3]].map(i => [i]).flat(Number.MAX_VALUE)",
        "const bar = [[1],[2],[3]].map(i => [i]).flat(Number.MAX_SAFE_INTEGER)",
        "const bar = [[1],[2],[3]].map(i => [i]).flat(...[1])",
        "const bar = [[1],[2],[3]].map(i => [i]).flat(0.4 +.6)",
        "const bar = [[1],[2],[3]].map(i => [i]).flat(+1)",
        "const bar = [[1],[2],[3]].map(i => [i]).flat(foo)",
        "const bar = [[1],[2],[3]].map(i => [i]).flat(foo.bar)",
        // Allowed
        "Children.map(children, fn).flat()", // `import {Children} from 'react';`
        "React.Children.map(children, fn).flat()",
        "array.filter(x => x).map(x => [x, x]);",
        "array.filter(x => x).flatMap(x => []);",
        "array.filter(x => x).flatMap(x => [x]);",
        "array.filter(x => x).flatMap(x => [...x]);",
        "array.filter(x => x).flatMap(x => x.children);",
        "array.filter(function(x) { return x; }).flatMap(x => [x, x]);",
        "array.filter(x => { return x; }).flatMap(x => [x, x]);",
        "array.filter(async x => x).flatMap(x => [x, x]);",
        "array.filter(x => x).flatMap(async x => [x, x]);",
        "array.filter(x => x).flatMap(x => { return [x, x]; });",
        "array.filter(x => x).flatMap(y => [y, y]);",
        "array.filter((x, i) => i).flatMap(x => [x, x]);",
        "array.filter(x => x).flatMap((x, i) => [x, i]);",
        "array.filter(() => true).flatMap(x => [x, x]);",
        "array.filter(x => x).flatMap(() => [1, 2]);",
        "array.filter((x = 1) => x).flatMap(x => [x, x]);",
        "array.filter(x => x).flatMap((x = 1) => [x, x]);",
        "array.filter(({x}) => x).flatMap(x => [x, x]);",
        "array.filter(x => x).flatMap(([x]) => [x, x]);",
        "array.filter((x, ...rest) => x).flatMap(x => [x, x]);",
        "array.filter(x => x).flatMap((x, ...rest) => [x, x]);",
        "array.filter(x => x, thisArg).flatMap(x => [x, x]);",
        "array.filter(x => x).flatMap(x => [x, x], thisArg);",
        "array.filter(...callbacks).flatMap(x => [x, x]);",
        "array.filter(x => x).flatMap(...callbacks);",
        "array[\"filter\"](x => x).flatMap(x => [x, x]);",
        "array.filter(x => x)[\"flatMap\"](x => [x, x]);",
        "array?.filter(x => x).flatMap(x => [x, x]);",
        "array.filter?.(x => x).flatMap(x => [x, x]);",
        "array.filter(x => x)?.flatMap(x => [x, x]);",
        "array.filter(x => x).flatMap?.(x => [x, x]);",
        "(array?.items).filter(x => x).flatMap(x => [x, x]);",
        "getArray?.().filter(x => x).flatMap(x => [x, x]);",
        "array.map?.(x => x).filter(x => x).flatMap(x => [x, x]);",
        "array.filter(x => /* comment */ x).flatMap(x => [x, x]);",
        "array.filter(x => x) /* comment */ .flatMap(x => [x, x]);",
        "array.filter(x => x).flatMap(x => [x, /* comment */ x]);",
        "array.filter<string>(x => x).flatMap(x => [x, x]);",
        "array.filter(x => x).flatMap<string>(x => [x, x]);",
        "array.filter((x: string) => x).flatMap(x => [x, x]);",
        "array.filter(x => x).flatMap((x: string) => [x, x]);",
        "array.filter((x): x is string => true).flatMap(x => [x, x]);",
        "array.filter(x => x).flatMap((x): string[] => [x, x]);",
        "array.filter(<T,>(x) => x).flatMap(x => [x, x]);",
        "array.filter(x => x).flatMap(<T,>(x) => [x, x]);",
        "array.filter((x?) => x).flatMap(x => [x, x]);",
        "array.filter(x => x).flatMap((x?) => [x, x]);",
        "({filter() { return [1, 2]; }}).filter(x => x > 0).flatMap(x => [x, x]);",
        "const collection = {filter() { return [1, 2]; }}; collection.filter(x => x > 0).flatMap(x => [x, x]);",
        "new Set().filter(x => x).flatMap(x => [x, x]);",
        "const collection = new Map(); collection.filter(x => x).flatMap(x => [x, x]);",
        "(function () {}).filter(x => x).flatMap(x => [x, x]);",
        "(() => []).filter(x => x).flatMap(x => [x, x]);",
        "(class {}).filter(x => x).flatMap(x => [x, x]);",
        "`items`.filter(x => x).flatMap(x => [x, x]);",
        "1 .filter(x => x).flatMap(x => [x, x]);",
    ];

    let fail = vec![
        "const bar = [[1],[2],[3]].map(i => [i]).flat()",
        "const bar = [[1],[2],[3]].map(i => [i]).flat(1)",
        "const bar = [[1],[2],[3]].map(i => [i]).flat(1.0)",
        "const bar = [[1],[2],[3]].map(i => [i]).flat(1.00)",
        // Parsed as 1.0.
        "const bar = [[1],[2],[3]].map(i => [i]).flat(0.99999999999999995)",
        // Parsed as 1.9999999999999998. Rounds down to 1.
        "const bar = [[1],[2],[3]].map(i => [i]).flat(1.99999999999999988)",
        "const bar = [[1],[2],[3]].map(i => [i]).flat(1,)",
        "const bar = [1,2,3].map(i => [i]).flat()",
        "const bar = [1,2,3].map((i) => [i]).flat()",
        "const bar = [1,2,3].map((i) => { return [i]; }).flat()",
        "const bar = [1,2,3].map(foo).flat()",
        "const bar = foo.map(i => [i]).flat()",
        "const bar = foo?.map(i => [i]).flat()",
        "const bar = { map: () => {} }.map(i => [i]).flat()",
        "const bar = [1,2,3].map(i => i).map(i => [i]).flat()",
        "const bar = [1,2,3].sort().map(i => [i]).flat()",
        "const bar = (([1,2,3].map(i => [i]))).flat()",
        "let bar = [1,2,3].map(i => {
                return [i];
            }).flat();",
        "let bar = [1,2,3].map(i => {
                return [i];
            })
            .flat();",
        "let bar = [1,2,3].map(i => {
                return [i];
            }) // comment
            .flat();",
        "let bar = [1,2,3].map(i => {
                return [i];
            }) // comment
            .flat(); // other",
        "let bar = [1,2,3]
                .map(i => { return [i]; })
                .flat();",
        "let bar = [1,2,3].map(i => { return [i]; })
                .flat();",
        "let bar = [1,2,3] . map( x => y ) . flat () // 🤪",
        "const bar = [1,2,3].map(i => [i]).flat(1);",
        "const result = values.filter((value) => value > 0).flatMap((value) => [value, value]);",
        "(array).filter(x => (x.active)).flatMap(x => ([x.id, x.slug]));",
        "array.filter(x => x.active && x.visible).flatMap(x => [x, ...x.children]);",
        "array.filter(x => x.a ? x.b : x.c).flatMap(x => [x, x]);",
        "array.filter(x => x.a = true).flatMap(x => [x, x]);",
        "array.filter(x => x as boolean).flatMap(x => [x, x]);",
        "((array.filter(x => x))).flatMap(x => [x, x]);",
        "(a ? b : c).filter(x => x).flatMap(x => [x, x]);",
        "array.filter(x => (x.a, x.b)).flatMap(x => [x, x]);",
    ];

    let fix = vec![
        (
            "const bar = [[1],[2],[3]].map(i => [i]).flat()",
            "const bar = [[1],[2],[3]].flatMap(i => [i])",
        ),
        (
            "const bar = [[1],[2],[3]].map(i => [i]).flat(1,)",
            "const bar = [[1],[2],[3]].flatMap(i => [i])",
        ),
        ("const bar = [1,2,3].map(i => [i]).flat()", "const bar = [1,2,3].flatMap(i => [i])"),
        ("const bar = [1,2,3].map((i) => [i]).flat()", "const bar = [1,2,3].flatMap((i) => [i])"),
        (
            "const bar = [1,2,3].map((i) => { return [i]; }).flat()",
            "const bar = [1,2,3].flatMap((i) => { return [i]; })",
        ),
        ("const bar = [1,2,3].map(foo).flat()", "const bar = [1,2,3].flatMap(foo)"),
        ("const bar = foo.map(i => [i]).flat()", "const bar = foo.flatMap(i => [i])"),
        (
            "const bar = { map: () => {} }.map(i => [i]).flat()",
            "const bar = { map: () => {} }.flatMap(i => [i])",
        ),
        (
            "const bar = [1,2,3].map(i => i).map(i => [i]).flat()",
            "const bar = [1,2,3].map(i => i).flatMap(i => [i])",
        ),
        (
            "const bar = [1,2,3].sort().map(i => [i]).flat()",
            "const bar = [1,2,3].sort().flatMap(i => [i])",
        ),
        (
            "const bar = (([1,2,3].map(i => [i]))).flat()",
            "const bar = (([1,2,3].flatMap(i => [i])))",
        ),
        (
            "let bar = [1,2,3] . map( x => y ) . flat () // 🤪",
            "let bar = [1,2,3] . flatMap( x => y ) // 🤪",
        ),
        ("const bar = [1,2,3].map(i => [i]).flat(1);", "const bar = [1,2,3].flatMap(i => [i]);"),
    ];

    let suggestions = vec![
        (
            "new Array().filter(x => x).flatMap(x => [x, x]);",
            "new Array().flatMap(x => x ? [x, x] : []);",
            None,
            FixKind::Suggestion,
        ),
        (
            "new Uint8Array().filter(x => x).flatMap(x => [x, x]);",
            "new Uint8Array().flatMap(x => x ? [x, x] : []);",
            None,
            FixKind::Suggestion,
        ),
        (
            "[1, 2].filter(x => x).flatMap(x => [x, x]);",
            "[1, 2].flatMap(x => x ? [x, x] : []);",
            None,
            FixKind::Suggestion,
        ),
        (
            "array.filter(x => x).flatMap(x => [x, x]);",
            "array.filter(x => x).flatMap(x => [x, x]);",
            None,
            FixKind::Fix,
        ),
        (
            "const result = values.filter((value) => value > 0).flatMap((value) => [value, value]);",
            "const result = values.flatMap(value => value > 0 ? [value, value] : []);",
            None,
            FixKind::Suggestion,
        ),
        (
            "(array).filter(x => (x.active)).flatMap(x => ([x.id, x.slug]));",
            "(array).flatMap(x => (x.active) ? [x.id, x.slug] : []);",
            None,
            FixKind::Suggestion,
        ),
        (
            "array.filter(x => x.active && x.visible).flatMap(x => [x, ...x.children]);",
            "array.flatMap(x => x.active && x.visible ? [x, ...x.children] : []);",
            None,
            FixKind::Suggestion,
        ),
        (
            "array.filter(x => x.a ? x.b : x.c).flatMap(x => [x, x]);",
            "array.flatMap(x => (x.a ? x.b : x.c) ? [x, x] : []);",
            None,
            FixKind::Suggestion,
        ),
        (
            "array.filter(x => x.a = true).flatMap(x => [x, x]);",
            "array.flatMap(x => (x.a = true) ? [x, x] : []);",
            None,
            FixKind::Suggestion,
        ),
        (
            "array.filter(x => x as boolean).flatMap(x => [x, x]);",
            "array.flatMap(x => (x as boolean) ? [x, x] : []);",
            None,
            FixKind::Suggestion,
        ),
        (
            "((array.filter(x => x))).flatMap(x => [x, x]);",
            "array.flatMap(x => x ? [x, x] : []);",
            None,
            FixKind::Suggestion,
        ),
        (
            "(a ? b : c).filter(x => x).flatMap(x => [x, x]);",
            "(a ? b : c).flatMap(x => x ? [x, x] : []);",
            None,
            FixKind::Suggestion,
        ),
        (
            "array.filter(x => (x.a, x.b)).flatMap(x => [x, x]);",
            "array.flatMap(x => (x.a, x.b) ? [x, x] : []);",
            None,
            FixKind::Suggestion,
        ),
    ];

    Tester::new(PreferArrayFlatMap::NAME, PreferArrayFlatMap::PLUGIN, pass, fail)
        .expect_fix(
            fix.into_iter()
                .map(ExpectFixTestCase::from)
                .chain(suggestions.into_iter().map(ExpectFixTestCase::from))
                .collect::<Vec<_>>(),
        )
        .test_and_snapshot();
}
