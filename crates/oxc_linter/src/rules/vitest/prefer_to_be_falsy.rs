use oxc_macros::declare_oxc_lint;

use crate::{context::LintContext, rule::Rule};

use super::prefer_to_be_truthy::prefer_to_be_simply_bool;

#[derive(Debug, Default, Clone)]
pub struct PreferToBeFalsy;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule warns when `toBe(false)` is used with `expect` or `expectTypeOf`.
    /// With `--fix`, it will be replaced with `toBeFalsy()`.
    ///
    /// ### Why is this bad?
    ///
    /// Using `toBe(false)` is less expressive and may not account for other falsy
    /// values like `0`, `null`, or `undefined`. `toBeFalsy()` provides a more
    /// comprehensive check for any falsy value, improving the robustness of the tests.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// expect(foo).toBe(false)
    /// expectTypeOf(foo).toBe(false)
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// expect(foo).toBeFalsy()
    /// expectTypeOf(foo).toBeFalsy()
    /// ```
    PreferToBeFalsy,
    style,
    fix
);

impl Rule for PreferToBeFalsy {
    fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &crate::utils::PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        prefer_to_be_simply_bool(jest_node, ctx, false);
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
    Tester::new(PreferToBeFalsy::NAME, pass, fail)
        .expect_fix(fix)
        .with_vitest_plugin(true)
        .test_and_snapshot();
}
