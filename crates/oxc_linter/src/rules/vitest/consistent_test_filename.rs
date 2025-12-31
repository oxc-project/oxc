use std::ffi::OsStr;

use lazy_regex::Regex;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use schemars::JsonSchema;

use crate::{context::LintContext, rule::Rule};

fn consistent_test_filename_diagnostic(file_path: &str, pattern: &str) -> OxcDiagnostic {
    let message = format!("The {file_path} is a test file but his name is not allowed");
    let help = format!("Rename the file that match the pattern {pattern}");

    OxcDiagnostic::warn(message).with_help(help)
}

#[derive(Debug, Default, Clone)]
pub struct ConsistentTestFilename(Box<ConsistentTestFilenameConfig>);

#[derive(Debug, Clone, JsonSchema)]
pub struct CompiledAllTestPattern(lazy_regex::Regex);

impl Default for CompiledAllTestPattern {
    fn default() -> Self {
        let default_regex = Regex::new(r".*\.(test|spec)\.[tj]sx?$");

        Self(default_regex.unwrap())
    }
}

impl std::ops::Deref for CompiledAllTestPattern {
    type Target = lazy_regex::Regex;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, JsonSchema)]
pub struct CompiledTestPatternName(lazy_regex::Regex);

impl Default for CompiledTestPatternName {
    fn default() -> Self {
        let default_regex = Regex::new(r".*\.test\.[tj]sx?$");

        Self(default_regex.unwrap())
    }
}

impl std::ops::Deref for CompiledTestPatternName {
    type Target = lazy_regex::Regex;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Default, Clone, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct ConsistentTestFilenameConfig {
    /// Regex pattern to ensure we are linting only test filenames.
    /// Decides whether a file is a testing file.
    all_test_pattern: CompiledAllTestPattern,
    /// Required regex to check if a test filename have a valid formart.
    /// Pattern doesn't have a default value, you must provide one.
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
    /// __tests__/2.ts
    ///
    /// An example of a **correct** file path for this rule configured as `{"allTestPattern": "__tests__",  "pattern": ".*\.spec\.ts$"}`:
    ///
    /// __tests__/2.spec.ts
    ///
    ConsistentTestFilename,
    vitest,
    style,
    config = ConsistentTestFilenameConfig,
);

fn compile_matcher_pattern(matcher_pattern: &serde_json::Value) -> Option<lazy_regex::Regex> {
    matcher_pattern.as_str().and_then(|regex_str| {
        if let Some(stripped) = regex_str.strip_prefix('/')
            && let Some(end) = stripped.rfind('/')
        {
            let (pat, _flags) = stripped.split_at(end);
            // For now, ignore flags and just use the pattern
            return Regex::new(pat).ok();
        }

        let reg_str = format!("(?u){regex_str}");
        Regex::new(&reg_str).ok()
    })
}

impl Rule for ConsistentTestFilename {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        let config = value.get(0);

        let all_test_pattern = config
            .and_then(|v| v.get("allTestPattern"))
            .and_then(|value| compile_matcher_pattern(value).map(CompiledAllTestPattern))
            .unwrap_or_default();

        let pattern = config
            .and_then(|v| v.get("pattern"))
            .and_then(|value| compile_matcher_pattern(value).map(CompiledTestPatternName))
            .unwrap_or_default();

        Ok(Self(Box::new(ConsistentTestFilenameConfig { all_test_pattern, pattern })))
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
