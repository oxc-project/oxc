use std::{
    env,
    path::{Path, PathBuf},
};

use oxc_allocator::Allocator;
use oxc_diagnostics::{miette::NamedSource, GraphicalReportHandler};
use oxc_diagnostics::{DiagnosticService, GraphicalTheme};
use serde_json::Value;

use crate::{
    config::parse_settings, rules::RULES, ESLintSettings, Fixer, LintOptions, LintService, Linter,
    RuleEnum,
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
    config: Option<Value>,
    settings: Option<Value>,
    path: Option<PathBuf>,
}

impl From<&str> for TestCase {
    fn from(source: &str) -> Self {
        Self { source: source.to_string(), ..Self::default() }
    }
}

impl From<(&str, Option<Value>)> for TestCase {
    fn from((source, config): (&str, Option<Value>)) -> Self {
        Self { source: source.to_string(), config, ..Self::default() }
    }
}

impl From<(&str, Option<Value>, Option<Value>)> for TestCase {
    fn from((source, config, settings): (&str, Option<Value>, Option<Value>)) -> Self {
        Self { source: source.to_string(), config, settings, ..Self::default() }
    }
}

impl From<(&str, Option<Value>, Option<Value>, Option<PathBuf>)> for TestCase {
    fn from(
        (source, config, settings, path): (&str, Option<Value>, Option<Value>, Option<PathBuf>),
    ) -> Self {
        Self { source: source.to_string(), config, settings, path }
    }
}

pub struct Tester {
    rule_name: &'static str,
    rule_path: PathBuf,
    expect_pass: Vec<TestCase>,
    expect_fail: Vec<TestCase>,
    expect_fix: Vec<(String, String, Option<Value>)>,
    snapshot: String,
    current_working_directory: Box<Path>,
    import_plugin: bool,
    jest_plugin: bool,
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
        }
    }

    /// Change the path
    pub fn change_rule_path(mut self, path: &str) -> Self {
        self.rule_path = self.current_working_directory.join(path);
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

    pub fn expect_fix<S: Into<String>>(mut self, expect_fix: Vec<(S, S, Option<Value>)>) -> Self {
        self.expect_fix =
            expect_fix.into_iter().map(|(s1, s2, r)| (s1.into(), s2.into(), r)).collect::<Vec<_>>();
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
        insta::with_settings!({ prepend_module_to_snapshot => false, }, {
            insta::assert_snapshot!(name.clone(), self.snapshot, &name);
        });
    }

    fn test_pass(&mut self) {
        for TestCase { source, config, settings, path } in self.expect_pass.clone() {
            let result = self.run(&source, config, false, &settings, &path);
            let passed = result == TestResult::Passed;
            assert!(passed, "expect test to pass: {source} {}", self.snapshot);
        }
    }

    fn test_fail(&mut self) {
        for TestCase { source, config, settings, path } in self.expect_fail.clone() {
            let result = self.run(&source, config, false, &settings, &path);
            let failed = result == TestResult::Failed;
            assert!(failed, "expect test to fail: {source}");
        }
    }

    fn test_fix(&mut self) {
        for (test, expected, config) in self.expect_fix.clone() {
            let result = self.run(&test, config, true, &None, &None);
            if let TestResult::Fixed(fixed_str) = result {
                assert_eq!(expected, fixed_str);
            } else {
                unreachable!()
            }
        }
    }

    fn run(
        &mut self,
        source_text: &str,
        config: Option<Value>,
        is_fix: bool,
        settings: &Option<Value>,
        path: &Option<PathBuf>,
    ) -> TestResult {
        let allocator = Allocator::default();
        let rule = self.find_rule().read_json(config);
        let lint_settings: ESLintSettings =
            settings.as_ref().map_or_else(ESLintSettings::default, parse_settings);
        let options = LintOptions::default()
            .with_fix(is_fix)
            .with_import_plugin(self.import_plugin)
            .with_jest_plugin(self.jest_plugin)
            .with_jsx_a11y_plugin(self.jsx_a11y_plugin)
            .with_nextjs_plugin(self.nextjs_plugin)
            .with_react_perf_plugin(self.react_perf_plugin);
        let linter = Linter::from_options(options)
            .unwrap()
            .with_rules(vec![rule])
            .with_settings(lint_settings);
        let path_to_lint = if self.import_plugin {
            assert!(path.is_none(), "import plugin does not support path");
            self.current_working_directory.join(&self.rule_path)
        } else if let Some(path) = path {
            self.current_working_directory.join(path)
        } else {
            self.rule_path.clone()
        };

        let lint_service = LintService::from_linter(
            self.current_working_directory.clone(),
            &[path_to_lint.into_boxed_path()],
            linter,
        );
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
            let diagnostic = diagnostic.error.with_source_code(source_text.to_string());
            let diagnostic = diagnostic.with_source_code(NamedSource::new(
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
