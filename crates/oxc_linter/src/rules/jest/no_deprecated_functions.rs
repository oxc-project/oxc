use oxc_ast::AstKind;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use phf::{phf_map, Map};
use std::borrow::Cow;

use crate::{context::LintContext, fixer::Fix, rule::Rule, utils::get_node_name_vec};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-jest(no-deprecated-functions): Disallow use of deprecated functions")]
#[diagnostic(severity(warning), help("{{0:?}}` has been deprecated in favor of `{1:?}"))]
pub struct DeprecatedFunction(pub String, pub String, #[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct JestConfig {
    pub version: String,
}

#[derive(Debug, Default, Clone)]
pub struct NoDeprecatedFunctions {
    pub jest: JestConfig,
}

declare_oxc_lint!(
    /// ### What it does
    /// Over the years Jest has accrued some debt in the form of functions that have
    /// either been renamed for clarity, or replaced with more powerful APIs.
    ///
    /// This rule can also autofix a number of these deprecations for you.
    /// #### `jest.resetModuleRegistry`
    /// This function was renamed to `resetModules` in Jest 15 and removed in Jest 27.
    ///
    /// #### `jest.addMatchers`
    /// This function was replaced with `expect.extend` in Jest 17 and removed in Jest 27.
    ///
    /// #### `require.requireActual` & `require.requireMock`
    /// These functions were replaced in Jest 21 and removed in Jest 26.
    ///
    /// Originally, the `requireActual` & `requireMock` the `requireActual`&
    /// `requireMock` functions were placed onto the `require` function.
    ///
    /// These functions were later moved onto the `jest` object in order to be easier
    /// for type checkers to handle, and their use via `require` deprecated. Finally,
    /// the release of Jest 26 saw them removed from the `require` function altogether.
    ///
    /// #### `jest.runTimersToTime`
    /// This function was renamed to `advanceTimersByTime` in Jest 22 and removed in Jest 27.
    ///
    /// #### `jest.genMockFromModule`
    /// This function was renamed to `createMockFromModule` in Jest 26, and is scheduled for removal in Jest 30.
    ///
    /// ### Why is this bad?
    ///
    /// While typically these deprecated functions are kept in the codebase for a number
    /// of majors, eventually they are removed completely.
    ///
    /// ### Example
    /// ```javascript
    /// jest.resetModuleRegistry // since Jest 15
    /// jest.addMatchers // since Jest 17
    /// ```
    NoDeprecatedFunctions,
    style,
);

const DEPRECATED_FUNCTIONS_MAP: Map<&'static str, (usize, &'static str)> = phf_map! {
    "jest.resetModuleRegistry" => (15, "jest.resetModules"),
    "jest.addMatchers" => (17, "expect.extend"),
    "require.requireMock" => (21, "jest.requireMock"),
    "require.requireActual" => (21, "jest.requireMock"),
    "jest.runTimersToTime" => (22, "jest.advanceTimersByTime"),
    "jest.genMockFromModule" => (26, "jest.createMockFromModule"),
};

impl Rule for NoDeprecatedFunctions {
    fn from_configuration(value: serde_json::Value) -> Self {
        let version = value
            .get(0)
            .and_then(|v| v.get("jest"))
            .and_then(|v| v.get("version"))
            .and_then(|v| serde_json::Value::as_str(v))
            .unwrap();

        Self { jest: JestConfig { version: version.to_string() } }
    }

    fn run<'a>(&self, node: &oxc_semantic::AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::MemberExpression(mem_expr) = node.kind() else {
            return;
        };
        let mut chain: Vec<Cow<'a, str>> = Vec::new();
        chain.extend(get_node_name_vec(mem_expr.object()));

        if let Some(name) = mem_expr.static_property_name() {
            chain.push(Cow::Borrowed(name));
        }

        let node_name = chain.join(".");
        let major: Vec<&str> = self.jest.version.split('.').collect();
        let jest_version_num: usize = major[0].parse().unwrap();

        if let Some((base_version, replacement)) = DEPRECATED_FUNCTIONS_MAP.get(&node_name) {
            if jest_version_num >= *base_version {
                ctx.diagnostic_with_fix(
                    DeprecatedFunction(node_name, (*replacement).to_string(), mem_expr.span()),
                    || Fix::new(*replacement, mem_expr.span()),
                );
            }
        }
    }
}

#[test]
fn tests() {
    use crate::tester::Tester;

    let pass = vec![
        ("jest", Some(serde_json::json!([{ "jest": { "version": "14" } }]))),
        ("require('fs')", Some(serde_json::json!([{ "jest": { "version": "14" } }]))),
        ("jest.resetModuleRegistry", Some(serde_json::json!([{ "jest": { "version": "14" } }]))),
        ("require.requireActual", Some(serde_json::json!([{ "jest": { "version": "17" } }]))),
        ("jest.genMockFromModule", Some(serde_json::json!([{ "jest": { "version": "25" } }]))),
        ("jest.genMockFromModule", Some(serde_json::json!([{ "jest": { "version": "25.1.1" } }]))),
        ("require.requireActual", Some(serde_json::json!([{ "jest": { "version": "17.2" } }]))),
    ];

    let fail = vec![
        // replace with `jest.resetModules` in Jest 15
        ("jest.resetModuleRegistry", Some(serde_json::json!([{ "jest": { "version": "16" }}]))),
        // replace with `jest.requireMock` in Jest 17.
        ("jest.addMatchers", Some(serde_json::json!([{ "jest": { "version": "18" }}]))),
        // replace with `jest.requireMock` in Jest 21.
        ("require.requireMock", Some(serde_json::json!([{ "jest": { "version": "22" }}]))),
        // replace with `jest.requireActual` in Jest 21.
        ("require.requireActual", Some(serde_json::json!([{ "jest": { "version": "22" }}]))),
        // replace with `jest.advanceTimersByTime` in Jest 22
        ("jest.runTimersToTime", Some(serde_json::json!([{ "jest": { "version": "23" }}]))),
        // replace with `jest.createMockFromModule` in Jest 26
        ("jest.genMockFromModule", Some(serde_json::json!([{ "jest": { "version": "27" }}]))),
    ];

    let fix = vec![
        (
            "jest.resetModuleRegistry()",
            "jest.resetModules()",
            Some(serde_json::json!([{ "jest": { "version": "21" } }])),
        ),
        (
            "jest.addMatchers",
            "expect.extend",
            Some(serde_json::json!([{ "jest": { "version": "24" } }])),
        ),
        (
            "jest.genMockFromModule",
            "jest.createMockFromModule",
            Some(serde_json::json!([{ "jest": { "version": "26" } }])),
        ),
        (
            "jest.genMockFromModule",
            "jest.createMockFromModule",
            Some(serde_json::json!([{ "jest": { "version": "26.0.0-next.11" } }])),
        ),
    ];

    Tester::new(NoDeprecatedFunctions::NAME, pass, fail)
        .with_jest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
