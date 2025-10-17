use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::Rule,
};

fn prefer_to_be_diagnostic(span: Span) -> OxcDiagnostic {
    // See <https://oxc.rs/docs/contribute/linter/adding-rules.html#diagnostics> for details
    OxcDiagnostic::warn("Should be an imperative statement about what is wrong")
        .with_help("Should be a command-like statement that tells the user how to fix the issue")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferToBe;

// See <https://github.com/oxc-project/oxc/issues/6050> for documentation details.
declare_oxc_lint!(
    /// ### What it does
    ///
    /// Briefly describe the rule's purpose.
    ///
    /// ### Why is this bad?
    ///
    /// Explain why violating this rule is problematic.
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
    PreferToBe,
    vitest,
    nursery, // TODO: change category to `correctness`, `suspicious`, `pedantic`, `perf`, `restriction`, or `style`
             // See <https://oxc.rs/docs/contribute/linter.html#rule-category> for details
    pending  // TODO: describe fix capabilities. Remove if no fix can be done,
             // keep at 'pending' if you think one could be added but don't know how.
             // Options are 'fix', 'fix_dangerous', 'suggestion', and 'conditional_fix_suggestion'
);

impl Rule for PreferToBe {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {}
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "expect(null).toBeNull();",
        "expect(null).not.toBeNull();",
        "expect(null).toBe(-1);",
        "expect(null).toBe(1);",
        "expect(obj).toStrictEqual([ x, 1 ]);",
        "expect(obj).toStrictEqual({ x: 1 });",
        "expect(obj).not.toStrictEqual({ x: 1 });",
        "expect(value).toMatchSnapshot();",
        "expect(catchError()).toStrictEqual({ message: 'oh noes!' })",
        r#"expect("something");"#,
        "expect(token).toStrictEqual(/[abc]+/g);",
        "expect(token).toStrictEqual(new RegExp('[abc]+', 'g'));",
        "expect(0.1 + 0.2).toEqual(0.3);",
        "expect(NaN).toBeNaN();",
        "expect(true).not.toBeNaN();",
        "expect({}).toEqual({});",
        "expect(something).toBe()",
        "expect(something).toBe(somethingElse)",
        "expect(something).toEqual(somethingElse)",
        "expect(something).not.toBe(somethingElse)",
        "expect(something).not.toEqual(somethingElse)",
        "expect(undefined).toBe",
        r#"expect("something");"#,
        "expect(null).toBeNull();",
        "expect(null).not.toBeNull();",
        "expect(null).toBe(1);",
        "expect(obj).toStrictEqual([ x, 1 ]);",
        "expect(obj).toStrictEqual({ x: 1 });",
        "expect(obj).not.toStrictEqual({ x: 1 });",
    ];

    let fail = vec![
        r#"expect(value).toEqual("my string");"#,
        r#"expect("a string").not.toEqual(null);"#,
        r#"expect("a string").not.toStrictEqual(null);"#,
        "expect(NaN).toBe(NaN);",
        r#"expect("a string").not.toBe(NaN);"#,
        r#"expect("a string").not.toStrictEqual(NaN);"#,
        "expect(null).toBe(null);",
        "expect(null).toEqual(null);", // { "parserOptions": { "ecmaVersion": 2017 } },
        r#"expect("a string").not.toEqual(null as number);"#,
        "expect(undefined).toBe(undefined as unknown as string as any);",
        r#"expect("a string").toEqual(undefined as number);"#,
    ];

    let fix = vec![
        (r#"expect(value).toEqual("my string");"#, r#"expect(value).toBe("my string");"#, None),
        (r#"expect("a string").not.toEqual(null);"#, r#"expect("a string").not.toBeNull();"#, None),
        (
            r#"expect("a string").not.toStrictEqual(null);"#,
            r#"expect("a string").not.toBeNull();"#,
            None,
        ),
        ("expect(NaN).toBe(NaN);", "expect(NaN).toBeNaN();", None),
        (r#"expect("a string").not.toBe(NaN);"#, r#"expect("a string").not.toBeNaN();"#, None),
        (
            r#"expect("a string").not.toStrictEqual(NaN);"#,
            r#"expect("a string").not.toBeNaN();"#,
            None,
        ),
        ("expect(null).toBe(null);", "expect(null).toBeNull();", None),
        ("expect(null).toEqual(null);", "expect(null).toBeNull();", None),
        (
            r#"expect("a string").not.toEqual(null as number);"#,
            r#"expect("a string").not.toBeNull();"#,
            None,
        ),
        (
            "expect(undefined).toBe(undefined as unknown as string as any);",
            "expect(undefined).toBeUndefined();",
            None,
        ),
        (
            r#"expect("a string").toEqual(undefined as number);"#,
            r#"expect("a string").toBeUndefined();"#,
            None,
        ),
    ];
    Tester::new(PreferToBe::NAME, PreferToBe::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
