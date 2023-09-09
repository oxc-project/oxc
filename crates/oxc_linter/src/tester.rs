use std::{
    env,
    path::{Path, PathBuf},
};

use oxc_allocator::Allocator;
use oxc_diagnostics::miette::{GraphicalReportHandler, GraphicalTheme, NamedSource};
use oxc_diagnostics::DiagnosticService;
use serde_json::Value;

use crate::{rules::RULES, Fixer, LintOptions, LintService, Linter, RuleEnum};

#[derive(Eq, PartialEq)]
enum TestResult {
    Passed,
    Failed,
    Fixed(String),
}

pub struct Tester {
    rule_name: &'static str,
    rule_path: PathBuf,
    expect_pass: Vec<(String, Option<Value>)>,
    expect_fail: Vec<(String, Option<Value>)>,
    expect_fix: Vec<(String, String, Option<Value>)>,
    snapshot: String,
    current_working_directory: Box<Path>,
    import_plugin: bool,
}

impl Tester {
    pub fn new<S: Into<String>>(
        rule_name: &'static str,
        expect_pass: Vec<(S, Option<Value>)>,
        expect_fail: Vec<(S, Option<Value>)>,
    ) -> Self {
        let rule_path = PathBuf::from(rule_name.replace('-', "_")).with_extension("tsx");
        let expect_pass = expect_pass.into_iter().map(|(s, r)| (s.into(), r)).collect::<Vec<_>>();
        let expect_fail = expect_fail.into_iter().map(|(s, r)| (s.into(), r)).collect::<Vec<_>>();
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
        }
    }

    pub fn new_without_config<S: Into<String>>(
        rule_name: &'static str,
        expect_pass: Vec<S>,
        expect_fail: Vec<S>,
    ) -> Self {
        let expect_pass = expect_pass.into_iter().map(|s| (s.into(), None)).collect::<Vec<_>>();
        let expect_fail = expect_fail.into_iter().map(|s| (s.into(), None)).collect::<Vec<_>>();
        Self::new(rule_name, expect_pass, expect_fail)
    }

    pub fn update_expect_pass_fail<S: Into<String>>(
        mut self,
        expect_pass: Vec<S>,
        expect_fail: Vec<S>,
    ) -> Self {
        self.expect_pass = expect_pass.into_iter().map(|s| (s.into(), None)).collect::<Vec<_>>();
        self.expect_fail = expect_fail.into_iter().map(|s| (s.into(), None)).collect::<Vec<_>>();
        self
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
        for (test, config) in self.expect_pass.clone() {
            let result = self.run(&test, config, false);
            let passed = result == TestResult::Passed;
            assert!(passed, "expect test to pass: {test} {}", self.snapshot);
        }
    }

    fn test_fail(&mut self) {
        for (test, config) in self.expect_fail.clone() {
            let result = self.run(&test, config, false);
            let failed = result == TestResult::Failed;
            assert!(failed, "expect test to fail: {test}");
        }
    }

    fn test_fix(&mut self) {
        for (test, expected, config) in self.expect_fix.clone() {
            let result = self.run(&test, config, true);
            if let TestResult::Fixed(fixed_str) = result {
                assert_eq!(expected, fixed_str);
            } else {
                unreachable!()
            }
        }
    }

    fn run(&mut self, source_text: &str, config: Option<Value>, is_fix: bool) -> TestResult {
        let allocator = Allocator::default();
        let rule = self.find_rule().read_json(config);
        let options =
            LintOptions::default().with_fix(is_fix).with_import_plugin(self.import_plugin);
        let linter = Linter::from_options(options).with_rules(vec![rule]);
        let path_to_lint = if self.import_plugin {
            self.current_working_directory.join(&self.rule_path)
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
        let handler = GraphicalReportHandler::new_themed(GraphicalTheme::unicode_nocolor());
        for diagnostic in result {
            let diagnostic = diagnostic.error.with_source_code(source_text.to_string());
            let diagnostic = diagnostic.with_source_code(NamedSource::new(
                diagnostic_path.clone(),
                source_text.to_string(),
            ));
            handler.render_report(&mut self.snapshot, diagnostic.as_ref()).unwrap();
            self.snapshot.push('\n');
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
