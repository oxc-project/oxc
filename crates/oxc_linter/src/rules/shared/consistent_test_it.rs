use std::borrow::Cow;

use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_semantic::ScopeId;
use oxc_span::{GetSpan, Span};
use rustc_hash::FxHashMap;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    context::LintContext,
    utils::{
        JestFnKind, JestGeneralFnKind, ParsedJestFnCallNew, PossibleJestNode,
        collect_possible_jest_call_node, parse_jest_fn_call,
    },
};

fn consistent_method(preferred_method: &str, other_method: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Enforce `test` and `it` usage conventions")
        .with_help(format!("Prefer using {preferred_method:?} instead of {other_method:?}"))
        .with_label(span)
}

fn consistent_method_within_describe(
    preferred_method: &str,
    other_method: &str,
    span: Span,
) -> OxcDiagnostic {
    OxcDiagnostic::warn("Enforce `test` and `it` usage conventions")
        .with_help(format!(
            "Prefer using {preferred_method:?} instead of {other_method:?} within describe"
        ))
        .with_label(span)
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Deserialize, JsonSchema, Serialize)]
#[serde(rename_all = "lowercase")]
enum TestCaseName {
    IT,
    Test,
}

impl TestCaseName {
    pub fn as_str(self) -> &'static str {
        match self {
            TestCaseName::IT => "it",
            TestCaseName::Test => "test",
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, JsonSchema, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ConsistentTestItConfig {
    /// Decides whether to use `test` or `it` within a `describe` scope.
    /// If only `fn` is provided, this will default to the value of `fn`.
    within_describe: TestCaseName,
    /// Decides whether to use `test` or `it`.
    r#fn: TestCaseName,
}

impl Default for ConsistentTestItConfig {
    fn default() -> Self {
        Self { within_describe: TestCaseName::IT, r#fn: TestCaseName::Test }
    }
}

pub const DOCUMENTATION: &str = r#"
### What it does

 Jest allows you to choose how you want to define your tests, using the `it` or
 the `test` keywords, with multiple permutations for each:

 - **it:** `it`, `xit`, `fit`, `it.only`, `it.skip`.
 - **test:** `test`, `xtest`, `test.only`, `test.skip`.

 ### Why is this bad?

 It's a good practice to be consistent in your test suite, so that all tests are written in the same way.

 ### Examples

 ```javascript
 /* jest/consistent-test-it: ["error", {"fn": "test"}] */
 test('foo'); // valid
 test.only('foo'); // valid

 it('foo'); // invalid
 it.only('foo'); // invalid
 ```

 ```javascript
 /* jest/consistent-test-it: ["error", {"fn": "it"}] */
 it('foo'); // valid
 it.only('foo'); // valid
 test('foo'); // invalid
 test.only('foo'); // invalid
 ```

 ```javascript
 /* jest/consistent-test-it: ["error", {"fn": "it", "withinDescribe": "test"}] */
 it('foo'); // valid
 describe('foo', function () {
     test('bar'); // valid
 });

 test('foo'); // invalid
 describe('foo', function () {
     it('bar'); // invalid
 });
 ```

 This rule is compatible with [eslint-plugin-vitest](https://github.com/vitest-dev/eslint-plugin-vitest/blob/v1.1.9/docs/rules/consistent-test-it.md),
 to use it, add the following configuration to your `.oxlintrc.json`:

 ```json
 {
   "rules": {
      "vitest/consistent-test-it": "error"
   }
 }
 ```
"#;

impl ConsistentTestItConfig {
    #[expect(clippy::unnecessary_wraps)] // TODO: fail on serde error
    pub fn from_configuration(value: &serde_json::Value) -> Result<Self, serde_json::error::Error> {
        if value.is_null() {
            return Ok(Self::default());
        }

        let config_value = value.get(0).unwrap_or(value);

        let mut config: ConsistentTestItConfig =
            serde_json::from_value(config_value.clone()).unwrap_or_default();

        // If withinDescribe wasn't provided, default it to the value of `fn` only if fn was explicitly provided
        if config_value.get("withinDescribe").is_none() && config_value.get("fn").is_some() {
            config.within_describe = config.r#fn;
        }

        Ok(config)
    }

    pub fn run_once(self, ctx: &LintContext) {
        let mut describe_nesting_hash: FxHashMap<ScopeId, i32> = FxHashMap::default();
        let mut possible_jest_nodes = collect_possible_jest_call_node(ctx);
        possible_jest_nodes.sort_unstable_by_key(|n| n.node.id());

        for possible_jest_node in &possible_jest_nodes {
            self.run(&mut describe_nesting_hash, possible_jest_node, ctx);
        }
    }
}

impl ConsistentTestItConfig {
    fn run<'a>(
        self,
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
        let fn_to_str = self.r#fn.as_str();

        if is_test && describe_nesting_hash.is_empty() && !jest_fn_call.name.ends_with(&fn_to_str) {
            let opposite_test_keyword = Self::get_opposite_test_case(self.r#fn);
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
            && !jest_fn_call.name.ends_with(&describe_to_str)
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
        if matches!(test_case_name, TestCaseName::Test) { "it" } else { "test" }
    }

    fn get_prefer_test_name_and_span<'s>(
        expr: &Expression,
        test_name: &str,
        fix_jest_name: &'s str,
    ) -> Option<(Span, Cow<'s, str>)> {
        match expr {
            Expression::Identifier(ident) => {
                if ident.name.eq("fit") {
                    return Some((ident.span(), Cow::Borrowed("test.only")));
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
