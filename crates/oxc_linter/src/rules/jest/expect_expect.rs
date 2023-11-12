use oxc_ast::{
    ast::{Argument, CallExpression, Expression, Statement},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use regex::Regex;

use crate::{
    ast_util::get_declaration_of_variable,
    context::LintContext,
    rule::Rule,
    utils::{
        collect_possible_jest_call_node, get_node_name, is_type_of_jest_fn_call, JestFnKind,
        JestGeneralFnKind, PossibleJestNode,
    },
};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-jest(expect-expect): Test has no assertions")]
#[diagnostic(severity(warning), help("Add assertion(s) in this Test"))]
struct ExpectExpectDiagnostic(#[label] pub Span);

#[derive(Debug, Clone)]
pub struct ExpectExpect {
    assert_function_names: Vec<String>,
    additional_test_block_functions: Vec<String>,
}

impl Default for ExpectExpect {
    fn default() -> Self {
        Self {
            assert_function_names: vec![String::from("expect")],
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
    ///  People may forget to add assertions.
    ///
    /// ### Example
    ///
    /// ```javascript
    /// it('should be a test', () => {
    ///     console.log('no assertion');
    /// });
    /// test('should assert something', () => {});
    /// ```
    ExpectExpect,
    correctness
);

impl Rule for ExpectExpect {
    fn from_configuration(value: serde_json::Value) -> Self {
        let default_assert_function_names = vec![String::from("expect")];
        let config = value.get(0);

        let assert_function_names = config
            .and_then(|config| config.get("assertFunctionNames"))
            .and_then(serde_json::Value::as_array)
            .map_or(default_assert_function_names, |v| {
                v.iter().filter_map(serde_json::Value::as_str).map(convert_pattern).collect()
            });

        let additional_test_block_functions = config
            .and_then(|config| config.get("additionalTestBlockFunctions"))
            .and_then(serde_json::Value::as_array)
            .map(|v| {
                v.iter().filter_map(serde_json::Value::as_str).map(ToString::to_string).collect()
            })
            .unwrap_or_default();

        Self { assert_function_names, additional_test_block_functions }
    }
    fn run_once(&self, ctx: &LintContext) {
        for possible_jest_node in &collect_possible_jest_call_node(ctx) {
            run(self, possible_jest_node, ctx);
        }
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
            if let Expression::MemberExpression(member_expr) = &call_expr.callee {
                let Some(property_name) = member_expr.static_property_name() else {
                    return;
                };
                if property_name == "todo" {
                    return;
                }
            }

            let has_assert_function = check_arguments(call_expr, &rule.assert_function_names, ctx);

            if !has_assert_function {
                ctx.diagnostic(ExpectExpectDiagnostic(call_expr.span));
            }
        }
    }
}

fn check_arguments<'a>(
    call_expr: &'a CallExpression<'a>,
    assert_function_names: &[String],
    ctx: &LintContext<'a>,
) -> bool {
    call_expr.arguments.iter().any(|argument| {
        if let Argument::Expression(expr) = argument {
            return check_assert_function_used(expr, assert_function_names, ctx);
        }
        false
    })
}

fn check_assert_function_used<'a>(
    expr: &'a Expression<'a>,
    assert_function_names: &[String],
    ctx: &LintContext<'a>,
) -> bool {
    match expr {
        Expression::FunctionExpression(fn_expr) => {
            let body = &fn_expr.body;
            if let Some(body) = body {
                return check_statements(&body.statements, assert_function_names, ctx);
            }
        }
        Expression::ArrowExpression(arrow_expr) => {
            let body = &arrow_expr.body;
            return check_statements(&body.statements, assert_function_names, ctx);
        }
        Expression::CallExpression(call_expr) => {
            let name = get_node_name(&call_expr.callee);
            if matches_assert_function_name(&name, assert_function_names) {
                return true;
            }

            let has_assert_function = check_arguments(call_expr, assert_function_names, ctx);

            return has_assert_function;
        }
        Expression::Identifier(ident) => {
            let Some(node) = get_declaration_of_variable(ident, ctx) else {
                return false;
            };
            let AstKind::Function(function) = node.kind() else {
                return false;
            };
            let Some(body) = &function.body else {
                return false;
            };
            return check_statements(&body.statements, assert_function_names, ctx);
        }
        Expression::AwaitExpression(expr) => {
            return check_assert_function_used(&expr.argument, assert_function_names, ctx);
        }
        _ => {}
    };

    false
}

fn check_statements<'a>(
    statements: &'a oxc_allocator::Vec<Statement<'a>>,
    assert_function_names: &[String],
    ctx: &LintContext<'a>,
) -> bool {
    statements.iter().any(|statement| {
        if let Statement::ExpressionStatement(expr_stmt) = statement {
            return check_assert_function_used(&expr_stmt.expression, assert_function_names, ctx);
        }
        false
    })
}

/// Checks if node names returned by getNodeName matches any of the given star patterns
fn matches_assert_function_name(name: &str, patterns: &[String]) -> bool {
    patterns.iter().any(|pattern| Regex::new(pattern).unwrap().is_match(name))
}

fn convert_pattern(pattern: &str) -> String {
    // Pre-process pattern, e.g.
    // request.*.expect -> request.[a-z\\d]*.expect
    // request.**.expect -> request.[a-z\\d\\.]*.expect
    // request.**.expect* -> request.[a-z\\d\\.]*.expect[a-z\\d]*
    let pattern = pattern
        .split('.')
        .map(|p| if p == "**" { String::from("[a-z\\d\\.]*") } else { p.replace('*', "[a-z\\d]*") })
        .collect::<Vec<_>>()
        .join("\\.");

    // 'a.b.c' -> /^a\.b\.c(\.|$)/iu
    format!("(?ui)^{pattern}(\\.|$)")
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
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
        (
            "it('should return undefined',() => expectSaga(mySaga).returns());",
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
    ];

    let fail = vec![
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
    ];

    Tester::new(ExpectExpect::NAME, pass, fail).with_jest_plugin(true).test_and_snapshot();
}
