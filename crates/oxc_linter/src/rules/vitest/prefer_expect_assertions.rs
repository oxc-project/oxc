use std::borrow::Cow;

use serde::Deserialize;

use oxc_ast::ast::{
    Argument, ArrayExpressionElement, BindingPattern, CallExpression, Expression, MemberExpression,
};
use oxc_ast_visit::VisitJs;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::NodeId;
use oxc_span::Span;
use oxc_str::CompactStr;

use crate::{
    context::LintContext,
    fixer::RuleFix,
    rule::{DefaultRuleConfig, Rule},
    rules::shared::prefer_expect_assertions::{
        CallbackBody, DOCUMENTATION, PreferExpectAssertionsConfig, PreferExpectAssertionsRuleImpl,
        resolve_expect_local_name, should_check,
    },
    utils::{collect_possible_jest_call_node, get_node_name},
};

fn have_expect_assertions(span: Span, prefix: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "This test should have either `{prefix}.assertions(<number of assertions>)` or `{prefix}.hasAssertions()` as its first expression.",
    ))
    .with_help(format!("Add `{prefix}.hasAssertions()` or `{prefix}.assertions(<number>)` as the first statement in this test."))
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
    vitest,
    style,
    docs = DOCUMENTATION,
    suggestion,
    version = "1.62.0",
    config = PreferExpectAssertionsConfig
);

impl Rule for PreferExpectAssertions {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        DefaultRuleConfig::<PreferExpectAssertionsConfig>::from_value(value)
            .map(|c| Self(Box::new(c.into_inner())))
    }

    fn run_once(&self, ctx: &LintContext) {
        let mut possible_jest_nodes = collect_possible_jest_call_node(ctx);
        possible_jest_nodes.sort_unstable_by_key(|n| n.node.id());

        // Resolve the file-level expect local name once (e.g. `"expect"` or `"e"`
        // for `import { expect as e }`). Per-callback vitest fixture overrides
        // are handled in `resolve_expect_source`.
        let file_expect_prefix =
            resolve_expect_local_name(ctx, &["vitest", "vite-plus/test", "@effect/vitest"]);

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
        _ctx: &LintContext<'a>,
    ) -> Option<Cow<'r, str>> {
        let Some(expect_param) = resolve_expect_parameter_prefix(callback, 0) else {
            return Some(Cow::Borrowed(file_expect_prefix));
        };

        Some(Cow::Owned(expect_param.to_string()))
    }

    fn resolve_expect_for_parameterized_test<'a, 'r>(
        &self,
        call_expr: &CallExpression<'a>,
        callback: &Expression<'a>,
        file_expect_prefix: &'r str,
        _ctx: &LintContext<'a>,
    ) -> Option<Cow<'r, str>> {
        if let Some(context_index) = parameterized_context_parameter_index(call_expr)
            && let Some(expect_param) = resolve_expect_parameter_prefix(callback, context_index)
        {
            return Some(Cow::Owned(expect_param.to_string()));
        }

        if let Some(expect_param) = resolve_used_parameterized_context_prefix(callback) {
            return Some(Cow::Owned(expect_param.to_string()));
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

    fn should_check_node(&self, body: CallbackBody<'_>, is_async: bool, prefix: &str) -> bool {
        should_check(self.0.as_ref(), body, is_async, prefix)
    }

    fn should_check_node_with_file_expect(
        &self,
        body: CallbackBody<'_>,
        is_async: bool,
        prefix: &str,
        file_expect_prefix: &str,
    ) -> bool {
        should_check(self.0.as_ref(), body, is_async, prefix)
            || (prefix != file_expect_prefix
                && should_check(self.0.as_ref(), body, is_async, file_expect_prefix))
    }
}

fn callback_uses_expect_prefix(callback: &Expression<'_>, prefix: &str) -> bool {
    let mut scanner = ExpectPrefixScanner {
        prefix,
        prefix_dot: CompactStr::from(format!("{prefix}.")),
        found: false,
    };
    scanner.visit_expression(callback);
    scanner.found
}

fn resolve_used_parameterized_context_prefix(callback: &Expression<'_>) -> Option<CompactStr> {
    let params = match callback {
        Expression::FunctionExpression(func) => &func.params,
        Expression::ArrowFunctionExpression(func) => &func.params,
        _ => return None,
    };

    params.items.iter().enumerate().rev().find_map(|(index, param)| {
        if index == 0 && !matches!(&param.pattern, BindingPattern::ObjectPattern(_)) {
            return None;
        }

        let prefix = resolve_expect_parameter_prefix_from_pattern(&param.pattern)?;
        callback_uses_expect_prefix(callback, &prefix).then_some(prefix)
    })
}

struct ExpectPrefixScanner<'p> {
    prefix: &'p str,
    prefix_dot: CompactStr,
    found: bool,
}

impl<'a> VisitJs<'a> for ExpectPrefixScanner<'_> {
    fn visit_call_expression(&mut self, call_expr: &CallExpression<'a>) {
        if self.found {
            return;
        }

        let name = get_node_name(&call_expr.callee);
        if name == self.prefix || name.starts_with(self.prefix_dot.as_str()) {
            self.found = true;
            return;
        }

        oxc_ast_visit::walk_js::walk_call_expression(self, call_expr);
    }
}

fn parameterized_context_parameter_index(call_expr: &CallExpression<'_>) -> Option<usize> {
    let Expression::CallExpression(parameterized_call) = &call_expr.callee else {
        return None;
    };
    let method = parameterized_call
        .callee
        .as_member_expression()
        .and_then(MemberExpression::static_property_name)?;

    if method == "for" {
        return Some(1);
    }
    if method != "each" {
        return None;
    }

    let Some(Expression::ArrayExpression(rows)) = parameterized_call
        .arguments
        .first()
        .and_then(Argument::as_expression)
        .map(Expression::get_inner_expression)
    else {
        return None;
    };

    let mut argument_counts = rows.elements.iter().map(each_row_argument_count);
    let argument_count = argument_counts.next()??;
    argument_counts.all(|count| count == Some(argument_count)).then_some(argument_count)
}

fn each_row_argument_count(row: &ArrayExpressionElement<'_>) -> Option<usize> {
    let expression = row.as_expression()?.get_inner_expression();
    let Expression::ArrayExpression(arguments) = expression else {
        return Some(1);
    };
    arguments
        .elements
        .iter()
        .all(|argument| !argument.is_spread())
        .then_some(arguments.elements.len())
}

fn resolve_expect_parameter_prefix(
    callback: &Expression<'_>,
    parameter_index: usize,
) -> Option<CompactStr> {
    let params = match callback {
        Expression::FunctionExpression(func) => &func.params,
        Expression::ArrowFunctionExpression(func) => &func.params,
        _ => return None,
    };

    let context_param = params.items.get(parameter_index)?;

    resolve_expect_parameter_prefix_from_pattern(&context_param.pattern)
}

fn resolve_expect_parameter_prefix_from_pattern(
    pattern: &BindingPattern<'_>,
) -> Option<CompactStr> {
    match pattern {
        BindingPattern::BindingIdentifier(id) => {
            Some(CompactStr::from(format!("{}.expect", id.name)))
        }
        BindingPattern::ObjectPattern(pattern) => {
            let prop = pattern
                .properties
                .iter()
                .find(|p| p.key.static_name().is_some_and(|name| name == "expect"))?;

            let local_name = match &prop.value {
                BindingPattern::BindingIdentifier(id) => id.name.as_str(),
                _ => "expect",
            };
            Some(CompactStr::from(local_name))
        }
        _ => None,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (r#"test("it1", () => {expect.assertions(0);})"#, None),
        (r#"test("it1", function() {expect.assertions(0);})"#, None),
        (r#"test("it1", function() {expect.hasAssertions();})"#, None),
        (r#"it("it1", function() {expect.assertions(0);})"#, None),
        (r#"test("it1")"#, None),
        (r#"itHappensToStartWithIt("foo", function() {})"#, None),
        (r#"testSomething("bar", function() {})"#, None),
        ("it(async () => {expect.assertions(0);})", None),
        (
            r#"import * as vi from 'vitest';
            test("example-fail", async ({ expect }) => {
                expect.assertions(1);
                await expect(Promise.resolve(null)).resolves.toBeNull();
              });
                "#,
            None,
        ),
        (
            r#"import { test } from 'vitest';
            test("ctx param", async (ctx) => {
                ctx.expect.assertions(1);
                await ctx.expect(Promise.resolve(null)).resolves.toBeNull();
              });
                "#,
            None,
        ),
        (
            r#"import { test } from 'vitest';
            test("renamed expect", async ({ expect: myExpect }) => {
                myExpect.assertions(1);
                await myExpect(Promise.resolve(null)).resolves.toBeNull();
              });
                "#,
            None,
        ),
        (
            r#"import { test } from 'vitest';
            test("renamed hasAssertions", async ({ expect: e }) => {
                e.hasAssertions();
                await e(Promise.resolve(null)).resolves.toBeNull();
              });
                "#,
            None,
        ),
        (
            r#"import { test } from 'vitest';
            test("ctx hasAssertions", async (t) => {
                t.expect.hasAssertions();
                await t.expect(Promise.resolve(null)).resolves.toBeNull();
              });
                "#,
            None,
        ),
        (
            r#"import { test, expect } from 'vitest';
            test("global expect", async () => {
                expect.assertions(1);
                await expect(Promise.resolve(null)).resolves.toBeNull();
              });
                "#,
            None,
        ),
        (
            r#"import { expect as e } from 'vitest';
            test("reassigned vitest import", () => {
                e.assertions(1);
                e(true).toBe(true);
              });
                "#,
            None,
        ),
        (
            r#"import { expect as e } from 'vite-plus/test';
            test("re-exported vitest", () => {
                e.assertions(1);
                e(true).toBe(true);
              });"#,
            None,
        ),
        (
            r#"import { expect } from 'vite-plus/test';
            test("re-exported vitest global", () => {
                expect.assertions(1);
                expect(true).toBe(true);
              });"#,
            None,
        ),
        (
            r#"import { expect } from '@effect/vitest';
            test("re-exported effect vitest global", () => {
                expect.assertions(1);
                expect(true).toBe(true);
              });"#,
            None,
        ),
        (
            "import { expect as e } from 'vitest';
            describe('suite', () => {
                beforeEach(() => { e.hasAssertions(); });
                it('test', () => {
                    e(true).toBe(true);
                });
            });",
            None,
        ),
        (
            r#"it("it1", () => {
                expect.assertions(0);
                const foo = { bar({ baz }) { baz(); } };
              });
                "#,
            None,
        ),
        (
            "
               const expectNumbersToBeGreaterThan = (numbers, value) => {
                for (let number of numbers) {
                expect(number).toBeGreaterThan(value);
               }
               };

               it('returns numbers that are greater than two', function () {
                expectNumbersToBeGreaterThan(getNumbers(), 2);
               });
               ",
            Some(serde_json::json!([{ "onlyFunctionsWithExpectInLoop": true }])),
        ),
        (
            r#"
               it("returns numbers that are greater than five", function () {
                expect.assertions(2);
                for (const number of getNumbers()) {
                expect(number).toBeGreaterThan(5);
               }
               });
               "#,
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
    ];

    let fail = vec![
        (r#"it("it1", () => foo())"#, None),
        (
            "
            import * as vi from 'vitest';
            it('my test description', ({ expect }) => {
              const a = 1;
              const b = 2;

              expect(sum(a, b)).toBe(a + b);
            })
            ",
            None,
        ),
        (
            "
            it('my test description', (context) => {
              const a = 1;
              const b = 2;

              context.expect(sum(a, b)).toBe(a + b);
            })
            ",
            None,
        ),
        ("it('resolves', () => expect(staged()).toBe(true));", None),
        ("it('resolves', async () => expect(await staged()).toBe(true));", None),
        (r#"it("it1", () => {})"#, None),
        (r#"it("it1", () => { foo()})"#, None),
        (r#"it("it1", function() {var a = 2;})"#, None),
        (r#"it("it1", function() {expect.assertions();})"#, None),
        (r#"it("it1", function() {expect.assertions(1,2);})"#, None),
        (r#"it("it1", function() {expect.assertions(1,2,);})"#, None),
        (r#"it("it1", function() {expect.assertions("1");})"#, None),
        (r#"it("it1", function() {expect.hasAssertions("1");})"#, None),
        (r#"it("it1", function() {expect.hasAssertions("1",);})"#, None),
        (r#"it("it1", function() {expect.hasAssertions("1", "2");})"#, None),
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
               });
                "#,
            Some(serde_json::json!([{ "onlyFunctionsWithExpectInLoop": true }])),
        ),
        (
            r#"test.each([[1]])("case", (value, { expect: e }) => {
                consume(() => e(value).toBe(1));
            });"#,
            Some(serde_json::json!([{ "onlyFunctionsWithExpectInCallback": true }])),
        ),
        (
            r#"test.each([{ value: 1 }])("case", ({ value }, ctx) => {
                consume(() => expect(value).toBe(1));
            });"#,
            Some(serde_json::json!([{ "onlyFunctionsWithExpectInCallback": true }])),
        ),
        (
            r#"test.each([{ value: 1 }])("case", ({ value }, ctx) => {
                ctx.expect(value).toBe(1);
                consume(() => expect(value).toBe(1));
            });"#,
            Some(serde_json::json!([{ "onlyFunctionsWithExpectInCallback": true }])),
        ),
        (
            r#"const rows = [[1]];
            test.each(rows)("case", (value, { expect: e }) => {
                consume(() => e(value).toBe(1));
            });"#,
            Some(serde_json::json!([{ "onlyFunctionsWithExpectInCallback": true }])),
        ),
        (
            r#"it("it1", () => {
                const foo = { bar({ baz }) { baz(); } };
              });
                "#,
            None,
        ),
        (
            "import * as vi from 'vitest';
            it('missing assertions', ({ expect: myExpect }) => {
              myExpect(true).toBe(true);
            })
            ",
            None,
        ),
        (
            "import * as vi from 'vitest';
            it('missing assertions', (ctx) => {
              ctx.expect(true).toBe(true);
            })
            ",
            None,
        ),
        (
            r#"import * as vi from 'vitest';
            it("it1", ({ expect: e }) => {e.assertions();})"#,
            None,
        ),
        (
            r#"import * as vi from 'vitest';
            it("it1", (ctx) => {ctx.expect.assertions("1");})"#,
            None,
        ),
        (
            r#"import * as vi from 'vitest';
            it("it1", ({ expect: e }) => {e.hasAssertions("1");})"#,
            None,
        ),
        (
            r#"import * as vi from 'vitest';
            it("it1", (ctx) => {ctx.expect.assertions(1, 2);})"#,
            None,
        ),
        (
            r#"import { expect as e } from 'vitest';
            test("reassigned", () => { e(true).toBe(true); });"#,
            None,
        ),
        (
            r#"import { expect as e } from 'vite-plus/test';
            test("re-exported missing", () => { e(true).toBe(true); });"#,
            None,
        ),
        (
            r#"import { expect as e } from '@effect/vitest';
            test("re-exported effect missing", () => { e(true).toBe(true); });"#,
            None,
        ),
        (
            "import { expect as e } from 'vitest';
            describe('suite', () => {
                beforeEach(() => { expect.hasAssertions(); });
                it('test', () => {
                    e(true).toBe(true);
                });
            });",
            None,
        ),
    ];

    let fix_import_reassignment = vec![(
        r#"import { expect as e } from 'vitest';
            test("reassigned", () => { e(true).toBe(true); });"#,
        (
            r#"import { expect as e } from 'vitest';
            test("reassigned", () => {e.hasAssertions(); e(true).toBe(true); });"#,
            r#"import { expect as e } from 'vitest';
            test("reassigned", () => {e.assertions(); e(true).toBe(true); });"#,
        ),
    )];

    let fix_two_suggestions = vec![
        // OG: empty body
        (
            r#"it("it1", () => {})"#,
            (
                r#"it("it1", () => {expect.hasAssertions();})"#,
                r#"it("it1", () => {expect.assertions();})"#,
            ),
        ),
        // OG: single statement
        (
            r#"it("it1", () => { foo()})"#,
            (
                r#"it("it1", () => {expect.hasAssertions(); foo()})"#,
                r#"it("it1", () => {expect.assertions(); foo()})"#,
            ),
        ),
        // OG: var declaration
        (
            r#"it("it1", function() {var a = 2;})"#,
            (
                r#"it("it1", function() {expect.hasAssertions();var a = 2;})"#,
                r#"it("it1", function() {expect.assertions();var a = 2;})"#,
            ),
        ),
        // OG: test() variant
        (
            r#"test("it1", () => {expect(true).toBe(true);})"#,
            (
                r#"test("it1", () => {expect.hasAssertions();expect(true).toBe(true);})"#,
                r#"test("it1", () => {expect.assertions();expect(true).toBe(true);})"#,
            ),
        ),
        // OG: multi-statement body
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
        // OG: object destructuring in body (not a fixture param)
        (
            r#"it("it1", () => {
                const foo = { bar({ baz }) { baz(); } };
              });"#,
            (
                r#"it("it1", () => {expect.hasAssertions();
                const foo = { bar({ baz }) { baz(); } };
              });"#,
                r#"it("it1", () => {expect.assertions();
                const foo = { bar({ baz }) { baz(); } };
              });"#,
            ),
        ),
        // Vitest fixture: destructured expect
        (
            "import * as vi from 'vitest';
            it('my test description', ({ expect }) => {
              const a = 1;
              const b = 2;

              expect(sum(a, b)).toBe(a + b);
            })",
            (
                "import * as vi from 'vitest';
            it('my test description', ({ expect }) => {expect.hasAssertions();
              const a = 1;
              const b = 2;

              expect(sum(a, b)).toBe(a + b);
            })",
                "import * as vi from 'vitest';
            it('my test description', ({ expect }) => {expect.assertions();
              const a = 1;
              const b = 2;

              expect(sum(a, b)).toBe(a + b);
            })",
            ),
        ),
        // Vitest fixture: context variable
        (
            "it('my test description', (context) => {
              const a = 1;
              const b = 2;

              context.expect(sum(a, b)).toBe(a + b);
            })",
            (
                "it('my test description', (context) => {context.expect.hasAssertions();
              const a = 1;
              const b = 2;

              context.expect(sum(a, b)).toBe(a + b);
            })",
                "it('my test description', (context) => {context.expect.assertions();
              const a = 1;
              const b = 2;

              context.expect(sum(a, b)).toBe(a + b);
            })",
            ),
        ),
        // Vitest fixture: renamed expect
        (
            "import * as vi from 'vitest';
            it('missing assertions', ({ expect: myExpect }) => {
              myExpect(true).toBe(true);
            })",
            (
                "import * as vi from 'vitest';
            it('missing assertions', ({ expect: myExpect }) => {myExpect.hasAssertions();
              myExpect(true).toBe(true);
            })",
                "import * as vi from 'vitest';
            it('missing assertions', ({ expect: myExpect }) => {myExpect.assertions();
              myExpect(true).toBe(true);
            })",
            ),
        ),
        // Vitest fixture: context variable shorthand
        (
            "import * as vi from 'vitest';
            it('missing assertions', (ctx) => {
              ctx.expect(true).toBe(true);
            })",
            (
                "import * as vi from 'vitest';
            it('missing assertions', (ctx) => {ctx.expect.hasAssertions();
              ctx.expect(true).toBe(true);
            })",
                "import * as vi from 'vitest';
            it('missing assertions', (ctx) => {ctx.expect.assertions();
              ctx.expect(true).toBe(true);
            })",
            ),
        ),
        // Direct test callbacks always receive TestContext first.
        (
            r#"test("x", ({ expect: e }) => doWork())"#,
            (
                r#"test("x", ({ expect: e }) => {e.hasAssertions();return doWork();})"#,
                r#"test("x", ({ expect: e }) => {e.assertions();return doWork();})"#,
            ),
        ),
        // The first parameter of a parameterized test is iteration data.
        (
            r#"it.each([{ abc: 123 }])("$abc", async (props) => {
              await expect(Promise.resolve(props.abc)).resolves.toBe(props.abc);
            });"#,
            (
                r#"it.each([{ abc: 123 }])("$abc", async (props) => {expect.hasAssertions();
              await expect(Promise.resolve(props.abc)).resolves.toBe(props.abc);
            });"#,
                r#"it.each([{ abc: 123 }])("$abc", async (props) => {expect.assertions();
              await expect(Promise.resolve(props.abc)).resolves.toBe(props.abc);
            });"#,
            ),
        ),
        (
            r#"test.only.for([{ abc: 123 }])("$abc", async (props) => {
              await expect(Promise.resolve(props.abc)).resolves.toBe(props.abc);
            });"#,
            (
                r#"test.only.for([{ abc: 123 }])("$abc", async (props) => {expect.hasAssertions();
              await expect(Promise.resolve(props.abc)).resolves.toBe(props.abc);
            });"#,
                r#"test.only.for([{ abc: 123 }])("$abc", async (props) => {expect.assertions();
              await expect(Promise.resolve(props.abc)).resolves.toBe(props.abc);
            });"#,
            ),
        ),
        // Parameterized tests append TestContext after the iteration values.
        (
            r#"test.each([[1]])("case", (value, { expect: e }) => {
              e(value).toBe(1);
            });"#,
            (
                r#"test.each([[1]])("case", (value, { expect: e }) => {e.hasAssertions();
              e(value).toBe(1);
            });"#,
                r#"test.each([[1]])("case", (value, { expect: e }) => {e.assertions();
              e(value).toBe(1);
            });"#,
            ),
        ),
        (
            r#"test.for([[1]])("case", (value, ctx) => {
              ctx.expect(value).toEqual([1]);
            });"#,
            (
                r#"test.for([[1]])("case", (value, ctx) => {ctx.expect.hasAssertions();
              ctx.expect(value).toEqual([1]);
            });"#,
                r#"test.for([[1]])("case", (value, ctx) => {ctx.expect.assertions();
              ctx.expect(value).toEqual([1]);
            });"#,
            ),
        ),
        // A declared context remains a valid suggestion target when global expect is used.
        (
            r#"test.each([{ value: 1 }])("case", ({ value }, ctx) => {
              expect(value).toBe(1);
            });"#,
            (
                r#"test.each([{ value: 1 }])("case", ({ value }, ctx) => {ctx.expect.hasAssertions();
              expect(value).toBe(1);
            });"#,
                r#"test.each([{ value: 1 }])("case", ({ value }, ctx) => {ctx.expect.assertions();
              expect(value).toBe(1);
            });"#,
            ),
        ),
        // Nonliteral and tagged tables resolve a used trailing TestContext.
        (
            r#"const rows = [[1]];
            test.each(rows)("case", (value, { expect: e }) => {
              e(value).toBe(1);
            });"#,
            (
                r#"const rows = [[1]];
            test.each(rows)("case", (value, { expect: e }) => {e.hasAssertions();
              e(value).toBe(1);
            });"#,
                r#"const rows = [[1]];
            test.each(rows)("case", (value, { expect: e }) => {e.assertions();
              e(value).toBe(1);
            });"#,
            ),
        ),
        (
            r#"test.each`
              value
              ${1}
            `("case", ({ value }, { expect: e }) => {
              e(value).toBe(1);
            });"#,
            (
                r#"test.each`
              value
              ${1}
            `("case", ({ value }, { expect: e }) => {e.hasAssertions();
              e(value).toBe(1);
            });"#,
                r#"test.each`
              value
              ${1}
            `("case", ({ value }, { expect: e }) => {e.assertions();
              e(value).toBe(1);
            });"#,
            ),
        ),
        // With no row arguments, `.each` passes TestContext first.
        (
            r#"test.each([[]])("uses context", ({ expect: e }) => {
              e(value).toBe(value);
            });"#,
            (
                r#"test.each([[]])("uses context", ({ expect: e }) => {e.hasAssertions();
              e(value).toBe(value);
            });"#,
                r#"test.each([[]])("uses context", ({ expect: e }) => {e.assertions();
              e(value).toBe(value);
            });"#,
            ),
        ),
        (
            r#"test.each([[]] as const)("uses context", (ctx) => {
              ctx.expect(value).toBe(value);
            });"#,
            (
                r#"test.each([[]] as const)("uses context", (ctx) => {ctx.expect.hasAssertions();
              ctx.expect(value).toBe(value);
            });"#,
                r#"test.each([[]] as const)("uses context", (ctx) => {ctx.expect.assertions();
              ctx.expect(value).toBe(value);
            });"#,
            ),
        ),
    ];

    // These fix cases require config options but `ExpectFixTestCase` doesn't support
    // `(S, (S, S), Option<Value>)` — the two-suggestion format with config.
    //
    // onlyFunctionsWithExpectInLoop (OG case 11):
    // (
    //     r#"it("it1", () => {
    //         expect.hasAssertions();
    //         for (const number of getNumbers()) {
    //           expect(number).toBeGreaterThan(0);
    //         }
    //       });
    //       it("it1", () => {
    //         for (const number of getNumbers()) {
    //           expect(number).toBeGreaterThan(0);
    //         }
    //       });"#,
    //     (
    //         // adds expect.hasAssertions(); to second test only
    //         // adds expect.assertions(); to second test only
    //     ),
    //     Some(serde_json::json!([{ "onlyFunctionsWithExpectInLoop": true }])),
    // ),
    //
    // onlyFunctionsWithExpectInLoop (OG case 12):
    // (
    //     r#"it("returns numbers > 4", async () => {
    //         for (const number of await getNumbers()) {
    //           expect(number).toBeGreaterThan(4);
    //         }
    //       });
    //       it("returns numbers > 5", () => {
    //         for (const number of getNumbers()) {
    //           expect(number).toBeGreaterThan(5);
    //         }
    //       });"#,
    //     (
    //         // adds expect.hasAssertions(); to both tests
    //         // adds expect.assertions(); to both tests
    //     ),
    //     Some(serde_json::json!([{ "onlyFunctionsWithExpectInLoop": true }])),
    // ),

    let fix_remove_args = vec![
        (
            r#"import * as vi from 'vitest';
            it("it1", ({ expect: e }) => {e.hasAssertions("1");})"#,
            r#"import * as vi from 'vitest';
            it("it1", ({ expect: e }) => {e.hasAssertions();})"#,
        ),
        (
            r#"import * as vi from 'vitest';
            it("it1", (ctx) => {ctx.expect.assertions(1, 2);})"#,
            r#"import * as vi from 'vitest';
            it("it1", (ctx) => {ctx.expect.assertions(1);})"#,
        ),
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
        (
            r#"beforeEach(() => { expect.hasAssertions("1") })"#,
            r"beforeEach(() => { expect.hasAssertions() })",
        ),
        (
            r#"afterEach(() => { expect.hasAssertions("1") })"#,
            r"afterEach(() => { expect.hasAssertions() })",
        ),
        (
            r#"import { expect as e } from 'vitest';
            beforeEach(() => { e.hasAssertions("1") })"#,
            "import { expect as e } from 'vitest';
            beforeEach(() => { e.hasAssertions() })",
        ),
    ];

    Tester::new(PreferExpectAssertions::NAME, PreferExpectAssertions::PLUGIN, pass, fail)
        .with_vitest_plugin(true)
        .expect_fix(fix_two_suggestions)
        .expect_fix(fix_import_reassignment)
        .expect_fix(fix_remove_args)
        .test_and_snapshot();
}
