use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, Span};

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
pub struct PreferLowercaseTitleConfig {
    allowed_prefixes: Vec<CompactStr>,
    ignore: Vec<CompactStr>,
    ignore_top_level_describe: bool,
    lowercase_first_character_only: bool,
}

impl std::ops::Deref for PreferLowercaseTitle {
    type Target = PreferLowercaseTitleConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Default, Clone)]
pub struct PreferLowercaseTitle(Box<PreferLowercaseTitleConfig>);

declare_oxc_lint!(
    /// ### What it does
    /// 
    /// Enforce `it`, `test`, and `describe` to have descriptions that begin with a
    /// lowercase letter. 
    ///
    /// ### Why is this bad?
    /// 
    /// Capitalized `it`, `test`, and `describe` descriptions may result in less 
    /// readable test failures.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// test('It works', () => {
	///     ...
    /// })
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// test('it works', () => {
	///     ...
    /// })
    /// ```
    PreferLowercaseTitle,
    style,
    pending  // TODO: describe fix capabilities. Remove if no fix can be done,
             // keep at 'pending' if you think one could be added but don't know how.
             // Options are 'fix', 'fix_dangerous', 'suggestion', and 'conditional_fix_suggestion'
);

impl Rule for PreferLowercaseTitle {
    fn from_configuration(value: serde_json::Value) -> Self {
        let obj = value.get(0);
        let ignore_top_level_describe = obj
            .and_then(|config| config.get("ignoreTopLevelDescribe"))
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        let lowercase_first_character_only = obj
            .and_then(|config| config.get("lowercaseFirstCharacterOnly"))
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        let ignore = obj
            .and_then(|config| config.get("ignore"))
            .and_then(serde_json::Value::as_array)
            .map(|v| v.iter().filter_map(serde_json::Value::as_str).map(CompactStr::from).collect())
            .unwrap_or_default();
        let allowed_prefixes = obj
            .and_then(|config| config.get("allowedPrefixes"))
            .and_then(serde_json::Value::as_array)
            .map(|v| v.iter().filter_map(serde_json::Value::as_str).map(CompactStr::from).collect())
            .unwrap_or_default();

        Self(Box::new(PreferLowercaseTitleConfig {
            allowed_prefixes,
            ignore,
            ignore_top_level_describe,
            lowercase_first_character_only
        }))
    }

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
