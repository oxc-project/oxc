use std::{borrow::Cow, path::PathBuf, rc::Rc};

use oxc_allocator::Allocator;
use oxc_diagnostics::miette::{GraphicalReportHandler, GraphicalTheme, NamedSource};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use serde_json::Value;

use crate::{rules::RULES, Fixer, LintContext, Linter, Message};

pub struct Tester {
    rule_name: &'static str,
    expect_pass: Vec<(String, Option<Value>)>,
    expect_fail: Vec<(String, Option<Value>)>,
    snapshot: String,
}

impl Tester {
    pub fn new<S: Into<String>>(
        rule_name: &'static str,
        expect_pass: Vec<(S, Option<Value>)>,
        expect_fail: Vec<(S, Option<Value>)>,
    ) -> Self {
        let expect_pass = expect_pass.into_iter().map(|(s, r)| (s.into(), r)).collect::<Vec<_>>();
        let expect_fail = expect_fail.into_iter().map(|(s, r)| (s.into(), r)).collect::<Vec<_>>();
        Self { rule_name, expect_pass, expect_fail, snapshot: String::new() }
    }

    pub fn new_without_config<S: Into<String>>(
        rule_name: &'static str,
        expect_pass: Vec<S>,
        expect_fail: Vec<S>,
    ) -> Self {
        let expect_pass = expect_pass.into_iter().map(|s| (s.into(), None)).collect::<Vec<_>>();
        let expect_fail = expect_fail.into_iter().map(|s| (s.into(), None)).collect::<Vec<_>>();
        Self { rule_name, expect_pass, expect_fail, snapshot: String::new() }
    }

    pub fn test(&mut self) {
        self.test_pass();
        self.test_fail();
    }

    pub fn test_and_snapshot(&mut self) {
        self.test();
        self.snapshot();
    }

    fn test_pass(&mut self) {
        for (test, config) in self.expect_pass.clone() {
            let passed = self.run(&test, config);
            assert!(passed, "expect test to pass: {test} {}", self.snapshot);
        }
    }

    fn test_fail(&mut self) {
        for (test, config) in self.expect_fail.clone() {
            let passed = self.run(&test, config);
            assert!(!passed, "expect test to fail: {test}");
        }
    }

    pub fn test_fix<S: Into<String>>(&mut self, expect_fix: Vec<(S, S, Option<Value>)>) {
        let allocator = Allocator::default();
        let expect_fix =
            expect_fix.into_iter().map(|(s1, s2, r)| (s1.into(), s2.into(), r)).collect::<Vec<_>>();

        for (source_text, expected, config) in expect_fix {
            let fixed_str = self.run_with_fix(&allocator, &source_text, config);
            if let Some(fixed_str) = fixed_str {
                assert_eq!(expected, fixed_str);
            }
        }
    }

    fn snapshot(&self) {
        let name = self.rule_name.replace('-', "_");
        insta::with_settings!({ prepend_module_to_snapshot => false, }, {
            insta::assert_snapshot!(name.clone(), self.snapshot, &name);
        });
    }

    fn run(&mut self, source_text: &str, config: Option<Value>) -> bool {
        let name = self.rule_name.replace('-', "_");
        let path = PathBuf::from(name).with_extension("tsx");
        let allocator = Allocator::default();
        let result = self.run_rules(&allocator, &path, source_text, config, false);
        if result.is_empty() {
            return true;
        }

        let handler = GraphicalReportHandler::new_themed(GraphicalTheme::unicode_nocolor());
        for diagnostic in result {
            let diagnostic = diagnostic.error.with_source_code(source_text.to_string());
            let diagnostic = diagnostic.with_source_code(NamedSource::new(
                path.to_string_lossy(),
                source_text.to_string(),
            ));
            handler.render_report(&mut self.snapshot, diagnostic.as_ref()).unwrap();
            self.snapshot.push('\n');
        }
        false
    }

    fn run_with_fix<'a>(
        &mut self,
        allocator: &'a Allocator,
        source_text: &'a str,
        config: Option<Value>,
    ) -> Option<Cow<'a, str>> {
        let name = self.rule_name.replace('-', "_");
        let path = PathBuf::from(name).with_extension("tsx");
        let result = self.run_rules(allocator, &path, source_text, config, true);
        if result.is_empty() {
            return None;
        }

        let fix_result = Fixer::new(source_text, result).fix();
        Some(fix_result.fixed_code)
    }

    fn run_rules<'a>(
        &mut self,
        allocator: &'a Allocator,
        path: &PathBuf,
        source_text: &'a str,
        config: Option<Value>,
        is_fix: bool,
    ) -> Vec<Message<'a>> {
        let source_type = SourceType::from_path(path).expect("incorrect {path:?}");
        let ret = Parser::new(allocator, source_text, source_type)
            .allow_return_outside_function(true)
            .parse();
        assert!(ret.errors.is_empty(), "{:?}", &ret.errors);
        let program = allocator.alloc(ret.program);
        let semantic_ret = SemanticBuilder::new(source_text, source_type)
            .with_trivias(&ret.trivias)
            .with_module_record_builder(true)
            .build(program);
        assert!(semantic_ret.errors.is_empty(), "{:?}", &semantic_ret.errors);
        let rule = RULES
            .iter()
            .find(|rule| rule.name() == self.rule_name)
            .unwrap_or_else(|| panic!("Rule not found: {}", &self.rule_name));
        let rule = rule.read_json(config);
        let lint_context = LintContext::new(&Rc::new(semantic_ret.semantic));
        Linter::from_rules(vec![rule]).with_fix(is_fix).run(lint_context)
    }
}
