use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::Semantic;
use oxc_span::{GetSpan, Span};
use serde_json::Value;

use crate::{
    AstNode,
    ast_util::{is_function_node, iter_outer_expressions},
    context::LintContext,
    rule::Rule,
};

fn max_nested_callbacks_diagnostic(num: usize, max: usize, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Too many nested callbacks ({num}). Maximum allowed is {max}."))
        .with_help("Reduce nesting with promises or refactoring your code.")
        .with_label(span)
}

#[derive(Debug, Clone)]
pub struct MaxNestedCallbacks {
    max: usize,
}

const DEFAULT_MAX_NESTED_CALLBACKS: usize = 10;

impl Default for MaxNestedCallbacks {
    fn default() -> Self {
        Self { max: DEFAULT_MAX_NESTED_CALLBACKS }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce a maximum depth that callbacks can be nested. This rule helps to limit
    /// the complexity of callback nesting, ensuring that callbacks do not become too
    /// deeply nested, improving code readability and maintainability.
    ///
    /// ### Why is this bad?
    ///
    /// Many JavaScript libraries use the callback pattern to manage asynchronous
    /// operations. A program of any complexity will most likely need to manage several
    /// asynchronous operations at various levels of concurrency. A common pitfall is
    /// nesting callbacks excessively, making code harder to read and understand.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule with the `{ "max": 3 }` option:
    /// ```js
    /// foo1(function() {
    ///     foo2(function() {
    ///         foo3(function() {
    ///             foo4(function() {
    ///                 // ...
    ///             });
    ///         });
    ///     });
    /// });
    /// ```
    ///
    /// Examples of **correct** code for this rule with the `{ "max": 3 }` option:
    /// ```js
    /// foo1(handleFoo1);
    ///
    /// function handleFoo1() {
    ///     foo2(handleFoo2);
    /// }
    ///
    /// function handleFoo2() {
    ///     foo3(handleFoo3);
    /// }
    ///
    /// function handleFoo3() {
    ///     foo4(handleFoo4);
    /// }
    ///
    /// function handleFoo4() {
    ///     foo5();
    /// }
    /// ```
    ///
    /// ### Options
    ///
    /// #### max
    ///
    /// `{ type: number, default: 10 }`
    ///
    /// The `max` enforces a maximum depth that callbacks can be nested.
    ///
    /// Example:
    ///
    /// ```json
    /// "eslint/max-nested-callbacks": ["error", 10]
    ///
    /// "eslint/max-nested-callbacks": [
    ///   "error",
    ///   {
    ///     max: 10
    ///   }
    /// ]
    /// ```
    MaxNestedCallbacks,
    eslint,
    pedantic
);

impl Rule for MaxNestedCallbacks {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if is_callback(node, ctx) {
            let depth = 1 + ctx
                .semantic()
                .nodes()
                .ancestors(node.id())
                .filter(|node| is_callback(node, ctx))
                .count();
            if depth > self.max {
                ctx.diagnostic(max_nested_callbacks_diagnostic(depth, self.max, node.span()));
            }
        }
    }

    fn from_configuration(value: serde_json::Value) -> Self {
        let config = value.get(0);
        let max = if let Some(max) = config
            .and_then(Value::as_number)
            .and_then(serde_json::Number::as_u64)
            .and_then(|v| usize::try_from(v).ok())
        {
            max
        } else {
            config
                .and_then(|config| config.get("max"))
                .and_then(Value::as_number)
                .and_then(serde_json::Number::as_u64)
                .map_or(DEFAULT_MAX_NESTED_CALLBACKS, |v| {
                    usize::try_from(v).unwrap_or(DEFAULT_MAX_NESTED_CALLBACKS)
                })
        };
        Self { max }
    }
}

fn is_callback<'a>(node: &AstNode<'a>, semantic: &Semantic<'a>) -> bool {
    is_function_node(node)
        && matches!(
            iter_outer_expressions(semantic.nodes(), node.id()).next(),
            Some(AstKind::Argument(_))
        )
}

#[test]
fn test() {
    use crate::tester::Tester;

    fn nested_functions(d: usize) -> String {
        ["foo(function() {".repeat(d), "});".repeat(d)].concat()
    }

    let defaults = MaxNestedCallbacks::default();
    assert_eq!(defaults.max, 10);

    let nested_10 = nested_functions(10);
    let nested_11 = nested_functions(11);

    let pass = vec![
        ("foo(function() { bar(thing, function(data) {}); });", Some(serde_json::json!([3]))),
        (
            "var foo = function() {}; bar(function(){ baz(function() { qux(foo); }) });",
            Some(serde_json::json!([2])),
        ),
        ("fn(function(){}, function(){}, function(){});", Some(serde_json::json!([2]))),
        ("fn(() => {}, function(){}, function(){});", Some(serde_json::json!([2]))), // { "ecmaVersion": 6 },
        (&nested_10, Some(serde_json::json!([{}]))),
        (
            "foo(function() { bar(thing, function(data) {}); });",
            Some(serde_json::json!([{ "max": 3 }])),
        ),
    ];

    let fail = vec![
        (
            "foo(function() { bar(thing, function(data) { baz(function() {}); }); });",
            Some(serde_json::json!([2])),
        ),
        (
            "foo(function() { bar(thing, (data) => { baz(function() {}); }); });",
            Some(serde_json::json!([2])),
        ), // { "ecmaVersion": 6 },
        (
            "foo(() => { bar(thing, (data) => { baz( () => {}); }); });",
            Some(serde_json::json!([2])),
        ), // { "ecmaVersion": 6 },
        (
            "foo(function() { if (isTrue) { bar(function(data) { baz(function() {}); }); } });",
            Some(serde_json::json!([2])),
        ),
        (&nested_11, Some(serde_json::json!([{}]))),
        ("foo(function() {})", Some(serde_json::json!([{ "max": 0 }]))),
        (
            "foo(function() { bar(thing, function(data) { baz(function() {}); }); });",
            Some(serde_json::json!([{ "max": 2 }])),
        ),
    ];

    Tester::new(MaxNestedCallbacks::NAME, MaxNestedCallbacks::PLUGIN, pass, fail)
        .test_and_snapshot();
}
