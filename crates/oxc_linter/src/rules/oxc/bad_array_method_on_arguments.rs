use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

fn bad_array_method_on_arguments_diagnostic(method_name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Bad array method on `arguments`.")
        .with_help(format!(
            "The `arguments` object does not have a `{method_name}()` method. If you intended to use an array method, consider using rest parameters instead or converting the `arguments` object to an array."
        ))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct BadArrayMethodOnArguments;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule applies when an array method is called on the arguments object itself.
    ///
    /// ### Why is this bad?
    ///
    /// The [arguments object](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Functions/arguments)
    /// is not an array, but an array-like object. It should be converted to a real array before calling an array method.
    /// Otherwise, a TypeError exception will be thrown because of the non-existent method.
    ///
    /// Note that you probably don't need this rule if you are using exclusively
    /// TypeScript, as it will catch these errors when typechecking.
    ///
    /// `arguments` usage is usually discouraged in modern JavaScript, and you should prefer using
    /// rest parameters instead, e.g. `function sum(...args)`.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// function add(x, y) {
    ///   return x + y;
    /// }
    /// function sum() {
    ///   return arguments.reduce(add, 0);
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// function add(x, y) {
    ///   return x + y;
    /// }
    ///
    /// function sum(...args) {
    ///   return args.reduce(add, 0);
    /// }
    /// ```
    BadArrayMethodOnArguments,
    oxc,
    correctness,
    version = "0.0.3",
);

impl Rule for BadArrayMethodOnArguments {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if !node.kind().is_specific_id_reference("arguments") {
            return;
        }
        let parent = ctx.nodes().parent_node(node.id());
        let Some(member_expr) = parent.kind().as_member_expression_kind() else {
            return;
        };
        let AstKind::CallExpression(_) = ctx.nodes().parent_kind(parent.id()) else {
            return;
        };
        let Some(name) = member_expr.static_property_name() else {
            return;
        };
        if ARRAY_METHODS.binary_search(&name.as_str()).is_ok() {
            ctx.diagnostic(bad_array_method_on_arguments_diagnostic(
                name.as_str(),
                member_expr.span(),
            ));
        }
    }
}

/// <https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array#instance_methods>
#[rustfmt::skip]
const ARRAY_METHODS: [&str; 38] = [
    "@@iterator",
    "at",
    "concat", "copyWithin",
    "entries", "every",
    "fill", "filter", "find", "findIndex", "findLast", "findLastIndex", "flat", "flatMap", "forEach",
    "groupBy",
    "includes", "indexOf",
    "join",
    "keys",
    "lastIndexOf",
    "map",
    "pop", "push",
    "reduce", "reduceRight", "reverse",
    "shift", "slice", "some", "sort", "splice",
    "toReversed", "toSorted", "toSpliced",
    "unshift",
    "values",
    "with",
];

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "function fn() {}",
        "function fn(...args) {return args.reduce((prev, cur) => prev + cur, 0)}",
        "function fn() {arguments.foo}",
        "function fn() {arguments.map}",
        "function fn() {arguments[method] }",
        "function fn() {let method='map'; arguments[method](() => {}) }",
        "function fn() {arguments['map']}",
        "function fn() {arguments[`map`]}",
        "function fn() {arg['map'](() => {})}",
        "function fn() {foo.arguments.map}",
        "function fn() {arguments[`map${''}`]((prev, cur) => prev + cur, 0)}",
        "function fn() {arguments[`${''}map`]((prev, cur) => prev + cur, 0)}",
        "function fn() {arguments[`${'map'}`]((prev, cur) => prev + cur, 0)}",
        "function fn() {arguments.toLocaleString(() => {})}",
        "function fn() {arguments.toString(() => {})}",
        "function fn() { Array.prototype.slice.call(arguments) }",
    ];

    let fail = vec![
        "function fn() {arguments['map'](() => {})}",
        "function fn() {arguments[`map`](() => {})}",
        "function fn() {arguments.at(0)}",
        "function fn() {arguments.concat([])}",
        "function fn() {arguments.copyWithin(0)}",
        "function fn() {arguments.entries()}",
        "function fn() {arguments.every(() => {})}",
        "function fn() {arguments.fill(() => {})}",
        "function fn() {arguments.filter(() => {})}",
        "function fn() {arguments.find(() => {})}",
        "function fn() {arguments.findIndex(() => {})}",
        "function fn() {arguments.findLast(() => {})}",
        "function fn() {arguments.findLastIndex(() => {})}",
        "function fn() {arguments.flat(() => {})}",
        "function fn() {arguments.flatMap(() => {})}",
        "function fn() {arguments.forEach(() => {})}",
        "function fn() {arguments.groupBy(() => {})}",
        "function fn() {arguments.includes(() => {})}",
        "function fn() {arguments.indexOf(() => {})}",
        "function fn() {arguments.join()}",
        "function fn() {arguments.keys()}",
        "function fn() {arguments.lastIndexOf('')}",
        "function fn() {arguments.map(() => {})}",
        "function fn() {arguments.pop()}",
        "function fn() {arguments.push('')}",
        "function fn() {arguments.reduce(() => {})}",
        "function fn() {arguments.reduceRight(() => {})}",
        "function fn() {arguments.reverse()}",
        "function fn() {arguments.shift()}",
        "function fn() {arguments.slice()}",
        "function fn() {arguments.some(() => {})}",
        "function fn() {arguments.sort(() => {})}",
        "function fn() {arguments.splice(() => {})}",
        "function fn() {arguments.toReversed(() => {})}",
        "function fn() {arguments.toSorted(() => {})}",
        "function fn() {arguments.toSpliced(0)}",
        "function fn() {arguments.unshift()}",
        "function fn() {arguments.values()}",
        "function fn() {arguments['@@iterator'](() => {})}",
        "const arr = [1, 2, 3, 4, 5];
         function fn() { arguments.with(2, 6) }
         fn(arr)",
    ];

    Tester::new(BadArrayMethodOnArguments::NAME, BadArrayMethodOnArguments::PLUGIN, pass, fail)
        .test_and_snapshot();
}

#[test]
// This needs to be sorted or else binary_search will not work correctly.
fn test_array_is_sorted() {
    assert!(ARRAY_METHODS.is_sorted());
}
