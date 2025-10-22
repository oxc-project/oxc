use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNodes;
use oxc_span::GetSpan;
use oxc_span::Span;
use schemars::JsonSchema;
use serde_json::Value;

use crate::{AstNode, ast_util::is_function_node, context::LintContext, rule::Rule};

fn max_depth_diagnostic(num: usize, max: usize, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Blocks are nested too deeply ({num}). Maximum allowed is {max}."))
        .with_help("Consider refactoring your code.")
        .with_label(span)
}

const DEFAULT_MAX_DEPTH: usize = 4;

#[derive(Debug, Clone, JsonSchema)]
pub struct MaxDepth {
    max: usize,
}

impl Default for MaxDepth {
    fn default() -> Self {
        Self { max: DEFAULT_MAX_DEPTH }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce a maximum depth that blocks can be nested. This rule helps to limit the complexity
    /// of nested blocks, improving readability and maintainability by ensuring that code does not
    /// become too deeply nested.
    ///
    /// ### Why is this bad?
    ///
    /// Many developers consider code difficult to read if blocks are nested beyond a certain depth.
    /// Excessive nesting can make it harder to follow the flow of the code, increasing cognitive load
    /// and making maintenance more error-prone. By enforcing a maximum block depth, this rule encourages
    /// cleaner, more readable code.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule with the default `{ "max": 3 }` option:
    /// ```js
    /// function foo() {
    ///   for (;;) { // Nested 1 deep
    ///     while (true) { // Nested 2 deep
    ///       if (true) { // Nested 3 deep
    ///         if (true) { // Nested 4 deep }
    ///       }
    ///     }
    ///   }
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule with the default `{ "max": 3 }` option:
    /// ```js
    /// function foo() {
    ///   for (;;) { // Nested 1 deep
    ///     while (true) { // Nested 2 deep
    ///       if (true) { // Nested 3 deep }
    ///     }
    ///   }
    /// }
    /// ```
    ///
    /// Note that class static blocks do not count as nested blocks, and that the depth in
    /// them is calculated separately from the enclosing context.
    ///
    /// Example:
    /// ```js
    /// function foo() {
    ///   if (true) { // Nested 1 deep
    ///     class C {
    ///       static {
    ///         if (true) { // Nested 1 deep
    ///           if (true) { // Nested 2 deep }
    ///         }
    ///       }
    ///     }
    ///   }
    /// }
    /// ```
    ///
    /// ### Options
    ///
    /// #### max
    ///
    /// `{ type: number, default: 4 }`
    ///
    /// The `max` enforces a maximum depth that blocks can be nested
    ///
    /// Example:
    ///
    /// ```json
    /// "eslint/max-depth": ["error", 4]
    ///
    /// "eslint/max-depth": [
    ///   "error",
    ///   {
    ///     max: 4
    ///   }
    /// ]
    /// ```
    MaxDepth,
    eslint,
    pedantic,
    config = MaxDepth,
);

impl Rule for MaxDepth {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if should_count(node, ctx.nodes()) {
            let depth = 1 + ctx
                .nodes()
                .ancestors(node.id())
                .take_while(|node| !should_stop(node))
                .filter(|node| should_count(node, ctx.nodes()))
                .count();
            if depth > self.max {
                ctx.diagnostic(max_depth_diagnostic(depth, self.max, node.span()));
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
                .map_or(DEFAULT_MAX_DEPTH, |v| usize::try_from(v).unwrap_or(DEFAULT_MAX_DEPTH))
        };
        Self { max }
    }
}

fn should_count(node: &AstNode<'_>, nodes: &AstNodes<'_>) -> bool {
    matches!(node.kind(), AstKind::IfStatement(_) if !matches!(nodes.parent_kind(node.id()), AstKind::IfStatement(_)))
        || matches!(node.kind(), |AstKind::SwitchStatement(_)| AstKind::TryStatement(_)
            | AstKind::DoWhileStatement(_)
            | AstKind::WhileStatement(_)
            | AstKind::WithStatement(_)
            | AstKind::ForStatement(_)
            | AstKind::ForInStatement(_)
            | AstKind::ForOfStatement(_))
}

fn should_stop(node: &AstNode<'_>) -> bool {
    is_function_node(node) || matches!(node.kind(), AstKind::Program(_) | AstKind::StaticBlock(_))
}

#[test]
fn test() {
    use crate::tester::Tester;

    let defaults = MaxDepth::default();
    assert_eq!(defaults.max, 4);

    let pass = vec![
        (
            "function foo() { if (true) { if (false) { if (true) { } } } }",
            Some(serde_json::json!([3])),
        ),
        (
            "function foo() { if (true) { } else if (false) { } else if (true) { } else if (false) {} }",
            Some(serde_json::json!([3])),
        ),
        (
            "var foo = () => { if (true) { if (false) { if (true) { } } } }",
            Some(serde_json::json!([3])),
        ), // { "ecmaVersion": 6 },
        ("function foo() { if (true) { if (false) { if (true) { } } } }", None),
        (
            "function foo() { if (true) { if (false) { if (true) { } } } }",
            Some(serde_json::json!([{ "max": 3 }])),
        ),
        ("class C { static { if (1) { if (2) {} } } }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        (
            "class C { static { if (1) { if (2) {} } if (1) { if (2) {} } } }",
            Some(serde_json::json!([2])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { static { if (1) { if (2) {} } } static { if (1) { if (2) {} } } }",
            Some(serde_json::json!([2])),
        ), // { "ecmaVersion": 2022 },
        ("if (1) { class C { static { if (1) { if (2) {} } } } }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        (
            "function foo() { if (1) { class C { static { if (1) { if (2) {} } } } } }",
            Some(serde_json::json!([2])),
        ), // { "ecmaVersion": 2022 },
        (
            "function foo() { if (1) { if (2) { class C { static { if (1) { if (2) {} } if (1) { if (2) {} } } } } } if (1) { if (2) {} } }",
            Some(serde_json::json!([2])),
        ), // { "ecmaVersion": 2022 }
    ];

    let fail = vec![
        (
            "function foo() { if (true) { if (false) { if (true) { } } } }",
            Some(serde_json::json!([2])),
        ),
        (
            "var foo = () => { if (true) { if (false) { if (true) { } } } }",
            Some(serde_json::json!([2])),
        ), // { "ecmaVersion": 6 },
        ("function foo() { if (true) {} else { for(;;) {} } }", Some(serde_json::json!([1]))),
        ("function foo() { while (true) { if (true) {} } }", Some(serde_json::json!([1]))),
        ("function foo() { for (let x of foo) { if (true) {} } }", Some(serde_json::json!([1]))), // { "ecmaVersion": 6 },
        (
            "function foo() { while (true) { if (true) { if (false) { } } } }",
            Some(serde_json::json!([1])),
        ),
        (
            "function foo() { if (true) { if (false) { if (true) { if (false) { if (true) { } } } } } }",
            None,
        ),
        (
            "function foo() { if (true) { if (false) { if (true) { } } } }",
            Some(serde_json::json!([{ "max": 2 }])),
        ),
        (
            "function foo() { if (a) { if (b) { if (c) { if (d) { if (e) {} } } } } }",
            Some(serde_json::json!([{}])),
        ),
        ("function foo() { if (true) {} }", Some(serde_json::json!([{ "max": 0 }]))),
        ("class C { static { if (1) { if (2) { if (3) {} } } } }", Some(serde_json::json!([2]))), // { "ecmaVersion": 2022 },
        (
            "if (1) { class C { static { if (1) { if (2) { if (3) {} } } } } }",
            Some(serde_json::json!([2])),
        ), // { "ecmaVersion": 2022 },
        (
            "function foo() { if (1) { class C { static { if (1) { if (2) { if (3) {} } } } } } }",
            Some(serde_json::json!([2])),
        ), // { "ecmaVersion": 2022 },
        (
            "function foo() { if (1) { class C { static { if (1) { if (2) {} } } } if (2) { if (3) {} } } }",
            Some(serde_json::json!([2])),
        ), // { "ecmaVersion": 2022 }
    ];

    Tester::new(MaxDepth::NAME, MaxDepth::PLUGIN, pass, fail).test_and_snapshot();
}
