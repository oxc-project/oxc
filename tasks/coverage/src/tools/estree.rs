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

    fn skip_test_case(&self) -> bool {
        // In an ideal world, we would not ignore these tests as we should be able to pass them.
        // But ignoring them for now just to reduce noise in the conformance snapshot file.
        // TODO: Re-enable these tests.
        static IGNORE_PATHS: &[&str] = &[
            // Missing `ParenthesizedExpression` on left side of assignment.
            // Oxc's parser does not support this, and we do not intend to fix.
            // https://github.com/oxc-project/oxc/issues/9029
            "test262/test/language/expressions/assignment/fn-name-lhs-cover.js",
            "test262/test/language/expressions/assignment/target-cover-id.js",
            "test262/test/language/expressions/postfix-decrement/target-cover-id.js",
            "test262/test/language/expressions/postfix-increment/target-cover-id.js",
            "test262/test/language/expressions/prefix-decrement/target-cover-id.js",
            "test262/test/language/expressions/prefix-increment/target-cover-id.js",
            "test262/test/language/statements/for-in/head-lhs-cover.js",
            "test262/test/language/statements/for-of/head-lhs-async-parens.js",
            "test262/test/language/statements/for-of/head-lhs-cover.js",

            // Lone surrogates in strings.
            // We cannot pass these tests at present, as Oxc's parser does not handle them correctly.
            // https://github.com/oxc-project/oxc/issues/3526#issuecomment-2650260735
            "test262/test/annexB/built-ins/RegExp/prototype/compile/pattern-string-u.js",
            "test262/test/annexB/built-ins/String/prototype/substr/surrogate-pairs.js",
            "test262/test/built-ins/Array/prototype/concat/Array.prototype.concat_spreadable-string-wrapper.js",
            "test262/test/built-ins/JSON/stringify/value-string-escape-unicode.js",
            "test262/test/built-ins/RegExp/dotall/with-dotall-unicode.js",
            "test262/test/built-ins/RegExp/dotall/with-dotall.js",
            "test262/test/built-ins/RegExp/dotall/without-dotall-unicode.js",
            "test262/test/built-ins/RegExp/dotall/without-dotall.js",
            "test262/test/built-ins/RegExp/escape/escaped-surrogates.js",
            "test262/test/built-ins/RegExp/named-groups/non-unicode-property-names-invalid.js",
            "test262/test/built-ins/RegExp/named-groups/unicode-property-names-invalid.js",
            "test262/test/built-ins/RegExp/prototype/Symbol.replace/coerce-unicode.js",
            "test262/test/built-ins/RegExp/prototype/exec/u-captured-value.js",
            "test262/test/built-ins/RegExp/regexp-modifiers/add-dotAll.js",
            "test262/test/built-ins/RegExp/regexp-modifiers/changing-dotAll-flag-does-not-affect-dotAll-modifier.js",
            "test262/test/built-ins/RegExp/regexp-modifiers/nesting-add-dotAll-within-remove-dotAll.js",
            "test262/test/built-ins/RegExp/regexp-modifiers/nesting-remove-dotAll-within-add-dotAll.js",
            "test262/test/built-ins/RegExp/regexp-modifiers/remove-dotAll.js",
            "test262/test/built-ins/String/prototype/at/returns-code-unit.js",
            "test262/test/built-ins/String/prototype/codePointAt/return-first-code-unit.js",
            "test262/test/built-ins/String/prototype/codePointAt/return-single-code-unit.js",
            "test262/test/built-ins/String/prototype/isWellFormed/returns-boolean.js",
            "test262/test/built-ins/String/prototype/match/regexp-prototype-match-v-u-flag.js",
            "test262/test/built-ins/String/prototype/padEnd/normal-operation.js",
            "test262/test/built-ins/String/prototype/padStart/normal-operation.js",
            "test262/test/built-ins/String/prototype/toWellFormed/returns-well-formed-string.js",
            "test262/test/built-ins/StringIteratorPrototype/next/next-iteration-surrogate-pairs.js",
            "test262/test/intl402/NumberFormat/prototype/format/format-non-finite-numbers.js",
            "test262/test/intl402/Segmenter/prototype/segment/containing/breakable-input.js",
            "test262/test/intl402/Segmenter/prototype/segment/containing/unbreakable-input.js",
            "test262/test/intl402/Segmenter/prototype/segment/containing/zero-index.js",
            "test262/test/language/literals/regexp/named-groups/invalid-lone-surrogate-groupname.js",
            "test262/test/language/literals/regexp/u-astral.js",
            "test262/test/language/literals/regexp/u-surrogate-pairs-atom-char-class.js",
            "test262/test/language/literals/regexp/u-surrogate-pairs-atom-escape-decimal.js",
            "test262/test/language/statements/for-of/string-astral-truncated.js",
        ];

        let path = &*self.path().to_string_lossy();
        IGNORE_PATHS.contains(&path)
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
