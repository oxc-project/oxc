use oxc_ast::ast::{
    ArrowFunctionExpression, BlockStatement, Class, Function, FunctionBody, StaticBlock,
};
use oxc_ast_visit::Visit;
use oxc_ast_visit::walk::{
    walk_arrow_function_expression, walk_class, walk_function, walk_function_body,
    walk_static_block,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::scope::ScopeFlags;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{context::LintContext, rule::Rule};

fn max_statements_diagnostic(
    name: Option<&str>,
    count: usize,
    max: usize,
    span: Span,
) -> OxcDiagnostic {
    let message = if let Some(name) = name {
        format!("function `{name}` has too many statements ({count}). Maximum allowed is {max}.")
    } else {
        format!("function has too many statements ({count}). Maximum allowed is {max}.")
    };

    OxcDiagnostic::warn(message)
        .with_help("Consider splitting it into smaller functions.")
        .with_label(span)
}

const DEFAULT_MAX_STATEMENTS: usize = 10;

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct MaxStatementsConfig {
    /// Maximum number of statements allowed per function.
    max: usize,
    /// Whether to ignore top-level functions.
    ignore_top_level_functions: bool,
}

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
    /// do a lot of things and can make it hard to follow what's going on.
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
    style,
    config = MaxStatementsConfig,
);

impl Rule for MaxStatements {
    fn from_configuration(value: Value) -> Result<Self, serde_json::error::Error> {
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

        Ok(Self(Box::new(MaxStatementsConfig { max, ignore_top_level_functions })))
    }

    fn run_once(&self, ctx: &LintContext<'_>) {
        let mut visitor = StatementCounter {
            max: self.max,
            ignore_top_level_functions: self.ignore_top_level_functions,
            function_stack: Vec::new(),
            top_level_functions: Vec::new(),
            diagnostics: Vec::new(),
        };
        visitor.visit_program(ctx.nodes().program());

        for (name, count, span) in visitor.diagnostics {
            ctx.diagnostic(max_statements_diagnostic(name, count, self.max, span));
        }

        if visitor.top_level_functions.len() > 1 {
            for (name, count, span) in visitor.top_level_functions {
                ctx.diagnostic(max_statements_diagnostic(name, count, self.max, span));
            }
        }
    }
}

struct StatementCounter<'a> {
    max: usize,
    ignore_top_level_functions: bool,
    /// Stack of statement counts for each function we're currently inside
    function_stack: Vec<usize>,
    /// Top-level functions that exceed the limit (reported only if > 1)
    top_level_functions: Vec<(Option<&'a str>, usize, Span)>,
    /// Diagnostics for non-top-level functions
    diagnostics: Vec<(Option<&'a str>, usize, Span)>,
}

impl<'a> StatementCounter<'a> {
    fn start_function(&mut self) {
        self.function_stack.push(0);
    }

    fn end_function(&mut self, name: Option<&'a str>, span: Span, is_static_block: bool) {
        let count = self.function_stack.pop().unwrap_or(0);

        if is_static_block {
            return;
        }

        if count > self.max {
            if self.ignore_top_level_functions && self.function_stack.is_empty() {
                self.top_level_functions.push((name, count, span));
            } else {
                self.diagnostics.push((name, count, span));
            }
        }
    }

    fn count_statements(&mut self, count: usize) {
        if let Some(current) = self.function_stack.last_mut() {
            *current += count;
        }
    }
}

impl<'a> Visit<'a> for StatementCounter<'a> {
    fn visit_function(&mut self, func: &Function<'a>, flags: ScopeFlags) {
        self.start_function();
        walk_function(self, func, flags);
        self.end_function(func.id.as_ref().map(|id| id.name.as_str()), func.span(), false);
    }

    fn visit_arrow_function_expression(&mut self, arrow: &ArrowFunctionExpression<'a>) {
        self.start_function();
        walk_arrow_function_expression(self, arrow);
        self.end_function(None, arrow.span(), false);
    }

    fn visit_static_block(&mut self, block: &StaticBlock<'a>) {
        self.start_function();
        walk_static_block(self, block);
        self.end_function(None, block.span, true);
    }

    fn visit_class(&mut self, class: &Class<'a>) {
        self.start_function();
        walk_class(self, class);
        self.function_stack.pop();
    }

    fn visit_function_body(&mut self, body: &FunctionBody<'a>) {
        self.count_statements(body.statements.len());
        walk_function_body(self, body);
    }

    fn visit_block_statement(&mut self, block: &BlockStatement<'a>) {
        self.count_statements(block.body.len());
        for stmt in &block.body {
            self.visit_statement(stmt);
        }
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
