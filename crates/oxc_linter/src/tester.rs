use std::path::PathBuf;

use oxc_allocator::Allocator;
use oxc_ast::SourceType;
use oxc_diagnostics::miette::{GraphicalReportHandler, GraphicalTheme, NamedSource};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;

use crate::{rules::RuleEnum, Linter};

pub struct Tester {
    rule_name: String,
    rule: RuleEnum,
    expect_pass: Vec<String>,
    expect_fail: Vec<String>,
    snapshot: String,
}

impl Tester {
    #[allow(clippy::needless_pass_by_value)]
    pub fn new<S: Into<String>>(
        rule_name: &str,
        rule: RuleEnum,
        expect_pass: Vec<S>,
        expect_fail: Vec<S>,
    ) -> Self {
        let rule_name = rule_name.replace('-', "_");
        let expect_pass = expect_pass.into_iter().map(Into::into).collect::<Vec<_>>();
        let expect_fail = expect_fail.into_iter().map(Into::into).collect::<Vec<_>>();
        Self { rule_name, rule, expect_pass, expect_fail, snapshot: String::new() }
    }

    pub fn test_and_snapshot(&mut self) {
        self.test_pass();
        self.test_fail();
        self.snapshot();
    }

    fn test_pass(&mut self) {
        for test in self.expect_pass.clone() {
            let passed = self.run(&test);
            assert!(passed, "expect test to pass: {test} {}", self.snapshot);
        }
    }

    fn test_fail(&mut self) {
        for test in self.expect_fail.clone() {
            let passed = self.run(&test);
            assert!(!passed, "expect test to fail: {test}");
        }
    }

    fn snapshot(&self) {
        insta::with_settings!({ prepend_module_to_snapshot => false, }, {
            insta::assert_snapshot!(self.rule_name.clone(), self.snapshot, &self.rule_name);
        });
    }

    fn run(&mut self, source_text: &str) -> bool {
        let allocator = Allocator::default();
        let path = PathBuf::from(self.rule_name.clone()).with_extension("tsx");
        let source_type = SourceType::from_path(&path).expect("incorrect {path:?}");
        let ret = Parser::new(&allocator, source_text, source_type).parse();
        assert!(ret.errors.is_empty(), "{:?}", &ret.errors);
        let program = allocator.alloc(ret.program);
        let semantic = SemanticBuilder::new().build(program);
        let semantic = std::rc::Rc::new(semantic);
        let diagnostics = Linter::from_rules(vec![self.rule.clone()]).run(&semantic);
        if diagnostics.is_empty() {
            return true;
        }
        let handler = GraphicalReportHandler::new_themed(GraphicalTheme::unicode_nocolor());
        for diagnostic in diagnostics {
            let diagnostic = diagnostic.with_source_code(source_text.to_string());
            let diagnostic = diagnostic.with_source_code(NamedSource::new(
                path.to_string_lossy(),
                source_text.to_string(),
            ));
            handler.render_report(&mut self.snapshot, diagnostic.as_ref()).unwrap();
        }
        false
    }
}
