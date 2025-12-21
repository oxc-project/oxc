use crate::ast_util::get_function_name_with_kind;
use crate::{AstNode, context::LintContext, rule::Rule};
use oxc_ast::AstKind;
use oxc_ast::ast::{ArrowFunctionExpression, BlockStatement, Class, Function, StaticBlock};
use oxc_ast_visit::Visit;
use oxc_ast_visit::walk::{walk_arrow_function_expression, walk_block_statement, walk_function};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::scope::ScopeFlags;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

fn max_statements_diagnostic(name: &str, count: usize, max: usize, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "{name} has too many statements ({count}). Maximum allowed is {max}."
    ))
    .with_help("Consider splitting it into smaller functions.")
    .with_label(span)
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct MaxStatementsConfig {
    /// Maximum number of statements allowed per function.
    max: usize,
    /// Whether to ignore top-level functions.
    ignore_top_level_functions: bool,
}

const DEFAULT_MAX_STATEMENTS: usize = 10;

impl Default for MaxStatementsConfig {
    fn default() -> Self {
        Self { max: DEFAULT_MAX_STATEMENTS, ignore_top_level_functions: false }
    }
}

#[derive(Debug, Default, Clone)]
pub struct MaxStatements(Box<MaxStatementsConfig>);

impl std::ops::Deref for MaxStatements {
    type Target = MaxStatementsConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce a maximum number of statements in a function. This rule ensures
    /// that functions do not exceed a specified statements count, promoting smaller,
    /// more focused functions that are easier to maintain and understand.
    ///
    /// ### Why is this bad?
    ///
    /// Some people consider large functions a code smell. Large functions tend to
    /// do a lot of things and can make it hard to follow what’s going on.
    /// This rule can help avoid large functions.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule with the default `{ "max": 10 }` option:
    /// ```js
    /// function foo() {
    ///   const foo1 = 1;
    ///   const foo2 = 2;
    ///   const foo3 = 3;
    ///   const foo4 = 4;
    ///   const foo5 = 5;
    ///   const foo6 = 6;
    ///   const foo7 = 7;
    ///   const foo8 = 8;
    ///   const foo9 = 9;
    ///   const foo10 = 10;
    ///
    ///   const foo11 = 11; // Too many.
    /// }
    ///
    /// const bar = () => {
    ///   const foo1 = 1;
    ///   const foo2 = 2;
    ///   const foo3 = 3;
    ///   const foo4 = 4;
    ///   const foo5 = 5;
    ///   const foo6 = 6;
    ///   const foo7 = 7;
    ///   const foo8 = 8;
    ///   const foo9 = 9;
    ///   const foo10 = 10;
    ///
    ///   const foo11 = 11; // Too many.
    /// };
    /// ```
    ///
    /// Examples of **correct** code for this rule with the default `{ "max": 10 }` option:
    /// ```js
    /// function foo() {
    ///   const foo1 = 1;
    ///   const foo2 = 2;
    ///   const foo3 = 3;
    ///   const foo4 = 4;
    ///   const foo5 = 5;
    ///   const foo6 = 6;
    ///   const foo7 = 7;
    ///   const foo8 = 8;
    ///   const foo9 = 9;
    ///   return function () { // 10
    ///
    ///     // The number of statements in the inner function does not count toward the
    ///     // statement maximum.
    ///
    ///     let bar;
    ///     let baz;
    ///     return 42;
    ///   };
    /// }
    ///
    /// const bar = () => {
    ///   const foo1 = 1;
    ///   const foo2 = 2;
    ///   const foo3 = 3;
    ///   const foo4 = 4;
    ///   const foo5 = 5;
    ///   const foo6 = 6;
    ///   const foo7 = 7;
    ///   const foo8 = 8;
    ///   const foo9 = 9;
    ///   return function () { // 10
    ///
    ///     // The number of statements in the inner function does not count toward the
    ///     // statement maximum.
    ///
    ///     let bar;
    ///     let baz;
    ///     return 42;
    ///   };
    /// }
    /// ```
    ///
    /// Note that this rule does not apply to class static blocks, and that statements in
    /// class static blocks do not count as statements in the enclosing function.
    ///
    /// Examples of **correct** code for this rule with `{ "max": 2 }` option:
    /// ```js
    /// function foo() {
    ///   let one;
    ///   let two = class {
    ///     static {
    ///       let three;
    ///       let four;
    ///       let five;
    ///       if (six) {
    ///         let seven;
    ///         let eight;
    ///         let nine;
    ///       }
    ///     }
    ///   };
    /// }
    /// ```
    ///
    /// Examples of additional **correct** code for this rule with the
    /// `{ "max": 10 }, { "ignoreTopLevelFunctions": true }` options:
    /// ```js
    /// function foo() {
    ///   const foo1 = 1;
    ///   const foo2 = 2;
    ///   const foo3 = 3;
    ///   const foo4 = 4;
    ///   const foo5 = 5;
    ///   const foo6 = 6;
    ///   const foo7 = 7;
    ///   const foo8 = 8;
    ///   const foo9 = 9;
    ///   const foo10 = 10;
    ///   const foo11 = 11;
    /// }
    /// ```
    MaxStatements,
    eslint,
    pedantic,
    config = MaxStatementsConfig,
);

impl Rule for MaxStatements {
    fn from_configuration(value: Value) -> Self {
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
                .map_or(DEFAULT_MAX_STATEMENTS, |v| {
                    usize::try_from(v).unwrap_or(DEFAULT_MAX_STATEMENTS)
                })
        };

        let ignore_top_level_functions = value
            .get(1)
            .and_then(|config| config.get("ignoreTopLevelFunctions"))
            .and_then(Value::as_bool)
            .unwrap_or(false);

        Self(Box::new(MaxStatementsConfig { max, ignore_top_level_functions }))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let func_body = match node.kind() {
            AstKind::Function(f) => f.body.as_ref(),
            AstKind::ArrowFunctionExpression(f) => Some(&f.body),
            _ => return,
        };

        let Some(func_body) = func_body else {
            return;
        };

        if self.ignore_top_level_functions && is_top_level_function(node.span(), ctx) {
            return;
        }

        let mut statements_visitor =
            StatementsCounter { function_depth: 0, statements_count: func_body.statements.len() };
        statements_visitor.visit_function_body(func_body);

        if statements_visitor.statements_count > self.max {
            let name = get_function_name_with_kind(node, ctx.nodes().parent_node(node.id()));
            ctx.diagnostic(max_statements_diagnostic(
                &name,
                statements_visitor.statements_count,
                self.max,
                node.span(),
            ));
        }
    }
}

struct TopLevelFunctionFinder {
    function_depth: usize,
    top_level_functions: Vec<Span>,
}

impl<'a> Visit<'a> for TopLevelFunctionFinder {
    fn enter_node(&mut self, kind: AstKind<'a>) {
        if is_function(kind) {
            self.function_depth += 1;
        }
    }

    fn leave_node(&mut self, kind: AstKind<'a>) {
        if is_function(kind) {
            self.function_depth -= 1;
        }
    }

    fn visit_function(&mut self, it: &Function<'a>, flags: ScopeFlags) {
        if self.function_depth == 0 {
            self.top_level_functions.push(it.span);
        }
        walk_function(self, it, flags);
    }

    fn visit_arrow_function_expression(&mut self, it: &ArrowFunctionExpression<'a>) {
        if self.function_depth == 0 {
            self.top_level_functions.push(it.span);
        }
        walk_arrow_function_expression(self, it);
    }

    fn visit_class(&mut self, _it: &Class<'a>) {
        // ignore classes since its functions are never top-level
    }
}

struct StatementsCounter {
    function_depth: usize,
    statements_count: usize,
}

impl<'a> Visit<'a> for StatementsCounter {
    fn visit_function(&mut self, it: &Function<'a>, flags: ScopeFlags) {
        self.function_depth += 1;
        walk_function(self, it, flags);
        self.function_depth -= 1;
    }

    fn visit_block_statement(&mut self, it: &BlockStatement<'a>) {
        if self.function_depth == 0 {
            self.statements_count += it.body.len();
            walk_block_statement(self, it);
        }
    }

    fn visit_static_block(&mut self, _: &StaticBlock<'a>) {
        // ignore static blocks
    }
}

fn is_function(kind: AstKind) -> bool {
    matches!(kind, AstKind::Function(_) | AstKind::ArrowFunctionExpression(_))
}

fn is_top_level_function(function_span: Span, ctx: &LintContext) -> bool {
    let mut top_level_functions_finder =
        TopLevelFunctionFinder { function_depth: 0, top_level_functions: vec![] };
    top_level_functions_finder.visit_program(ctx.nodes().program());

    // If there are several top-level functions, it means that the actual top-level function is the module (i.e., the file) itself.
    // If there is only one top-level function, it should be ignored.
    if top_level_functions_finder.top_level_functions.len() > 1 {
        return false;
    }
    match top_level_functions_finder.top_level_functions.pop() {
        Some(top_level_function) => top_level_function.span() == function_span,
        None => false,
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
            "(() => { var bar = 1; return function () { return 42; }; })()",
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
            ";(() => { var bar = 1; return () => { var z; return 42; }; })()",
            Some(serde_json::json!([1, { "ignoreTopLevelFunctions": true }])),
        ),
        (
            ";(function() { var bar = 1; var baz = 2; })(); (function() { var bar = 1; var baz = 2; })()",
            Some(serde_json::json!([1, { "ignoreTopLevelFunctions": true }])),
        ),
        (
            ";(() => { var bar = 1; var baz = 2; })(); (() => { var bar = 1; var baz = 2; })()",
            Some(serde_json::json!([1, { "ignoreTopLevelFunctions": true }])),
        ),
        (
            "define(['foo', 'qux'], function(foo, qux) { var bar = 1; var baz = 2; return function () { var z; return 42; }; })",
            Some(serde_json::json!([1, { "ignoreTopLevelFunctions": true }])),
        ),
        (
            "define(['foo', 'qux'], (foo, qux) => { var bar = 1; var baz = 2; return () => { var z; return 42; }; })",
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
