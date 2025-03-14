use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use oxc::{
    allocator::Allocator, ast_visit::utf8_to_utf16::Utf8ToUtf16, diagnostics::OxcDiagnostic,
    parser::Parser, span::SourceType,
};

use crate::{
    suite::{Case, Suite, TestResult},
    test262::Test262Case,
    typescript::TypeScriptCase,
    workspace_root,
};

pub struct EstreeTest262Case {
    base: Test262Case,
    acorn_json_path: PathBuf,
}

impl Case for EstreeTest262Case {
    fn new(path: PathBuf, code: String) -> Self {
        let acorn_json_path = Path::new("./tasks/coverage/acorn-test262")
            .join(path.strip_prefix("test262").unwrap())
            .with_extension("json");

        Self { base: Test262Case::new(path, code), acorn_json_path }
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
        // Skip tests where fixture starts with a hashbang.
        // We intentionally diverge from Acorn, by including an extra `hashbang` field on `Program`.
        // `acorn-test262` adapts Acorn's AST to add a `hashbang: null` field to `Program`,
        // in order to match Oxc's output.
        // But these fixtures *do* include hashbangs, so there's a mismatch, because `hashbang`
        // field is (correctly) not `null` in these cases.
        // `napi/parser` contains tests for correct parsing of hashbangs.
        if self.path().starts_with("test262/test/language/comments/hashbang/") {
            return true;
        }

        // These tests fail, due to lack of support in Oxc's parser.
        // We don't filter them out because they are genuine test fails, but leaving this list here so
        // can uncomment this block when debugging any new test failures, to filter out "known bad".
        /*
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
        if IGNORE_PATHS.contains(&path) {
            return true;
        }
        */

        // Skip tests where no Acorn JSON file
        matches!(fs::exists(&self.acorn_json_path), Ok(false))
    }

    fn run(&mut self) {
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
        Utf8ToUtf16::new(source_text).convert_program(&mut program);

        let acorn_json = match fs::read_to_string(&self.acorn_json_path) {
            Ok(acorn_json) => acorn_json,
            Err(e) => {
                self.base.set_result(TestResult::GenericError(
                    "Error reading Acorn JSON",
                    e.to_string(),
                ));
                return;
            }
        };

        let oxc_json = program.to_pretty_estree_js_json();

        if oxc_json == acorn_json {
            self.base.set_result(TestResult::Passed);
            return;
        }

        // Mismatch found.
        // Write diff to `acorn-test262-diff` directory, unless running on CI.
        let is_ci = std::option_env!("CI") == Some("true");
        if !is_ci {
            self.write_diff(&oxc_json, &acorn_json);
        }

        self.base.set_result(TestResult::Mismatch("Mismatch", oxc_json, acorn_json));
    }
}

impl EstreeTest262Case {
    /// Write diff to `acorn-test262-diff` directory.
    fn write_diff(&self, oxc_json: &str, acorn_json: &str) {
        let diff_path = Path::new("./tasks/coverage/acorn-test262-diff")
            .join(self.path().strip_prefix("test262").unwrap())
            .with_extension("diff");
        std::fs::create_dir_all(diff_path.parent().unwrap()).unwrap();
        write!(
            std::fs::File::create(diff_path).unwrap(),
            "{}",
            similar::TextDiff::from_lines(acorn_json, oxc_json)
                .unified_diff()
                .missing_newline_hint(false)
        )
        .unwrap();
    }
}

pub struct AcornJsxSuite<T: Case> {
    path: PathBuf,
    test_cases: Vec<T>,
}

impl<T: Case> AcornJsxSuite<T> {
    pub fn new() -> Self {
        Self { path: Path::new("acorn-test262/test-acorn-jsx").to_path_buf(), test_cases: vec![] }
    }
}

impl<T: Case> Suite<T> for AcornJsxSuite<T> {
    fn get_test_root(&self) -> &Path {
        &self.path
    }

    fn skip_test_path(&self, path: &Path) -> bool {
        let path = path.to_string_lossy();
        !path.ends_with(".jsx")
    }

    fn save_test_cases(&mut self, cases: Vec<T>) {
        self.test_cases = cases;
    }

    fn get_test_cases(&self) -> &Vec<T> {
        &self.test_cases
    }

    fn get_test_cases_mut(&mut self) -> &mut Vec<T> {
        &mut self.test_cases
    }
}

pub struct AcornJsxCase {
    path: PathBuf,
    code: String,
    result: TestResult,
}

impl Case for AcornJsxCase {
    fn new(path: PathBuf, code: String) -> Self {
        Self { path, code, result: TestResult::ToBeRun }
    }

    fn code(&self) -> &str {
        &self.code
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn test_result(&self) -> &TestResult {
        &self.result
    }

    fn skip_test_case(&self) -> bool {
        false
    }

    fn should_fail(&self) -> bool {
        self.path.parent().unwrap().file_name().unwrap() == "fail"
    }

    fn run(&mut self) {
        let source_text = &self.code;
        let source_type = SourceType::default().with_module(true).with_jsx(true);
        let allocator = Allocator::new();
        let ret = Parser::new(&allocator, source_text, source_type).parse();
        let mut program = ret.program;

        let is_parse_error = ret.panicked || !ret.errors.is_empty();
        if self.should_fail() {
            self.result =
                if is_parse_error { TestResult::Passed } else { TestResult::IncorrectlyPassed };
            return;
        }
        if is_parse_error {
            let error =
                ret.errors.first().map_or_else(|| "Panicked".to_string(), OxcDiagnostic::to_string);
            self.result = TestResult::ParseError(error, ret.panicked);
            return;
        }

        // Convert spans to UTF16
        Utf8ToUtf16::new(source_text).convert_program(&mut program);

        let acorn_json_path = workspace_root().join(self.path.with_extension("json"));
        let acorn_json = match fs::read_to_string(&acorn_json_path) {
            Ok(acorn_json) => acorn_json,
            Err(e) => {
                self.result = TestResult::GenericError("Error reading Acorn JSON", e.to_string());
                return;
            }
        };

        let oxc_json = program.to_pretty_estree_js_json();

        if oxc_json == acorn_json {
            self.result = TestResult::Passed;
            return;
        }

        // Mismatch found.
        // Write diff to `acorn-test262-diff` directory, unless running on CI.
        let is_ci = std::option_env!("CI") == Some("true");
        if !is_ci {
            let diff_path = Path::new("./tasks/coverage/acorn-test262-diff/acorn-jsx")
                .join(self.path.file_name().unwrap())
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
        }

        self.result = TestResult::Mismatch("Mismatch", oxc_json, acorn_json);
    }
}

pub struct EstreeTypescriptCase {
    path: PathBuf,
    base: TypeScriptCase,
    estree_file_path: PathBuf,
}

impl Case for EstreeTypescriptCase {
    fn new(path: PathBuf, code: String) -> Self {
        let estree_file_path = workspace_root()
            .join("./acorn-test262/test-typescript")
            .join(path.strip_prefix("typescript").unwrap())
            .with_extension(format!("{}.md", path.extension().unwrap().to_str().unwrap()));
        Self { path: path.clone(), base: TypeScriptCase::new(path, code), estree_file_path }
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
        matches!(fs::exists(&self.estree_file_path), Ok(false))
    }

    fn run(&mut self) {
        let estree_file_content = fs::read_to_string(&self.estree_file_path).unwrap();

        let estree_units: Vec<String> = estree_file_content
            .split("__ESTREE_TEST__")
            .skip(1)
            .map(|s| {
                let s = s.strip_prefix(":PASS:\n```json\n").unwrap();
                let s = s.strip_suffix("\n```\n").unwrap();
                s.to_string()
            })
            .collect();

        if estree_units.len() != self.base.units.len() {
            // likely a bug in acorn-test262 script
            self.base.result = TestResult::GenericError(
                "Unexpected estree file content",
                format!("{} != {}", estree_units.len(), self.base.units.len()),
            );
            return;
        }

        for (unit, estree_json) in self.base.units.iter().zip(estree_units.into_iter()) {
            let source_text = &unit.content;
            let allocator = Allocator::new();
            let ret = Parser::new(&allocator, source_text, unit.source_type).parse();

            if ret.panicked || !ret.errors.is_empty() {
                let error = ret
                    .errors
                    .first()
                    .map_or_else(|| "Panicked".to_string(), OxcDiagnostic::to_string);
                self.base.result = TestResult::ParseError(error + "\n", ret.panicked);
                return;
            }

            let mut program = ret.program;
            Utf8ToUtf16::new(source_text).convert_program(&mut program);

            let oxc_json = program.to_pretty_estree_ts_json();

            if oxc_json == estree_json {
                continue;
            }

            // Mismatch found.
            // Write diff to `acorn-test262-diff` directory only when SAVE_DIFF=true since it's slow
            if std::option_env!("SAVE_DIFF") == Some("true") {
                let diff_path = Path::new("./tasks/coverage/acorn-test262-diff")
                    .join(&self.path)
                    .with_extension("diff");
                std::fs::create_dir_all(diff_path.parent().unwrap()).unwrap();
                write!(
                    std::fs::File::create(diff_path).unwrap(),
                    "{}",
                    similar::TextDiff::from_lines(&estree_json, &oxc_json)
                        .unified_diff()
                        .missing_newline_hint(false)
                )
                .unwrap();
            }

            self.base.result = TestResult::Mismatch("Mismatch", oxc_json, estree_json);
            return;
        }

        self.base.result = TestResult::Passed;
    }
}
