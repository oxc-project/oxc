use std::{
    env,
    path::{Path, PathBuf},
};

use oxc_allocator::Allocator;
use oxc_diagnostics::{DiagnosticService, GraphicalReportHandler, GraphicalTheme, NamedSource};
use serde::Deserialize;
use serde_json::Value;

use crate::{
    fixer::FixKind, options::LintPluginOptions, rules::RULES, AllowWarnDeny, Fixer, LintOptions,
    LintService, LintServiceOptions, Linter, OxlintConfig, RuleEnum, RuleWithSeverity,
};

#[derive(Eq, PartialEq)]
enum TestResult {
    Passed,
    Failed,
    Fixed(String),
}

#[derive(Debug, Clone, Default)]
pub struct TestCase {
    source: String,
    rule_config: Option<Value>,
    eslint_config: Option<Value>,
    path: Option<PathBuf>,
}

impl From<&str> for TestCase {
    fn from(source: &str) -> Self {
        Self { source: source.to_string(), ..Self::default() }
    }
}

impl From<String> for TestCase {
    fn from(source: String) -> Self {
        Self { source, ..Self::default() }
    }
}

impl From<(&str, Option<Value>)> for TestCase {
    fn from((source, rule_config): (&str, Option<Value>)) -> Self {
        Self { source: source.to_string(), rule_config, ..Self::default() }
    }
}

impl From<(&str, Option<Value>, Option<Value>)> for TestCase {
    fn from((source, rule_config, eslint_config): (&str, Option<Value>, Option<Value>)) -> Self {
        Self { source: source.to_string(), rule_config, eslint_config, ..Self::default() }
    }
}

impl From<(&str, Option<Value>, Option<Value>, Option<PathBuf>)> for TestCase {
    fn from(
        (source, rule_config, eslint_config, path): (
            &str,
            Option<Value>,
            Option<Value>,
            Option<PathBuf>,
        ),
    ) -> Self {
        Self { source: source.to_string(), rule_config, eslint_config, path }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ExpectFixKind {
    /// We expect no fix to be applied
    #[default]
    None,
    /// We expect some fix to be applied, but don't care what kind it is
    Any,
    /// We expect a fix of a certain [`FixKind`] to be applied
    Specific(FixKind),
}

impl ExpectFixKind {
    #[inline]
    pub fn is_none(self) -> bool {
        matches!(self, Self::None)
    }

    #[inline]
    pub fn is_some(self) -> bool {
        !self.is_none()
    }
}

impl From<FixKind> for ExpectFixKind {
    fn from(kind: FixKind) -> Self {
        Self::Specific(kind)
    }
}
impl From<ExpectFixKind> for FixKind {
    fn from(expected_kind: ExpectFixKind) -> Self {
        match expected_kind {
            ExpectFixKind::None => FixKind::None,
            ExpectFixKind::Any => FixKind::All,
            ExpectFixKind::Specific(kind) => kind,
        }
    }
}

impl From<Option<FixKind>> for ExpectFixKind {
    fn from(maybe_kind: Option<FixKind>) -> Self {
        match maybe_kind {
            Some(kind) => Self::Specific(kind),
            None => Self::Any, // intentionally not None
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExpectFix {
    /// Source code being tested
    source: String,
    /// Expected source code after fix has been applied
    expected: String,
    kind: ExpectFixKind,
    rule_config: Option<Value>,
}

impl<S: Into<String>> From<(S, S, Option<Value>)> for ExpectFix {
    fn from(value: (S, S, Option<Value>)) -> Self {
        Self {
            source: value.0.into(),
            expected: value.1.into(),
            kind: ExpectFixKind::Any,
            rule_config: value.2,
        }
    }
}

impl<S: Into<String>> From<(S, S)> for ExpectFix {
    fn from(value: (S, S)) -> Self {
        Self {
            source: value.0.into(),
            expected: value.1.into(),
            kind: ExpectFixKind::Any,
            rule_config: None,
        }
    }
}
impl<S, F> From<(S, S, Option<Value>, F)> for ExpectFix
where
    S: Into<String>,
    F: Into<ExpectFixKind>,
{
    fn from((source, expected, config, kind): (S, S, Option<Value>, F)) -> Self {
        Self {
            source: source.into(),
            expected: expected.into(),
            kind: kind.into(),
            rule_config: config,
        }
    }
}

pub struct Tester {
    rule_name: &'static str,
    rule_path: PathBuf,
    expect_pass: Vec<TestCase>,
    expect_fail: Vec<TestCase>,
    expect_fix: Vec<ExpectFix>,
    snapshot: String,
    /// Suffix added to end of snapshot name.
    ///
    /// See: [insta::Settings::set_snapshot_suffix]
    snapshot_suffix: Option<&'static str>,
    current_working_directory: Box<Path>,
    // import_plugin: bool,
    // jest_plugin: bool,
    // vitest_plugin: bool,
    // jsx_a11y_plugin: bool,
    // nextjs_plugin: bool,
    // react_perf_plugin: bool,
    plugins: LintPluginOptions,
}

impl Tester {
    pub fn new<T: Into<TestCase>>(
        rule_name: &'static str,
        expect_pass: Vec<T>,
        expect_fail: Vec<T>,
    ) -> Self {
        let rule_path = PathBuf::from(rule_name.replace('-', "_")).with_extension("tsx");
        let expect_pass = expect_pass.into_iter().map(Into::into).collect::<Vec<_>>();
        let expect_fail = expect_fail.into_iter().map(Into::into).collect::<Vec<_>>();
        let current_working_directory =
            env::current_dir().unwrap().join("fixtures/import").into_boxed_path();
        Self {
            rule_name,
            rule_path,
            expect_pass,
            expect_fail,
            expect_fix: vec![],
            snapshot: String::new(),
            snapshot_suffix: None,
            current_working_directory,
            plugins: LintPluginOptions::none(),
        }
    }

    /// Change the path
    pub fn change_rule_path(mut self, path: &str) -> Self {
        self.rule_path = self.current_working_directory.join(path);
        self
    }

    /// Change the extension of the path
    pub fn change_rule_path_extension(mut self, ext: &str) -> Self {
        self.rule_path = self.rule_path.with_extension(ext);
        self
    }

    pub fn with_snapshot_suffix(mut self, suffix: &'static str) -> Self {
        self.snapshot_suffix = Some(suffix);
        self
    }

    pub fn with_import_plugin(mut self, yes: bool) -> Self {
        self.plugins.import = yes;
        self
    }

    pub fn with_jest_plugin(mut self, yes: bool) -> Self {
        self.plugins.jest = yes;
        self
    }

    pub fn with_vitest_plugin(mut self, yes: bool) -> Self {
        self.plugins.vitest = yes;
        self
    }

    pub fn with_jsx_a11y_plugin(mut self, yes: bool) -> Self {
        self.plugins.jsx_a11y = yes;
        self
    }

    pub fn with_nextjs_plugin(mut self, yes: bool) -> Self {
        self.plugins.nextjs = yes;
        self
    }

    pub fn with_react_perf_plugin(mut self, yes: bool) -> Self {
        self.plugins.react_perf = yes;
        self
    }

    /// Add cases that should fix problems found in the source code.
    ///
    /// These cases will fail if no fixes are produced or if the fixed source
    /// code does not match the expected result.
    ///
    /// ```
    /// use oxc_linter::tester::Tester;
    ///
    /// let pass = vec![
    ///     ("let x = 1", None)
    /// ];
    /// let fail = vec![];
    /// // You can omit the rule_config if you never use it,
    /// //otherwise its an Option<Value>
    /// let fix = vec![
    ///     // source, expected, rule_config?
    ///     ("let x = 1", "let x = 1", None)
    /// ];
    ///
    /// // the first argument is normally `MyRuleStruct::NAME`.
    /// Tester::new("no-undef", pass, fail).expect_fix(fix).test();
    /// ```
    pub fn expect_fix<F: Into<ExpectFix>>(mut self, expect_fix: Vec<F>) -> Self {
        self.expect_fix = expect_fix.into_iter().map(std::convert::Into::into).collect::<Vec<_>>();
        self
    }

    pub fn test(&mut self) {
        self.test_pass();
        self.test_fail();
        self.test_fix();
    }

    pub fn test_and_snapshot(&mut self) {
        self.test();
        self.snapshot();
    }

    fn snapshot(&self) {
        let name = self.rule_name.replace('-', "_");
        let mut settings = insta::Settings::clone_current();

        settings.set_prepend_module_to_snapshot(false);
        settings.set_omit_expression(true);
        if let Some(suffix) = self.snapshot_suffix {
            settings.set_snapshot_suffix(suffix);
        }

        settings.bind(|| {
            insta::assert_snapshot!(name, self.snapshot);
        });
    }

    fn test_pass(&mut self) {
        for TestCase { source, rule_config, eslint_config, path } in self.expect_pass.clone() {
            let result = self.run(&source, rule_config, &eslint_config, path, ExpectFixKind::None);
            let passed = result == TestResult::Passed;
            assert!(passed, "expect test to pass: {source} {}", self.snapshot);
        }
    }

    fn test_fail(&mut self) {
        for TestCase { source, rule_config, eslint_config, path } in self.expect_fail.clone() {
            let result = self.run(&source, rule_config, &eslint_config, path, ExpectFixKind::None);
            let failed = result == TestResult::Failed;
            assert!(failed, "expect test to fail: {source}");
        }
    }

    fn test_fix(&mut self) {
        for fix in self.expect_fix.clone() {
            let ExpectFix { source, expected, kind, rule_config: config } = fix;
            let result = self.run(&source, config, &None, None, kind);
            match result {
                TestResult::Fixed(fixed_str) => assert_eq!(
                    expected, fixed_str,
                    r#"Expected "{source}" to be fixed into "{expected}""#
                ),
                TestResult::Passed => panic!("Expected a fix, but test passed: {source}"),
                TestResult::Failed => panic!("Expected a fix, but test failed: {source}"),
            }
        }
    }

    fn run(
        &mut self,
        source_text: &str,
        rule_config: Option<Value>,
        eslint_config: &Option<Value>,
        path: Option<PathBuf>,
        fix: ExpectFixKind,
    ) -> TestResult {
        let allocator = Allocator::default();
        let rule = self.find_rule().read_json(rule_config.unwrap_or_default());
        let options = LintOptions::default()
            .with_fix(fix.into())
            .with_import_plugin(self.plugins.import)
            .with_jest_plugin(self.plugins.jest)
            .with_vitest_plugin(self.plugins.vitest)
            .with_jsx_a11y_plugin(self.plugins.jsx_a11y)
            .with_nextjs_plugin(self.plugins.nextjs)
            .with_react_perf_plugin(self.plugins.react_perf);
        let eslint_config = eslint_config
            .as_ref()
            .map_or_else(OxlintConfig::default, |v| OxlintConfig::deserialize(v).unwrap());
        let linter = Linter::from_options(options)
            .unwrap()
            .with_rules(vec![RuleWithSeverity::new(rule, AllowWarnDeny::Warn)])
            .with_eslint_config(eslint_config.into());
        let path_to_lint = if self.plugins.import {
            assert!(path.is_none(), "import plugin does not support path");
            self.current_working_directory.join(&self.rule_path)
        } else if let Some(path) = path {
            self.current_working_directory.join(path)
        } else if self.plugins.jest {
            self.rule_path.with_extension("test.tsx")
        } else {
            self.rule_path.clone()
        };

        let cwd = self.current_working_directory.clone();
        let paths = vec![path_to_lint.into_boxed_path()];
        let options = LintServiceOptions { cwd, paths, tsconfig: None };
        let lint_service = LintService::from_linter(linter, options);
        let diagnostic_service = DiagnosticService::default();
        let tx_error = diagnostic_service.sender();
        let result = lint_service.run_source(&allocator, source_text, false, tx_error);

        if result.is_empty() {
            return TestResult::Passed;
        }

        if fix.is_some() {
            let fix_result = Fixer::new(source_text, result).fix();
            return TestResult::Fixed(fix_result.fixed_code.to_string());
        }

        let diagnostic_path = if self.plugins.import {
            self.rule_path.strip_prefix(&self.current_working_directory).unwrap()
        } else {
            &self.rule_path
        }
        .to_string_lossy();

        let handler = GraphicalReportHandler::new()
            .with_links(false)
            .with_theme(GraphicalTheme::unicode_nocolor());
        for diagnostic in result {
            let diagnostic = diagnostic.error.with_source_code(NamedSource::new(
                diagnostic_path.clone(),
                source_text.to_string(),
            ));
            handler.render_report(&mut self.snapshot, diagnostic.as_ref()).unwrap();
        }
        TestResult::Failed
    }

    fn find_rule(&self) -> &RuleEnum {
        RULES
            .iter()
            .find(|rule| rule.name() == self.rule_name)
            .unwrap_or_else(|| panic!("Rule not found: {}", &self.rule_name))
    }
}
