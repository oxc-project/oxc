use std::path::PathBuf;

use oxc_allocator::Allocator;
use oxc_ast::SourceType;
use oxc_diagnostics::miette::{GraphicalReportHandler, GraphicalTheme, NamedSource};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;

use crate::{rules::RULES, Linter};

pub struct Tester {
    rule_name: &'static str,
    expect_pass: Vec<String>,
    expect_fail: Vec<String>,
    snapshot: String,
}

impl Tester {
    #[allow(clippy::needless_pass_by_value)]
    pub fn new<S: Into<String>>(
        rule_name: &'static str,
        expect_pass: Vec<S>,
        expect_fail: Vec<S>,
    ) -> Self {
        let expect_pass = expect_pass.into_iter().map(Into::into).collect::<Vec<_>>();
        let expect_fail = expect_fail.into_iter().map(Into::into).collect::<Vec<_>>();
        Self { rule_name, expect_pass, expect_fail, snapshot: String::new() }
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
        let name = self.rule_name.replace('-', "_");
        insta::with_settings!({ prepend_module_to_snapshot => false, }, {
            insta::assert_snapshot!(name.clone(), self.snapshot, &name);
        });
    }

    fn run(&mut self, source_text: &str) -> bool {
        let name = self.rule_name.replace('-', "_");
        let allocator = Allocator::default();
        let path = PathBuf::from(name).with_extension("tsx");
        let source_type = SourceType::from_path(&path).expect("incorrect {path:?}");
        let ret = Parser::new(&allocator, source_text, source_type).parse();
        assert!(ret.errors.is_empty(), "{:?}", &ret.errors);
        let program = allocator.alloc(ret.program);
        let semantic = SemanticBuilder::new().build(program, ret.trivias);
        let semantic = std::rc::Rc::new(semantic);
        let rule = RULES.iter().find(|rule| rule.name() == self.rule_name).unwrap();
        let diagnostics = Linter::from_rules(vec![rule.clone()]).run(&semantic);
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
