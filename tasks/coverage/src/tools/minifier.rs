use std::path::{Path, PathBuf};
use std::process::Command;

use oxc::minifier::{CompressOptions, CompressOptionsKeepNames};
use oxc::span::SourceType;

use crate::{
    Driver,
    babel::BabelCase,
    node_compat_table::NodeCompatCase,
    suite::{Case, TestResult},
    test262::Test262Case,
};

/// Idempotency test
fn get_result(source_text: &str, source_type: SourceType) -> TestResult {
    Driver { compress: Some(CompressOptions::smallest()), codegen: true, ..Driver::default() }
        .idempotency("Compress", source_text, source_type)
}

pub struct MinifierTest262Case {
    base: Test262Case,
}

impl Case for MinifierTest262Case {
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
            // Unable to minify non-strict code, which may contain syntaxes that the minifier do not support (e.g. `with`).
            || self.base.is_no_strict()
    }

    fn run(&mut self) {
        let source_text = self.base.code();
        let is_module = self.base.is_module();
        let source_type = SourceType::default().with_module(is_module);
        let result = get_result(source_text, source_type);
        self.base.set_result(result);
    }
}

pub struct MinifierBabelCase {
    base: BabelCase,
}

impl Case for MinifierBabelCase {
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
        self.base.skip_test_case()
            || self.base.should_fail()
            || self.base.source_type().is_typescript()
    }

    fn run(&mut self) {
        let source_text = self.base.code();
        let source_type = self.base.source_type();
        let result = get_result(source_text, source_type);
        self.base.set_result(result);
    }
}

pub struct MinifierNodeCompatCase {
    base: NodeCompatCase,
}

impl Case for MinifierNodeCompatCase {
    fn new(path: PathBuf, code: String) -> Self {
        Self { base: NodeCompatCase::new(path, code) }
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
        let path = self.path().to_str().unwrap();
        self.base.skip_test_case()
        || path.contains("temporal dead zone") // TDZ errors are ignored by assumptions
        || path == "ES2015/built-ins›well-known symbols›Symbol.toPrimitive" // [Symbol.toPrimitive] is assumed to not have a side effect
        || path == "ES2015/misc›Proxy, internal 'get' calls›ClassDefinitionEvaluation" // extending a class is assumed to not have a side effect
        || path == "ES2015/annex b›non-strict function semantics›hoisted block-level function declaration" // this is a pathological case in non-strict mode, terser and SWC fails as well, ignore it
    }

    fn run(&mut self) {
        let source_text = self.base.code();
        let source_type = NodeCompatCase::source_type();
        let keep_names = self.path().to_str().unwrap().contains("\"name\" property");
        let result = test_minification_preserves_execution(source_text, source_type, keep_names);
        self.base.set_result(result);
    }
}

fn test_minification_preserves_execution(
    code: &str,
    source_type: SourceType,
    keep_names: bool,
) -> TestResult {
    let Ok(original_result) = execute_node_code(code) else {
        return TestResult::ParseError("Original code failed to execute".to_string(), false);
    };

    let Ok(minified_code) = minify_code(code, source_type, keep_names) else {
        return TestResult::ParseError("Failed to minify code".to_string(), false);
    };

    let Ok(minified_result) = execute_node_code(&minified_code) else {
        return TestResult::GenericError(
            "minified_execution",
            "Minified code failed to execute".to_string(),
        );
    };

    if original_result == minified_result {
        TestResult::Passed
    } else {
        TestResult::Mismatch("execution_result", minified_result, original_result)
    }
}

fn execute_node_code(code: &str) -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("node").arg("-e").arg(code).output()?;
    Ok(String::from_utf8(output.stdout)?)
}

fn minify_code(
    source_text: &str,
    source_type: SourceType,
    keep_names: bool,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut driver = Driver {
        path: PathBuf::from("test.js"),
        compress: Some(CompressOptions {
            keep_names: if keep_names {
                CompressOptionsKeepNames::all_true()
            } else {
                CompressOptionsKeepNames::all_false()
            },
            ..CompressOptions::smallest()
        }),
        codegen: true,
        remove_whitespace: true,
        ..Driver::default()
    };

    driver.run(source_text, source_type);

    if !driver.errors().is_empty() {
        return Err("Compilation errors".into());
    }

    Ok(driver.printed)
}
