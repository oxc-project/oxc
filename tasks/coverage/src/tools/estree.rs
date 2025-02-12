use std::io::Write;
use std::path::{Path, PathBuf};

use serde_json::Value;

use oxc::{
    allocator::Allocator, ast::utf8_to_utf16::Utf8ToUtf16, diagnostics::OxcDiagnostic,
    parser::Parser, span::SourceType,
};

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

    fn run(&mut self) {
        let acorn_path = Path::new("./tasks/coverage/acorn-test262")
            .join(self.path().strip_prefix("test262").unwrap())
            .with_extension("json");
        let Ok(acorn_file) = std::fs::read_to_string(acorn_path) else {
            // JSON file not found
            self.base.set_result(TestResult::Passed);
            return;
        };

        // Parse
        let source_text = self.base.code();
        let is_module = self.base.is_module();
        let source_type = SourceType::default().with_module(is_module);
        let allocator = Allocator::new();
        let ret = Parser::new(&allocator, source_text, source_type).parse();
        let mut program = ret.program;

        if ret.panicked || !ret.errors.is_empty() {
            let error =
                ret.errors.first().map_or_else(|| "Panicked".to_string(), OxcDiagnostic::to_string);
            self.base.set_result(TestResult::ParseError(error, ret.panicked));
            return;
        }

        // Convert spans to UTF16
        Utf8ToUtf16::new().convert(&mut program);

        // Remove extra properties from Oxc AST where there is no corresponding property in Acorn AST
        let acorn_json_value = match deserialize_json(&acorn_file) {
            Err(e) => {
                self.base.set_result(TestResult::GenericError("serde_json", e.to_string()));
                return;
            }
            Ok(acorn_json) => acorn_json,
        };
        let mut oxc_json_value = match deserialize_json(&program.to_json()) {
            Err(e) => {
                self.base.set_result(TestResult::GenericError("serde_json", e.to_string()));
                return;
            }
            Ok(oxc_json) => oxc_json,
        };
        remove_extra_properties_from_oxc_ast(&mut oxc_json_value, &acorn_json_value);

        // Compare JSON between Acorn and Oxc
        let acorn_json = serde_json::to_string_pretty(&acorn_json_value).unwrap();
        let oxc_json = serde_json::to_string_pretty(&oxc_json_value).unwrap();

        if acorn_json == oxc_json {
            self.base.set_result(TestResult::Passed);
            return;
        }

        // Mismatch found
        let diff_path = Path::new("./tasks/coverage/acorn-test262-diff")
            .join(self.path().strip_prefix("test262").unwrap())
            .with_extension("diff");
        std::fs::create_dir_all(diff_path.parent().unwrap()).unwrap();
        write!(
            std::fs::File::create(diff_path).unwrap(),
            "{}",
            similar::TextDiff::from_lines(&acorn_json, &oxc_json)
                .unified_diff()
                .missing_newline_hint(false)
        )
        .unwrap();
        self.base.set_result(TestResult::Mismatch("Mismatch", oxc_json, acorn_json));
    }
}

/// Deserialize JSON string to `serde_json::Value`.
///
/// Identical to `serde_json::from_str::<serde_json::Value>(json)`,
/// except with no limit on how deeply nested the JSON can be.
fn deserialize_json(json: &str) -> Result<Value, serde_json::Error> {
    use serde::Deserialize;

    let s = serde_json::de::StrRead::new(json);
    let mut deserializer = serde_json::Deserializer::new(s);
    deserializer.disable_recursion_limit();
    let value = Value::deserialize(&mut deserializer)?;
    deserializer.end()?;
    Ok(value)
}

/// Remove extra properties from Oxc AST where there is no corresponding property in Acorn AST.
///
/// Intention is to ignore extra properties in Oxc AST which are Typescript-related extensions to AST,
/// and don't appear in Acorn AST.
fn remove_extra_properties_from_oxc_ast(oxc: &mut Value, acorn: &Value) {
    match (oxc, acorn) {
        (Value::Object(oxc), Value::Object(acorn)) => {
            oxc.retain(|key, oxc_value| {
                if let Some(acorn_value) = acorn.get(key) {
                    remove_extra_properties_from_oxc_ast(oxc_value, acorn_value);
                    true
                } else {
                    false
                }
            });
        }
        (Value::Array(oxc), Value::Array(acorn)) => {
            for (oxc_value, acorn_value) in oxc.iter_mut().zip(acorn) {
                remove_extra_properties_from_oxc_ast(oxc_value, acorn_value);
            }
        }
        _ => {}
    }
}
