use std::ffi::OsStr;

use lazy_regex::{Regex, RegexBuilder};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

fn consistent_test_filename_diagnostic(file_path: &str, pattern: &str) -> OxcDiagnostic {
    let message = format!(
        "The file {file_path} is a test file, but its name does not match the expected pattern."
    );
    let help = format!("Rename the file that match the pattern {pattern}");

    OxcDiagnostic::warn(message).with_help(help)
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct ConsistentTestFilename(Box<ConsistentTestFilenameConfig>);

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct CompiledAllTestPattern(
    #[serde(deserialize_with = "deserialize_matcher_pattern")] lazy_regex::Regex,
);

impl Default for CompiledAllTestPattern {
    fn default() -> Self {
        Self(
            Regex::new(r".*\.(test|spec)\.[tj]sx?$")
                .expect("default all-test pattern should be valid"),
        )
    }
}

impl std::ops::Deref for CompiledAllTestPattern {
    type Target = lazy_regex::Regex;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct CompiledTestPatternName(
    #[serde(deserialize_with = "deserialize_matcher_pattern")] lazy_regex::Regex,
);

impl Default for CompiledTestPatternName {
    fn default() -> Self {
        Self(
            Regex::new(r".*\.test\.[tj]sx?$")
                .expect("default test filename pattern should be valid"),
        )
    }
}

impl std::ops::Deref for CompiledTestPatternName {
    type Target = lazy_regex::Regex;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Default, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct ConsistentTestFilenameConfig {
    /// Regex pattern to ensure we are linting only test filenames.
    /// Decides whether a file is a testing file.
    #[schemars(with = "Option<String>")]
    all_test_pattern: CompiledAllTestPattern,
    /// Required regex to check if a test filename have a valid formart.
    /// Pattern doesn't have a default value, you must provide one.
    #[schemars(with = "Option<String>")]
    pattern: CompiledTestPatternName,
}

impl std::ops::Deref for ConsistentTestFilename {
    type Target = ConsistentTestFilenameConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule triggers an error when a file is considered a test file, but its name
    /// does not match an expected filename format.
    ///
    /// ### Why is this bad?
    ///
    /// Files that are tests but with an unexpected filename make it hard to distinguish between
    /// source code files and test files.
    ///
    /// ### Examples
    ///
    /// An example of an **incorrect** file path for this rule configured as `{"allTestPattern": "__tests__",  "pattern": ".*\.spec\.ts$"}`:
    ///
    /// `__tests__/2.ts`
    ///
    /// An example of a **correct** file path for this rule configured as `{"allTestPattern": "__tests__",  "pattern": ".*\.spec\.ts$"}`:
    ///
    /// `__tests__/2.spec.ts`
    ///
    ConsistentTestFilename,
    vitest,
    style,
    config = ConsistentTestFilenameConfig,
    version = "1.36.0",
    short_description = "This rule triggers an error when a file is considered a test file, but its name does not match an expected filename format.",
);

fn deserialize_matcher_pattern<'de, D>(deserializer: D) -> Result<Regex, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;

    let regex_str = String::deserialize(deserializer)?;
    if let Some(stripped) = regex_str.strip_prefix('/')
        && let Some(end) = stripped.rfind('/')
    {
        let (pattern, _flags) = stripped.split_at(end);
        // For now, ignore flags and just use the pattern
        return Regex::new(pattern).map_err(D::Error::custom);
    }

    RegexBuilder::new(&regex_str).unicode(true).build().map_err(D::Error::custom)
}

impl Rule for ConsistentTestFilename {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        DefaultRuleConfig::<Self>::from_value(value).map(DefaultRuleConfig::into_inner)
    }

    fn run_once(&self, ctx: &LintContext) {
        let Some(file_path) = ctx.file_path().to_str() else { return };

        let Some(file_name) = ctx.file_path().file_name().and_then(OsStr::to_str) else { return };

        if !self.all_test_pattern.is_match(file_path) {
            return;
        }

        if !self.pattern.is_match(file_path) {
            ctx.diagnostic(consistent_test_filename_diagnostic(file_name, self.pattern.as_str()));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    use std::path::PathBuf;

    let pass = vec![
        ("export {}", None, None, Some(PathBuf::from("1.test.ts"))),
        (
            "export {}",
            Some(serde_json::json!([{ "pattern": r".*\.spec\.ts$" }])),
            None,
            Some(PathBuf::from("1.spec.ts")),
        ),
        (
            "export {}",
            Some(serde_json::json!([{ "pattern": r"/.*\.spec\.ts$/u" }])),
            None,
            Some(PathBuf::from("1.spec.ts")),
        ),
    ];

    let fail = vec![
        ("export {}", None, None, Some(PathBuf::from("1.spec.ts"))),
        (
            "export {}",
            Some(
                serde_json::json!([  {  "allTestPattern": "__tests__",  "pattern": r".*\.spec\.ts$",  },  ]),
            ),
            None,
            Some(PathBuf::from("__tests__/2.ts")),
        ),
    ];

    Tester::new(ConsistentTestFilename::NAME, ConsistentTestFilename::PLUGIN, pass, fail)
        .test_and_snapshot();
}

#[test]
fn invalid_patterns_are_rejected() {
    let configs =
        [serde_json::json!([{ "allTestPattern": "[" }]), serde_json::json!([{ "pattern": "[" }])];

    for config in configs {
        let error = ConsistentTestFilename::from_configuration(config)
            .expect_err("invalid filename pattern should be rejected");
        assert_eq!(error.classify(), serde_json::error::Category::Data);
    }
}
