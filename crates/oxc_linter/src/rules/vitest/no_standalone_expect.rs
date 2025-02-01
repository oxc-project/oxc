use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::Rule,
    AstNode,
};

fn no_standalone_expect_diagnostic(span: Span) -> OxcDiagnostic {
    // See <https://oxc.rs/docs/contribute/linter/adding-rules.html#diagnostics> for details
    OxcDiagnostic::warn("Should be an imperative statement about what is wrong")
        .with_help("Should be a command-like statement that tells the user how to fix the issue")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoStandaloneExpect;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prevents `expect` statements outside of a `test` or `it` block. An `expect`
    /// within a helper function (but outside of a `test` or `it` block) will not
    /// trigger this rule.
    ///
    /// Statements like `expect.hasAssertions()` will NOT trigger this rule since these
    /// calls will execute if they are not in a test block.
    ///
    /// ### Example
    /// ```javascript
    /// describe('a test', () => {
    ///     expect(1).toBe(1);
    /// });
    /// ```
    NoStandaloneExpect,
    vitest,
    correctness,
);

impl Rule for NoStandaloneExpect {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {}
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("beforeEach(() => { doSomething(); });", None),
        ("expect.any(String)", None),
        ("expect.extend({})", None),
        (r#"bench("a bench", () => {})"#, None),
        (r#"describe("a test", () => { it("an it", () => {expect(1).toBe(1); }); });"#, None),
        (
            r#"describe("a test", () => { it("an it", () => { const func = () => { expect(1).toBe(1); }; }); });"#,
            None,
        ),
        (r#"describe("a test", () => { const func = () => { expect(1).toBe(1); }; });"#, None),
        (r#"describe("a test", () => { function func() { expect(1).toBe(1); }; });"#, None),
        (r#"describe("a test", () => { const func = function(){ expect(1).toBe(1); }; });"#, None),
        (
            r#"describe.only.concurrent.todo("a test", () => { const func = function(){ expect(1).toBe(1); }; });"#,
            None,
        ),
        (r#"it("an it", () => expect(1).toBe(1))"#, None),
        (r#"it.only("an it", () => expect(1).toBe(1))"#, None),
        (r#"it.concurrent("an it", () => expect(1).toBe(1))"#, None),
        (r#"it.extend.skip("an it", ()  => expect(1).toBe(1))"#, None),
        (r#"test("a test", () => expect(1).toBe(1))"#, None),
        (r#"test.skip("a skipped test", () => expect(1).toBe(1))"#, None),
        (r#"test.fails("a failing test", () => expect(1).toBe(1))"#, None),
        ("const func = function(){ expect(1).toBe(1); };", None),
        ("const func = () => expect(1).toBe(1);", None),
        ("{}", None),
        (r#"it.each([1, true])("trues", value => { expect(value).toBe(true); });"#, None),
        (
            r#"it.each([1, true])("trues", value => { expect(value).toBe(true); }); it("an it", () => { expect(1).toBe(1) });"#,
            None,
        ),
    ];

    let fail = vec![
        ("(() => {})('testing', () => expect(true).toBe(false))", None),
        ("expect.hasAssertions()", None),
        (
            "
			       describe('scenario', () => {
			      const t = Math.random() ? it.only : it;
			      t('testing', () => expect(true).toBe(false));
			       });
			     ",
            None,
        ),
        (
            "describe('scenario', () => {
			      const t = Math.random() ? it.only : it;
			      t('testing', () => expect(true).toBe(false));
			       });",
            None,
        ),
        (r#"describe("a test", () => { expect(1).toBe(1); });"#, None),
        (r#"describe("a test", () => expect(1).toBe(1));"#, None),
        (
            r#"describe("a test", () => { const func = () => { expect(1).toBe(1); }; expect(1).toBe(1); });"#,
            None,
        ),
        (
            r#"describe("a test", () => {  it(() => { expect(1).toBe(1); }); expect(1).toBe(1); });"#,
            None,
        ),
        ("expect(1).toBe(1);", None),
        ("{expect(1).toBe(1)}", None),
        (
            "
			     each([
			       [1, 1, 2],
			       [1, 2, 3],
			       [2, 1, 3],
			     ]).test('returns the result of adding %d to %d', (a, b, expected) => {
			       expect(a + b).toBe(expected);
			     });",
            Some(serde_json::json!([{ "additionalTestBlockFunctions": ["test"] }])),
        ),
    ];

    Tester::new(NoStandaloneExpect::NAME, NoStandaloneExpect::PLUGIN, pass, fail)
        .with_vitest_plugin(true)
        .with_snapshot_suffix("vitest")
        .test_and_snapshot();
}
