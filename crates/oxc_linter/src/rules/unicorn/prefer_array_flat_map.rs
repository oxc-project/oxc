use oxc_ast::{
    ast::{Argument, Expression},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{ast_util::is_method_call, context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-unicorn(prefer-array-flat-map): `Array.flatMap` performs `Array.map` and `Array.flat` in one step.")]
#[diagnostic(severity(warning), help("Prefer `.flatMap(…)` over `.map(…).flat()`."))]
struct PreferArrayFlatMapDiagnostic(#[label] pub Span);

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
    /// ### Example
    /// ```javascript
    /// const bar = [1,2,3].map(i => [i]).flat(); // ✗ fail
    ///
    /// const bar = [1,2,3].flatMap(i => [i]); // ✓ pass
    /// ```
    PreferArrayFlatMap,
    style
);

impl Rule for PreferArrayFlatMap {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(flat_call_expr) = node.kind() else { return };

        if flat_call_expr.arguments.len() > 1 {
            return;
        }

        if !is_method_call(flat_call_expr, None, Some(&["flat"]), None, None) {
            return;
        }
        let Expression::MemberExpression(member_expr) = &flat_call_expr.callee else { return };
        let Expression::CallExpression(call_expr) = &member_expr.object().without_parenthesized()
        else {
            return;
        };
        if !is_method_call(call_expr, None, Some(&["map"]), None, None) {
            return;
        }

        if let Some(first_arg) = flat_call_expr.arguments.first() {
            if let Argument::Expression(Expression::NumberLiteral(number_lit)) = first_arg {
                if number_lit.raw != "1" {
                    return;
                }
            } else {
                return;
            }
        }

        ctx.diagnostic(PreferArrayFlatMapDiagnostic(flat_call_expr.span));
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
        ("const bar = [[1],[2],[3]].map(i => [i]).flat(1, null)", None),
        ("const bar = [[1],[2],[3]].map(i => [i]).flat(Infinity)", None),
        ("const bar = [[1],[2],[3]].map(i => [i]).flat(Number.POSITIVE_INFINITY)", None),
        ("const bar = [[1],[2],[3]].map(i => [i]).flat(Number.MAX_VALUE)", None),
        ("const bar = [[1],[2],[3]].map(i => [i]).flat(Number.MAX_SAFE_INTEGER)", None),
        ("const bar = [[1],[2],[3]].map(i => [i]).flat(...[1])", None),
        ("const bar = [[1],[2],[3]].map(i => [i]).flat(0.4 +.6)", None),
        ("const bar = [[1],[2],[3]].map(i => [i]).flat(+1)", None),
        ("const bar = [[1],[2],[3]].map(i => [i]).flat(foo)", None),
        ("const bar = [[1],[2],[3]].map(i => [i]).flat(foo.bar)", None),
        ("const bar = [[1],[2],[3]].map(i => [i]).flat(1.00)", None),
    ];

    let fail = vec![
        ("const bar = [[1],[2],[3]].map(i => [i]).flat()", None),
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

    Tester::new(PreferArrayFlatMap::NAME, pass, fail).test_and_snapshot();
}
