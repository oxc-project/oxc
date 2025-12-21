use std::ops::Deref;

use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use schemars::JsonSchema;

use crate::{
    AstNode,
    ast_util::{get_function_name_with_kind, is_function_node},
    context::LintContext,
    rule::Rule,
};

fn max_statements_diagnostic(name: &str, count: usize, max: usize, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "{name} has too many statements ({count}). Maximum allowed is {max}."
    ))
    .with_help("Consider refactoring this function to reduce the number of statements.")
    .with_label(span)
}

const DEFAULT_MAX_STATEMENTS: usize = 10;

#[derive(Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct MaxStatementsConfig {
    /// Maximum number of statements allowed in a function.
    max: usize,
    /// When set to true, a single top-level function with too many statements
    /// is ignored, but if multiple top-level functions exceed the limit,
    /// they are all reported (matching ESLint behavior).
    ignore_top_level_functions: bool,
}

impl Default for MaxStatementsConfig {
    fn default() -> Self {
        Self { max: DEFAULT_MAX_STATEMENTS, ignore_top_level_functions: false }
    }
}

#[derive(Debug, Default, Clone)]
pub struct MaxStatements(Box<MaxStatementsConfig>);

impl Deref for MaxStatements {
    type Target = MaxStatementsConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces a maximum number of statements allowed in function blocks.
    /// This rule counts statements in function bodies and reports functions
    /// that exceed the configured maximum.
    ///
    /// ### Why is this bad?
    ///
    /// Functions with too many statements are harder to understand, test, and
    /// maintain. Breaking large functions into smaller, more focused functions
    /// improves code readability and modularity.
    ///
    /// ### Options
    ///
    /// - `max` (default: `10`): Maximum number of statements allowed.
    /// - `ignoreTopLevelFunctions` (default: `false`): When `true`, a single
    ///   top-level function with too many statements is ignored, but if multiple
    ///   top-level functions exceed the limit they are all reported (matching ESLint).
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule with `{ "max": 2 }`:
    /// ```js
    /// function foo() {
    ///     var bar = 1;
    ///     var baz = 2;
    ///     var qux = 3;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule with `{ "max": 2 }`:
    /// ```js
    /// function foo() {
    ///     var bar = 1;
    ///     var baz = 2;
    /// }
    /// ```
    MaxStatements,
    eslint,
    pedantic,
    config = MaxStatementsConfig,
);

impl Rule for MaxStatements {
    fn from_configuration(value: serde_json::Value) -> Self {
        let config = value.get(0);

        // Handle both number and object forms
        let max = config
            .and_then(|c| {
                // First try as number directly
                c.as_number()
                    .and_then(serde_json::Number::as_u64)
                    .and_then(|v| usize::try_from(v).ok())
                    .or_else(|| {
                        // Then try as object with max property
                        c.get("max")
                            .and_then(serde_json::Value::as_number)
                            .and_then(serde_json::Number::as_u64)
                            .and_then(|v| usize::try_from(v).ok())
                            .or_else(|| {
                                // Also support deprecated "maximum" property
                                c.get("maximum")
                                    .and_then(serde_json::Value::as_number)
                                    .and_then(serde_json::Number::as_u64)
                                    .and_then(|v| usize::try_from(v).ok())
                            })
                    })
            })
            .unwrap_or(DEFAULT_MAX_STATEMENTS);

        // Get ignoreTopLevelFunctions from second config element or from first if object
        let ignore_top_level_functions = value
            .get(1)
            .and_then(|c| c.get("ignoreTopLevelFunctions"))
            .and_then(serde_json::Value::as_bool)
            .or_else(|| {
                config
                    .and_then(|c| c.get("ignoreTopLevelFunctions"))
                    .and_then(serde_json::Value::as_bool)
            })
            .unwrap_or(false);

        Self(Box::new(MaxStatementsConfig { max, ignore_top_level_functions }))
    }

    fn run_once(&self, ctx: &LintContext<'_>) {
        let mut top_level_violations: Vec<(String, usize, Span)> = Vec::new();

        for node in ctx.nodes() {
            if !is_function_node(node) {
                continue;
            }

            let statement_count = count_statements(node);
            if statement_count <= self.max {
                continue;
            }

            let parent = ctx.nodes().parent_node(node.id());
            let name = get_function_name_with_kind(node, parent).into_owned();
            let span = get_function_span(node);

            // Check if this is a top-level function (not nested inside another function)
            if self.ignore_top_level_functions && is_outermost_function(node, ctx) {
                top_level_violations.push((name, statement_count, span));
            } else {
                ctx.diagnostic(max_statements_diagnostic(&name, statement_count, self.max, span));
            }
        }

        // Only report top-level violations if there's more than one
        // (single top-level function is ignored)
        if top_level_violations.len() > 1 {
            for (name, count, span) in top_level_violations {
                ctx.diagnostic(max_statements_diagnostic(&name, count, self.max, span));
            }
        }
    }
}

/// Check if a function is at the top level (program/module scope).
/// This is used for the `ignoreTopLevelFunctions` option.
/// A function is "outermost" if it's not nested inside another function or static block.
fn is_outermost_function(node: &AstNode, ctx: &LintContext) -> bool {
    let mut current = ctx.nodes().parent_node(node.id());

    loop {
        match current.kind() {
            AstKind::Program(_) => {
                // Reached the program level without finding another function or static block
                return true;
            }
            AstKind::Function(_)
            | AstKind::ArrowFunctionExpression(_)
            | AstKind::StaticBlock(_) => {
                // Found a parent function or static block - this function is nested
                return false;
            }
            _ => {
                // Keep walking up
                current = ctx.nodes().parent_node(current.id());
            }
        }
    }
}

/// Count statements in a function body, excluding nested functions
fn count_statements(node: &AstNode) -> usize {
    match node.kind() {
        AstKind::Function(func) => {
            func.body.as_ref().map_or(0, |body| count_statements_in_block(&body.statements))
        }
        AstKind::ArrowFunctionExpression(arrow) => {
            count_statements_in_block(&arrow.body.statements)
        }
        _ => 0,
    }
}

/// Count statements in a block, recursively counting nested blocks but not nested functions
fn count_statements_in_block(statements: &[oxc_ast::ast::Statement]) -> usize {
    let mut count = 0;

    for stmt in statements {
        count += 1;

        // Recursively count statements in nested blocks (if, for, while, etc.)
        // but NOT in nested function declarations (those count as 1 statement)
        match stmt {
            oxc_ast::ast::Statement::BlockStatement(block) => {
                // Block statements don't count as a statement themselves in ESLint
                count -= 1;
                count += count_statements_in_block(&block.body);
            }
            oxc_ast::ast::Statement::IfStatement(if_stmt) => {
                count += count_statement_body(&if_stmt.consequent);
                if let Some(alternate) = &if_stmt.alternate {
                    count += count_statement_body(alternate);
                }
            }
            oxc_ast::ast::Statement::ForStatement(for_stmt) => {
                count += count_statement_body(&for_stmt.body);
            }
            oxc_ast::ast::Statement::ForInStatement(for_in) => {
                count += count_statement_body(&for_in.body);
            }
            oxc_ast::ast::Statement::ForOfStatement(for_of) => {
                count += count_statement_body(&for_of.body);
            }
            oxc_ast::ast::Statement::WhileStatement(while_stmt) => {
                count += count_statement_body(&while_stmt.body);
            }
            oxc_ast::ast::Statement::DoWhileStatement(do_while) => {
                count += count_statement_body(&do_while.body);
            }
            oxc_ast::ast::Statement::SwitchStatement(switch) => {
                for case in &switch.cases {
                    count += count_statements_in_block(&case.consequent);
                }
            }
            oxc_ast::ast::Statement::TryStatement(try_stmt) => {
                count += count_statements_in_block(&try_stmt.block.body);
                if let Some(handler) = &try_stmt.handler {
                    count += count_statements_in_block(&handler.body.body);
                }
                if let Some(finalizer) = &try_stmt.finalizer {
                    count += count_statements_in_block(&finalizer.body);
                }
            }
            oxc_ast::ast::Statement::WithStatement(with_stmt) => {
                count += count_statement_body(&with_stmt.body);
            }
            oxc_ast::ast::Statement::LabeledStatement(labeled) => {
                // The labeled statement itself counts as 1, plus its body
                count += count_statement_body(&labeled.body);
            }
            // Function/class declarations count as 1 statement but we don't recurse into them
            // Other statements just count as 1 (already counted at the start)
            _ => {}
        }
    }

    count
}

/// Count statements for a single statement that might be a block
fn count_statement_body(stmt: &oxc_ast::ast::Statement) -> usize {
    match stmt {
        oxc_ast::ast::Statement::BlockStatement(block) => count_statements_in_block(&block.body),
        _ => 0, // Single statements are already counted by the parent
    }
}

/// Get the span for the function declaration (just the signature, not the whole body)
fn get_function_span(node: &AstNode) -> Span {
    match node.kind() {
        AstKind::Function(func) => {
            // Use the span from start to the opening brace
            if let Some(body) = &func.body {
                Span::new(func.span.start, body.span.start)
            } else {
                func.span
            }
        }
        AstKind::ArrowFunctionExpression(arrow) => {
            // Use the span from start to the body
            Span::new(arrow.span.start, arrow.body.span.start)
        }
        _ => node.span(),
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "function foo() { var bar = 1; function qux () { var noCount = 2; } return 3; }",
            Some(serde_json::json!([3])),
        ),
        (
            "function foo() { var bar = 1; if (true) { for (;;) { var qux = null; } } else { quxx(); } return 3; }",
            Some(serde_json::json!([6])),
        ),
        (
            "function foo() { var x = 5; function bar() { var y = 6; } bar(); z = 10; baz(); }",
            Some(serde_json::json!([5])),
        ),
        (
            "function foo() { var a; var b; var c; var x; var y; var z; bar(); baz(); qux(); quxx(); }",
            None,
        ),
        (
            "(function() { var bar = 1; return function () { return 42; }; })()",
            Some(serde_json::json!([1, { "ignoreTopLevelFunctions": true }])),
        ),
        (
            "function foo() { var bar = 1; var baz = 2; }",
            Some(serde_json::json!([1, { "ignoreTopLevelFunctions": true }])),
        ),
        (
            "define(['foo', 'qux'], function(foo, qux) { var bar = 1; var baz = 2; })",
            Some(serde_json::json!([1, { "ignoreTopLevelFunctions": true }])),
        ),
        (
            "var foo = { thing: function() { var bar = 1; var baz = 2; } }",
            Some(serde_json::json!([2])),
        ),
        ("var foo = { thing() { var bar = 1; var baz = 2; } }", Some(serde_json::json!([2]))), // { "ecmaVersion": 6 },
        ("var foo = { ['thing']() { var bar = 1; var baz = 2; } }", Some(serde_json::json!([2]))), // { "ecmaVersion": 6 },
        ("var foo = { thing: () => { var bar = 1; var baz = 2; } }", Some(serde_json::json!([2]))), // { "ecmaVersion": 6 },
        (
            "var foo = { thing: function() { var bar = 1; var baz = 2; } }",
            Some(serde_json::json!([{ "max": 2 }])),
        ),
        (
            "class C { static { one; two; three; { four; five; six; } } }",
            Some(serde_json::json!([2])),
        ), // { "ecmaVersion": 2022 },
        (
            "function foo() { class C { static { one; two; three; { four; five; six; } } } }",
            Some(serde_json::json!([2])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { static { one; two; three; function foo() { 1; 2; } four; five; six; } }",
            Some(serde_json::json!([2])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { static { { one; two; three; function foo() { 1; 2; } four; five; six; } } }",
            Some(serde_json::json!([2])),
        ), // { "ecmaVersion": 2022 },
        (
            "function top_level() { 1; /* 2 */ class C { static { one; two; three; { four; five; six; } } } 3;}",
            Some(serde_json::json!([2, { "ignoreTopLevelFunctions": true }])),
        ), // { "ecmaVersion": 2022 },
        (
            "function top_level() { 1; 2; } class C { static { one; two; three; { four; five; six; } } }",
            Some(serde_json::json!([1, { "ignoreTopLevelFunctions": true }])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { static { one; two; three; { four; five; six; } } } function top_level() { 1; 2; } ",
            Some(serde_json::json!([1, { "ignoreTopLevelFunctions": true }])),
        ), // { "ecmaVersion": 2022 },
        (
            "function foo() { let one; let two = class { static { let three; let four; let five; if (six) { let seven; let eight; let nine; } } }; }",
            Some(serde_json::json!([2])),
        ), // { "ecmaVersion": 2022 }
    ];

    let fail = vec![
        ("function foo() { var bar = 1; var baz = 2; var qux = 3; }", Some(serde_json::json!([2]))),
        (
            "var foo = () => { var bar = 1; var baz = 2; var qux = 3; };",
            Some(serde_json::json!([2])),
        ), // { "ecmaVersion": 6 },
        (
            "var foo = function() { var bar = 1; var baz = 2; var qux = 3; };",
            Some(serde_json::json!([2])),
        ),
        (
            "function foo() { var bar = 1; if (true) { while (false) { var qux = null; } } return 3; }",
            Some(serde_json::json!([4])),
        ),
        (
            "function foo() { var bar = 1; if (true) { for (;;) { var qux = null; } } return 3; }",
            Some(serde_json::json!([4])),
        ),
        (
            "function foo() { var bar = 1; if (true) { for (;;) { var qux = null; } } else { quxx(); } return 3; }",
            Some(serde_json::json!([5])),
        ),
        (
            "function foo() { var x = 5; function bar() { var y = 6; } bar(); z = 10; baz(); }",
            Some(serde_json::json!([3])),
        ),
        (
            "function foo() { var x = 5; function bar() { var y = 6; } bar(); z = 10; baz(); }",
            Some(serde_json::json!([4])),
        ),
        (
            ";(function() { var bar = 1; return function () { var z; return 42; }; })()",
            Some(serde_json::json!([1, { "ignoreTopLevelFunctions": true }])),
        ),
        (
            ";(function() { var bar = 1; var baz = 2; })(); (function() { var bar = 1; var baz = 2; })()",
            Some(serde_json::json!([1, { "ignoreTopLevelFunctions": true }])),
        ),
        (
            "define(['foo', 'qux'], function(foo, qux) { var bar = 1; var baz = 2; return function () { var z; return 42; }; })",
            Some(serde_json::json!([1, { "ignoreTopLevelFunctions": true }])),
        ),
        (
            "function foo() { var a; var b; var c; var x; var y; var z; bar(); baz(); qux(); quxx(); foo(); }",
            None,
        ),
        (
            "var foo = { thing: function() { var bar = 1; var baz = 2; var baz2; } }",
            Some(serde_json::json!([2])),
        ),
        (
            "var foo = { thing() { var bar = 1; var baz = 2; var baz2; } }",
            Some(serde_json::json!([2])),
        ), // { "ecmaVersion": 6 },
        (
            "var foo = { thing: () => { var bar = 1; var baz = 2; var baz2; } }",
            Some(serde_json::json!([2])),
        ), // { "ecmaVersion": 6 },
        (
            "var foo = { thing: function() { var bar = 1; var baz = 2; var baz2; } }",
            Some(serde_json::json!([{ "max": 2 }])),
        ),
        ("function foo() { 1; 2; 3; 4; 5; 6; 7; 8; 9; 10; 11; }", Some(serde_json::json!([{}]))),
        ("function foo() { 1; }", Some(serde_json::json!([{ "max": 0 }]))),
        (
            "function foo() { foo_1; /* foo_ 2 */ class C { static { one; two; three; four; { five; six; seven; eight; } } } foo_3 }",
            Some(serde_json::json!([2])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { static { one; two; three; four; function not_top_level() { 1; 2; 3; } five; six; seven; eight; } }",
            Some(serde_json::json!([2, { "ignoreTopLevelFunctions": true }])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { static { { one; two; three; four; function not_top_level() { 1; 2; 3; } five; six; seven; eight; } } }",
            Some(serde_json::json!([2, { "ignoreTopLevelFunctions": true }])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { static { { one; two; three; four; } function not_top_level() { 1; 2; 3; } { five; six; seven; eight; } } }",
            Some(serde_json::json!([2, { "ignoreTopLevelFunctions": true }])),
        ), // { "ecmaVersion": 2022 }
    ];

    Tester::new(MaxStatements::NAME, MaxStatements::PLUGIN, pass, fail).test_and_snapshot();
}
