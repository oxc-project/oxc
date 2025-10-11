use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    AstNode,
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::Rule,
};

fn no_process_env_diagnostic(span: Span) -> OxcDiagnostic {
    // See <https://oxc.rs/docs/contribute/linter/adding-rules.html#diagnostics> for details
    OxcDiagnostic::warn("Should be an imperative statement about what is wrong")
        .with_help("Should be a command-like statement that tells the user how to fix the issue")
        .with_label(span)
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename_all = "camelCase")]
struct ConfigElement0 {
    allowed_variables: Vec<String>,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
pub struct NoProcessEnv(ConfigElement0);

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
    NoProcessEnv,
    node,
    nursery, // TODO: change category to `correctness`, `suspicious`, `pedantic`, `perf`, `restriction`, or `style`
             // See <https://oxc.rs/docs/contribute/linter.html#rule-category> for details
    pending  // TODO: describe fix capabilities. Remove if no fix can be done,
             // keep at 'pending' if you think one could be added but don't know how.
             // Options are 'fix', 'fix_dangerous', 'suggestion', and 'conditional_fix_suggestion'
    config = NoProcessEnv,
);

impl Rule for NoProcessEnv {
    fn from_configuration(value: serde_json::Value) -> Self {
        serde_json::from_value(value).unwrap()
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {}
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("Process.env", None),
        ("process[env]", None),
        ("process.nextTick", None),
        ("process.execArgv", None),
        ("process.env.NODE_ENV", Some(serde_json::json!([{ "allowedVariables": ["NODE_ENV"] }]))),
        (
            "process.env['NODE_ENV']",
            Some(serde_json::json!([{ "allowedVariables": ["NODE_ENV"] }])),
        ),
        (
            "process['env'].NODE_ENV",
            Some(serde_json::json!([{ "allowedVariables": ["NODE_ENV"] }])),
        ),
        (
            "process['env']['NODE_ENV']",
            Some(serde_json::json!([{ "allowedVariables": ["NODE_ENV"] }])),
        ),
    ];

    let fail = vec![
        ("process.env", None),
        ("process['env']", None),
        ("process.env.ENV", None),
        ("f(process.env)", None),
        (
            "process.env['OTHER_VARIABLE']",
            Some(serde_json::json!([{ "allowedVariables": ["NODE_ENV"] }])),
        ),
        (
            "process.env.OTHER_VARIABLE",
            Some(serde_json::json!([{ "allowedVariables": ["NODE_ENV"] }])),
        ),
        (
            "process['env']['OTHER_VARIABLE']",
            Some(serde_json::json!([{ "allowedVariables": ["NODE_ENV"] }])),
        ),
        (
            "process['env'].OTHER_VARIABLE",
            Some(serde_json::json!([{ "allowedVariables": ["NODE_ENV"] }])),
        ),
        ("process.env[NODE_ENV]", Some(serde_json::json!([{ "allowedVariables": ["NODE_ENV"] }]))),
        (
            "process['env'][NODE_ENV]",
            Some(serde_json::json!([{ "allowedVariables": ["NODE_ENV"] }])),
        ),
    ];

    Tester::new(NoProcessEnv::NAME, NoProcessEnv::PLUGIN, pass, fail).test_and_snapshot();
}
