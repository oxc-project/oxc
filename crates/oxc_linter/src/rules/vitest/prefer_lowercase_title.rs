use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::Rule,
    AstNode,
};

fn prefer_lowercase_title_diagnostic(span: Span) -> OxcDiagnostic {
    // See <https://oxc.rs/docs/contribute/linter/adding-rules.html#diagnostics> for details
    OxcDiagnostic::warn("Should be an imperative statement about what is wrong")
        .with_help("Should be a command-like statement that tells the user how to fix the issue")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferLowercaseTitle;

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
    PreferLowercaseTitle,
    nursery, // TODO: change category to `correctness`, `suspicious`, `pedantic`, `perf`, `restriction`, or `style`
             // See <https://oxc.rs/docs/contribute/linter.html#rule-category> for details

    pending  // TODO: describe fix capabilities. Remove if no fix can be done,
             // keep at 'pending' if you think one could be added but don't know how.
             // Options are 'fix', 'fix_dangerous', 'suggestion', and 'conditional_fix_suggestion'
);

impl Rule for PreferLowercaseTitle {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {}
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("it.each()", None),
        ("it.each()(1)", None),
        ("it.todo();", None),
        (r#"describe("oo", function () {})"#, None),
        (r#"test("foo", function () {})"#, None),
        ("test(`123`, function () {})", None),
    ];

    let fail = vec![
        (r#"it("Foo MM mm", function () {})"#, None),
        ("test(`Foo MM mm`, function () {})", None),
        (
            "test(`SFC Compile`, function () {})",
            Some(
                serde_json::json!([        {          "lowercaseFirstCharacterOnly": false        }      ]),
            ),
        ),
        ("bench(`Foo MM mm`, function () {})", None),
    ];

    let fix = vec![
        (r#"it("Foo MM mm", function () {})"#, r#"it("foo MM mm", function () {})"#, None),
        ("test(`Foo MM mm`, function () {})", "test(`foo MM mm`, function () {})", None),
        (
            "test(`SFC Compile`, function () {})",
            "test(`sfc compile`, function () {})",
            Some(
                serde_json::json!([        {          "lowercaseFirstCharacterOnly": false        }      ]),
            ),
        ),
        ("bench(`Foo MM mm`, function () {})", "bench(`foo MM mm`, function () {})", None),
    ];
    Tester::new(PreferLowercaseTitle::NAME, PreferLowercaseTitle::CATEGORY, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
