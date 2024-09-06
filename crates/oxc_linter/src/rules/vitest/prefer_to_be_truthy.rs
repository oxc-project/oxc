use oxc_macros::declare_oxc_lint;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{collect_possible_jest_call_node, prefer_to_be_simply_bool},
};

#[derive(Debug, Default, Clone)]
pub struct PreferToBeTruthy;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule warns when `toBe(true)` is used with `expect` or `expectTypeOf`.
    /// With `--fix`, it will be replaced with `toBeTruthy()`.
    ///
    /// ### Why is this bad?
    ///
    /// Using `toBe(true)` is less flexible and may not account for other truthy
    /// values like non-empty strings or objects. `toBeTruthy()` checks for any
    /// truthy value, which makes the tests more comprehensive and robust.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// expect(foo).toBe(true)
    /// expectTypeOf(foo).toBe(true)
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// expect(foo).toBeTruthy()
    /// expectTypeOf(foo).toBeTruthy()
    /// ```
    PreferToBeTruthy,
    style,
    fix
);

impl Rule for PreferToBeTruthy {
    fn run_once(&self, ctx: &LintContext) {
        for possible_vitest_node in &collect_possible_jest_call_node(ctx) {
            prefer_to_be_simply_bool(possible_vitest_node, ctx, true);
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "[].push(true)",
        r#"expect("something");"#,
        "expect(true).toBeTrue();",
        "expect(false).toBeTrue();",
        "expect(fal,se).toBeFalse();",
        "expect(true).toBeFalse();",
        "expect(value).toEqual();",
        "expect(value).not.toBeTrue();",
        "expect(value).not.toEqual();",
        "expect(value).toBe(undefined);",
        "expect(value).not.toBe(undefined);",
        "expect(true).toBe(false)",
        "expect(value).toBe();",
        "expect(true).toMatchSnapshot();",
        r#"expect("a string").toMatchSnapshot(true);"#,
        r#"expect("a string").not.toMatchSnapshot();"#,
        "expect(something).toEqual('a string');",
        "expect(true).toBe",
        "expectTypeOf(true).toBe()",
    ];

    let fail = vec![
        "expect(false).toBe(true);",
        "expectTypeOf(false).toBe(true);",
        "expect(wasSuccessful).toEqual(true);",
        "expect(fs.existsSync('/path/to/file')).toStrictEqual(true);",
        r#"expect("a string").not.toBe(true);"#,
        r#"expect("a string").not.toEqual(true);"#,
        r#"expectTypeOf("a string").not.toStrictEqual(true);"#,
    ];

    let fix = vec![
        ("expect(false).toBe(true);", "expect(false).toBeTruthy();", None),
        ("expectTypeOf(false).toBe(true);", "expectTypeOf(false).toBeTruthy();", None),
        ("expect(wasSuccessful).toEqual(true);", "expect(wasSuccessful).toBeTruthy();", None),
        (
            "expect(fs.existsSync('/path/to/file')).toStrictEqual(true);",
            "expect(fs.existsSync('/path/to/file')).toBeTruthy();",
            None,
        ),
        (r#"expect("a string").not.toBe(true);"#, r#"expect("a string").not.toBeTruthy();"#, None),
        (
            r#"expect("a string").not.toEqual(true);"#,
            r#"expect("a string").not.toBeTruthy();"#,
            None,
        ),
        (
            r#"expectTypeOf("a string").not.toStrictEqual(true);"#,
            r#"expectTypeOf("a string").not.toBeTruthy();"#,
            None,
        ),
    ];
    Tester::new(PreferToBeTruthy::NAME, pass, fail).expect_fix(fix).test_and_snapshot();
}
