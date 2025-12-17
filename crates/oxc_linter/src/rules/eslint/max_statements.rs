use oxc_ast::AstKind;
use oxc_ast::ast::Statement;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::NodeId;
use oxc_span::{GetSpan, Span};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

use crate::{AstNode, context::LintContext, rule::Rule};

fn max_statements_diagnostic(span: Span, count: usize, max: i32) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Function has too many statements ({count}). Maximum allowed is {max}."
    ))
    .with_help("Consider breaking this function into smaller functions")
    .with_label(span)
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(untagged, rename_all = "camelCase")]
enum MaxLineConfig {
    #[serde(rename = "max")]
    Max(i32),
    #[serde(rename = "maximum")]
    Maximum(i32),
}

impl Default for MaxLineConfig {
    fn default() -> Self {
        MaxLineConfig::Max(10)
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename_all = "camelCase")]
struct ConfigElement1 {
    #[serde(rename = "ignoreTopLevelFunctions")]
    ignore_top_level_functions: bool,
}

#[derive(Debug, Clone, Serialize)]
struct TopLevelFunctionInfo {
    node_id: NodeId,
    span: Span,
    count: usize,
    done_diagnostic: bool,
}

#[derive(Debug, Default, Clone)]
pub struct MaxStatements {
    max: i32,
    ignore_top_level_functions: bool,
    top_level_functions: Arc<Mutex<Vec<TopLevelFunctionInfo>>>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MaxStatementsConfig(MaxLineConfig, ConfigElement1);

// See <https://github.com/oxc-project/oxc/issues/6050> for documentation details.
// See <https://eslint.org/docs/latest/rules/max-statements> for rule details
declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce a maximum number of statements allowed in function blocks
    ///
    /// ### Why is this bad?
    ///
    /// The max-statements rule allows you to specify the maximum number of statements allowed in a function.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// function foo() {
    ///   const bar = 1; // one statement
    ///   const baz = 2; // two statements
    ///   const qux = 3; // three statements
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
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
    /// ```
    MaxStatements,
    eslint,
    style,
    pending,
    config = MaxStatementsConfig,
);

impl Rule for MaxStatements {
    fn from_configuration(value: serde_json::Value) -> Self {
        // [{max,maximum}, {ignoreTopLevelFunctions}]
        let config: Result<MaxStatementsConfig, ()> =
            serde_json::from_value::<MaxStatementsConfig>(value.clone())
                .or_else(|_| {
                    // [number, {ignoreTopLevelFunctions}]
                    serde_json::from_value::<(i32, ConfigElement1)>(value.clone())
                        .map(|(n, c1)| MaxStatementsConfig(MaxLineConfig::Max(n as i32), c1))
                })
                .or_else(|_| {
                    // [{max,maximum}]
                    serde_json::from_value::<(MaxLineConfig,)>(value.clone())
                        .map(|(c0,)| MaxStatementsConfig(c0, ConfigElement1::default()))
                })
                .or_else(|_| {
                    // config: [number]
                    serde_json::from_value::<(i32,)>(value.clone()).map(|(n,)| {
                        MaxStatementsConfig(MaxLineConfig::Max(n as i32), ConfigElement1::default())
                    })
                })
                .or_else(|_| {
                    // final fallback
                    Ok(MaxStatementsConfig(MaxLineConfig::default(), ConfigElement1::default()))
                });

        let config = config.unwrap();

        Self::new(config)
    }

    fn to_configuration(&self) -> Option<Result<serde_json::Value, serde_json::Error>> {
        Some(serde_json::to_value((
            match &self.max {
                n if *n == 10 => MaxLineConfig::default(),
                n => MaxLineConfig::Max(*n),
            },
            ConfigElement1 { ignore_top_level_functions: self.ignore_top_level_functions },
        )))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::Program(_) => {
                self.top_level_functions.lock().unwrap().clear();
            }
            _ => self.check_and_fix(node, ctx),
        };
    }
}

impl MaxStatements {
    fn new(config: MaxStatementsConfig) -> Self {
        Self {
            max: match config.0 {
                MaxLineConfig::Max(n) | MaxLineConfig::Maximum(n) => n,
            },
            ignore_top_level_functions: config.1.ignore_top_level_functions,
            top_level_functions: Arc::new(Mutex::new(vec![])),
        }
    }

    fn check_is_top_level_function<'a>(
        &self,
        node: &oxc_semantic::AstNode<'a>,
        ctx: &LintContext<'a>,
    ) -> bool {
        let nodes = ctx.nodes();
        let parent_id = nodes.parent_id(node.id());
        let parent = nodes.get_node(parent_id);

        match parent.kind() {
            AstKind::Function(_) => false,
            AstKind::ArrowFunctionExpression(_) => false,
            AstKind::Program(_) => true,
            AstKind::Class(_) => false,
            _ => self.check_is_top_level_function(&parent, ctx),
        }
    }

    fn count_statement(&self, statement: &Statement<'_>) -> usize {
        match statement {
            Statement::EmptyStatement(_) => 0,

            Statement::BlockStatement(block) => {
                // directly access block.body, which is a Vec<Statement>
                self.count_statements_recursive(&block.body)
            }

            Statement::IfStatement(if_stmt) => {
                let mut count = 1; // if

                // handle consequent (then branch)
                count += self.count_statement(&if_stmt.consequent);

                // handle alternate (else branch, optional)
                if let Some(alternate) = &if_stmt.alternate {
                    count += self.count_statement(alternate);
                }

                count
            }

            Statement::WhileStatement(while_stmt) => 1 + self.count_statement(&while_stmt.body),

            Statement::ForStatement(for_stmt) => 1 + self.count_statement(&for_stmt.body),

            Statement::ForInStatement(for_in) => 1 + self.count_statement(&for_in.body),

            Statement::ForOfStatement(for_of) => 1 + self.count_statement(&for_of.body),

            Statement::DoWhileStatement(do_while) => 1 + self.count_statement(&do_while.body),

            Statement::TryStatement(try_stmt) => {
                let mut count = 1; // try
                // try 块
                count += self.count_statements_recursive(&try_stmt.block.body);

                // catch 块
                if let Some(handler) = &try_stmt.handler {
                    count += self.count_statements_recursive(&handler.body.body);
                }

                // finally 块
                if let Some(finalizer) = &try_stmt.finalizer {
                    count += self.count_statements_recursive(&finalizer.body);
                }

                count
            }

            Statement::SwitchStatement(switch) => {
                let mut count = 1; // switch

                for case in &switch.cases {
                    count += self.count_statements_recursive(&case.consequent);
                }

                count
            }

            Statement::LabeledStatement(labeled) => {
                // no count label
                self.count_statement(&labeled.body)
            }

            Statement::WithStatement(with_stmt) => 1 + self.count_statement(&with_stmt.body),

            // simple statement
            _ => 1,
        }
    }

    fn count_statements_recursive<'a>(
        &self,
        statements: &oxc_allocator::Vec<'a, Statement<'a>>,
    ) -> usize {
        statements.iter().map(|stmt| self.count_statement(stmt)).sum()
    }

    fn check_and_fix<'a>(&self, node: &oxc_semantic::AstNode<'a>, ctx: &LintContext<'a>) {
        let max_lines = self.max;

        match node.kind() {
            AstKind::ArrowFunctionExpression(expr) => {
                let statements_count = self.count_statements_recursive(&expr.body.statements);

                let is_top_level = self.check_is_top_level_function(node, ctx);

                if self.ignore_top_level_functions && is_top_level {
                    self.top_level_functions.lock().unwrap().push(TopLevelFunctionInfo {
                        node_id: node.id(),
                        span: expr.span(),
                        count: statements_count,
                        done_diagnostic: false,
                    });

                    self.diagnostic_top_level_function(ctx);
                    return;
                }

                if statements_count as i32 > max_lines {
                    let span = expr.span();
                    let diagnostic = max_statements_diagnostic(span, statements_count, max_lines);
                    ctx.diagnostic(diagnostic);
                };
            }
            AstKind::Function(func) => {
                let Some(body) = func.body.as_ref() else {
                    return;
                };

                let statements_count = self.count_statements_recursive(&body.statements);

                let is_top_level = self.check_is_top_level_function(node, ctx);

                if self.ignore_top_level_functions && is_top_level {
                    self.top_level_functions.lock().unwrap().push(TopLevelFunctionInfo {
                        node_id: node.id(),
                        span: func.span(),
                        count: statements_count,
                        done_diagnostic: false,
                    });

                    self.diagnostic_top_level_function(ctx);

                    return;
                }

                if statements_count as i32 > max_lines {
                    let span = func.span();
                    let diagnostic = max_statements_diagnostic(span, statements_count, max_lines);
                    ctx.diagnostic(diagnostic);
                };
            }
            _ => {}
        };
    }

    fn diagnostic_top_level_function(&self, ctx: &LintContext) {
        let mut top_level_functions = self.top_level_functions.lock().unwrap();

        if top_level_functions.len() == 1 {
            return;
        }

        top_level_functions.iter_mut().for_each(|func_info| {
            if func_info.count as i32 > self.max && func_info.done_diagnostic == false {
                func_info.done_diagnostic = true;
                let diagnostic =
                    max_statements_diagnostic(func_info.span, func_info.count, self.max);
                ctx.diagnostic(diagnostic);
            }
        });
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
        (
            "class C { static { { one; two; three; four; } function not_top_level() { 1; 2; 3; } { five; six; seven; eight; } } }",
            Some(serde_json::json!([{ "maximum" : 2 }, { "ignoreTopLevelFunctions": true }])),
        ), // { "ecmaVersion": 2022 }
        (
            "class C { static { { one; two; three; four; } function not_top_level() { 1; 2; 3; } { five; six; seven; eight; } } }",
            Some(serde_json::json!([{ "max" : 2 }, { "ignoreTopLevelFunctions": true }])),
        ), // { "ecmaVersion": 2022 }
    ];

    Tester::new(MaxStatements::NAME, MaxStatements::PLUGIN, pass, fail).test_and_snapshot();
}
