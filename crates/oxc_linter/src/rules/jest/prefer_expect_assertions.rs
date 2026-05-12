use oxc_ast::ast::{Expression, FunctionBody};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::NodeId;
use oxc_span::Span;
use oxc_syntax::scope::ScopeId;
use serde::Deserialize;
use std::borrow::Cow;

use crate::{
    context::LintContext,
    fixer::RuleFix,
    rule::{DefaultRuleConfig, Rule},
    rules::shared::prefer_expect_assertions::{
        DOCUMENTATION, PreferExpectAssertionsConfig, PreferExpectAssertionsRuleImpl,
        resolve_expect_local_name, should_check,
    },
    utils::collect_possible_jest_call_node,
};

fn have_expect_assertions(span: Span, prefix: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Every test should have either `{prefix}.assertions(<number of assertions>)` or `{prefix}.hasAssertions()` as its first expression.",
    ))
    .with_help(format!("Add `{prefix}.hasAssertions()` or `{prefix}.assertions(<number>)` as the first statement in the test."))
    .with_label(span)
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct PreferExpectAssertions(Box<PreferExpectAssertionsConfig>);

impl std::ops::Deref for PreferExpectAssertions {
    type Target = PreferExpectAssertionsConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    PreferExpectAssertions,
    jest,
    style,
    docs = DOCUMENTATION,
    suggestion,
    version = "1.62.0",
    config = PreferExpectAssertionsConfig
);

impl Rule for PreferExpectAssertions {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<PreferExpectAssertionsConfig>>(value)
            .map(|c| Self(Box::new(c.into_inner())))
    }

    fn run_once(&self, ctx: &LintContext) {
        let mut possible_jest_nodes = collect_possible_jest_call_node(ctx);
        possible_jest_nodes.sort_unstable_by_key(|n| n.node.id());

        // Resolve the file-level expect local name once (e.g. `"expect"` or `"e"`
        // for `import { expect as e }`). Per-callback vitest fixture overrides
        // are handled in `resolve_expect_source`.
        let file_expect_prefix = resolve_expect_local_name(ctx, &["@jest/globals"]);

        let mut covered_describe_ids: Vec<NodeId> = Vec::new();

        for jest_node in &possible_jest_nodes {
            self.check_node(jest_node, &file_expect_prefix, &mut covered_describe_ids, ctx);
        }
    }
}

impl PreferExpectAssertionsRuleImpl for PreferExpectAssertions {
    fn has_options(&self) -> bool {
        self.0.compute_config()
    }

    fn resolve_expect<'a, 'r>(
        &self,
        callback: &Expression<'a>,
        file_expect_prefix: &'r str,
        ctx: &LintContext<'a>,
    ) -> Option<Cow<'r, str>> {
        if is_expect_shadowed_in(callback, ctx) {
            return None;
        }

        Some(Cow::Borrowed(file_expect_prefix))
    }

    fn report_have_expect_assertions(
        &self,
        span: Span,
        prefix: &str,
        suggestions: [RuleFix; 2],
        ctx: &LintContext<'_>,
    ) {
        ctx.diagnostic_with_suggestions(have_expect_assertions(span, prefix), suggestions);
    }

    fn should_check_node(&self, body: &FunctionBody<'_>, is_async: bool, prefix: &str) -> bool {
        should_check(self.0.as_ref(), body, is_async, prefix)
    }
}

fn is_expect_shadowed_in(callback: &Expression<'_>, ctx: &LintContext<'_>) -> bool {
    callback_scope_id(callback)
        .is_some_and(|id| ctx.scoping().get_binding(id, "expect".into()).is_some())
}

fn callback_scope_id(callback: &Expression<'_>) -> Option<ScopeId> {
    match callback {
        Expression::FunctionExpression(func) => func.scope_id.get(),
        Expression::ArrowFunctionExpression(func) => func.scope_id.get(),
        _ => None,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (r#"test("nonsense", [])"#, None),
        (r#"test("it1", () => {expect.assertions(0);})"#, None),
        (r#"test("it1", function() {expect.assertions(0);})"#, None),
        (r#"test("it1", function() {expect.hasAssertions();})"#, None),
        (r#"it("it1", function() {expect.assertions(0);})"#, None),
        (
            r#"it("it1", function() {
              expect.assertions(1);
              expect(someValue).toBe(true)
            });"#,
            None,
        ),
        (r#"test("it1")"#, None),
        (r#"itHappensToStartWithIt("foo", function() {})"#, None),
        (r#"testSomething("bar", function() {})"#, None),
        ("it(async () => {expect.assertions(0);})", None),
        (
            r#"it("returns numbers that are greater than four", function() {
              expect.assertions(2);
              for(let thing in things) {
                expect(number).toBeGreaterThan(4);
              }
            });"#,
            None,
        ),
        (
            r#"it("returns numbers that are greater than four", function() {
              expect.hasAssertions();
              for (let i = 0; i < things.length; i++) {
                expect(number).toBeGreaterThan(4);
              }
            });"#,
            None,
        ),
        (
            r#"it("it1", async () => {
              expect.assertions(1);
              expect(someValue).toBe(true)
            });"#,
            Some(serde_json::json!([{ "onlyFunctionsWithAsyncKeyword": true }])),
        ),
        (
            r#"it("it1", function() {
              expect(someValue).toBe(true)
            });"#,
            Some(serde_json::json!([{ "onlyFunctionsWithAsyncKeyword": true }])),
        ),
        (
            r#"it("it1", () => {})"#,
            Some(serde_json::json!([{ "onlyFunctionsWithAsyncKeyword": true }])),
        ),
        (
            r#"it("returns numbers that are greater than four", async () => {
              expect.assertions(2);
              for(let thing in things) {
                expect(number).toBeGreaterThan(4);
              }
            });"#,
            Some(serde_json::json!([{ "onlyFunctionsWithAsyncKeyword": true }])),
        ),
        (
            r#"it("returns numbers that are greater than four", () => {
              for(let thing in things) {
                expect(number).toBeGreaterThan(4);
              }
            });"#,
            Some(serde_json::json!([{ "onlyFunctionsWithAsyncKeyword": true }])),
        ),
        (
            r#"import { expect as pleaseExpect } from '@jest/globals';
            it("returns numbers that are greater than four", function() {
              pleaseExpect.assertions(2);
              for(let thing in things) {
                pleaseExpect(number).toBeGreaterThan(4);
              }
            });"#,
            None,
        ),
        (
            r#"beforeEach(() => expect.hasAssertions());
            it('responds ok', function () {
              client.get('/user', response => {
                expect(response.status).toBe(200);
              });
            });
            it("is a number that is greater than four", () => {
              expect(number).toBeGreaterThan(4);
            });"#,
            None,
        ),
        (
            r#"afterEach(() => {
              expect.hasAssertions();
            });
            it('responds ok', function () {
              client.get('/user', response => {
                expect(response.status).toBe(200);
              });
            });
            it("is a number that is greater than four", () => {
              expect(number).toBeGreaterThan(4);
            });"#,
            None,
        ),
        (
            r#"afterEach(() => {
              expect.hasAssertions();
            });
            it('responds ok', function () {
              client.get('/user', response => {
                expect(response.status).toBe(200);
              });
            });
            it("is a number that is greater than four", () => {
              expect.hasAssertions();
              expect(number).toBeGreaterThan(4);
            });"#,
            None,
        ),
        (
            r#"beforeEach(() => { expect.hasAssertions(); });
            describe('my tests', () => {
              it('responds ok', function () {
                client.get('/user', response => {
                  expect(response.status).toBe(200);
                });
              });
              it("is a number that is greater than four", () => {
                expect.hasAssertions();
                expect(number).toBeGreaterThan(4);
              });
            });"#,
            None,
        ),
        (
            r#"describe('my tests', () => {
              beforeEach(() => { expect.hasAssertions(); });
              describe('left', () => {
                describe('inner', () => {
                  it('responds ok', function () {
                    client.get('/user', response => {
                      expect(response.status).toBe(200);
                    });
                  });
                });
              });
              describe('right', () => {
                it("is a number that is greater than four", () => {
                  expect(number).toBeGreaterThan(4);
                });
              });
            });"#,
            None,
        ),
        (
            r#"describe('my tests', () => {
              beforeEach(() => { expect.hasAssertions(); });
              describe('left', () => {
                it('responds ok', function () {
                  client.get('/user', response => {
                    expect(response.status).toBe(200);
                  });
                });
              });
              describe('right', () => {
                it("is a number that is greater than four", () => {
                  expect(number).toBeGreaterThan(4);
                });
              });
            });"#,
            None,
        ),
        (
            r#"describe('my tests', () => {
              beforeEach(() => { expect.hasAssertions(); });
              describe('left', () => {
                beforeEach(() => { expect.hasAssertions(); });
                it('responds ok', function () {
                  client.get('/user', response => {
                    expect(response.status).toBe(200);
                  });
                });
              });
              describe('right', () => {
                it("is a number that is greater than four", () => {
                  expect(number).toBeGreaterThan(4);
                });
              });
            });"#,
            None,
        ),
        (
            r#"describe('my tests', () => {
              beforeEach(() => { expect.hasAssertions(); });
              describe('left', () => {
                afterEach(() => { expect.hasAssertions(); });
                it('responds ok', function () {
                  client.get('/user', response => {
                    expect(response.status).toBe(200);
                  });
                });
              });
              describe('right', () => {
                it("is a number that is greater than four", () => {
                  expect(number).toBeGreaterThan(4);
                });
              });
            });"#,
            None,
        ),
        (
            r#"describe('my tests', () => {
              beforeEach(() => { expect.hasAssertions(); });
              it('responds ok', function () {
                client.get('/user', response => {
                  expect(response.status).toBe(200);
                });
              });
              it("is a number that is greater than four", () => {
                expect.hasAssertions();
                expect(number).toBeGreaterThan(4);
              });
            });"#,
            None,
        ),
        (
            "beforeEach(() => {
              setTimeout(() => expect.hasAssertions(), 5000);
            });
            it('only returns numbers that are greater than six', () => {
              for (const number of getNumbers()) {
                expect(number).toBeGreaterThan(6);
              }
            });",
            None,
        ),
        (
            "const expectNumbersToBeGreaterThan = (numbers, value) => {
              for (let number of numbers) {
                expect(number).toBeGreaterThan(value);
              }
            };
            it('returns numbers that are greater than two', function () {
              expectNumbersToBeGreaterThan(getNumbers(), 2);
            });",
            Some(serde_json::json!([{ "onlyFunctionsWithExpectInLoop": true }])),
        ),
        (
            r#"it("returns numbers that are greater than five", function () {
              expect.assertions(2);
              for (const number of getNumbers()) {
                expect(number).toBeGreaterThan(5);
              }
            });"#,
            Some(serde_json::json!([{ "onlyFunctionsWithExpectInLoop": true }])),
        ),
        (
            r#"it("returns things that are less than ten", function () {
              expect.hasAssertions();
              for (const thing in things) {
                expect(thing).toBeLessThan(10);
              }
            });"#,
            Some(serde_json::json!([{ "onlyFunctionsWithExpectInLoop": true }])),
        ),
        (
            "const expectNumbersToBeGreaterThan = (numbers, value) => {
              numbers.forEach(number => {
                expect(number).toBeGreaterThan(value);
              });
            };
            it('returns numbers that are greater than two', function () {
              expectNumbersToBeGreaterThan(getNumbers(), 2);
            });",
            Some(serde_json::json!([{ "onlyFunctionsWithExpectInCallback": true }])),
        ),
        (
            "it('returns numbers that are greater than two', function () {
              expect.assertions(2);
              const expectNumbersToBeGreaterThan = (numbers, value) => {
                for (let number of numbers) {
                  expect(number).toBeGreaterThan(value);
                }
              };
              expectNumbersToBeGreaterThan(getNumbers(), 2);
            });",
            Some(serde_json::json!([{ "onlyFunctionsWithExpectInCallback": true }])),
        ),
        (
            "beforeEach(() => expect.hasAssertions());
            it('returns numbers that are greater than two', function () {
              const expectNumbersToBeGreaterThan = (numbers, value) => {
                for (let number of numbers) {
                  expect(number).toBeGreaterThan(value);
                }
              };
              expectNumbersToBeGreaterThan(getNumbers(), 2);
            });",
            Some(serde_json::json!([{ "onlyFunctionsWithExpectInCallback": true }])),
        ),
        (
            r#"it("returns numbers that are greater than five", function () {
              expect.assertions(2);
              getNumbers().forEach(number => {
                expect(number).toBeGreaterThan(5);
              });
            });"#,
            Some(serde_json::json!([{ "onlyFunctionsWithExpectInCallback": true }])),
        ),
        (
            r#"it("returns things that are less than ten", function () {
              expect.hasAssertions();
              things.forEach(thing => {
                expect(thing).toBeLessThan(10);
              });
            });"#,
            Some(serde_json::json!([{ "onlyFunctionsWithExpectInCallback": true }])),
        ),
        (
            "it('sends the data as a string', () => {
              expect.hasAssertions();
              const stream = openStream();
              stream.on('data', data => {
                expect(data).toBe(expect.any(String));
              });
            });",
            Some(serde_json::json!([{ "onlyFunctionsWithExpectInCallback": true }])),
        ),
        (
            "it('responds ok', function () {
              expect.assertions(1);
              client.get('/user', response => {
                expect(response.status).toBe(200);
              });
            });",
            Some(serde_json::json!([{ "onlyFunctionsWithExpectInCallback": true }])),
        ),
        (
            r#"it.each([1, 2, 3])("returns ok", id => {
              expect.assertions(3);
              client.get(`/users/${id}`, response => {
                expect(response.status).toBe(200);
              });
            });
            it("is a number that is greater than four", () => {
              expect(number).toBeGreaterThan(4);
            });"#,
            Some(serde_json::json!([{ "onlyFunctionsWithExpectInCallback": true }])),
        ),
        (
            "it('is a test', () => {
              expect(expected).toBe(actual);
            });
            describe('my test', () => {
              it('is another test', () => {
                expect(expected).toBe(actual);
              });
            });",
            Some(serde_json::json!([{ "onlyFunctionsWithExpectInCallback": true }])),
        ),
        (
            "it('responds ok', function () {
              expect.assertions(1);
              client.get('/user', response => {
                expect(response.status).toBe(200);
              });
            });
            describe('my test', () => {
              beforeEach(() => expect.hasAssertions());
              it('responds ok', function () {
                client.get('/user', response => {
                  expect(response.status).toBe(200);
                });
              });
            });",
            Some(serde_json::json!([{ "onlyFunctionsWithExpectInCallback": true }])),
        ),
        (
            "it('responds ok', function () {
              expect.assertions(1);
              client.get('/user', response => {
                expect(response.status).toBe(200);
              });
            });
            describe('my test', () => {
              afterEach(() => expect.hasAssertions());
              it('responds ok', function () {
                client.get('/user', response => {
                  expect(response.status).toBe(200);
                });
              });
            });",
            Some(serde_json::json!([{ "onlyFunctionsWithExpectInCallback": true }])),
        ),
        (
            "it('only returns numbers that are greater than zero', async () => {
              expect.hasAssertions();
              for (const number of await getNumbers()) {
                expect(number).toBeGreaterThan(0);
              }
            });",
            Some(
                serde_json::json!([ { "onlyFunctionsWithAsyncKeyword": true, "onlyFunctionsWithExpectInLoop": true, }, ]),
            ),
        ),
        (
            "it('only returns numbers that are greater than zero', async () => {
              expect.assertions(2);
              for (const number of await getNumbers()) {
                expect(number).toBeGreaterThan(0);
              }
            });",
            Some(
                serde_json::json!([ { "onlyFunctionsWithAsyncKeyword": true, "onlyFunctionsWithExpectInLoop": true, }, ]),
            ),
        ),
        (r#"test.each()("is fine", () => { expect.assertions(0); })"#, None),
        (r#"test.each``("is fine", () => { expect.assertions(0); })"#, None),
        (r#"test.each()("is fine", () => { expect.hasAssertions(); })"#, None),
        (r#"test.each``("is fine", () => { expect.hasAssertions(); })"#, None),
        (r#"it.each()("is fine", () => { expect.assertions(0); })"#, None),
        (r#"it.each``("is fine", () => { expect.assertions(0); })"#, None),
        (r#"it.each()("is fine", () => { expect.hasAssertions(); })"#, None),
        (r#"it.each``("is fine", () => { expect.hasAssertions(); })"#, None),
        (
            r#"test.each()("is fine", () => {})"#,
            Some(serde_json::json!([{ "onlyFunctionsWithAsyncKeyword": true }])),
        ),
        (
            r#"test.each``("is fine", () => {})"#,
            Some(serde_json::json!([{ "onlyFunctionsWithAsyncKeyword": true }])),
        ),
        (
            r#"it.each()("is fine", () => {})"#,
            Some(serde_json::json!([{ "onlyFunctionsWithAsyncKeyword": true }])),
        ),
        (
            r#"it.each``("is fine", () => {})"#,
            Some(serde_json::json!([{ "onlyFunctionsWithAsyncKeyword": true }])),
        ),
        (
            "describe.each(['hello'])('%s', () => {
              it('is fine', () => {
                expect.assertions(0);
              });
            });",
            None,
        ),
        (
            r"describe.each``('%s', () => {
              it('is fine', () => {
                expect.assertions(0);
              });
            });",
            None,
        ),
        (
            "describe.each(['hello'])('%s', () => {
              it('is fine', () => {
                expect.hasAssertions();
              });
            });",
            None,
        ),
        (
            r"describe.each``('%s', () => {
              it('is fine', () => {
                expect.hasAssertions();
              });
            });",
            None,
        ),
        (
            "describe.each(['hello'])('%s', () => {
              it.each()('is fine', () => {
                expect.assertions(0);
              });
            });",
            None,
        ),
        (
            r"describe.each``('%s', () => {
              it.each()('is fine', () => {
                expect.assertions(0);
              });
            });",
            None,
        ),
        (
            "describe.each(['hello'])('%s', () => {
              it.each()('is fine', () => {
                expect.hasAssertions();
              });
            });",
            None,
        ),
        (
            r"describe.each``('%s', () => {
              it.each()('is fine', () => {
                expect.hasAssertions();
              });
            });",
            None,
        ),
        (
            r#"import { expect as e } from '@jest/globals';
            test("reassigned jest import", () => {
                e.assertions(1);
                e(true).toBe(true);
              });"#,
            None,
        ),
    ];

    let fail = vec![
        (r#"it("it1", () => foo())"#, None),
        ("it('resolves', () => expect(staged()).toBe(true));", None),
        ("it('resolves', async () => expect(await staged()).toBe(true));", None),
        (r#"it("it1", () => {})"#, None),
        (r#"it("it1", () => { foo()})"#, None),
        (
            r#"it("it1", function() {
              someFunctionToDo();
              someFunctionToDo2();
            });"#,
            None,
        ),
        (
            r#"it("it1", function() {
              someFunctionToDo();
              someFunctionToDo2();
            });
            describe('some tests', () => {
              beforeEach(() => { expect.hasAssertions(); });
              it("it1", function() {
                someFunctionToDo();
                someFunctionToDo2();
              });
            });"#,
            None,
        ),
        (
            r#"it("it1", function() {
              someFunctionToDo();
              someFunctionToDo2();
            });
            describe('some tests', () => {
              afterEach(() => { expect.hasAssertions(); });
              it("it1", function() {
                someFunctionToDo();
                someFunctionToDo2();
              });
            });"#,
            None,
        ),
        (
            r#"describe('some tests', () => {
              it("it1", function() {
                someFunctionToDo();
                someFunctionToDo2();
              });
              beforeEach(() => { expect.hasAssertions(); });
              it("it1", function() {
                someFunctionToDo();
                someFunctionToDo2();
              });
            });"#,
            None,
        ),
        (
            r#"describe('some tests', () => {
              beforeEach(() => { expect.hasAssertions(); });
              it("it1", function() {
                someFunctionToDo();
                someFunctionToDo2();
              });
            });
            it("it1", function() {
              someFunctionToDo();
              someFunctionToDo2();
            });"#,
            None,
        ),
        (
            r#"describe('some tests', () => {
              beforeEach(() => { expect.hasAssertions(); });
              it("it1", function() {
                someFunctionToDo();
                someFunctionToDo2();
              });
            });
            describe('more tests', () => {
              it("it1", function() {
                someFunctionToDo();
                someFunctionToDo2();
              });
            });"#,
            None,
        ),
        (r#"it("it1", function() {var a = 2;})"#, None),
        (r#"it("it1", function() {expect.assertions();})"#, None),
        (r#"it("it1", function() {expect.assertions(1,2);})"#, None),
        (r#"it("it1", function() {expect.assertions(1,2,);})"#, None),
        (r#"it("it1", function() {expect.assertions("1");})"#, None),
        (r#"beforeEach(() => { expect.hasAssertions("1") })"#, None),
        (r#"beforeEach(() => expect.hasAssertions("1"))"#, None),
        (r#"afterEach(() => { expect.hasAssertions("1") })"#, None),
        (r#"afterEach(() => expect.hasAssertions("1"))"#, None),
        (r#"it("it1", function() {expect.hasAssertions("1");})"#, None),
        (r#"it("it1", function() {expect.hasAssertions("1",);})"#, None),
        (r#"it("it1", function() {expect.hasAssertions("1", "2");})"#, None),
        (
            r#"it("it1", function() {
              expect.hasAssertions(() => {
                someFunctionToDo();
                someFunctionToDo2();
              });
            });"#,
            None,
        ),
        (
            r#"it("it1", async function() {
              expect(someValue).toBe(true);
            });"#,
            Some(serde_json::json!([{ "onlyFunctionsWithAsyncKeyword": true }])),
        ),
        (
            r#"it("returns numbers that are greater than four", async () => {
              for(let thing in things) {
                expect(number).toBeGreaterThan(4);
              }
            });"#,
            Some(serde_json::json!([{ "onlyFunctionsWithAsyncKeyword": true }])),
        ),
        (
            r#"it("returns numbers that are greater than four", async () => {
              for (const number of getNumbers()) {
                expect(number).toBeGreaterThan(4);
              }
            });"#,
            Some(serde_json::json!([{ "onlyFunctionsWithAsyncKeyword": true }])),
        ),
        (
            r#"it("returns numbers that are greater than four", async () => {
              for (const number of getNumbers()) {
                expect(number).toBeGreaterThan(4);
              }
            });
            it("returns numbers that are greater than five", () => {
              for (const number of getNumbers()) {
                expect(number).toBeGreaterThan(5);
              }
            });"#,
            Some(serde_json::json!([{ "onlyFunctionsWithAsyncKeyword": true }])),
        ),
        (
            r#"beforeAll(() => { expect.hasAssertions(); });
            it("returns numbers that are greater than four", async () => {
              for (const number of getNumbers()) {
                expect(number).toBeGreaterThan(4);
              }
            });
            it("returns numbers that are greater than five", () => {
              for (const number of getNumbers()) {
                expect(number).toBeGreaterThan(5);
              }
            });"#,
            Some(serde_json::json!([{ "onlyFunctionsWithAsyncKeyword": true }])),
        ),
        (
            r#"afterAll(() => { expect.hasAssertions(); });
            it("returns numbers that are greater than four", async () => {
              for (const number of getNumbers()) {
                expect(number).toBeGreaterThan(4);
              }
            });
            it("returns numbers that are greater than five", () => {
              for (const number of getNumbers()) {
                expect(number).toBeGreaterThan(5);
              }
            });"#,
            Some(serde_json::json!([{ "onlyFunctionsWithAsyncKeyword": true }])),
        ),
        (
            "it('only returns numbers that are greater than six', () => {
              for (const number of getNumbers()) {
                expect(number).toBeGreaterThan(6);
              }
            });",
            Some(serde_json::json!([{ "onlyFunctionsWithExpectInLoop": true }])),
        ),
        (
            "it('returns numbers that are greater than two', function () {
              const expectNumbersToBeGreaterThan = (numbers, value) => {
                for (let number of numbers) {
                  expect(number).toBeGreaterThan(value);
                }
              };
              expectNumbersToBeGreaterThan(getNumbers(), 2);
            });",
            Some(serde_json::json!([{ "onlyFunctionsWithExpectInLoop": true }])),
        ),
        (
            r#"it("only returns numbers that are greater than seven", function () {
              const numbers = getNumbers();
              for (let i = 0; i < numbers.length; i++) {
                expect(numbers[i]).toBeGreaterThan(7);
              }
            });"#,
            Some(serde_json::json!([{ "onlyFunctionsWithExpectInLoop": true }])),
        ),
        (
            "it('has the number two', () => {
              expect(number).toBe(2);
            });
            it('only returns numbers that are less than twenty', () => {
              for (const number of getNumbers()) {
                expect(number).toBeLessThan(20);
              }
            });",
            Some(serde_json::json!([{ "onlyFunctionsWithExpectInLoop": true }])),
        ),
        (
            r#"it("is wrong");
            it("is a test", () => {
              for (const number of getNumbers()) {
                expect(number).toBeGreaterThan(4);
              }
            });"#,
            Some(serde_json::json!([{ "onlyFunctionsWithExpectInLoop": true }])),
        ),
        (
            r#"it("is a number that is greater than four", () => {
              expect(number).toBeGreaterThan(4);
            });
            it("returns numbers that are greater than four", () => {
              for (const number of getNumbers()) {
                expect(number).toBeGreaterThan(4);
              }
            });
            it("returns numbers that are greater than five", () => {
              expect(number).toBeGreaterThan(5);
            });"#,
            Some(serde_json::json!([{ "onlyFunctionsWithExpectInLoop": true }])),
        ),
        (
            r#"describe('my tests', () => {
              beforeEach(expect.hasAssertions);
              it("is a number that is greater than four", () => {
                expect(number).toBeGreaterThan(4);
              });
            });
            describe('more tests', () => {
              it("returns numbers that are greater than four", () => {
                for (const number of getNumbers()) {
                  expect(number).toBeGreaterThan(4);
                }
              });
            });
            it("returns numbers that are greater than five", () => {
              expect(number).toBeGreaterThan(5);
            });"#,
            Some(serde_json::json!([{ "onlyFunctionsWithExpectInLoop": true }])),
        ),
        (
            r#"it.each([1, 2, 3])("returns numbers that are greater than four", () => {
              for (const number of getNumbers()) {
                expect(number).toBeGreaterThan(4);
              }
            });
            it("is a number that is greater than four", () => {
              expect(number).toBeGreaterThan(4);
            });"#,
            Some(serde_json::json!([{ "onlyFunctionsWithExpectInLoop": true }])),
        ),
        (
            r#"it("returns numbers that are greater than four", () => {
              for (const number of getNumbers()) {
                expect(number).toBeGreaterThan(4);
              }
            });
            it("is a number that is greater than four", () => {
              expect(number).toBeGreaterThan(4);
            });"#,
            Some(serde_json::json!([{ "onlyFunctionsWithExpectInLoop": true }])),
        ),
        (
            r#"it("returns numbers that are greater than four", () => {
              for (const number of getNumbers()) {
                expect(number).toBeGreaterThan(4);
              }
            });
            it("is a number that is greater than four", () => {
              expect.hasAssertions();
              expect(number).toBeGreaterThan(4);
            });"#,
            Some(serde_json::json!([{ "onlyFunctionsWithExpectInLoop": true }])),
        ),
        (
            r#"it("it1", () => {
              expect.hasAssertions();
              for (const number of getNumbers()) {
                expect(number).toBeGreaterThan(0);
              }
            });
            it("it1", () => {
              for (const number of getNumbers()) {
                expect(number).toBeGreaterThan(0);
              }
            });"#,
            Some(serde_json::json!([{ "onlyFunctionsWithExpectInLoop": true }])),
        ),
        (
            r#"it("returns numbers that are greater than four", async () => {
              for (const number of await getNumbers()) {
                expect(number).toBeGreaterThan(4);
              }
            });
            it("returns numbers that are greater than five", () => {
              for (const number of getNumbers()) {
                expect(number).toBeGreaterThan(5);
              }
            });"#,
            Some(serde_json::json!([{ "onlyFunctionsWithExpectInLoop": true }])),
        ),
        (
            r#"it("it1", async () => {
              expect.hasAssertions();
              for (const number of getNumbers()) {
                expect(number).toBeGreaterThan(4);
              }
            });
            it("it1", () => {
              for (const number of getNumbers()) {
                expect(number).toBeGreaterThan(4);
              }
            });"#,
            Some(serde_json::json!([{ "onlyFunctionsWithExpectInLoop": true }])),
        ),
        (
            r#"describe('my tests', () => {
              beforeEach(() => { expect.hasAssertions(); });
              it("it1", async () => {
                for (const number of getNumbers()) {
                  expect(number).toBeGreaterThan(4);
                }
              });
            });
            it("it1", () => {
              for (const number of getNumbers()) {
                expect(number).toBeGreaterThan(4);
              }
            });"#,
            Some(serde_json::json!([{ "onlyFunctionsWithExpectInLoop": true }])),
        ),
        (
            r#"describe('my tests', () => {
              afterEach(() => { expect.hasAssertions(); });
              it("it1", async () => {
                for (const number of getNumbers()) {
                  expect(number).toBeGreaterThan(4);
                }
              });
            });
            it("it1", () => {
              for (const number of getNumbers()) {
                expect(number).toBeGreaterThan(4);
              }
            });"#,
            Some(serde_json::json!([{ "onlyFunctionsWithExpectInLoop": true }])),
        ),
        (
            r#"it.skip.each``("it1", async () => {
              expect.hasAssertions();
              for (const number of getNumbers()) {
                expect(number).toBeGreaterThan(4);
              }
            });
            it("it1", () => {
              for (const number of getNumbers()) {
                expect(number).toBeGreaterThan(4);
              }
            });"#,
            Some(serde_json::json!([{ "onlyFunctionsWithExpectInLoop": true }])),
        ),
        (
            r#"it("it1", async () => {
              for (const number of getNumbers()) {
                expect(number).toBeGreaterThan(4);
              }
            });
            it("it1", () => {
              expect.hasAssertions();
              for (const number of getNumbers()) {
                expect(number).toBeGreaterThan(4);
              }
            });"#,
            Some(serde_json::json!([{ "onlyFunctionsWithExpectInLoop": true }])),
        ),
        (
            r#"describe('my tests', () => {
              it("it1", async () => {
                for (const number of getNumbers()) {
                  expect(number).toBeGreaterThan(4);
                }
              });
            });
            it("it1", () => {
              expect.hasAssertions();
              for (const number of getNumbers()) {
                expect(number).toBeGreaterThan(4);
              }
            });"#,
            Some(serde_json::json!([{ "onlyFunctionsWithExpectInLoop": true }])),
        ),
        (
            "it('sends the data as a string', () => {
              const stream = openStream();
              stream.on('data', data => {
                expect(data).toBe(expect.any(String));
              });
            });",
            Some(serde_json::json!([{ "onlyFunctionsWithExpectInCallback": true }])),
        ),
        (
            "it('responds ok', function () {
              client.get('/user', response => {
                expect(response.status).toBe(200);
              });
            });",
            Some(serde_json::json!([{ "onlyFunctionsWithExpectInCallback": true }])),
        ),
        (
            "it('responds ok', function () {
              client.get('/user', response => {
                expect.assertions(1);
                expect(response.status).toBe(200);
              });
            });",
            Some(serde_json::json!([{ "onlyFunctionsWithExpectInCallback": true }])),
        ),
        (
            "it('responds ok', function () {
              const expectOkResponse = response => {
                expect.assertions(1);
                expect(response.status).toBe(200);
              };
              client.get('/user', expectOkResponse);
            });",
            Some(serde_json::json!([{ "onlyFunctionsWithExpectInCallback": true }])),
        ),
        (
            "it('returns numbers that are greater than two', function () {
              const expectNumberToBeGreaterThan = (number, value) => {
                expect(number).toBeGreaterThan(value);
              };
              expectNumberToBeGreaterThan(1, 2);
            });",
            Some(serde_json::json!([{ "onlyFunctionsWithExpectInCallback": true }])),
        ),
        (
            "it('returns numbers that are greater than two', function () {
              const expectNumbersToBeGreaterThan = (numbers, value) => {
                for (let number of numbers) {
                  expect(number).toBeGreaterThan(value);
                }
              };
              expectNumbersToBeGreaterThan(getNumbers(), 2);
            });",
            Some(serde_json::json!([{ "onlyFunctionsWithExpectInCallback": true }])),
        ),
        (
            "it('only returns numbers that are greater than six', () => {
              getNumbers().forEach(number => {
                expect(number).toBeGreaterThan(6);
              });
            });",
            Some(serde_json::json!([{ "onlyFunctionsWithExpectInCallback": true }])),
        ),
        (
            r#"it("is wrong");
            it('responds ok', function () {
              const expectOkResponse = response => {
                expect.assertions(1);
                expect(response.status).toBe(200);
              };
              client.get('/user', expectOkResponse);
            });"#,
            Some(serde_json::json!([{ "onlyFunctionsWithExpectInCallback": true }])),
        ),
        (
            r#"it("is a number that is greater than four", () => {
              expect(number).toBeGreaterThan(4);
            });
            it('responds ok', function () {
              const expectOkResponse = response => {
                expect(response.status).toBe(200);
              };
              client.get('/user', expectOkResponse);
            });
            it("returns numbers that are greater than five", () => {
              expect(number).toBeGreaterThan(5);
            });"#,
            Some(serde_json::json!([{ "onlyFunctionsWithExpectInCallback": true }])),
        ),
        (
            r#"it("is a number that is greater than four", () => {
              expect(number).toBeGreaterThan(4);
            });
            it("returns numbers that are greater than four", () => {
              getNumbers().map(number => {
                expect(number).toBeGreaterThan(0);
              });
            });
            it("returns numbers that are greater than five", () => {
              expect(number).toBeGreaterThan(5);
            });"#,
            Some(serde_json::json!([{ "onlyFunctionsWithExpectInCallback": true }])),
        ),
        (
            r#"it.each([1, 2, 3])("returns ok", id => {
              client.get(`/users/${id}`, response => {
                expect(response.status).toBe(200);
              });
            });
            it("is a number that is greater than four", () => {
              expect(number).toBeGreaterThan(4);
            });"#,
            Some(serde_json::json!([{ "onlyFunctionsWithExpectInCallback": true }])),
        ),
        (
            r#"it('responds ok', function () {
              client.get('/user', response => {
                expect(response.status).toBe(200);
              });
            });
            it("is a number that is greater than four", () => {
              expect(number).toBeGreaterThan(4);
            });"#,
            Some(serde_json::json!([{ "onlyFunctionsWithExpectInCallback": true }])),
        ),
        (
            r#"it('responds ok', function () {
              client.get('/user', response => {
                expect(response.status).toBe(200);
              });
            });
            it("is a number that is greater than four", () => {
              expect.hasAssertions();
              expect(number).toBeGreaterThan(4);
            });"#,
            Some(serde_json::json!([{ "onlyFunctionsWithExpectInCallback": true }])),
        ),
        (
            r#"it("it1", () => {
              expect.hasAssertions();
              getNumbers().forEach(number => {
                expect(number).toBeGreaterThan(0);
              });
            });
            it("it1", () => {
              getNumbers().forEach(number => {
                expect(number).toBeGreaterThan(0);
              });
            });"#,
            Some(serde_json::json!([{ "onlyFunctionsWithExpectInCallback": true }])),
        ),
        (
            "it('responds ok', function () {
              expect.hasAssertions();
              client.get('/user', response => {
                expect(response.status).toBe(200);
              });
            });
            it('responds not found', function () {
              client.get('/user', response => {
                expect(response.status).toBe(404);
              });
            });",
            Some(serde_json::json!([{ "onlyFunctionsWithExpectInCallback": true }])),
        ),
        (
            r#"it.skip.each``("it1", async () => {
              expect.hasAssertions();
              client.get('/user', response => {
                expect(response.status).toBe(200);
              });
            });
            it("responds ok", () => {
              client.get('/user', response => {
                expect(response.status).toBe(200);
              });
            });"#,
            Some(serde_json::json!([{ "onlyFunctionsWithExpectInCallback": true }])),
        ),
        (
            r#"it("returns numbers that are greater than four", function(expect) {
              expect.assertions(2);
              for(let thing in things) {
                expect(number).toBeGreaterThan(4);
              }
            });"#,
            None,
        ),
        (
            r#"it('only returns numbers that are greater than zero', () => {
              for (const number of getNumbers()) {
                expect(number).toBeGreaterThan(0);
              }
            });
            it("is zero", () => {
              expect.hasAssertions();
              expect(0).toBe(0);
            });"#,
            Some(serde_json::json!([{ "onlyFunctionsWithExpectInLoop": true }])),
        ),
        (
            "it('only returns numbers that are greater than zero', () => {
              expect.hasAssertions();
              for (const number of getNumbers()) {
                expect(number).toBeGreaterThan(0);
              }
            });
            it('only returns numbers that are less than 100', () => {
              for (const number of getNumbers()) {
                expect(number).toBeLessThan(0);
              }
            });",
            Some(serde_json::json!([{ "onlyFunctionsWithExpectInLoop": true }])),
        ),
        (
            r#"it("to be true", async function() {
              expect(someValue).toBe(true);
            });"#,
            Some(
                serde_json::json!([ { "onlyFunctionsWithAsyncKeyword": true, "onlyFunctionsWithExpectInLoop": true, }, ]),
            ),
        ),
        (
            "it('only returns numbers that are greater than zero', async () => {
              for (const number of getNumbers()) {
                expect(number).toBeGreaterThan(0);
              }
            });",
            Some(
                serde_json::json!([ { "onlyFunctionsWithAsyncKeyword": true, "onlyFunctionsWithExpectInLoop": true, }, ]),
            ),
        ),
        (
            r#"test.each()("is not fine", () => {
              expect(someValue).toBe(true);
            });"#,
            None,
        ),
        (
            r#"describe.each()('something', () => {
              it("is not fine", () => {
                expect(someValue).toBe(true);
              });
            });"#,
            None,
        ),
        (
            r#"describe.each()('something', () => {
              test.each()("is not fine", () => {
                expect(someValue).toBe(true);
              });
            });"#,
            None,
        ),
        (
            r#"test.each()("is not fine", async () => {
              expect(someValue).toBe(true);
            });"#,
            Some(serde_json::json!([{ "onlyFunctionsWithAsyncKeyword": true }])),
        ),
        (
            r#"it.each()("is not fine", async () => {
              expect(someValue).toBe(true);
            });"#,
            Some(serde_json::json!([{ "onlyFunctionsWithAsyncKeyword": true }])),
        ),
        (
            r#"describe.each()('something', () => {
              test.each()("is not fine", async () => {
                expect(someValue).toBe(true);
              });
            });"#,
            Some(serde_json::json!([{ "onlyFunctionsWithAsyncKeyword": true }])),
        ),
        (
            // jest import reassignment: missing assertions
            r#"import { expect as e } from '@jest/globals';
            test("reassigned", () => { e(true).toBe(true); });"#,
            None,
        ),
    ];

    let fix_two_suggestions = vec![
        // OG test 1: empty body
        (
            r#"it("it1", () => {})"#,
            (
                r#"it("it1", () => {expect.hasAssertions();})"#,
                r#"it("it1", () => {expect.assertions();})"#,
            ),
        ),
        // OG test 2: single statement
        (
            r#"it("it1", () => { foo()})"#,
            (
                r#"it("it1", () => {expect.hasAssertions(); foo()})"#,
                r#"it("it1", () => {expect.assertions(); foo()})"#,
            ),
        ),
        // OG test 4: var declaration
        (
            r#"it("it1", function() {var a = 2;})"#,
            (
                r#"it("it1", function() {expect.hasAssertions();var a = 2;})"#,
                r#"it("it1", function() {expect.assertions();var a = 2;})"#,
            ),
        ),
        // OG test 1 variant: test() instead of it()
        (
            r#"test("it1", () => {expect(true).toBe(true);})"#,
            (
                r#"test("it1", () => {expect.hasAssertions();expect(true).toBe(true);})"#,
                r#"test("it1", () => {expect.assertions();expect(true).toBe(true);})"#,
            ),
        ),
    ];

    let fix_import_reassignment = vec![(
        r#"import { expect as e } from '@jest/globals';
            test("reassigned", () => { e(true).toBe(true); });"#,
        (
            r#"import { expect as e } from '@jest/globals';
            test("reassigned", () => {e.hasAssertions(); e(true).toBe(true); });"#,
            r#"import { expect as e } from '@jest/globals';
            test("reassigned", () => {e.assertions(); e(true).toBe(true); });"#,
        ),
    )];

    // Two suggestions for multi-statement function body and describe/hook interactions
    let fix_multi_statement = vec![
        // OG test 3: multi-statement body
        (
            r#"it("it1", function() {
              someFunctionToDo();
              someFunctionToDo2();
            });"#,
            (
                r#"it("it1", function() {expect.hasAssertions();
              someFunctionToDo();
              someFunctionToDo2();
            });"#,
                r#"it("it1", function() {expect.assertions();
              someFunctionToDo();
              someFunctionToDo2();
            });"#,
            ),
        ),
        // OG test 15: test outside describe not covered by beforeEach inside describe
        (
            r#"it("it1", function() {
              someFunctionToDo();
              someFunctionToDo2();
            });

            describe('some tests', () => {
              beforeEach(() => { expect.hasAssertions(); });
              it("it1", function() {
                someFunctionToDo();
                someFunctionToDo2();
              });
            });"#,
            (
                r#"it("it1", function() {expect.hasAssertions();
              someFunctionToDo();
              someFunctionToDo2();
            });

            describe('some tests', () => {
              beforeEach(() => { expect.hasAssertions(); });
              it("it1", function() {
                someFunctionToDo();
                someFunctionToDo2();
              });
            });"#,
                r#"it("it1", function() {expect.assertions();
              someFunctionToDo();
              someFunctionToDo2();
            });

            describe('some tests', () => {
              beforeEach(() => { expect.hasAssertions(); });
              it("it1", function() {
                someFunctionToDo();
                someFunctionToDo2();
              });
            });"#,
            ),
        ),
        // OG test 18: test after describe not covered by beforeEach inside describe
        (
            r#"describe('some tests', () => {
              beforeEach(() => { expect.hasAssertions(); });
              it("it1", function() {
                someFunctionToDo();
                someFunctionToDo2();
              });
            });

            it("it1", function() {
              someFunctionToDo();
              someFunctionToDo2();
            });"#,
            (
                r#"describe('some tests', () => {
              beforeEach(() => { expect.hasAssertions(); });
              it("it1", function() {
                someFunctionToDo();
                someFunctionToDo2();
              });
            });

            it("it1", function() {expect.hasAssertions();
              someFunctionToDo();
              someFunctionToDo2();
            });"#,
                r#"describe('some tests', () => {
              beforeEach(() => { expect.hasAssertions(); });
              it("it1", function() {
                someFunctionToDo();
                someFunctionToDo2();
              });
            });

            it("it1", function() {expect.assertions();
              someFunctionToDo();
              someFunctionToDo2();
            });"#,
            ),
        ),
        // OG test 19: test in second describe not covered by beforeEach in first describe
        (
            r#"describe('some tests', () => {
              beforeEach(() => { expect.hasAssertions(); });
              it("it1", function() {
                someFunctionToDo();
                someFunctionToDo2();
              });
            });

            describe('more tests', () => {
              it("it1", function() {
                someFunctionToDo();
                someFunctionToDo2();
              });
            });"#,
            (
                r#"describe('some tests', () => {
              beforeEach(() => { expect.hasAssertions(); });
              it("it1", function() {
                someFunctionToDo();
                someFunctionToDo2();
              });
            });

            describe('more tests', () => {
              it("it1", function() {expect.hasAssertions();
                someFunctionToDo();
                someFunctionToDo2();
              });
            });"#,
                r#"describe('some tests', () => {
              beforeEach(() => { expect.hasAssertions(); });
              it("it1", function() {
                someFunctionToDo();
                someFunctionToDo2();
              });
            });

            describe('more tests', () => {
              it("it1", function() {expect.assertions();
                someFunctionToDo();
                someFunctionToDo2();
              });
            });"#,
            ),
        ),
    ];

    // hasAssertions with callback arg — remove everything after hasAssertions
    let fix_has_assertions_callback_arg = vec![(
        r#"it("it1", function() {
              expect.hasAssertions(() => {
                someFunctionToDo();
                someFunctionToDo2();
              });
            });"#,
        r#"it("it1", function() {
              expect.hasAssertions();
            });"#,
    )];

    // These fix cases require config options but `ExpectFixTestCase` doesn't support
    // `(S, (S, S), Option<Value>)` — the two-suggestion format with config.
    // The options only control WHETHER the diagnostic fires, not WHAT the fix produces,
    // so the fix output is the same regardless.
    //
    // onlyFunctionsWithAsyncKeyword:
    // (
    //     r#"it("it1", async function() { expect(someValue).toBe(true); });"#,
    //     (
    //         r#"it("it1", async function() {expect.hasAssertions(); expect(someValue).toBe(true); });"#,
    //         r#"it("it1", async function() {expect.assertions(); expect(someValue).toBe(true); });"#,
    //     ),
    //     Some(serde_json::json!([{ "onlyFunctionsWithAsyncKeyword": true }])),
    // ),
    //
    // onlyFunctionsWithExpectInLoop:
    // (
    //     r#"it('numbers > 6', () => { for (const n of getNumbers()) { expect(n).toBeGreaterThan(6); } });"#,
    //     (
    //         r#"it('numbers > 6', () => {expect.hasAssertions(); for (const n of getNumbers()) { expect(n).toBeGreaterThan(6); } });"#,
    //         r#"it('numbers > 6', () => {expect.assertions(); for (const n of getNumbers()) { expect(n).toBeGreaterThan(6); } });"#,
    //     ),
    //     Some(serde_json::json!([{ "onlyFunctionsWithExpectInLoop": true }])),
    // ),
    //
    // onlyFunctionsWithExpectInCallback:
    // (
    //     r#"it('data', () => { stream.on('data', data => { expect(data).toBe(expect.any(String)); }); });"#,
    //     (
    //         r#"it('data', () => {expect.hasAssertions(); stream.on('data', data => { expect(data).toBe(expect.any(String)); }); });"#,
    //         r#"it('data', () => {expect.assertions(); stream.on('data', data => { expect(data).toBe(expect.any(String)); }); });"#,
    //     ),
    //     Some(serde_json::json!([{ "onlyFunctionsWithExpectInCallback": true }])),
    // ),

    let fix_remove_args = vec![
        (
            r#"it("it1", function() {expect.hasAssertions("1");})"#,
            r#"it("it1", function() {expect.hasAssertions();})"#,
        ),
        (
            r#"it("it1", function() {expect.hasAssertions("1",);})"#,
            r#"it("it1", function() {expect.hasAssertions();})"#,
        ),
        (
            r#"it("it1", function() {expect.hasAssertions("1", "2");})"#,
            r#"it("it1", function() {expect.hasAssertions();})"#,
        ),
        (
            r#"it("it1", function() {expect.assertions(1,2);})"#,
            r#"it("it1", function() {expect.assertions(1);})"#,
        ),
        (
            r#"it("it1", function() {expect.assertions(1,2,);})"#,
            r#"it("it1", function() {expect.assertions(1);})"#,
        ),
        // hooks with block body
        (
            r#"beforeEach(() => { expect.hasAssertions("1") })"#,
            r"beforeEach(() => { expect.hasAssertions() })",
        ),
        (
            r#"afterEach(() => { expect.hasAssertions("1") })"#,
            r"afterEach(() => { expect.hasAssertions() })",
        ),
    ];

    Tester::new(PreferExpectAssertions::NAME, PreferExpectAssertions::PLUGIN, pass, fail)
        .with_jest_plugin(true)
        .expect_fix(fix_two_suggestions)
        .expect_fix(fix_import_reassignment)
        .expect_fix(fix_multi_statement)
        .expect_fix(fix_has_assertions_callback_arg)
        .expect_fix(fix_remove_args)
        .test_and_snapshot();
}
