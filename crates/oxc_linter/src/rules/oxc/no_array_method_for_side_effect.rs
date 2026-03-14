use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule};

fn no_array_method_for_side_effect_diagnostic(method_name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Array.prototype.{method_name}() called without using its return value"
    ))
    .with_help(format!(
        "`.{method_name}()` returns a new value. Either assign the result to a variable or use `.forEach()` for side effects."
    ))
    .with_label(span)
}

/// Methods on `Array.prototype` whose return value is almost always meaningful.
/// Calling these as expression statements (discarding the result) is a code smell.
const TARGET_METHODS: [&str; 16] = [
    "every",
    "filter",
    "find",
    "findIndex",
    "findLast",
    "findLastIndex",
    "flat",
    "flatMap",
    "map",
    "reduce",
    "reduceRight",
    "some",
    "toReversed",
    "toSorted",
    "toSpliced",
    "with",
];

#[derive(Debug, Default, Clone)]
pub struct NoArrayMethodForSideEffect;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows calling array methods like `map()`, `filter()`, `toSorted()`, or
    /// `reduce()` without using the return value.
    ///
    /// ### Why is this bad?
    ///
    /// Methods such as `Array.prototype.map()` and `Array.prototype.toSorted()`
    /// return a new value. Calling them as a bare expression statement means the
    /// result is discarded, which is almost always a mistake. If you only need
    /// the side effects of the callback, use `forEach()` instead. If you intended
    /// to use the new value, assign it to a variable.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// array.toSorted();
    ///
    /// array.map((item) => {
    ///   console.log(item + 1);
    /// });
    ///
    /// array.filter(Boolean);
    ///
    /// array.reduce((acc, item) => acc + item, 0);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// const sorted = array.toSorted();
    ///
    /// const incremented = array.map((item) => {
    ///   console.log(++item);
    ///   return item;
    /// });
    ///
    /// array.forEach((item) => {
    ///   console.log(item + 1);
    /// });
    ///
    /// const filtered = array.filter(Boolean);
    ///
    /// const sum = array.reduce((acc, item) => acc + item, 0);
    /// ```
    NoArrayMethodForSideEffect,
    oxc,
    correctness
);

impl Rule for NoArrayMethodForSideEffect {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        let mut ancestors = ctx
            .nodes()
            .ancestors(node.id())
            .filter(|a| !matches!(a.kind(), AstKind::ChainExpression(_)));
        let Some(parent) = ancestors.next() else { return };

        if !matches!(parent.kind(), AstKind::ExpressionStatement(_)) {
            return;
        }

        // The callee must be a member expression like `array.map(...)`.
        let callee = call_expr.callee.get_inner_expression();
        let member_expr = callee.as_member_expression().or_else(|| {
            if let oxc_ast::ast::Expression::ChainExpression(chain) = callee {
                chain.expression.as_member_expression()
            } else {
                None
            }
        });
        let Some(member_expr) = member_expr else {
            return;
        };

        let Some((_span, method_name)) = member_expr.static_property_info() else {
            return;
        };

        if !TARGET_METHODS.contains(&method_name) {
            return;
        }

        ctx.diagnostic(no_array_method_for_side_effect_diagnostic(method_name, call_expr.span));
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // Return value is used — assigned to a variable
        ("const sorted = array.toSorted();", None),
        ("const mapped = array.map(x => x + 1);", None),
        ("const filtered = array.filter(Boolean);", None),
        ("const found = array.find(x => x > 0);", None),
        ("const idx = array.findIndex(x => x > 0);", None),
        ("const foundLast = array.findLast(x => x > 0);", None),
        ("const idxLast = array.findLastIndex(x => x > 0);", None),
        ("const flatMapped = array.flatMap(x => [x, x]);", None),
        ("const flattened = array.flat();", None),
        ("const sum = array.reduce((a, b) => a + b, 0);", None),
        ("const sum = array.reduceRight((a, b) => a + b, 0);", None),
        ("const allPositive = array.every(x => x > 0);", None),
        ("const hasPositive = array.some(x => x > 0);", None),
        ("const reversed = array.toReversed();", None),
        ("const spliced = array.toSpliced(0, 1);", None),
        ("const replaced = array.with(0, 'a');", None),
        // Used in boolean context
        ("if (array.every(x => x > 0)) {}", None),
        ("while (array.some(x => x > 0)) {}", None),
        // Used as argument
        ("console.log(array.map(x => x + 1));", None),
        ("foo(array.filter(Boolean));", None),
        // Used in return
        ("function f() { return array.map(x => x + 1); }", None),
        // forEach is fine — it is designed for side effects
        ("array.forEach(x => console.log(x));", None),
        // Mutating methods are fine as expression statements
        ("array.sort();", None),
        ("array.reverse();", None),
        ("array.splice(0, 1);", None),
        ("array.fill(0);", None),
        ("array.copyWithin(0, 1);", None),
        // Non-array methods
        ("foo.bar();", None),
        ("foo.customMethod();", None),
        // Used in ternary
        ("const x = cond ? array.map(fn) : [];", None),
        // Used in logical expression
        ("const x = array.find(fn) || 'default';", None),
        // Chained — final method is forEach so result is not discarded
        ("array.map(fn).forEach(x => console.log(x));", None),
    ];

    let fail = vec![
        // Basic expression statement — return value discarded
        ("array.toSorted();", None),
        ("array.map(x => x + 1);", None),
        ("array.map((item) => { console.log(item + 1); });", None),
        ("array.filter(Boolean);", None),
        ("array.find(x => x > 0);", None),
        ("array.findIndex(x => x > 0);", None),
        ("array.findLast(x => x > 0);", None),
        ("array.findLastIndex(x => x > 0);", None),
        ("array.flat();", None),
        ("array.flatMap(x => [x, x]);", None),
        ("array.reduce((a, b) => a + b, 0);", None),
        ("array.reduceRight((a, b) => a + b, 0);", None),
        ("array.every(x => x > 0);", None),
        ("array.some(x => x > 0);", None),
        ("array.toReversed();", None),
        ("array.toSpliced(0, 1);", None),
        ("array.with(0, 'a');", None),
        // Optional chaining — still unused
        ("array?.map(x => x + 1);", None),
        ("array?.filter(Boolean);", None),
        // Deeply nested member expression
        ("foo.bar.baz.map(x => x + 1);", None),
        // Chained — final result still discarded
        ("array.filter(Boolean).map(fn);", None),
    ];

    Tester::new(NoArrayMethodForSideEffect::NAME, NoArrayMethodForSideEffect::PLUGIN, pass, fail)
        .test_and_snapshot();
}
