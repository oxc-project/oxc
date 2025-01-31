use std::path::{Path, PathBuf};

use oxc::{allocator::Allocator, parser::Parser, span::SourceType};

use crate::{
    suite::{Case, TestResult},
    test262::Test262Case,
};

pub struct EstreeTest262Case {
    base: Test262Case,
}

impl Case for EstreeTest262Case {
    fn new(path: PathBuf, code: String) -> Self {
        Self { base: Test262Case::new(path, code) }
    }

    fn code(&self) -> &str {
        self.base.code()
    }

    fn path(&self) -> &Path {
        self.base.path()
    }

    fn test_result(&self) -> &TestResult {
        self.base.test_result()
    }

    fn skip_test_case(&self) -> bool {
        self.base.should_fail() || self.base.skip_test_case()
    }

    fn run(&mut self) {
        let acorn_path = Path::new("../acorn-test262")
            .join(self.path().strip_prefix("test262").unwrap())
            .with_extension("json");
        let Ok(acorn_file) = std::fs::read_to_string(acorn_path) else {
            // JSON file not found.
            self.base.set_result(TestResult::Passed);
            return;
        };
        // FIXME: `called `Result::unwrap()` on an `Err` value: Error("unexpected end of hex escape", line: 307, column: 33)`
        let acorn_json = match serde_json::from_str::<serde_json::Value>(&acorn_file) {
            Err(e) => {
                self.base.set_result(TestResult::GenericError("serde_json", e.to_string()));
                return;
            }
            Ok(acorn_json) => acorn_json,
        };
        let source_text = self.base.code();
        let is_module = self.base.is_module();
        let source_type = SourceType::default().with_module(is_module);
        let allocator = Allocator::new();
        let ret = Parser::new(&allocator, source_text, source_type).parse();
        // Ignore empty AST or parse errors.
        if ret.program.is_empty() || ret.panicked || !ret.errors.is_empty() {
            self.base.set_result(TestResult::GenericError("Parser", "".to_string()));
            return;
        }
        let oxc_json = serde_json::from_str::<serde_json::Value>(&ret.program.to_json()).unwrap();

        // TODO: check what mismatches
        // - span
        // - extra properties
        // - others
        // let mut mismatch_span = false;
        // let mut mismatch_extra = false;
        // let mut mismatch_other = false;

        // check excess property
        // check span difference
        // check other differences

        let acorn_json = serde_json::to_string_pretty(&acorn_json).unwrap();
        let oxc_json = serde_json::to_string_pretty(&oxc_json).unwrap();

        if acorn_json == oxc_json {
            self.base.set_result(TestResult::Passed);
        } else {
            let diff_path = Path::new("tasks/coverage/estree-diff")
                .join(self.path().strip_prefix("test262").unwrap())
                .with_extension("diff");
            std::fs::create_dir_all(diff_path.parent().unwrap()).unwrap();
            std::fs::write(
                diff_path,
                format!(
                    "{}",
                    similar::TextDiff::from_lines(&acorn_json, &oxc_json)
                        .unified_diff()
                        .missing_newline_hint(false)
                ),
            )
            .unwrap();
            self.base.set_result(TestResult::Mismatch("Mismatch", "".to_string(), "".to_string()));
        }
    }
}
