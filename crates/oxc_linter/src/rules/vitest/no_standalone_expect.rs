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
    ///
    /// ### Why is this bad?
    ///
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// FIXME: Tests will fail if examples are missing or syntactically incorrect.
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// FIXME: Tests will fail if examples are missing or syntactically incorrect.
    /// ```
    NoStandaloneExpect,
    vitest,
    nursery, // TODO: change category to `correctness`, `suspicious`, `pedantic`, `perf`, `restriction`, or `style`
             // See <https://oxc.rs/docs/contribute/linter.html#rule-category> for details

    pending  // TODO: describe fix capabilities. Remove if no fix can be done,
             // keep at 'pending' if you think one could be added but don't know how.
             // Options are 'fix', 'fix_dangerous', 'suggestion', and 'conditional_fix_suggestion'
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
        .test_and_snapshot();
}
