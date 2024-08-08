use oxc_macros::declare_oxc_lint;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{collect_possible_jest_call_node, prefer_to_be_simply_bool},
};

#[derive(Debug, Default, Clone)]
pub struct PreferToBeFalsy;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule warns when `toBe(false)` is used with `expect` or `expectTypeOf`. With `--fix`, it will be replaced with `toBeFalsy()`.
    ///
    /// ### Examples
    ///
    /// ```javascript
    /// // bad
    /// expect(foo).toBe(false)
    /// expectTypeOf(foo).toBe(false)
    ///
    /// // good
    /// expect(foo).toBeFalsy()
    /// expectTypeOf(foo).toBeFalsy()
    /// ```
    PreferToBeFalsy,
    style,
    fix
);

impl Rule for PreferToBeFalsy {
    fn run_once(&self, ctx: &LintContext) {
        for possible_vitest_node in &collect_possible_jest_call_node(ctx) {
            prefer_to_be_simply_bool(possible_vitest_node, ctx, false);
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "[].push(false)",
        r#"expect("something");"#,
        "expect(true).toBeTrue();",
        "expect(false).toBeTrue();",
        "expect(false).toBeFalsy();",
        "expect(true).toBeFalsy();",
        "expect(value).toEqual();",
        "expect(value).not.toBeFalsy();",
        "expect(value).not.toEqual();",
        "expect(value).toBe(undefined);",
        "expect(value).not.toBe(undefined);",
        "expect(false).toBe(true)",
        "expect(value).toBe();",
        "expect(true).toMatchSnapshot();",
        r#"expect("a string").toMatchSnapshot(false);"#,
        r#"expect("a string").not.toMatchSnapshot();"#,
        "expect(something).toEqual('a string');",
        "expect(false).toBe",
        "expectTypeOf(false).toBe",
    ];

    let fail = vec![
        "expect(true).toBe(false);",
        "expect(wasSuccessful).toEqual(false);",
        "expect(fs.existsSync('/path/to/file')).toStrictEqual(false);",
        r#"expect("a string").not.toBe(false);"#,
        r#"expect("a string").not.toEqual(false);"#,
        r#"expectTypeOf("a string").not.toEqual(false);"#,
    ];

    let fix = vec![
        ("expect(true).toBe(false);", "expect(true).toBeFalsy();", None),
        ("expect(wasSuccessful).toEqual(false);", "expect(wasSuccessful).toBeFalsy();", None),
        (
            "expect(fs.existsSync('/path/to/file')).toStrictEqual(false);",
            "expect(fs.existsSync('/path/to/file')).toBeFalsy();",
            None,
        ),
        (r#"expect("a string").not.toBe(false);"#, r#"expect("a string").not.toBeFalsy();"#, None),
        (
            r#"expect("a string").not.toEqual(false);"#,
            r#"expect("a string").not.toBeFalsy();"#,
            None,
        ),
        (
            r#"expectTypeOf("a string").not.toEqual(false);"#,
            r#"expectTypeOf("a string").not.toBeFalsy();"#,
            None,
        ),
    ];
    Tester::new(PreferToBeFalsy::NAME, pass, fail).expect_fix(fix).test_and_snapshot();
}
