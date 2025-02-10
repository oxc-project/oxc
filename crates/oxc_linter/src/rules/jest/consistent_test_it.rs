use std::{borrow::Cow, str::FromStr};

use oxc_ast::{ast::Expression, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::ScopeId;
use oxc_span::{GetSpan, Span};
use rustc_hash::FxHashMap;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{
        collect_possible_jest_call_node, parse_jest_fn_call, JestFnKind, JestGeneralFnKind,
        ParsedJestFnCallNew, PossibleJestNode,
    },
};

fn consistent_method(x1: &str, x2: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Enforce `test` and `it` usage conventions")
        .with_help(format!("Prefer using {x1:?} instead of {x2:?}"))
        .with_label(span)
}

fn consistent_method_within_describe(x1: &str, x2: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Enforce `test` and `it` usage conventions")
        .with_help(format!("Prefer using {x1:?} instead of {x2:?} within describe"))
        .with_label(span)
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum TestCaseName {
    Fit,
    IT,
    Test,
    Xit,
    Xtest,
}
impl TestCaseName {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Fit => "fit",
            Self::IT => "it",
            Self::Test => "test",
            Self::Xit => "xit",
            Self::Xtest => "xtest",
        }
    }
}

impl std::fmt::Display for TestCaseName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_str().fmt(f)
    }
}

impl std::str::FromStr for TestCaseName {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "fit" => Ok(TestCaseName::Fit),
            "it" => Ok(TestCaseName::IT),
            "test" => Ok(TestCaseName::Test),
            "xit" => Ok(TestCaseName::Xit),
            "xtest" => Ok(TestCaseName::Xtest),
            _ => Err("Unknown Test case name"),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct ConsistentTestIt(Box<ConsistentTestItConfig>);

#[derive(Debug, Clone)]
pub struct ConsistentTestItConfig {
    within_describe: TestCaseName,
    within_fn: TestCaseName,
}

impl std::ops::Deref for ConsistentTestIt {
    type Target = ConsistentTestItConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for ConsistentTestItConfig {
    fn default() -> Self {
        Self { within_describe: TestCaseName::IT, within_fn: TestCaseName::Test }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Jest allows you to choose how you want to define your tests, using the `it` or
    /// the `test` keywords, with multiple permutations for each:
    ///
    /// - **it:** `it`, `xit`, `fit`, `it.only`, `it.skip`.
    /// - **test:** `test`, `xtest`, `test.only`, `test.skip`.
    ///
    /// ### Why is this bad?
    ///
    /// It's a good practice to be consistent in your test suite, so that all tests are written in the same way.
    ///
    /// ### Example
    ///
    /// ```javascript
    /// /*eslint jest/consistent-test-it: ["error", {"fn": "test"}]*/
    /// test('foo'); // valid
    /// test.only('foo'); // valid
    ///
    /// it('foo'); // invalid
    /// it.only('foo'); // invalid
    /// ```
    ///
    /// ```javascript
    /// /*eslint jest/consistent-test-it: ["error", {"fn": "it"}]*/
    /// it('foo'); // valid
    /// it.only('foo'); // valid
    /// test('foo'); // invalid
    /// test.only('foo'); // invalid
    /// ```
    ///
    /// ```javascript
    /// /*eslint jest/consistent-test-it: ["error", {"fn": "it", "withinDescribe": "test"}]*/
    /// it('foo'); // valid
    /// describe('foo', function () {
    ///     test('bar'); // valid
    /// });
    ///
    /// test('foo'); // invalid
    /// describe('foo', function () {
    ///     it('bar'); // invalid
    /// });
    /// ```
    ///
    /// #### Options
    ///
    /// This rule can be configured as follows
    /// ```json5
    /// {
    ///     type: 'object',
    ///     properties: {
    ///         fn: {
    ///             enum: ['it', 'test'],
    ///         },
    ///         withinDescribe: {
    ///             enum: ['it', 'test'],
    ///         },
    ///     },
    ///     additionalProperties: false,
    /// }
    /// ```
    ///
    /// ##### fn
    /// Decides whether to use `test` or `it`.
    ///
    /// ##### withinDescribe
    /// Decides whether to use `test` or `it` within a `describe` scope.
    ///
    ///
    /// This rule is compatible with [eslint-plugin-vitest](https://github.com/veritem/eslint-plugin-vitest/blob/v1.1.9/docs/rules/consistent-test-it.md),
    /// to use it, add the following configuration to your `.eslintrc.json`:
    ///
    /// ```json
    /// {
    ///   "rules": {
    ///      "vitest/consistent-test-it": "error"
    ///   }
    /// }
    ConsistentTestIt,
    jest,
    style,
    fix
);

impl Rule for ConsistentTestIt {
    fn from_configuration(value: serde_json::Value) -> Self {
        let config = value.get(0);

        let within_fn = config
            .and_then(|config| config.get("fn"))
            .and_then(serde_json::Value::as_str)
            .and_then(|x| TestCaseName::from_str(x).ok())
            .unwrap_or(TestCaseName::Test);

        let within_describe = config
            .and_then(|config| config.get("withinDescribe"))
            .and_then(serde_json::Value::as_str)
            .and_then(|x| TestCaseName::from_str(x).ok())
            .unwrap_or(
                config
                    .and_then(|config| config.get("fn"))
                    .and_then(serde_json::Value::as_str)
                    .and_then(|x| TestCaseName::from_str(x).ok())
                    .unwrap_or(TestCaseName::IT),
            );

        Self(Box::new(ConsistentTestItConfig { within_describe, within_fn }))
    }

    fn run_once(&self, ctx: &LintContext) {
        let mut describe_nesting_hash: FxHashMap<ScopeId, i32> = FxHashMap::default();
        let mut possible_jest_nodes = collect_possible_jest_call_node(ctx);
        possible_jest_nodes.sort_by_key(|n| n.node.id());

        for possible_jest_node in &possible_jest_nodes {
            self.run(&mut describe_nesting_hash, possible_jest_node, ctx);
        }
    }
}

impl ConsistentTestIt {
    fn run<'a>(
        &self,
        describe_nesting_hash: &mut FxHashMap<ScopeId, i32>,
        possible_jest_node: &PossibleJestNode<'a, '_>,
        ctx: &LintContext<'a>,
    ) {
        let node = possible_jest_node.node;
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };
        let Some(ParsedJestFnCallNew::GeneralJest(jest_fn_call)) =
            parse_jest_fn_call(call_expr, possible_jest_node, ctx)
        else {
            return;
        };

        if matches!(jest_fn_call.kind, JestFnKind::General(JestGeneralFnKind::Describe)) {
            let scope_id = node.scope_id();
            let current_count = describe_nesting_hash.get(&scope_id).unwrap_or(&0);
            describe_nesting_hash.insert(scope_id, *current_count + 1);
            return;
        }

        let is_test = matches!(jest_fn_call.kind, JestFnKind::General(JestGeneralFnKind::Test));
        let fn_to_str = self.within_fn.as_str();

        if is_test && describe_nesting_hash.is_empty() && !jest_fn_call.name.ends_with(fn_to_str) {
            let opposite_test_keyword = Self::get_opposite_test_case(self.within_fn);
            if let Some((span, prefer_test_name)) = Self::get_prefer_test_name_and_span(
                call_expr.callee.get_inner_expression(),
                &jest_fn_call.name,
                fn_to_str,
            ) {
                ctx.diagnostic_with_fix(
                    consistent_method(fn_to_str, opposite_test_keyword, span),
                    |fixer| fixer.replace(span, prefer_test_name),
                );
            }
        }

        let describe_to_str = self.within_describe.as_str();

        if is_test
            && !describe_nesting_hash.is_empty()
            && !jest_fn_call.name.ends_with(describe_to_str)
        {
            let opposite_test_keyword = Self::get_opposite_test_case(self.within_describe);
            if let Some((span, prefer_test_name)) = Self::get_prefer_test_name_and_span(
                call_expr.callee.get_inner_expression(),
                &jest_fn_call.name,
                describe_to_str,
            ) {
                ctx.diagnostic_with_fix(
                    consistent_method_within_describe(describe_to_str, opposite_test_keyword, span),
                    |fixer| fixer.replace(span, prefer_test_name),
                );
            }
        }
    }

    fn get_opposite_test_case(test_case_name: TestCaseName) -> &'static str {
        if matches!(test_case_name, TestCaseName::Test) {
            TestCaseName::IT.as_str()
        } else {
            TestCaseName::Test.as_str()
        }
    }

    fn get_prefer_test_name_and_span<'s>(
        expr: &Expression,
        test_name: &str,
        fix_jest_name: &'s str,
    ) -> Option<(Span, Cow<'s, str>)> {
        match expr {
            Expression::Identifier(ident) => {
                if ident.name.eq("fit") {
                    return Some((ident.span, Cow::Borrowed("test.only")));
                }

                let prefer_test_name = match test_name.chars().next() {
                    Some('x') => Cow::Owned(format!("x{fix_jest_name}")),
                    Some('f') => Cow::Owned(format!("f{fix_jest_name}")),
                    _ => Cow::Borrowed(fix_jest_name),
                };
                Some((ident.span(), prefer_test_name))
            }
            Expression::StaticMemberExpression(expr) => {
                Self::get_prefer_test_name_and_span(&expr.object, test_name, fix_jest_name)
            }
            Expression::CallExpression(call_expr) => Self::get_prefer_test_name_and_span(
                call_expr.callee.get_inner_expression(),
                test_name,
                fix_jest_name,
            ),
            Expression::TaggedTemplateExpression(expr) => Self::get_prefer_test_name_and_span(
                expr.tag.get_inner_expression(),
                test_name,
                fix_jest_name,
            ),
            _ => None,
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let mut pass = vec![
        // consistent-test-it with fn=test
        ("test(\"foo\")", Some(serde_json::json!([{ "fn": "test" }]))),
        ("test.only(\"foo\")", Some(serde_json::json!([{ "fn": "test" }]))),
        ("test.skip(\"foo\")", Some(serde_json::json!([{ "fn": "test" }]))),
        ("test.concurrent(\"foo\")", Some(serde_json::json!([{ "fn": "test" }]))),
        ("xtest(\"foo\")", Some(serde_json::json!([{ "fn": "test" }]))),
        ("test.each([])(\"foo\")", Some(serde_json::json!([{ "fn": "test" }]))),
        ("test.each``(\"foo\")", Some(serde_json::json!([{ "fn": "test" }]))),
        (
            "describe(\"suite\", () => { test(\"foo\") })",
            Some(serde_json::json!([{ "fn": "test" }])),
        ),
        // consistent-test-it with fn=it
        ("it(\"foo\")", Some(serde_json::json!([{ "fn": "it" }]))),
        ("fit(\"foo\")", Some(serde_json::json!([{ "fn": "it" }]))),
        ("xit(\"foo\")", Some(serde_json::json!([{ "fn": "it" }]))),
        ("it.only(\"foo\")", Some(serde_json::json!([{ "fn": "it" }]))),
        ("it.skip(\"foo\")", Some(serde_json::json!([{ "fn": "it" }]))),
        ("it.concurrent(\"foo\")", Some(serde_json::json!([{ "fn": "it" }]))),
        ("it.each([])(\"foo\")", Some(serde_json::json!([{ "fn": "it" }]))),
        ("it.each``(\"foo\")", Some(serde_json::json!([{ "fn": "it" }]))),
        ("describe(\"suite\", () => { it(\"foo\") })", Some(serde_json::json!([{ "fn": "it" }]))),
        // consistent-test-it with fn=test and withinDescribe=it
        ("test(\"foo\")", Some(serde_json::json!([{ "fn": "test", "withinDescribe": "it" }]))),
        ("test.only(\"foo\")", Some(serde_json::json!([{ "fn": "test", "withinDescribe": "it" }]))),
        ("test.skip(\"foo\")", Some(serde_json::json!([{ "fn": "test", "withinDescribe": "it" }]))),
        (
            "test.concurrent(\"foo\")",
            Some(serde_json::json!([{ "fn": "test", "withinDescribe": "it" }])),
        ),
        ("xtest(\"foo\")", Some(serde_json::json!([{ "fn": "test", "withinDescribe": "it" }]))),
        (
            "[1,2,3].forEach(() => { test(\"foo\") })",
            Some(serde_json::json!([{ "fn": "test", "withinDescribe": "it" }])),
        ),
        // consistent-test-it with fn=it and withinDescribe=test
        ("it(\"foo\")", Some(serde_json::json!([{ "fn": "it", "withinDescribe": "test" }]))),
        ("it.only(\"foo\")", Some(serde_json::json!([{ "fn": "it", "withinDescribe": "test" }]))),
        ("it.skip(\"foo\")", Some(serde_json::json!([{ "fn": "it", "withinDescribe": "test" }]))),
        (
            "it.concurrent(\"foo\")",
            Some(serde_json::json!([{ "fn": "it", "withinDescribe": "test" }])),
        ),
        ("xit(\"foo\")", Some(serde_json::json!([{ "fn": "it", "withinDescribe": "test" }]))),
        (
            "[1,2,3].forEach(() => { it(\"foo\") })",
            Some(serde_json::json!([{ "fn": "it", "withinDescribe": "test" }])),
        ),
        // consistent-test-it with fn=test and withinDescribe=test
        (
            "describe(\"suite\", () => { test(\"foo\") })",
            Some(serde_json::json!([{ "fn": "test", "withinDescribe": "test" }])),
        ),
        ("test(\"foo\");", Some(serde_json::json!([{ "fn": "test", "withinDescribe": "test" }]))),
        // consistent-test-it with fn=it and withinDescribe=it
        (
            "describe(\"suite\", () => { it(\"foo\") })",
            Some(serde_json::json!([{ "fn": "it", "withinDescribe": "it" }])),
        ),
        ("it(\"foo\")", Some(serde_json::json!([{ "fn": "it", "withinDescribe": "it" }]))),
        // consistent-test-it defaults without config object
        ("test(\"foo\")", None),
        // consistent-test-it with withinDescribe=it
        ("test(\"foo\")", Some(serde_json::json!([{ "withinDescribe": "it" }]))),
        (
            "describe(\"suite\", () => { it(\"foo\") })",
            Some(serde_json::json!([{ "withinDescribe": "it" }])),
        ),
        // consistent-test-it with withinDescribe=test
        ("test(\"foo\")", Some(serde_json::json!([{ "withinDescribe": "test" }]))),
        (
            "describe(\"suite\", () => { test(\"foo\") })",
            Some(serde_json::json!([{ "withinDescribe": "test" }])),
        ),
    ];

    let mut fail = vec![
        // consistent-test-it with fn=test
        ("it(\"foo\")", Some(serde_json::json!([{ "fn": "test" }]))),
        (
            "
                import { it } from '@jest/globals';

                it(\"foo\")
            ",
            Some(serde_json::json!([{ "fn": "test" }])),
        ),
        (
            "
                import { it as testThisThing } from '@jest/globals';

                testThisThing(\"foo\")
            ",
            Some(serde_json::json!([{ "fn": "test" }])),
        ),
        ("xit(\"foo\")", Some(serde_json::json!([{ "fn": "test" }]))),
        ("fit(\"foo\")", Some(serde_json::json!([{ "fn": "test" }]))),
        ("it.skip(\"foo\")", Some(serde_json::json!([{ "fn": "test" }]))),
        ("it.concurrent(\"foo\")", Some(serde_json::json!([{ "fn": "test" }]))),
        ("it.only(\"foo\")", Some(serde_json::json!([{ "fn": "test" }]))),
        ("it.each([])(\"foo\")", Some(serde_json::json!([{ "fn": "test" }]))),
        ("it.each``(\"foo\")", Some(serde_json::json!([{ "fn": "test" }]))),
        (
            "describe.each``(\"foo\", () => { it.each``(\"bar\") })",
            Some(serde_json::json!([{ "fn": "test" }])),
        ),
        (
            "describe.each``(\"foo\", () => { test.each``(\"bar\") })",
            Some(serde_json::json!([{ "fn": "it" }])),
        ),
        (
            "
                describe.each()(\"%s\", () => {
                    test(\"is valid, but should not be\", () => {});

                    it(\"is not valid, but should be\", () => {});
                });
            ",
            Some(serde_json::json!([{ "fn": "test", "withinDescribe": "it" }])),
        ),
        (
            "
                describe.only.each()(\"%s\", () => {
                    test(\"is valid, but should not be\", () => {});

                    it(\"is not valid, but should be\", () => {});
                });
            ",
            Some(serde_json::json!([{ "fn": "test", "withinDescribe": "it" }])),
        ),
        ("describe(\"suite\", () => { it(\"foo\") })", Some(serde_json::json!([{ "fn": "test" }]))),
        // consistent-test-it with fn=it
        ("test(\"foo\")", Some(serde_json::json!([{ "fn": "it" }]))),
        ("xtest(\"foo\")", Some(serde_json::json!([{ "fn": "it" }]))),
        ("test.skip(\"foo\")", Some(serde_json::json!([{ "fn": "it" }]))),
        ("test.concurrent(\"foo\")", Some(serde_json::json!([{ "fn": "it" }]))),
        ("test.only(\"foo\")", Some(serde_json::json!([{ "fn": "it" }]))),
        ("test.each([])(\"foo\")", Some(serde_json::json!([{ "fn": "it" }]))),
        (
            "describe.each``(\"foo\", () => { test.each``(\"bar\") })",
            Some(serde_json::json!([{ "fn": "it" }])),
        ),
        ("test.each``(\"foo\")", Some(serde_json::json!([{ "fn": "it" }]))),
        ("describe(\"suite\", () => { test(\"foo\") })", Some(serde_json::json!([{ "fn": "it" }]))),
        // consistent-test-it with fn=test and withinDescribe=it
        (
            "describe(\"suite\", () => { test(\"foo\") })",
            Some(serde_json::json!([{ "fn": "test", "withinDescribe": "it" }])),
        ),
        (
            "describe(\"suite\", () => { test.only(\"foo\") })",
            Some(serde_json::json!([{ "fn": "test", "withinDescribe": "it" }])),
        ),
        (
            "describe(\"suite\", () => { xtest(\"foo\") })",
            Some(serde_json::json!([{ "fn": "test", "withinDescribe": "it" }])),
        ),
        (
            "
                import { xtest as dontTestThis } from '@jest/globals';

                describe(\"suite\", () => { dontTestThis(\"foo\") });
            ",
            Some(serde_json::json!([{ "fn": "test", "withinDescribe": "it" }])),
        ),
        (
            "
                import { describe as context, xtest as dontTestThis } from '@jest/globals';

                context(\"suite\", () => { dontTestThis(\"foo\") });
            ",
            Some(serde_json::json!([{ "fn": "test", "withinDescribe": "it" }])),
        ),
        (
            "describe(\"suite\", () => { test.skip(\"foo\") })",
            Some(serde_json::json!([{ "fn": "test", "withinDescribe": "it" }])),
        ),
        (
            "describe(\"suite\", () => { test.concurrent(\"foo\") })",
            Some(serde_json::json!([{ "fn": "test", "withinDescribe": "it" }])),
        ),
        // consistent-test-it with fn=it and withinDescribe=test
        (
            "describe(\"suite\", () => { test(\"foo\") })",
            Some(serde_json::json!([{ "fn": "test", "withinDescribe": "it" }])),
        ),
        (
            "describe(\"suite\", () => { test.only(\"foo\") })",
            Some(serde_json::json!([{ "fn": "test", "withinDescribe": "it" }])),
        ),
        (
            "describe(\"suite\", () => { xtest(\"foo\") })",
            Some(serde_json::json!([{ "fn": "test", "withinDescribe": "it" }])),
        ),
        (
            "
                import { xtest as dontTestThis } from '@jest/globals';

                describe(\"suite\", () => { dontTestThis(\"foo\") });
            ",
            Some(serde_json::json!([{ "fn": "test", "withinDescribe": "it" }])),
        ),
        (
            "
            import { describe as context, xtest as dontTestThis } from '@jest/globals';

            context(\"suite\", () => { dontTestThis(\"foo\") });
        ",
            Some(serde_json::json!([{ "fn": "test", "withinDescribe": "it" }])),
        ),
        (
            "describe(\"suite\", () => { test.skip(\"foo\") })",
            Some(serde_json::json!([{ "fn": "test", "withinDescribe": "it" }])),
        ),
        (
            "describe(\"suite\", () => { test.concurrent(\"foo\") })",
            Some(serde_json::json!([{ "fn": "test", "withinDescribe": "it" }])),
        ),
        // consistent-test-it with fn=test and withinDescribe=test
        (
            "describe(\"suite\", () => { it(\"foo\") })",
            Some(serde_json::json!([{ "fn": "test", "withinDescribe": "test" }])),
        ),
        ("it(\"foo\")", Some(serde_json::json!([{ "fn": "test", "withinDescribe": "test" }]))),
        // consistent-test-it with fn=it and withinDescribe=it
        (
            "describe(\"suite\", () => { test(\"foo\") })",
            Some(serde_json::json!([{ "fn": "it", "withinDescribe": "it" }])),
        ),
        ("test(\"foo\")", Some(serde_json::json!([{ "fn": "it", "withinDescribe": "it" }]))),
        // consistent-test-it defaults without config object
        ("describe(\"suite\", () => { test(\"foo\") })", None),
        // consistent-test-it with withinDescribe=it
        ("it(\"foo\")", Some(serde_json::json!([{ "withinDescribe": "it" }]))),
        (
            "describe(\"suite\", () => { test(\"foo\") })",
            Some(serde_json::json!([{ "withinDescribe": "it" }])),
        ),
        // consistent-test-it with withinDescribe=test
        ("it(\"foo\")", Some(serde_json::json!([{ "withinDescribe": "test" }]))),
        (
            "describe(\"suite\", () => { it(\"foo\") })",
            Some(serde_json::json!([{ "withinDescribe": "test" }])),
        ),
    ];

    let mut fix = vec![
        // consistent-test-it with fn=test
        ("it(\"foo\")", "test(\"foo\")"),
        (
            "
                import { it } from '@jest/globals';
                it(\"foo\")
            ",
            "
                import { it } from '@jest/globals';
                test(\"foo\")
            ",
        ),
        (
            "
                import { it as testThisThing } from '@jest/globals';
                testThisThing(\"foo\")
            ",
            "
                import { it as testThisThing } from '@jest/globals';
                test(\"foo\")
            ",
        ),
        ("xit(\"foo\")", "xtest(\"foo\")"),
        ("fit(\"foo\")", "test.only(\"foo\")"),
        ("it.skip(\"foo\")", "test.skip(\"foo\")"),
        ("it.concurrent(\"foo\")", "test.concurrent(\"foo\")"),
        ("it.only(\"foo\")", "test.only(\"foo\")"),
        ("it.each([])(\"foo\")", "test.each([])(\"foo\")"),
        ("it.each``(\"foo\")", "test.each``(\"foo\")"),
        // Note: couldn't fix
        // Todo: this need to fixer support option configuration.
        // (
        //     "describe.each``(\"foo\", () => { it.each``(\"bar\") })",
        //     "describe.each``(\"foo\", () => { test.each``(\"bar\") })",
        // ),
        (
            "describe.each``(\"foo\", () => { test.each``(\"bar\") })",
            "describe.each``(\"foo\", () => { it.each``(\"bar\") })",
        ),
        (
            "
                describe.each()(\"%s\", () => {
                    test(\"is valid, but should not be\", () => {});
                    it(\"is not valid, but should be\", () => {});
                });
            ",
            "
                describe.each()(\"%s\", () => {
                    it(\"is valid, but should not be\", () => {});
                    it(\"is not valid, but should be\", () => {});
                });
            ",
        ),
        (
            "
                describe.only.each()(\"%s\", () => {
                    test(\"is valid, but should not be\", () => {});
                    it(\"is not valid, but should be\", () => {});
                });
            ",
            "
                describe.only.each()(\"%s\", () => {
                    it(\"is valid, but should not be\", () => {});
                    it(\"is not valid, but should be\", () => {});
                });
            ",
        ),
        // Note: couldn't fix, because the fixer couldn't be set option `fn=it`
        // (
        //     "describe(\"suite\", () => { it(\"foo\") })",
        //     "describe(\"suite\", () => { test(\"foo\") })",
        // ),
        // consistent-test-it with fn=it
        // ("test(\"foo\")", "it(\"foo\")"),
        // ("xtest(\"foo\")", "xit(\"foo\")"),
        // ("test.skip(\"foo\")", "it.skip(\"foo\")"),
        // ("test.concurrent(\"foo\")", "it.concurrent(\"foo\")"),
        // ("test.only(\"foo\")", "it.only(\"foo\")"),
        // ("test.each([])(\"foo\")", "it.each([])(\"foo\")"),
        // ("test.each``(\"foo\")", "it.each``(\"foo\")"),
        (
            "describe.each``(\"foo\", () => { test.each``(\"bar\") })",
            "describe.each``(\"foo\", () => { it.each``(\"bar\") })",
        ),
        (
            "describe(\"suite\", () => { test(\"foo\") })",
            "describe(\"suite\", () => { it(\"foo\") })",
        ),
        //
        // consistent-test-it with fn=test and withinDescribe=it
        (
            "describe(\"suite\", () => { test(\"foo\") })",
            "describe(\"suite\", () => { it(\"foo\") })",
        ),
        (
            "describe(\"suite\", () => { test.only(\"foo\") })",
            "describe(\"suite\", () => { it.only(\"foo\") })",
        ),
        (
            "describe(\"suite\", () => { xtest(\"foo\") })",
            "describe(\"suite\", () => { xit(\"foo\") })",
        ),
        (
            "
                import { xtest as dontTestThis } from '@jest/globals';
                describe(\"suite\", () => { dontTestThis(\"foo\") });
            ",
            "
                import { xtest as dontTestThis } from '@jest/globals';
                describe(\"suite\", () => { xit(\"foo\") });
            ",
        ),
        (
            "
                import { describe as context, xtest as dontTestThis } from '@jest/globals';
                context(\"suite\", () => { dontTestThis(\"foo\") });
            ",
            "
                import { describe as context, xtest as dontTestThis } from '@jest/globals';
                context(\"suite\", () => { xit(\"foo\") });
            ",
        ),
        (
            "describe(\"suite\", () => { test.skip(\"foo\") })",
            "describe(\"suite\", () => { it.skip(\"foo\") })",
        ),
        (
            "describe(\"suite\", () => { test.concurrent(\"foo\") })",
            "describe(\"suite\", () => { it.concurrent(\"foo\") })",
        ),
        // consistent-test-it with fn=it and withinDescribe=test
        (
            "describe(\"suite\", () => { test(\"foo\") })",
            "describe(\"suite\", () => { it(\"foo\") })",
        ),
        (
            "describe(\"suite\", () => { test.only(\"foo\") })",
            "describe(\"suite\", () => { it.only(\"foo\") })",
        ),
        (
            "describe(\"suite\", () => { xtest(\"foo\") })",
            "describe(\"suite\", () => { xit(\"foo\") })",
        ),
        (
            "
                import { xtest as dontTestThis } from '@jest/globals';
                describe(\"suite\", () => { dontTestThis(\"foo\") });
            ",
            "
                import { xtest as dontTestThis } from '@jest/globals';
                describe(\"suite\", () => { xit(\"foo\") });
            ",
        ),
        (
            "
                import { describe as context, xtest as dontTestThis } from '@jest/globals';
                context(\"suite\", () => { dontTestThis(\"foo\") });
            ",
            "
                import { describe as context, xtest as dontTestThis } from '@jest/globals';
                context(\"suite\", () => { xit(\"foo\") });
            ",
        ),
        (
            "describe(\"suite\", () => { test.skip(\"foo\") })",
            "describe(\"suite\", () => { it.skip(\"foo\") })",
        ),
        (
            "describe(\"suite\", () => { test.concurrent(\"foo\") })",
            "describe(\"suite\", () => { it.concurrent(\"foo\") })",
        ),
        // Note: couldn't fix
        // Todo: this need to fixer support option configuration.
        // consistent-test-it with fn=test and withinDescribe=test
        // (
        //     "describe(\"suite\", () => { it(\"foo\") })",
        //     "describe(\"suite\", () => { test(\"foo\") })",
        // ),
        // ("it(\"foo\")", "test(\"foo\")"),
        //
        // consistent-test-it with fn=it and withinDescribe=it
        (
            "describe(\"suite\", () => { test(\"foo\") })",
            "describe(\"suite\", () => { it(\"foo\") })",
        ),
        // ("test(\"foo\")", "it(\"foo\")"),
        // consistent-test-it defaults without config object
        (
            "describe(\"suite\", () => { test(\"foo\") })",
            "describe(\"suite\", () => { it(\"foo\") })",
        ),
        // consistent-test-it with withinDescribe=it
        ("it(\"foo\")", "test(\"foo\")"),
        (
            "describe(\"suite\", () => { test(\"foo\") })",
            "describe(\"suite\", () => { it(\"foo\") })",
        ),
        // consistent-test-it with withinDescribe=test
        ("it(\"foo\")", "test(\"foo\")"),
        // Note: couldn't fixed
        // Todo: this need to fixer support option configuration.
        // (
        //     "describe(\"suite\", () => { it(\"foo\") })",
        //     "describe(\"suite\", () => { test(\"foo\") })",
        // ),
    ];

    let pass_vitest = vec![
        (
            "
                it(\"shows error\", () => {
                    expect(true).toBe(false);
                });
            ",
            Some(serde_json::json!([{ "fn": "it" }])),
        ),
        (
            "
                it(\"foo\", function () {
                    expect(true).toBe(false);
                })
            ",
            Some(serde_json::json!([{ "fn": "it" }])),
        ),
        (
            "
                it('foo', () => {
                        expect(true).toBe(false);
                    });
                function myTest() { if ('bar') {} }
            ",
            Some(serde_json::json!([{ "fn": "it" }])),
        ),
        (
            "
                test(\"shows error\", () => {
                    expect(true).toBe(false);
                });
            ",
            Some(serde_json::json!([{ "fn": "test" }])),
        ),
        ("test.skip(\"foo\")", Some(serde_json::json!([{ "fn": "test" }]))),
        ("test.concurrent(\"foo\")", Some(serde_json::json!([{ "fn": "test" }]))),
        ("xtest(\"foo\")", Some(serde_json::json!([{ "fn": "test" }]))),
        ("test.each([])(\"foo\")", Some(serde_json::json!([{ "fn": "test" }]))),
        ("test.each``(\"foo\")", Some(serde_json::json!([{ "fn": "test" }]))),
        (
            "describe(\"suite\", () => { test(\"foo\") })",
            Some(serde_json::json!([{ "fn": "test" }])),
        ),
        (
            "describe(\"suite\", () => { it(\"foo\") })",
            Some(serde_json::json!([{ "fn": "it", "withinDescribe": "it" }])),
        ),
        ("it(\"foo\")", Some(serde_json::json!([{ "fn": "it", "withinDescribe": "it" }]))),
        ("test(\"shows error\", () => {});", None),
        ("test(\"foo\")", Some(serde_json::json!([{ "withinDescribe": "it" }]))),
        (
            "describe(\"suite\", () => { it(\"foo\") })",
            Some(serde_json::json!([{ "withinDescribe": "it" }])),
        ),
        ("test(\"foo\")", Some(serde_json::json!([{ "withinDescribe": "test" }]))),
        (
            "describe(\"suite\", () => { test(\"foo\") })",
            Some(serde_json::json!([{ "withinDescribe": "test" }])),
        ),
    ];

    let fail_vitest = vec![
        ("test(\"shows error\", () => {});", Some(serde_json::json!([{ "fn": "it" }]))),
        ("test.skip(\"shows error\");", Some(serde_json::json!([{ "fn": "it" }]))),
        ("test.only('shows error');", Some(serde_json::json!([{ "fn": "it" }]))),
        (
            "describe('foo', () => { it('bar', () => {}); });",
            Some(serde_json::json!([{ "fn": "it", "withinDescribe": "test" }])),
        ),
        (
            "import { test } from \"vitest\"\ntest(\"shows error\", () => {});",
            Some(serde_json::json!([{ "fn": "it" }])),
        ),
        ("it(\"shows error\", () => {});", Some(serde_json::json!([{ "fn": "test" }]))),
        ("describe(\"suite\", () => { it(\"foo\") })", Some(serde_json::json!([{ "fn": "test" }]))),
        (
            "describe(\"suite\", () => { test(\"foo\") })",
            Some(serde_json::json!([{ "fn": "it", "withinDescribe": "it" }])),
        ),
        ("test(\"foo\")", Some(serde_json::json!([{ "fn": "it", "withinDescribe": "it" }]))),
        ("describe(\"suite\", () => { test(\"foo\") })", None),
        ("it(\"foo\")", Some(serde_json::json!([{ "withinDescribe": "it" }]))),
        (
            "describe(\"suite\", () => { test(\"foo\") })",
            Some(serde_json::json!([{ "withinDescribe": "it" }])),
        ),
        ("it(\"foo\")", Some(serde_json::json!([{ "withinDescribe": "test" }]))),
        (
            "import { it } from \"vitest\"\nit(\"foo\")",
            Some(serde_json::json!([{ "withinDescribe": "test" }])),
        ),
        (
            "describe(\"suite\", () => { it(\"foo\") })",
            Some(serde_json::json!([{ "withinDescribe": "test" }])),
        ),
    ];

    let fix_vitest = vec![
        // Note: couldn't  fixed, because the fixer doesn't support to set the options for the fix cases.
        // Todo: this need to fixer support option configuration.
        // ("test(\"shows error\", () => {});", "it(\"shows error\", () => {});"),
        // ("test.skip(\"shows error\");", "it.skip(\"shows error\");"),
        // ("test.only('shows error');", "it.only('shows error');"),
        // (
        //     "describe('foo', () => { it('bar', () => {}); });",
        //     "describe('foo', () => { test('bar', () => {}); });"
        // ),
        // (
        //     "import { test } from \"vitest\"\ntest(\"shows error\", () => {});",
        //     "import { it } from \"vitest\"\nit(\"shows error\", () => {});",
        // ),
        // ("describe(\"suite\", () => { it(\"foo\") })", "describe(\"suite\", () => { test(\"foo\") })"),
        // ("test(\"foo\")", "it(\"foo\")"),
        // ("describe(\"suite\", () => { it(\"foo\") })", "describe(\"suite\", () => { test(\"foo\") })"),
        //
        ("it(\"shows error\", () => {});", "test(\"shows error\", () => {});"),
        (
            "describe(\"suite\", () => { test(\"foo\") })",
            "describe(\"suite\", () => { it(\"foo\") })",
        ),
        (
            "describe(\"suite\", () => { test(\"foo\") })",
            "describe(\"suite\", () => { it(\"foo\") })",
        ),
        ("it(\"foo\")", "test(\"foo\")"),
        (
            "describe(\"suite\", () => { test(\"foo\") })",
            "describe(\"suite\", () => { it(\"foo\") })",
        ),
        ("it(\"foo\")", "test(\"foo\")"),
        // Todo: need to be fixed
        // (
        //     "import { it } from \"vitest\"\nit(\"foo\")",
        //     "import { test } from \"vitest\"\ntest(\"foo\")"
        // ),
    ];

    pass.extend(pass_vitest);
    fail.extend(fail_vitest);
    fix.extend(fix_vitest);

    Tester::new(ConsistentTestIt::NAME, ConsistentTestIt::PLUGIN, pass, fail)
        .with_jest_plugin(true)
        .with_vitest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
