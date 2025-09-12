use oxc_ast::{
    AstKind,
    ast::{Argument, CallExpression, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, ast_util::is_method_call, context::LintContext, fixer::Fix, rule::Rule};

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
    /// Prefers the use of `.flatMap()` when `map().flat()` are used together.
    ///
    /// ### Why is this bad?
    ///
    /// It is slightly more efficient to use `.flatMap(…)` instead of `.map(…).flat()`.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// const bar = [1,2,3].map(i => [i]).flat();
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// const bar = [1,2,3].flatMap(i => [i]);
    /// ```
    PreferArrayFlatMap,
    unicorn,
    perf,
    fix
);

// skip React.Children because we are only looking at `StaticMemberExpression.property` and not its object
const IGNORE_OBJECTS: [&str; 1] = [/* "React.Children", */ "Children"];

impl Rule for PreferArrayFlatMap {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(flat_call_expr) = node.kind() else {
            return;
        };

        if flat_call_expr.arguments.len() > 1 {
            return;
        }

        if !is_method_call(flat_call_expr, None, Some(&["flat"]), None, None) {
            return;
        }
        let Some(member_expr) = flat_call_expr.callee.as_member_expression() else {
            return;
        };
        let Expression::CallExpression(call_expr) = &member_expr.object().without_parentheses()
        else {
            return;
        };
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
    use crate::tester::Tester;

    let pass = vec![
        ("const bar = [1,2,3].map()", None),
        ("const bar = [1,2,3].map(i => i)", None),
        ("const bar = [1,2,3].map((i) => i)", None),
        ("const bar = [1,2,3].map((i) => { return i; })", None),
        ("const bar = foo.map(i => i)", None),
        ("const bar = [[1],[2],[3]].flat()", None),
        ("const bar = [1,2,3].map(i => [i]).sort().flat()", None),
        ("const bar = [[1],[2],[3]].map(i => [i]).flat(2)", None),
        ("const bar = [[1],[2],[3]].map(i => [i]).flat(2.0)", None),
        // Parsed as 0.9999999999999999. Rounds down to 0.
        ("const bar = [[1],[2],[3]].map(i => [i]).flat(0.99999999999999994)", None),
        // Parsed as 2.0.
        ("const bar = [[1],[2],[3]].map(i => [i]).flat(1.99999999999999989)", None),
        ("const bar = [[1],[2],[3]].map(i => [i]).flat(1, null)", None),
        ("const bar = [[1],[2],[3]].map(i => [i]).flat(-1)", None),
        ("const bar = [[1],[2],[3]].map(i => [i]).flat(Infinity)", None),
        ("const bar = [[1],[2],[3]].map(i => [i]).flat(Number.POSITIVE_INFINITY)", None),
        ("const bar = [[1],[2],[3]].map(i => [i]).flat(Number.MAX_VALUE)", None),
        ("const bar = [[1],[2],[3]].map(i => [i]).flat(Number.MAX_SAFE_INTEGER)", None),
        ("const bar = [[1],[2],[3]].map(i => [i]).flat(...[1])", None),
        ("const bar = [[1],[2],[3]].map(i => [i]).flat(0.4 +.6)", None),
        ("const bar = [[1],[2],[3]].map(i => [i]).flat(+1)", None),
        ("const bar = [[1],[2],[3]].map(i => [i]).flat(foo)", None),
        ("const bar = [[1],[2],[3]].map(i => [i]).flat(foo.bar)", None),
        // Allowed
        ("Children.map(children, fn).flat()", None), // `import {Children} from 'react';`
        ("React.Children.map(children, fn).flat()", None),
    ];

    let fail = vec![
        ("const bar = [[1],[2],[3]].map(i => [i]).flat()", None),
        ("const bar = [[1],[2],[3]].map(i => [i]).flat(1)", None),
        ("const bar = [[1],[2],[3]].map(i => [i]).flat(1.0)", None),
        ("const bar = [[1],[2],[3]].map(i => [i]).flat(1.00)", None),
        // Parsed as 1.0.
        ("const bar = [[1],[2],[3]].map(i => [i]).flat(0.99999999999999995)", None),
        // Parsed as 1.9999999999999998. Rounds down to 1.
        ("const bar = [[1],[2],[3]].map(i => [i]).flat(1.99999999999999988)", None),
        ("const bar = [[1],[2],[3]].map(i => [i]).flat(1,)", None),
        ("const bar = [1,2,3].map(i => [i]).flat()", None),
        ("const bar = [1,2,3].map((i) => [i]).flat()", None),
        ("const bar = [1,2,3].map((i) => { return [i]; }).flat()", None),
        ("const bar = [1,2,3].map(foo).flat()", None),
        ("const bar = foo.map(i => [i]).flat()", None),
        ("const bar = { map: () => {} }.map(i => [i]).flat()", None),
        ("const bar = [1,2,3].map(i => i).map(i => [i]).flat()", None),
        ("const bar = [1,2,3].sort().map(i => [i]).flat()", None),
        ("const bar = (([1,2,3].map(i => [i]))).flat()", None),
        ("let bar = [1,2,3] . map( x => y ) . flat () // 🤪", None),
        ("const bar = [1,2,3].map(i => [i]).flat(1);", None),
    ];

    let fix = vec![
        (
            "const bar = [[1],[2],[3]].map(i => [i]).flat()",
            "const bar = [[1],[2],[3]].flatMap(i => [i])",
            None,
        ),
        (
            "const bar = [[1],[2],[3]].map(i => [i]).flat(1,)",
            "const bar = [[1],[2],[3]].flatMap(i => [i])",
            None,
        ),
        ("const bar = [1,2,3].map(i => [i]).flat()", "const bar = [1,2,3].flatMap(i => [i])", None),
        (
            "const bar = [1,2,3].map((i) => [i]).flat()",
            "const bar = [1,2,3].flatMap((i) => [i])",
            None,
        ),
        (
            "const bar = [1,2,3].map((i) => { return [i]; }).flat()",
            "const bar = [1,2,3].flatMap((i) => { return [i]; })",
            None,
        ),
        ("const bar = [1,2,3].map(foo).flat()", "const bar = [1,2,3].flatMap(foo)", None),
        ("const bar = foo.map(i => [i]).flat()", "const bar = foo.flatMap(i => [i])", None),
        (
            "const bar = { map: () => {} }.map(i => [i]).flat()",
            "const bar = { map: () => {} }.flatMap(i => [i])",
            None,
        ),
        (
            "const bar = [1,2,3].map(i => i).map(i => [i]).flat()",
            "const bar = [1,2,3].map(i => i).flatMap(i => [i])",
            None,
        ),
        (
            "const bar = [1,2,3].sort().map(i => [i]).flat()",
            "const bar = [1,2,3].sort().flatMap(i => [i])",
            None,
        ),
        (
            "const bar = (([1,2,3].map(i => [i]))).flat()",
            "const bar = (([1,2,3].flatMap(i => [i])))",
            None,
        ),
        (
            "let bar = [1,2,3] . map( x => y ) . flat () // 🤪",
            "let bar = [1,2,3] . flatMap( x => y ) // 🤪",
            None,
        ),
        (
            "const bar = [1,2,3].map(i => [i]).flat(1);",
            "const bar = [1,2,3].flatMap(i => [i]);",
            None,
        ),
    ];

    Tester::new(PreferArrayFlatMap::NAME, PreferArrayFlatMap::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
