use oxc_ast::{
    AstKind,
    ast::{Argument, Expression, FunctionBody, Statement},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_str::CompactStr;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    context::LintContext,
    rule::Rule,
    rules::PossibleJestNode,
    utils::{
        JestGeneralFnKind, convert_pattern, get_node_name, matches_assert_function_name,
        parse_expect_jest_fn_call, parse_general_jest_fn_call,
    },
};

fn prefer_ending_with_an_expect_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Test must end with an assertion")
        .with_help("Add an `expect` or assertion call as the last statement in the test block.")
        .with_label(span)
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct PreferEndingWithAnExpect(Box<PreferEndingWithAnExpectConfig>);

#[derive(Debug, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PreferEndingWithAnExpectConfig {
    /// An array of function names that should also be treated as test blocks.
    additional_test_block_functions: Vec<CompactStr>,
    /// A list of function names that should be treated as assertion functions.
    assert_function_names: Vec<CompactStr>,
}

impl std::ops::Deref for PreferEndingWithAnExpect {
    type Target = PreferEndingWithAnExpectConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for PreferEndingWithAnExpectConfig {
    fn default() -> Self {
        Self {
            assert_function_names: vec!["expect".into()],
            additional_test_block_functions: vec![],
        }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces that test blocks end with an assertion (`expect` or a configured
    /// assertion function).
    ///
    /// ### Why is this bad?
    ///
    /// A test that doesn't end with an assertion may be performing side effects
    /// or setup after its last check, which makes the test harder to understand
    /// and can hide failures. Ending with an assertion ensures the test's final
    /// action is verifying behavior.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule with default values:
    /// ```js
    /// it('lets me change the selected option', () => {
    ///   const container = render(MySelect, {
    ///     props: { options: [1, 2, 3], selected: 1 },
    ///   });
    ///
    ///   expect(container).toBeDefined();
    ///   expect(container.toHTML()).toContain('<option value="1" selected>');
    ///
    ///   container.setProp('selected', 2);
    /// });
    /// ```
    ///
    /// Examples of **correct** code for this rule with defaults values:
    /// ```js
    /// it('lets me change the selected option', () => {
    ///   const container = render(MySelect, {
    ///     props: { options: [1, 2, 3], selected: 1 },
    ///   });
    ///
    ///   expect(container).toBeDefined();
    ///   expect(container.toHTML()).toContain('<option value="1" selected>');
    ///
    ///   container.setProp('selected', 2);
    ///
    ///   expect(container.toHTML()).not.toContain('<option value="1" selected>');
    ///   expect(container.toHTML()).toContain('<option value="2" selected>');
    /// });
    /// ```
    ///
    /// Examples of **incorrect** code for this rule with `{ "assertFunctionNames": ["expect"] }`:
    /// ```js
    /// import { expectSaga } from 'redux-saga-test-plan';
    /// import { addSaga } from '../src/sagas';
    ///
    /// test('returns sum', () => {
    ///   expectSaga(addSaga, 1, 1).returns(2).run();
    /// });
    /// ```
    ///
    /// Examples of **correct** code for this rule with `{ "assertFunctionNames": ["expect"] }`:
    /// ```js
    /// import { expectSaga } from 'redux-saga-test-plan';
    /// import { addSaga } from '../src/sagas';
    ///
    /// test('returns sum', () => {
    ///   expectSaga(addSaga, 1, 1).returns(2).run();
    /// });
    /// ```
    ///
    /// Examples of **correct** code for this rule with `{ "additionalTestBlockFunctions": ["each.test"] }`:
    /// ```js
    /// each([
    ///   [2, 3],
    ///   [1, 3],
    /// ]).test(
    ///   'the selection can change from %d to %d',
    ///   (firstSelection, secondSelection) => {
    ///     const container = render(MySelect, {
    ///       props: { options: [1, 2, 3], selected: firstSelection },
    ///     });
    ///
    ///     expect(container).toBeDefined();
    ///     expect(container.toHTML()).toContain(
    ///       `<option value="${firstSelection}" selected>`,
    ///     );
    ///
    ///     container.setProp('selected', secondSelection);
    ///
    ///     expect(container.toHTML()).not.toContain(
    ///       `<option value="${firstSelection}" selected>`,
    ///     );
    ///     expect(container.toHTML()).toContain(
    ///       `<option value="${secondSelection}" selected>`,
    ///     );
    ///   },
    /// );
    /// ```
    PreferEndingWithAnExpect,
    jest,
    style,
    config = PreferEndingWithAnExpectConfig,
    version = "1.60.0"
);

impl Rule for PreferEndingWithAnExpect {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        let config = value.get(0);

        let assert_function_names = config
            .and_then(|config| config.get("assertFunctionNames"))
            .and_then(serde_json::Value::as_array)
            .map(|v| {
                v.iter()
                    .filter_map(serde_json::Value::as_str)
                    .map(convert_pattern)
                    .collect::<Vec<_>>()
            })
            .unwrap_or(vec!["expect".into()]);

        let additional_test_block_functions = config
            .and_then(|config| config.get("additionalTestBlockFunctions"))
            .and_then(serde_json::Value::as_array)
            .map(|v| v.iter().filter_map(serde_json::Value::as_str).map(CompactStr::from).collect())
            .unwrap_or_default();

        Ok(Self(Box::new(PreferEndingWithAnExpectConfig {
            additional_test_block_functions,
            assert_function_names,
        })))
    }

    fn run_on_jest_node<'a, 'c>(
        &self,
        possible_jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        let node = possible_jest_node.node;

        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        let name = get_node_name(&call_expr.callee);

        let Some(test_fn_argument) = call_expr.arguments.get(1) else {
            return;
        };

        let Some(function_body) = function_argument(test_fn_argument) else {
            return;
        };

        let Some(parsed_jest_fn) = parse_general_jest_fn_call(call_expr, possible_jest_node, ctx)
        else {
            return;
        };

        // Only real test callbacks should be checked here. Other Jest/Vitest APIs
        // such as `vi.mock(..., factory)` also accept functions in the second slot.
        let is_test_block = parsed_jest_fn
            .kind
            .to_general()
            .is_some_and(|test_kind| matches!(test_kind, JestGeneralFnKind::Test));
        let is_additional_test_block = self.additional_test_block_functions.contains(&name);

        if !is_test_block && !is_additional_test_block {
            return;
        }

        if self.is_valid_last_statement(function_body, ctx) {
            return;
        }

        ctx.diagnostic(prefer_ending_with_an_expect_diagnostic(call_expr.callee.span()));
    }
}

impl PreferEndingWithAnExpect {
    fn is_valid_last_statement<'a>(
        &self,
        function_body: &'a FunctionBody<'a>,
        ctx: &LintContext<'a>,
    ) -> bool {
        let Some(statement) = function_body.statements.last() else {
            return false;
        };

        let statement_expression = match statement {
            Statement::ExpressionStatement(expression_statement) => {
                &expression_statement.expression
            }
            _ => {
                return false;
            }
        };

        let call_expression = match &statement_expression {
            Expression::AwaitExpression(awaited) => {
                let Expression::CallExpression(ref call_expression_awaited) = awaited.argument
                else {
                    return false;
                };

                call_expression_awaited
            }
            Expression::CallExpression(last_call_expression) => last_call_expression,
            _ => {
                return false;
            }
        };

        let possible_jest_node = PossibleJestNode {
            node: ctx.nodes().get_node(call_expression.node_id()),
            original: None,
        };

        if parse_expect_jest_fn_call(call_expression, &possible_jest_node, ctx).is_some() {
            return true;
        }

        let node_name = get_node_name(&call_expression.callee);

        matches_assert_function_name(&node_name, &self.assert_function_names)
    }
}

fn function_argument<'a>(argument: &'a Argument<'a>) -> Option<&'a FunctionBody<'a>> {
    match argument {
        Argument::ArrowFunctionExpression(array_fn) => Some(&array_fn.body),
        Argument::FunctionExpression(function) => function.body.as_ref().map(AsRef::as_ref),
        _ => None,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (r#"it.todo("will test something eventually")"#, None),
        (r#"test.todo("will test something eventually")"#, None),
        ("['x']();", None),
        (r#"it("is weird", "because this should be a function")"#, None),
        (r#"it("is weird", "because this should be a function", () => {})"#, None),
        (r#"it("should pass", () => expect(true).toBeDefined())"#, None),
        (r#"test("should pass", () => expect(true).toBeDefined())"#, None),
        (r#"it("should pass", myTest); function myTest() { expect(true).toBeDefined() }"#, None),
        (
            "test('should pass', () => {
              expect(true).toBeDefined();
              foo(true).toBe(true);
            });",
            Some(serde_json::json!([{ "assertFunctionNames": ["expect", "foo"] }])),
        ),
        (
            r#"it("should return undefined",() => expectSaga(mySaga).returns());"#,
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
            "test('verifies deep expect method call', () => {
              tester.foo().expect(123);
            });",
            Some(serde_json::json!([{ "assertFunctionNames": ["tester.foo.expect"] }])),
        ),
        (
            "test('verifies chained expect method call', () => {
              doSomething();
              tester
                .foo()
                .bar()
                .expect(456);
            });",
            Some(serde_json::json!([{ "assertFunctionNames": ["tester.foo.bar.expect"] }])),
        ),
        (
            r#"test("verifies the function call", () => {
              td.verify(someFunctionCall())
            })"#,
            Some(serde_json::json!([{ "assertFunctionNames": ["td.verify"] }])),
        ),
        (r#"it("should pass", async () => expect(true).toBeDefined())"#, None),
        (
            r#"it("should pass", () => expect(true).toBeDefined())"#,
            Some(
                serde_json::json!([ { "assertFunctionNames": "undefined", "additionalTestBlockFunctions": "undefined", }, ]),
            ),
        ),
        (r#"it("should pass", () => { expect(true).toBeDefined() })"#, None),
        (r#"it("should pass", function () { expect(true).toBeDefined() })"#, None),
        (
            "it('is a complete test', () => {
              const container = render(Greeter);
              expect(container).toBeDefined();
              container.setProp('name', 'Bob');
              expect(container.toHTML()).toContain('Hello Bob!');
            });",
            None,
        ),
        (
            "it('is a complete test', async () => {
              const container = render(Greeter);
              expect(container).toBeDefined();
              container.setProp('name', 'Bob');
              await expect(container.toHTML()).resolve.toContain('Hello Bob!');
            });",
            None,
        ),
        (
            "it('is a complete test', async function () {
              const container = render(Greeter);
              expect(container).toBeDefined();
              container.setProp('name', 'Bob');
              await expect(container.toHTML()).resolve.toContain('Hello Bob!');
            });",
            None,
        ),
        (
            "describe('GET /user', function () {
              it('responds with json', function (done) {
                doSomething();
                request(app).get('/user').expect('Content-Type', /json/).expect(200, done);
              });
            });",
            Some(serde_json::json!([{ "assertFunctionNames": ["expect", "request.**.expect"] }])),
        ),
        (
            r#"each([
              [2, 3],
              [1, 3],
            ]).test(
              'the selection can change from %d to %d',
              (firstSelection, secondSelection) => {
                const container = render(MySelect, {
                  props: { options: [1, 2, 3], selected: firstSelection },
                });
                expect(container).toBeDefined();
                expect(container.toHTML()).toContain(
                  `<option value="${firstSelection}" selected>`
                );
                container.setProp('selected', secondSelection);
                expect(container.toHTML()).not.toContain(
                  `<option value="${firstSelection}" selected>`
                );
                expect(container.toHTML()).toContain(
                  `<option value="${secondSelection}" selected>`
                );
              }
            );"#,
            Some(serde_json::json!([{ "additionalTestBlockFunctions": ["each.test"] }])),
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
            "import { test } from '@jest/globals';
            test('should pass', () => {
              expect(true).toBeDefined();
              foo(true).toBe(true);
            });",
            Some(serde_json::json!([{ "assertFunctionNames": ["expect", "foo"] }])),
        ),
        (
            "import { test as checkThat } from '@jest/globals';
            checkThat('this passes', () => {
              expect(true).toBeDefined();
              foo(true).toBe(true);
            });",
            Some(serde_json::json!([{ "assertFunctionNames": ["expect", "foo"] }])),
        ),
        (
            "const { test } = require('@jest/globals');
            test('verifies chained expect method call', () => {
              tester
                .foo()
                .bar()
                .expect(456);
            });",
            Some(serde_json::json!([{ "assertFunctionNames": ["tester.foo.bar.expect"] }])),
        ),
        (
            "import { vi } from 'vitest';
            vi.mock(import('foo'), () => vi.fn());",
            None,
        ),
    ];

    let fail = vec![
        (r#"it("should fail", () => {});"#, None),
        (r#"test("should fail", () => {});"#, None),
        (r#"test.skip("should fail", () => {});"#, None),
        (r#"it("should fail", () => { somePromise.then(() => {}); });"#, None),
        (
            r#"test("should fail", () => { foo(true).toBe(true); })"#,
            Some(serde_json::json!([{ "assertFunctionNames": ["expect"] }])),
        ),
        (
            r#"it("should also fail",() => expectSaga(mySaga).returns());"#,
            Some(serde_json::json!([{ "assertFunctionNames": ["expect"] }])),
        ),
        (r#"it("should pass", () => somePromise().then(() => expect(true).toBeDefined()))"#, None),
        (r#"it("should pass", () => render(Greeter))"#, None),
        (r#"it("should pass", () => { render(Greeter) })"#, None),
        (r#"it("should pass", function () { render(Greeter) })"#, None),
        (r#"it("should not pass", () => class {})"#, None),
        (r#"it("should not pass", () => ([]))"#, None),
        (r#"it("should not pass", () => { const x = []; })"#, None),
        (r#"it("should not pass", function () { class Mx {} })"#, None),
        (
            "it('is a complete test', () => {
              const container = render(Greeter);
              expect(container).toBeDefined();
              container.setProp('name', 'Bob');
            });",
            None,
        ),
        (
            "it('is a complete test', async () => {
              const container = render(Greeter);
              await expect(container).toBeDefined();
              await container.setProp('name', 'Bob');
            });",
            None,
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
            "import { test as checkThat } from '@jest/globals';
            checkThat('this passes', () => {
              // ...
            });",
            Some(serde_json::json!([{ "assertFunctionNames": ["expect", "foo"] }])),
        ),
        (
            "import { test as checkThat } from '@jest/globals';
            checkThat.skip('this passes', () => {
              // ...
            });",
            None,
        ),
    ];

    Tester::new(PreferEndingWithAnExpect::NAME, PreferEndingWithAnExpect::PLUGIN, pass, fail)
        .with_jest_plugin(true)
        .test_and_snapshot();
}
