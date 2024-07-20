use std::{
    env,
    path::{Path, PathBuf},
};

use oxc_allocator::Allocator;
use oxc_diagnostics::{DiagnosticService, GraphicalReportHandler, GraphicalTheme, NamedSource};
use serde::Deserialize;
use serde_json::Value;

use crate::{
    fixer::FixKind, rules::RULES, AllowWarnDeny, Fixer, LintOptions, LintService,
    LintServiceOptions, Linter, OxlintConfig, RuleEnum, RuleWithSeverity,
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

#[derive(Debug, Clone)]
pub struct ExpectFix {
    /// Source code being tested
    source: String,
    /// Expected source code after fix has been applied
    expected: String,
    rule_config: Option<Value>,
}

impl<S: Into<String>> From<(S, S, Option<Value>)> for ExpectFix {
    fn from(value: (S, S, Option<Value>)) -> Self {
        Self { source: value.0.into(), expected: value.1.into(), rule_config: value.2 }
    }
}

impl<S: Into<String>> From<(S, S)> for ExpectFix {
    fn from(value: (S, S)) -> Self {
        Self { source: value.0.into(), expected: value.1.into(), rule_config: None }
    }
}

pub struct Tester {
    rule_name: &'static str,
    rule_path: PathBuf,
    expect_pass: Vec<TestCase>,
    expect_fail: Vec<TestCase>,
    expect_fix: Vec<ExpectFix>,
    snapshot: String,
    current_working_directory: Box<Path>,
    import_plugin: bool,
    jest_plugin: bool,
    vitest_plugin: bool,
    jsx_a11y_plugin: bool,
    nextjs_plugin: bool,
    react_perf_plugin: bool,
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
            current_working_directory,
            import_plugin: false,
            jest_plugin: false,
            jsx_a11y_plugin: false,
            nextjs_plugin: false,
            react_perf_plugin: false,
            vitest_plugin: false,
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

    pub fn with_import_plugin(mut self, yes: bool) -> Self {
        self.import_plugin = yes;
        self
    }

    pub fn with_jest_plugin(mut self, yes: bool) -> Self {
        self.jest_plugin = yes;
        self
    }

    pub fn with_vitest_plugin(mut self, yes: bool) -> Self {
        self.vitest_plugin = yes;
        self
    }

    pub fn with_jsx_a11y_plugin(mut self, yes: bool) -> Self {
        self.jsx_a11y_plugin = yes;
        self
    }

    pub fn with_nextjs_plugin(mut self, yes: bool) -> Self {
        self.nextjs_plugin = yes;
        self
    }

    pub fn with_react_perf_plugin(mut self, yes: bool) -> Self {
        self.react_perf_plugin = yes;
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

    pub fn snapshot(&self) {
        let name = self.rule_name.replace('-', "_");
        insta::with_settings!({ prepend_module_to_snapshot => false, omit_expression => true }, {
            insta::assert_snapshot!(name, self.snapshot);
        });
    }

    fn test_pass(&mut self) {
        for TestCase { source, rule_config, eslint_config, path } in self.expect_pass.clone() {
            let result = self.run(&source, rule_config, &eslint_config, path, false);
            let passed = result == TestResult::Passed;
            assert!(passed, "expect test to pass: {source} {}", self.snapshot);
        }
    }

    fn test_fail(&mut self) {
        for TestCase { source, rule_config, eslint_config, path } in self.expect_fail.clone() {
            let result = self.run(&source, rule_config, &eslint_config, path, false);
            let failed = result == TestResult::Failed;
            assert!(failed, "expect test to fail: {source}");
        }
    }

    fn test_fix(&mut self) {
        for fix in self.expect_fix.clone() {
            let ExpectFix { source, expected, rule_config: config } = fix;
            let result = self.run(&source, config, &None, None, true);
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
        is_fix: bool,
    ) -> TestResult {
        let allocator = Allocator::default();
        let rule = self.find_rule().read_json(rule_config.unwrap_or_default());
        let options = LintOptions::default()
            .with_fix(
                is_fix
                    .then_some(FixKind::DangerousFix.union(FixKind::Suggestion))
                    .unwrap_or_default(),
            )
            .with_import_plugin(self.import_plugin)
            .with_jest_plugin(self.jest_plugin)
            .with_vitest_plugin(self.vitest_plugin)
            .with_jsx_a11y_plugin(self.jsx_a11y_plugin)
            .with_nextjs_plugin(self.nextjs_plugin)
            .with_react_perf_plugin(self.react_perf_plugin);
        let eslint_config = eslint_config
            .as_ref()
            .map_or_else(OxlintConfig::default, |v| OxlintConfig::deserialize(v).unwrap());
        let linter = Linter::from_options(options)
            .unwrap()
            .with_rules(vec![RuleWithSeverity::new(rule, AllowWarnDeny::Warn)])
            .with_eslint_config(eslint_config);
        let path_to_lint = if self.import_plugin {
            assert!(path.is_none(), "import plugin does not support path");
            self.current_working_directory.join(&self.rule_path)
        } else if let Some(path) = path {
            self.current_working_directory.join(path)
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

        if is_fix {
            let fix_result = Fixer::new(source_text, result).fix();
            return TestResult::Fixed(fix_result.fixed_code.to_string());
        }

        let diagnostic_path = if self.import_plugin {
            self.rule_path.strip_prefix(&self.current_working_directory).unwrap()
        } else {
            &self.rule_path
        }
        .to_string_lossy();

        let handler = GraphicalReportHandler::new().with_theme(GraphicalTheme::unicode_nocolor());
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
