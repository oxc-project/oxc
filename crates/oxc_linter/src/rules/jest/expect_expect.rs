use cow_utils::CowUtils;
use lazy_regex::Regex;
use rustc_hash::FxHashSet;

use oxc_ast::{
    AstKind,
    ast::{CallExpression, Expression, FormalParameter, Function, Statement},
};
use oxc_ast_visit::{Visit, walk};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, GetSpan, Span};
use oxc_syntax::scope::ScopeFlags;
use schemars::JsonSchema;

use crate::{
    ast_util::get_declaration_of_variable,
    context::LintContext,
    rule::Rule,
    utils::{
        JestFnKind, JestGeneralFnKind, PossibleJestNode, get_node_name, is_type_of_jest_fn_call,
    },
};

fn expect_expect_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Test has no assertions")
        .with_help("Add assertion(s) in this Test")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct ExpectExpect(Box<ExpectExpectConfig>);

#[derive(Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct ExpectExpectConfig {
    /// A list of function names that should be treated as assertion functions.
    ///
    /// NOTE: The default value is `["expect"]` for Jest and
    /// `["expect", "expectTypeOf", "assert", "assertType"]` for Vitest.
    #[serde(rename = "assertFunctionNames")]
    assert_function_names_jest: Vec<CompactStr>,
    #[schemars(skip)] // Skipped because this field isn't exposed to the user.
    assert_function_names_vitest: Vec<CompactStr>,
    /// An array of function names that should also be treated as test blocks.
    additional_test_block_functions: Vec<CompactStr>,
}

impl std::ops::Deref for ExpectExpect {
    type Target = ExpectExpectConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for ExpectExpectConfig {
    fn default() -> Self {
        Self {
            assert_function_names_jest: vec!["expect".into()],
            assert_function_names_vitest: vec![
                "expect".into(),
                "expectTypeOf".into(),
                "assert".into(),
                "assertType".into(),
            ],
            additional_test_block_functions: vec![],
        }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule triggers when there is no call made to `expect` in a test, ensure that there is at least one `expect` call made in a test.
    ///
    /// ### Why is this bad?
    ///
    /// People may forget to add assertions.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// it('should be a test', () => {
    ///     console.log('no assertion');
    /// });
    /// test('should assert something', () => {});
    /// ```
    ///
    /// This rule is compatible with [eslint-plugin-vitest](https://github.com/veritem/eslint-plugin-vitest/blob/v1.1.9/docs/rules/expect-expect.md),
    /// to use it, add the following configuration to your `.oxlintrc.json`:
    ///
    /// ```json
    /// {
    ///   "rules": {
    ///      "vitest/expect-expect": "error"
    ///   }
    /// }
    /// ```
    ExpectExpect,
    jest,
    correctness,
    config = ExpectExpectConfig,
);

impl Rule for ExpectExpect {
    fn from_configuration(value: serde_json::Value) -> Self {
        let default_assert_function_names_jest = vec!["expect".into()];
        let default_assert_function_names_vitest =
            vec!["expect".into(), "expectTypeOf".into(), "assert".into(), "assertType".into()];
        let config = value.get(0);

        let assert_function_names = config
            .and_then(|config| config.get("assertFunctionNames"))
            .and_then(serde_json::Value::as_array)
            .map(|v| {
                v.iter()
                    .filter_map(serde_json::Value::as_str)
                    .map(convert_pattern)
                    .collect::<Vec<_>>()
            });

        let assert_function_names_jest =
            assert_function_names.clone().unwrap_or(default_assert_function_names_jest);
        let assert_function_names_vitest =
            assert_function_names.unwrap_or(default_assert_function_names_vitest);

        let additional_test_block_functions = config
            .and_then(|config| config.get("additionalTestBlockFunctions"))
            .and_then(serde_json::Value::as_array)
            .map(|v| v.iter().filter_map(serde_json::Value::as_str).map(CompactStr::from).collect())
            .unwrap_or_default();

        Self(Box::new(ExpectExpectConfig {
            assert_function_names_jest,
            assert_function_names_vitest,
            additional_test_block_functions,
        }))
    }

    fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        run(self, jest_node, ctx);
    }
}

fn run<'a>(
    rule: &ExpectExpect,
    possible_jest_node: &PossibleJestNode<'a, '_>,
    ctx: &LintContext<'a>,
) {
    let node = possible_jest_node.node;
    if let AstKind::CallExpression(call_expr) = node.kind() {
        let name = get_node_name(&call_expr.callee);
        if is_type_of_jest_fn_call(
            call_expr,
            possible_jest_node,
            ctx,
            &[JestFnKind::General(JestGeneralFnKind::Test)],
        ) || rule.additional_test_block_functions.contains(&name)
        {
            if let Some(member_expr) = call_expr.callee.as_member_expression() {
                let Some(property_name) = member_expr.static_property_name() else {
                    return;
                };
                if property_name == "todo" {
                    return;
                }
                if property_name == "skip" && ctx.frameworks().is_vitest() {
                    return;
                }
            }

            let assert_function_names = if ctx.frameworks().is_vitest() {
                &rule.assert_function_names_vitest
            } else {
                &rule.assert_function_names_jest
            };

            let mut visitor = AssertionVisitor::new(ctx, assert_function_names);

            // Visit each argument of the test call
            for argument in &call_expr.arguments {
                if let Some(expr) = argument.as_expression() {
                    visitor.check_expression(expr);
                    if visitor.found_assertion {
                        return;
                    }
                }
            }

            if !visitor.found_assertion {
                ctx.diagnostic(expect_expect_diagnostic(call_expr.callee.span()));
            }
        }
    }
}

struct AssertionVisitor<'a, 'b> {
    ctx: &'b LintContext<'a>,
    assert_function_names: &'b [CompactStr],
    visited: FxHashSet<Span>,
    found_assertion: bool,
}

impl<'a, 'b> AssertionVisitor<'a, 'b> {
    fn new(ctx: &'b LintContext<'a>, assert_function_names: &'b [CompactStr]) -> Self {
        Self { ctx, assert_function_names, visited: FxHashSet::default(), found_assertion: false }
    }

    fn check_expression(&mut self, expr: &Expression<'a>) {
        // Avoid infinite loops by tracking visited expressions
        if !self.visited.insert(expr.span()) {
            return;
        }

        match expr {
            Expression::FunctionExpression(fn_expr) => {
                if let Some(body) = &fn_expr.body {
                    self.visit_function_body(body);
                }
            }
            Expression::ArrowFunctionExpression(arrow_expr) => {
                self.visit_function_body(&arrow_expr.body);
            }
            Expression::CallExpression(call_expr) => {
                self.visit_call_expression(call_expr);
            }
            Expression::Identifier(ident) => {
                self.check_identifier(ident);
            }
            Expression::AwaitExpression(expr) => {
                self.check_expression(&expr.argument);
            }
            Expression::ArrayExpression(array_expr) => {
                for element in &array_expr.elements {
                    if let Some(element_expr) = element.as_expression() {
                        self.check_expression(element_expr);
                        if self.found_assertion {
                            return;
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn check_identifier(&mut self, ident: &oxc_ast::ast::IdentifierReference<'a>) {
        let Some(node) = get_declaration_of_variable(ident, self.ctx) else {
            return;
        };
        let AstKind::Function(function) = node.kind() else {
            return;
        };
        if let Some(body) = &function.body {
            self.visit_function_body(body);
        }
    }
}

impl<'a> Visit<'a> for AssertionVisitor<'a, '_> {
    fn visit_call_expression(&mut self, call_expr: &CallExpression<'a>) {
        let name = get_node_name(&call_expr.callee);
        if matches_assert_function_name(&name, self.assert_function_names) {
            self.found_assertion = true;
            return;
        }

        for argument in &call_expr.arguments {
            if let Some(expr) = argument.as_expression() {
                self.check_expression(expr);
                if self.found_assertion {
                    return;
                }
            }
        }

        walk::walk_call_expression(self, call_expr);
    }

    fn visit_expression_statement(&mut self, stmt: &oxc_ast::ast::ExpressionStatement<'a>) {
        self.check_expression(&stmt.expression);
        if !self.found_assertion {
            walk::walk_expression_statement(self, stmt);
        }
    }

    fn visit_block_statement(&mut self, block: &oxc_ast::ast::BlockStatement<'a>) {
        for stmt in &block.body {
            self.visit_statement(stmt);
            if self.found_assertion {
                return;
            }
        }
    }

    fn visit_if_statement(&mut self, if_stmt: &oxc_ast::ast::IfStatement<'a>) {
        if let Statement::BlockStatement(block_stmt) = &if_stmt.consequent {
            self.visit_block_statement(block_stmt);
        }
        if self.found_assertion {
            return;
        }
        if let Some(alternate) = &if_stmt.alternate {
            self.visit_statement(alternate);
        }
    }

    fn visit_function(&mut self, _func: &Function<'a>, _flags: ScopeFlags) {}

    fn visit_formal_parameter(&mut self, _param: &FormalParameter<'a>) {}
}

/// Checks if node names returned by getNodeName matches any of the given star patterns
fn matches_assert_function_name(name: &str, patterns: &[CompactStr]) -> bool {
    patterns.iter().any(|pattern| Regex::new(pattern).unwrap().is_match(name))
}

fn convert_pattern(pattern: &str) -> CompactStr {
    // Pre-process pattern, e.g.
    // request.*.expect -> request.[a-z\\d]*.expect
    // request.**.expect -> request.[a-z\\d\\.]*.expect
    // request.**.expect* -> request.[a-z\\d\\.]*.expect[a-z\\d]*
    let pattern = pattern
        .split('.')
        .map(|p| {
            if p == "**" {
                CompactStr::from("[a-z\\d\\.]*")
            } else {
                p.cow_replace('*', "[a-z\\d]*").into()
            }
        })
        .collect::<Vec<_>>()
        .join("\\.");

    // 'a.b.c' -> /^a\.b\.c(\.|$)/iu
    format!("(?ui)^{pattern}(\\.|$)").into()
}

#[test]
fn test() {
    use crate::tester::Tester;

    let mut pass = vec![
        ("it.todo('will test something eventually')", None),
        ("test.todo('will test something eventually')", None),
        ("['x']();", None),
        ("it('should pass', () => expect(true).toBeDefined())", None),
        ("test('should pass', () => expect(true).toBeDefined())", None),
        ("it('should pass', () => somePromise().then(() => expect(true).toBeDefined()))", None),
        ("it('should pass', myTest); function myTest() { expect(true).toBeDefined() }", None),
        (
            "
            test('should pass', () => {
                expect(true).toBeDefined();
                foo(true).toBe(true);
            });
            ",
            Some(serde_json::json!([{ "assertFunctionNames": ["expect", "foo"] }])),
        ),
        ("it('should return undefined',() => expectSaga(mySaga).returns());", Some(serde_json::json!([{ "assertFunctionNames": ["expectSaga"] }]))),
        ("test('verifies expect method call', () => expect$(123));", Some(serde_json::json!([{ "assertFunctionNames": ["expect\\$"] }]))),
        ("test('verifies expect method call', () => new Foo().expect(123));", Some(serde_json::json!([{ "assertFunctionNames": ["Foo.expect"] }]))),
        (
            "
        	test('verifies deep expect method call', () => {
        	tester.foo().expect(123);
        	});
        ",
            Some(serde_json::json!([{ "assertFunctionNames": ["tester.foo.expect"] }])),
        ),
        (
            "
        	test('verifies chained expect method call', () => {
        	tester
        		.foo()
        		.bar()
        		.expect(456);
        	});
        ",
            Some(serde_json::json!([{ "assertFunctionNames": ["tester.foo.bar.expect"] }])),
        ),
        (
            "
        	test('verifies the function call', () => {
        	td.verify(someFunctionCall())
        	})
        ",
            Some(serde_json::json!([{ "assertFunctionNames": ["td.verify"] }])),
        ),
        (
            "it('should pass', () => expect(true).toBeDefined())",
            Some(serde_json::json!([
                {
                "assertFunctionNames": "undefined",
                "additionalTestBlockFunctions": "undefined",
                },
            ])),
        ),
        (
            "
            theoretically('the number {input} is correctly translated to string', theories, theory => {
                const output = NumberToLongString(theory.input);
                expect(output).toBe(theory.expected);
            })
            ",
            Some(serde_json::json!([{ "additionalTestBlockFunctions": ["theoretically"] }])),
        ),
        ("test('should pass *', () => expect404ToBeLoaded());", Some(serde_json::json!([{ "assertFunctionNames": ["expect*"] }]))),
        ("test('should pass *', () => expect.toHaveStatus404());", Some(serde_json::json!([{ "assertFunctionNames": ["expect.**"] }]))),
        ("test('should pass', () => tester.foo().expect(123));", Some(serde_json::json!([{ "assertFunctionNames": ["tester.*.expect"] }]))),
        ("test('should pass **', () => tester.foo().expect(123));", Some(serde_json::json!([{ "assertFunctionNames": ["**"] }]))),
        ("test('should pass *', () => tester.foo().expect(123));", Some(serde_json::json!([{ "assertFunctionNames": ["*"] }]))),
        ("test('should pass', () => tester.foo().expect(123));", Some(serde_json::json!([{ "assertFunctionNames": ["tester.**"] }]))),
        ("test('should pass', () => tester.foo().expect(123));", Some(serde_json::json!([{ "assertFunctionNames": ["tester.*"] }]))),
        ("test('should pass', () => tester.foo().bar().expectIt(456));", Some(serde_json::json!([{ "assertFunctionNames": ["tester.**.expect*"] }]))),
        ("test('should pass', () => request.get().foo().expect(456));", Some(serde_json::json!([{ "assertFunctionNames": ["request.**.expect"] }]))),
        ("test('should pass', () => request.get().foo().expect(456));", Some(serde_json::json!([{ "assertFunctionNames": ["request.**.e*e*t"] }]))),
        (
            "
        	import { test } from '@jest/globals';

        	test('should pass', () => {
        	expect(true).toBeDefined();
        	foo(true).toBe(true);
        	});
        ",
            Some(serde_json::json!([{ "assertFunctionNames": ["expect", "foo"] }])),
        ),
        (
            "
        	import { test as checkThat } from '@jest/globals';

        	checkThat('this passes', () => {
        	expect(true).toBeDefined();
        	foo(true).toBe(true);
        	});
        ",
            Some(serde_json::json!([{ "assertFunctionNames": ["expect", "foo"] }])),
        ),
        (
            "
        	const { test } = require('@jest/globals');

        	test('verifies chained expect method call', () => {
        	tester
        		.foo()
        		.bar()
        		.expect(456);
        	});
        ",
            Some(serde_json::json!([{ "assertFunctionNames": ["tester.foo.bar.expect"] }])),
        ),
        (
            r#"
            it("should not warn on await expect", async () => {
                const asyncFunction = async () => {
                    throw new Error('nope')
                };
                await expect(asyncFunction()).rejects.toThrow();
            });
            "#,
            None,
        ),
        (
            r"
            it('should not warn on expect in Promise.all', async () => {
                await Promise.all([
                    expect(new Promise((resolve) => { resolve(1); }) ).resolves.toBe(1),
                    expect(new Promise((_, reject) => { reject(new Error('Failed')); })).rejects.toThrowError('Failed'),
                ]);
            });
            ",
            None,
        ),
        (
            r#"
            it("should not warn on await expect", async () => {
                if(true) {
                    const asyncFunction = async () => {
                        throw new Error('nope')
                    };
                    await expect(asyncFunction()).rejects.toThrow();
                }
            });
            "#,
            None,
        ),
        (
            r#"
            it("should not warn on await expect", async () => {
                {
                    const asyncFunction = async () => {
                        throw new Error('nope')
                    };
                    await expect(asyncFunction()).rejects.toThrow();
                }
            });
            "#,
            None,
        ),
        ("it('test', async () => { const array = [1]; for (const element of array) { expect(element).toBe(1); } });", None),
        (r"it('msg', async () => { const r = foo(); return expect(r).rejects.toThrow(); });", None),
    ];

    let mut fail = vec![
        ("it(\"should fail\", () => {});", None),
        ("it(\"should fail\", myTest); function myTest() {}", None),
        ("test(\"should fail\", () => {});", None),
        ("test.skip(\"should fail\", () => {});", None),
        (
            "afterEach(() => {});",
            Some(serde_json::json!([{ "additionalTestBlockFunctions": ["afterEach"] }])),
        ),
        // TODO: is this case usual? not support this now, which need visit all call expression and get it's node name
        // (
        //     "
        // 	theoretically('the number {input} is correctly translated to string', theories, theory => {
        // 	const output = NumberToLongString(theory.input);
        // 	})
        // ",
        //     Some(serde_json::json!([{ "additionalTestBlockFunctions": ["theoretically"] }])),
        // ),
        (r#"it("should fail", () => { somePromise.then(() => {}); });"#, None),
        (
            "test(\"should fail\", () => { foo(true).toBe(true); })",
            Some(serde_json::json!([{ "assertFunctionNames": ["expect"] }])),
        ),
        (
            "it(\"should also fail\",() => expectSaga(mySaga).returns());",
            Some(serde_json::json!([{ "assertFunctionNames": ["expect"] }])),
        ),
        (
            "test('should fail', () => request.get().foo().expect(456));",
            Some(serde_json::json!([{ "assertFunctionNames": ["request.*.expect"] }])),
        ),
        (
            "test('should fail', () => request.get().foo().bar().expect(456));",
            Some(serde_json::json!([{ "assertFunctionNames": ["request.foo**.expect"] }])),
        ),
        (
            "test('should fail', () => tester.request(123));",
            Some(serde_json::json!([{ "assertFunctionNames": ["request.*"] }])),
        ),
        (
            "test('should fail', () => request(123));",
            Some(serde_json::json!([{ "assertFunctionNames": ["request.*"] }])),
        ),
        (
            "test('should fail', () => request(123));",
            Some(serde_json::json!([{ "assertFunctionNames": ["request.**"] }])),
        ),
        (
            "
        	import { test as checkThat } from '@jest/globals';

        	checkThat('this passes', () => {
        	// ...
        	});
        ",
            Some(serde_json::json!([{ "assertFunctionNames": ["expect", "foo"] }])),
        ),
        (
            "
        	import { test as checkThat } from '@jest/globals';

        	checkThat.skip('this passes', () => {
        	// ...
        	});
        ",
            None,
        ),
        (
            r#"
            it("should warn on non-assert await expression", async () => {
                const asyncFunction = async () => {
                    throw new Error('nope')
                };
                await foo(asyncFunction()).rejects.toThrow();
            });
            "#,
            None,
        ),
        (
            r#"
            test("event emitters bound to CLS context", function(t) {
                t.test("emitter with newListener that removes handler", function(t) {
                    ee.on("newListener", function handler(event: any) {
                        this.removeListener("newListener", handler);
                    });
                });
            });
            "#,
            None,
        ),
    ];

    let pass_vitest = vec![
        (
            "
                import { test } from 'vitest';
                test.skip(\"skipped test\", () => {})
            ",
            None,
        ),
        ("it.todo(\"will test something eventually\")", None),
        ("test.todo(\"will test something eventually\")", None),
        ("['x']();", None),
        ("it(\"should pass\", () => expect(true).toBeDefined())", None),
        ("test(\"should pass\", () => expect(true).toBeDefined())", None),
        ("it(\"should pass\", () => somePromise().then(() => expect(true).toBeDefined()))", None),
        ("it(\"should pass\", myTest); function myTest() { expect(true).toBeDefined() }", None),
        (
            "
                test('should pass', () => {
                    expect(true).toBeDefined();
                    foo(true).toBe(true);
                });
            ",
            Some(serde_json::json!([{ "assertFunctionNames": ["expect", "foo"] }]))
        ),
        (
            "
                import { bench } from 'vitest'

                bench('normal sorting', () => {
                    const x = [1, 5, 4, 2, 3]
                    x.sort((a, b) => {
                        return a - b
                    })
                }, { time: 1000 })
            ",
            None,
        ),
        (
            "it(\"should return undefined\", () => expectSaga(mySaga).returns());",
            Some(serde_json::json!([{ "assertFunctionNames": ["expectSaga"] }])),
        ),
        (
            "test('verifies expect method call', () => expect$(123));",
            Some(serde_json::json!([{ "assertFunctionNames": ["expect\\$"] }])),
        ),
        (
            "test('verifies expect method call', () => new Foo().expect(123));",
            Some(serde_json::json!([{ "assertFunctionNames": ["Foo.expect"] }])),
        ),
        (
            "
                test('verifies deep expect method call', () => {
                    tester.foo().expect(123);
                });
            ",
            Some(serde_json::json!([{ "assertFunctionNames": ["tester.foo.expect"] }])),
        ),
        (
            "
                    test('verifies chained expect method call', () => {
                        tester
                            .foo()
                            .bar()
                            .expect(456);
                    });
            ",
            Some(serde_json::json!([{ "assertFunctionNames": ["tester.foo.bar.expect"] }])),
        ),
        (
            "
                test(\"verifies the function call\", () => {
                    td.verify(someFunctionCall())
                })
            ",
            Some(serde_json::json!([{ "assertFunctionNames": ["td.verify"] }])),
        ),
        (
            "it(\"should pass\", () => expect(true).toBeDefined())",
            Some(serde_json::json!([{
                "assertFunctionNames": "undefined",
                "additionalTestBlockFunctions": "undefined",
            }])),
        ),
        (
            "
                theoretically('the number {input} is correctly translated to string', theories, theory => {
                    const output = NumberToLongString(theory.input);
                    expect(output).toBe(theory.expected);
                })
            ",
            Some(serde_json::json!([{ "additionalTestBlockFunctions": ["theoretically"] }])),
        ),
        (
            "test('should pass *', () => expect404ToBeLoaded());",
            Some(serde_json::json!([{ "assertFunctionNames": ["expect*"] }])),
        ),
        (
            "test('should pass *', () => expect.toHaveStatus404());",
            Some(serde_json::json!([{ "assertFunctionNames": ["expect.**"] }])),
        ),
        (
            "test('should pass', () => tester.foo().expect(123));",
            Some(serde_json::json!([{ "assertFunctionNames": ["tester.*.expect"] }])),
        ),
        (
            "test('should pass **', () => tester.foo().expect(123));",
            Some(serde_json::json!([{ "assertFunctionNames": ["**"] }])),
        ),
        (
            "test('should pass *', () => tester.foo().expect(123));",
            Some(serde_json::json!([{ "assertFunctionNames": ["*"] }])),
        ),
        (
            "test('should pass', () => tester.foo().expect(123));",
            Some(serde_json::json!([{ "assertFunctionNames": ["tester.**"] }])),
        ),
        (
            "test('should pass', () => tester.foo().expect(123));",
            Some(serde_json::json!([{ "assertFunctionNames": ["tester.*"] }])),
        ),
        (
            "test('should pass', () => tester.foo().bar().expectIt(456));",
            Some(serde_json::json!([{ "assertFunctionNames": ["tester.**.expect*"] }])),
        ),
        (
            "test('should pass', () => request.get().foo().expect(456));",
            Some(serde_json::json!([{ "assertFunctionNames": ["request.**.expect"] }])),
        ),
        (
            "test('should pass', () => request.get().foo().expect(456));",
            Some(serde_json::json!([{ "assertFunctionNames": ["request.**.e*e*t"] }])),
        ),
        (
            "
                import { test } from 'vitest';

                test('should pass', () => {
                    expect(true).toBeDefined();
                    foo(true).toBe(true);
                });
            ",
            Some(serde_json::json!([{ "assertFunctionNames": ["expect", "foo"] }])),
        ),
        (
            "
                import { test as checkThat } from 'vitest';

                checkThat('this passes', () => {
                    expect(true).toBeDefined();
                    foo(true).toBe(true);
                });
            ",
            Some(serde_json::json!([{ "assertFunctionNames": ["expect", "foo"] }])),
        ),
        (
            "
                const { test } = require('vitest');

                test('verifies chained expect method call', () => {
                    tester
                    .foo()
                    .bar()
                    .expect(456);
                });
            ",
            Some(serde_json::json!([{ "assertFunctionNames": ["tester.foo.bar.expect"] }])),
        ),
        (
            "
                it(\"should pass with 'typecheck' enabled\", () => {
                    expectTypeOf({ a: 1 }).toEqualTypeOf<{ a: number }>()
                });
            ",
            None
        ),
        (
            "
                import { assert, it } from 'vitest';

                it('test', () => {
                    assert.throws(() => {
                        throw Error('Invalid value');
                    });
                });
            ",
            Some(serde_json::json!([{ "assertFunctionNames": ["assert"] }])),
        ),
        (
            "
                import { expectTypeOf } from 'vitest'

                expectTypeOf({ a: 1 }).toEqualTypeOf<{ a: number }>()
            ",
            Some(serde_json::json!([{ "assertFunctionNames": ["expectTypeOf"] }])),
        ),
        (
            "
                import { assertType } from 'vitest'

                function concat(a: string, b: string): string
                function concat(a: number, b: number): number
                function concat(a: string | number, b: string | number): string | number

                assertType<string>(concat('a', 'b'))
                assertType<number>(concat(1, 2))
                // @ts-expect-error wrong types
                assertType(concat('a', 2))
            ",
            Some(serde_json::json!([{ "assertFunctionNames": ["assertType"] }])),
        ),
    ];

    let fail_vitest = vec![
        ("it(\"should fail\", () => {});", None),
        ("it(\"should fail\", myTest); function myTest() {}", None),
        ("test(\"should fail\", () => {});", None),
        (
            "afterEach(() => {});",
            Some(serde_json::json!([{ "additionalTestBlockFunctions": ["afterEach"] }])),
        ),
        // Todo: currently it's not support
        // (
        //     "
        //         theoretically('the number {input} is correctly translated to string', theories, theory => {
        //             const output = NumberToLongString(theory.input);
        //         })
        //     ",
        //     Some(serde_json::json!([{ "additionalTestBlockFunctions": ["theoretically"] }])),
        // ),
        ("it(\"should fail\", () => { somePromise.then(() => {}); });", None),
        (
            "test(\"should fail\", () => { foo(true).toBe(true); })",
            Some(serde_json::json!([{ "assertFunctionNames": ["expect"] }])),
        ),
        (
            "it(\"should also fail\",() => expectSaga(mySaga).returns());",
            Some(serde_json::json!([{ "assertFunctionNames": ["expect"] }])),
        ),
        (
            "test('should fail', () => request.get().foo().expect(456));",
            Some(serde_json::json!([{ "assertFunctionNames": ["request.*.expect"] }])),
        ),
        (
            "test('should fail', () => request.get().foo().bar().expect(456));",
            Some(serde_json::json!([{ "assertFunctionNames": ["request.foo**.expect"] }])),
        ),
        (
            "test('should fail', () => tester.request(123));",
            Some(serde_json::json!([{ "assertFunctionNames": ["request.*"] }])),
        ),
        (
            "test('should fail', () => request(123));",
            Some(serde_json::json!([{ "assertFunctionNames": ["request.*"] }])),
        ),
        (
            "test('should fail', () => request(123));",
            Some(serde_json::json!([{ "assertFunctionNames": ["request.**"] }])),
        ),
        (
            "
                import { test as checkThat } from 'vitest';

                checkThat('this passes', () => {
                    // ...
                });
            ",
            Some(serde_json::json!([{ "assertFunctionNames": ["expect", "foo"] }])),
        ),
        // Todo: currently we couldn't support ignore the typecheck option.
        // (
        //     "
        //         it(\"should fail without 'typecheck' enabled\", () => {
        //             expectTypeOf({ a: 1 }).toEqualTypeOf<{ a: number }>()
        //         });
        //     ",
        //     None,
        // ),
    ];

    pass.extend(pass_vitest);
    fail.extend(fail_vitest);

    Tester::new(ExpectExpect::NAME, ExpectExpect::PLUGIN, pass, fail)
        .with_jest_plugin(true)
        .with_vitest_plugin(true)
        .test_and_snapshot();
}
