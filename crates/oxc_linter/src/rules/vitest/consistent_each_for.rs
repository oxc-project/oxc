use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, Span};
use rustc_hash::FxHashMap;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
    utils::{JestFnKind, JestGeneralFnKind, PossibleJestNode, parse_general_jest_fn_call},
};

fn consistent_each_for_diagnostic(
    span: Span,
    fn_kind: &str,
    method_used: &str,
    method_name: &CompactStr,
) -> OxcDiagnostic {
    let message =
        format!("`{fn_kind}` can not be used with `.{method_used}` to create parameterized test.");
    let help = format!(
        "To create parameterized test with `{fn_kind}` function you should use `{method_name}`"
    );

    OxcDiagnostic::warn(message).with_help(help).with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct ConsistentEachFor(Box<ConsistentEachForConfig>);

impl std::ops::Deref for ConsistentEachFor {
    type Target = ConsistentEachForConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, JsonSchema, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MemberNames {
    For,
    Each,
}

impl MemberNames {
    fn not_allowed_method(&self, method: &str) -> bool {
        match self {
            MemberNames::For => "each" == method,
            MemberNames::Each => "for" == method,
        }
    }

    fn allowed_method_from_disallowed_method(&self) -> CompactStr {
        match self {
            MemberNames::For => ".for".into(),
            MemberNames::Each => ".each".into(),
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
enum MatchKind {
    Describe,
    It,
    Test,
    Suite,
}

impl MatchKind {
    fn from(name: &str) -> Option<Self> {
        match name {
            "describe" => Some(Self::Describe),
            "it" => Some(Self::It),
            "test" => Some(Self::Test),
            "suite" => Some(Self::Suite),
            _ => None,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct ConsistentEachForConfig {
    methods: FxHashMap<MatchKind, MemberNames>,
}

#[derive(Debug, Default, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct ConsistentEachForJson {
    describe: Option<MemberNames>,
    suite: Option<MemberNames>,
    test: Option<MemberNames>,
    it: Option<MemberNames>,
}

impl ConsistentEachForJson {
    fn into_consistent_each_for_config(self) -> ConsistentEachForConfig {
        let mut members = FxHashMap::default();

        if let Some(describe) = self.describe {
            members.insert(MatchKind::Describe, describe);
        }

        if let Some(it) = self.it {
            members.insert(MatchKind::It, it);
        }

        if let Some(suite) = self.suite {
            members.insert(MatchKind::Suite, suite);
        }

        if let Some(test) = self.test {
            members.insert(MatchKind::Test, test);
        }

        ConsistentEachForConfig { methods: members }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule ensure consistency on which method used to create parameterized test.
    /// This configuration affects to different test function types (`test`, `it`, `describe`, `suite`).
    ///
    /// ### Why is this bad?
    ///
    /// Not having a consistent way to create parametrized tests, we rely on the developer to remember that
    /// `.for` spread the values as different arguments and `.each` pass the array as an unique argument.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// // { test: 'for' }
    /// test.each([[1, 1, 2]])('test', (a, b, expected) => {
    ///   expect(a + b).toBe(expected)
    /// })
    ///
    /// // { describe: 'for' }
    /// describe.each([[1], [2]])('suite %s', (n) => {
    ///   test('test', () => {})
    /// })
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// // { test: 'for' }
    /// test.for([[1, 1, 2]])('test', ([a, b, expected]) => {
    ///   expect(a + b).toBe(expected)
    /// })
    ///
    /// // { describe: 'for' }
    /// describe.for([[1], [2]])('suite %s', ([n]) => {
    ///   test('test', () => {})
    /// })
    /// ```
    ConsistentEachFor,
    vitest,
    correctness,
    config = ConsistentEachForJson
);

impl Rule for ConsistentEachFor {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        Ok(Self(Box::new(
            serde_json::from_value::<DefaultRuleConfig<ConsistentEachForJson>>(value)
                .unwrap_or_default()
                .into_inner()
                .into_consistent_each_for_config(),
        )))
    }

    fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        self.run(jest_node, ctx);
    }
}

impl ConsistentEachFor {
    fn run<'a>(&self, possible_jest_node: &PossibleJestNode<'a, '_>, ctx: &LintContext<'a>) {
        let node = possible_jest_node.node;

        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        let Some(jest_fn_call) = parse_general_jest_fn_call(call_expr, possible_jest_node, ctx)
        else {
            return;
        };

        if !matches!(
            jest_fn_call.kind,
            JestFnKind::General(JestGeneralFnKind::Describe | JestGeneralFnKind::Test)
        ) {
            return;
        }

        let Some(fn_kind) = MatchKind::from(jest_fn_call.name.as_ref()) else {
            return;
        };

        let Some(member_to_check) = self.methods.get(&fn_kind) else {
            return;
        };

        let Some(last_method) = jest_fn_call.members.last() else {
            return;
        };

        let Some(method_name) = last_method.name() else {
            return;
        };

        if member_to_check.not_allowed_method(method_name.as_ref()) {
            ctx.diagnostic(consistent_each_for_diagnostic(
                last_method.span,
                jest_fn_call.name.as_ref(),
                method_name.as_ref(),
                &member_to_check.allowed_method_from_disallowed_method(),
            ));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    /*
     * Currently the responsible to set what frameworks are active or not is not `with_vitest_plugin` or oxlint config.
     * The code that set what test framewors are active is ContextHost::sniff_for_frameworks, and the current detection lead to a
     * a false negative. To detect if the current source code belongs to vitest is based if a `vitest` import exist, if not, assumes
     * we are on a possible jest test. On top of that, the method `frameworks::is_jestlike_file` most of the times is going to be true, at least in
     * our current situation. So this lead that the ContextHost can have jest and vitest active **at same time**.
     *
     * This detection isn't compatible on how `parse_general_jest_fn_call` handle if a node is valid or not. To make it simple:
     *
     * - Jest file: ctx.frameworks().is_jest() is true && ctx.frameworks().is_vitest() is false
     * - Vitest file: ctx.frameworks().is_jest() is true && ctx.frameworks().is_vitest is true
     *
     * And if you are dealing with non compatible modifiers or methods, that only exists in vitest, it will fail as in jest doesn't exist.
     *
     * In case of dealing with syntax that only exists in vitest, add an import of `vitest` to force the ContextHost to detect we are dealing with vitest.
     * This probably will allow reuse allow of the methods that rely on this false negative detection.
     */
    macro_rules! vitest_context {
        ($test: literal) => {
            concat!("import * as vi from 'vitest'\n\n", $test)
        };
    }

    let pass = vec![
        (vitest_context!("test.each([1, 2, 3])('test', (n) => { expect(n).toBeDefined() })"), None),
        (
            vitest_context!("test.for([1, 2, 3])('test', ([n]) => { expect(n).toBeDefined() })"),
            None,
        ),
        (
            vitest_context!("describe.each([1, 2, 3])('suite', (n) => { test('test', () => {}) })"),
            None,
        ),
        (
            vitest_context!("test.each([1, 2, 3])('test', (n) => { expect(n).toBeDefined() })"),
            Some(serde_json::json!([{ "test": "each" }])),
        ),
        (
            vitest_context!(
                "test.skip.each([1, 2, 3])('test', (n) => { expect(n).toBeDefined() })"
            ),
            Some(serde_json::json!([{ "test": "each" }])),
        ),
        (
            vitest_context!(
                "test.only.each([1, 2, 3])('test', (n) => { expect(n).toBeDefined() })"
            ),
            Some(serde_json::json!([{ "test": "each" }])),
        ),
        (
            vitest_context!(
                "test.concurrent.each([1, 2, 3])('test', (n) => { expect(n).toBeDefined() })"
            ),
            Some(serde_json::json!([{ "test": "each" }])),
        ),
        (
            vitest_context!("test.for([1, 2, 3])('test', ([n]) => { expect(n).toBeDefined() })"),
            Some(serde_json::json!([{ "test": "for" }])),
        ),
        (
            vitest_context!(
                "test.skip.for([1, 2, 3])('test', ([n]) => { expect(n).toBeDefined() })"
            ),
            Some(serde_json::json!([{ "test": "for" }])),
        ),
        (
            vitest_context!(
                "test.only.for([1, 2, 3])('test', ([n]) => { expect(n).toBeDefined() })"
            ),
            Some(serde_json::json!([{ "test": "for" }])),
        ),
        (
            vitest_context!("it.each([1, 2, 3])('test', (n) => { expect(n).toBeDefined() })"),
            Some(serde_json::json!([{ "it": "each" }])),
        ),
        (
            vitest_context!("it.for([1, 2, 3])('test', ([n]) => { expect(n).toBeDefined() })"),
            Some(serde_json::json!([{ "it": "for" }])),
        ),
        (
            vitest_context!("describe.each([1, 2, 3])('suite', (n) => { test('test', () => {}) })"),
            Some(serde_json::json!([{ "describe": "each" }])),
        ),
        (
            vitest_context!(
                "describe.skip.each([1, 2, 3])('suite', (n) => { test('test', () => {}) })"
            ),
            Some(serde_json::json!([{ "describe": "each" }])),
        ),
        (
            vitest_context!(
                "describe.for([1, 2, 3])('suite', ([n]) => { test('test', () => {}) })"
            ),
            Some(serde_json::json!([{ "describe": "for" }])),
        ),
        (
            vitest_context!("suite.each([1, 2, 3])('suite', (n) => { test('test', () => {}) })"),
            Some(serde_json::json!([{ "suite": "each" }])),
        ),
        (
            vitest_context!("suite.for([1, 2, 3])('suite', ([n]) => { test('test', () => {}) })"),
            Some(serde_json::json!([{ "suite": "for" }])),
        ),
        (
            vitest_context!(
                "
			        test.each([1, 2, 3])('test', (n) => { expect(n).toBeDefined() })
			        describe.for([1, 2, 3])('suite', ([n]) => { test('test', () => {}) })
			      "
            ),
            Some(serde_json::json!([{ "test": "each", "describe": "for" }])),
        ),
    ];

    let fail = vec![
        (
            vitest_context!("test.for([1, 2, 3])('test', ([n]) => { expect(n).toBeDefined() })"),
            Some(serde_json::json!([{ "test": "each" }])),
        ),
        (
            vitest_context!(
                "test.skip.for([1, 2, 3])('test', ([n]) => { expect(n).toBeDefined() })"
            ),
            Some(serde_json::json!([{ "test": "each" }])),
        ),
        (
            vitest_context!(
                "test.only.for([1, 2, 3])('test', ([n]) => { expect(n).toBeDefined() })"
            ),
            Some(serde_json::json!([{ "test": "each" }])),
        ),
        (
            vitest_context!("test.each([1, 2, 3])('test', (n) => { expect(n).toBeDefined() })"),
            Some(serde_json::json!([{ "test": "for" }])),
        ),
        (
            vitest_context!(
                "test.skip.each([1, 2, 3])('test', (n) => { expect(n).toBeDefined() })"
            ),
            Some(serde_json::json!([{ "test": "for" }])),
        ),
        (
            vitest_context!("it.for([1, 2, 3])('test', ([n]) => { expect(n).toBeDefined() })"),
            Some(serde_json::json!([{ "it": "each" }])),
        ),
        (
            vitest_context!("it.each([1, 2, 3])('test', (n) => { expect(n).toBeDefined() })"),
            Some(serde_json::json!([{ "it": "for" }])),
        ),
        (
            vitest_context!(
                "describe.for([1, 2, 3])('suite', ([n]) => { test('test', () => {}) })"
            ),
            Some(serde_json::json!([{ "describe": "each" }])),
        ),
        (
            vitest_context!("describe.each([1, 2, 3])('suite', (n) => { test('test', () => {}) })"),
            Some(serde_json::json!([{ "describe": "for" }])),
        ),
        (
            vitest_context!("suite.for([1, 2, 3])('suite', ([n]) => { test('test', () => {}) })"),
            Some(serde_json::json!([{ "suite": "each" }])),
        ),
        (
            vitest_context!("suite.each([1, 2, 3])('suite', (n) => { test('test', () => {}) })"),
            Some(serde_json::json!([{ "suite": "for" }])),
        ),
        (
            vitest_context!(
                "
			        test.for([1, 2])('test1', ([n]) => {})
			        test.for([3, 4])('test2', ([n]) => {})
			      "
            ),
            Some(serde_json::json!([{ "test": "each" }])),
        ),
    ];

    Tester::new(ConsistentEachFor::NAME, ConsistentEachFor::PLUGIN, pass, fail)
        .with_vitest_plugin(true)
        .test_and_snapshot();
}
