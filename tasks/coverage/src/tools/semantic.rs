use std::path::{Path, PathBuf};

use oxc::{
    span::SourceType,
    transformer::{JsxOptions, JsxRuntime, TransformOptions},
};

use crate::{
    babel::BabelCase,
    driver::Driver,
    misc::MiscCase,
    suite::{Case, TestResult},
    test262::Test262Case,
    typescript::TypeScriptCase,
};

fn get_default_transformer_options() -> TransformOptions {
    TransformOptions {
        jsx: JsxOptions {
            jsx_plugin: true,
            jsx_self_plugin: true,
            jsx_source_plugin: true,
            ..Default::default()
        },
        ..TransformOptions::enable_all()
    }
}

/// Idempotency test
fn get_result(
    source_text: &str,
    source_type: SourceType,
    source_path: &Path,
    options: Option<TransformOptions>,
) -> TestResult {
    let mut driver = Driver {
        path: source_path.to_path_buf(),
        transform: Some(options.unwrap_or_else(get_default_transformer_options)),
        check_semantic: true,
        ..Driver::default()
    };

    driver.run(source_text, source_type);
    let errors = driver.errors();
    if errors.is_empty() {
        return TestResult::Passed;
    }

    let messages = errors.into_iter().map(|e| e.message.to_string()).collect::<Vec<_>>().join("\n");
    TestResult::GenericError("semantic", messages)
}

pub struct SemanticTest262Case {
    base: Test262Case,
}

impl Case for SemanticTest262Case {
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
        let source_text = self.base.code();
        let is_module = self.base.is_module();
        let source_type = SourceType::default().with_module(is_module);
        let result = get_result(source_text, source_type, self.path(), None);
        self.base.set_result(result);
    }
}

pub struct SemanticBabelCase {
    base: BabelCase,
}

impl Case for SemanticBabelCase {
    fn new(path: PathBuf, code: String) -> Self {
        Self { base: BabelCase::new(path, code) }
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
        self.base.skip_test_case() || self.base.should_fail()
    }

    fn run(&mut self) {
        let source_text = self.base.code();
        let source_type = self.base.source_type();
        let result = get_result(source_text, source_type, self.path(), None);
        self.base.set_result(result);
    }
}

pub struct SemanticTypeScriptCase {
    base: TypeScriptCase,
}

impl Case for SemanticTypeScriptCase {
    fn new(path: PathBuf, code: String) -> Self {
        Self { base: TypeScriptCase::new(path, code) }
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
        self.base.skip_test_case() || self.base.should_fail()
    }

    fn execute(&mut self, source_type: SourceType) -> TestResult {
        let mut options = get_default_transformer_options();
        let mut source_type = source_type;
        // handle @jsx: react, `react` of behavior is match babel following options
        if self.base.settings.jsx.last().is_some_and(|jsx| jsx == "react") {
            source_type = source_type.with_module(true);
            options.jsx.runtime = JsxRuntime::Classic;
        }
        get_result(self.base.code(), source_type, self.path(), Some(options))
    }

    fn run(&mut self) {
        let units = self.base.units.clone();
        for unit in units {
            self.base.code = unit.content.to_string();
            let result = self.execute(unit.source_type);
            if result != TestResult::Passed {
                self.base.result = result;
                return;
            }
        }
        self.base.result = TestResult::Passed;
    }
}

pub struct SemanticMiscCase {
    base: MiscCase,
}

impl Case for SemanticMiscCase {
    fn new(path: PathBuf, code: String) -> Self {
        Self { base: MiscCase::new(path, code) }
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
        self.base.skip_test_case() || self.base.should_fail()
    }

    fn run(&mut self) {
        let result = get_result(self.base.code(), self.base.source_type(), self.path(), None);
        self.base.set_result(result);
    }
}
