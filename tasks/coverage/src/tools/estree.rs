use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use constcat::concat_slices;
use similar::TextDiff;

use oxc::{
    allocator::Allocator,
    ast_visit::utf8_to_utf16::Utf8ToUtf16,
    diagnostics::OxcDiagnostic,
    parser::{ParseOptions, Parser},
    span::SourceType,
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
        let acorn_json_path =
            workspace_root().join("estree-conformance/tests").join(&path).with_extension("json");

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
        // `estree-conformance` adapts Acorn's AST to add a `hashbang: null` field to `Program`,
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
        #[expect(clippy::items_after_statements)]
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
        ];

        if self.path().to_str().is_some_and(|path| IGNORE_PATHS.contains(&path)) {
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
        Utf8ToUtf16::new(source_text).convert_program_with_ascending_order_checks(&mut program);

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

        let oxc_json = program.to_pretty_estree_js_json(false);

        if oxc_json == acorn_json {
            self.base.set_result(TestResult::Passed);
            return;
        }

        // Mismatch found
        write_diff(self.path(), &oxc_json, &acorn_json);
        self.base.set_result(TestResult::Mismatch("Mismatch", oxc_json, acorn_json));
    }
}

pub struct AcornJsxSuite<T: Case> {
    path: PathBuf,
    test_cases: Vec<T>,
}

impl<T: Case> AcornJsxSuite<T> {
    pub fn new() -> Self {
        Self {
            path: workspace_root().join("estree-conformance/tests/acorn-jsx"),
            test_cases: vec![],
        }
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

pub struct EstreeJsxCase {
    path: PathBuf,
    code: String,
    result: TestResult,
}

impl Case for EstreeJsxCase {
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
        Utf8ToUtf16::new(source_text).convert_program_with_ascending_order_checks(&mut program);

        let acorn_json_path = workspace_root().join(self.path.with_extension("json"));
        let acorn_json = match fs::read_to_string(&acorn_json_path) {
            Ok(acorn_json) => acorn_json,
            Err(e) => {
                self.result = TestResult::GenericError("Error reading Acorn JSON", e.to_string());
                return;
            }
        };

        let oxc_json = program.to_pretty_estree_js_json(false);

        if oxc_json == acorn_json {
            self.result = TestResult::Passed;
            return;
        }

        // Mismatch found
        let diff_path = Path::new("acorn-jsx").join(self.path.file_name().unwrap());
        write_diff(&diff_path, &oxc_json, &acorn_json);

        self.result = TestResult::Mismatch("Mismatch", oxc_json, acorn_json);
    }
}

pub struct EstreeTypescriptCase {
    base: TypeScriptCase,
    estree_file_path: PathBuf,
}

impl Case for EstreeTypescriptCase {
    fn new(path: PathBuf, code: String) -> Self {
        let estree_file_path = workspace_root()
            .join("estree-conformance/tests")
            .join(&path)
            .with_extension(format!("{}.md", path.extension().unwrap().to_str().unwrap()));
        Self { base: TypeScriptCase::new(path, code), estree_file_path }
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

    fn always_strict(&self) -> bool {
        self.base.always_strict()
    }

    fn skip_test_case(&self) -> bool {
        // Skip cases which are failing in parser conformance tests.
        // Some of these should parse correctly, but the cause is not related to ESTree serialization,
        // so they're not relevant here. If we fix them, that'll register in the parser snapshot.
        // TODO: If we fix any of these in parser, remove them from the list below.
        const PARSE_ERROR_PATHS: &[&str] = &[
            // Fails because fixture is not loaded as an ESM module (bug in tester)
            "typescript/tests/cases/compiler/arrayFromAsync.ts",
            // Differences between TS's recoverable parser and Oxc's non-recoverable parser
            "typescript/tests/cases/conformance/classes/propertyMemberDeclarations/staticPropertyNameConflicts.ts",
            "typescript/tests/cases/conformance/es2019/importMeta/importMeta.ts",
            // Decorators - probably should be parsed correctly (bug in parser)
            "typescript/tests/cases/compiler/sourceMapValidationDecorators.ts",
            "typescript/tests/cases/conformance/esDecorators/esDecorators-decoratorExpression.1.ts",
        ];

        // Skip tests where `@typescript-eslint/parser` is incorrect, and we can't massage the AST
        // to align with it
        const INCORRECT_PATHS: &[&str] = &[
            // TS-ESLint includes `\r` in `raw` field of `TemplateElement`.
            // This is incorrect - `\r` should be converted to `\n`, as both Acorn and Espree do.
            // This matches the `raw` value that you get at runtime in a tagged template in JS.
            // We perform the `\r` -> `\n` conversion in parser, so we can't match TS-ESTree without
            // breaking plain JS ESTree.
            "typescript/tests/cases/conformance/es6/templates/templateStringMultiline3.ts",
        ];

        // Skip tests where fixture starts with a hashbang.
        // We intentionally diverge from TS-ESTree, by including an extra `hashbang` field on `Program`.
        // `estree-conformance` adapts TS-ESLint's AST to add a `hashbang: null` field to `Program`,
        // in order to match Oxc's output.
        // But these fixtures *do* include hashbangs, so there's a mismatch, because `hashbang`
        // field is (correctly) not `null` in these cases.
        // `napi/parser` contains tests for correct parsing of hashbangs.
        const HASHBANG_PATHS: &[&str] = &[
            "typescript/tests/cases/compiler/emitBundleWithShebang1.ts",
            "typescript/tests/cases/compiler/emitBundleWithShebang2.ts",
            "typescript/tests/cases/compiler/emitBundleWithShebangAndPrologueDirectives1.ts",
            "typescript/tests/cases/compiler/emitBundleWithShebangAndPrologueDirectives2.ts",
            "typescript/tests/cases/compiler/shebang.ts",
            "typescript/tests/cases/compiler/shebangBeforeReferences.ts",
        ];

        static IGNORE_PATHS: &[&str] = concat_slices!(
            [&str]: PARSE_ERROR_PATHS, INCORRECT_PATHS, HASHBANG_PATHS
        );

        // Skip cases where expected to fail to parse
        if self.base.should_fail() {
            return true;
        }

        // Skip ignored cases
        if self.path().to_str().is_some_and(|path| IGNORE_PATHS.contains(&path)) {
            return true;
        }

        // Skip cases where no JSON file for case in `estree-conformance`
        matches!(fs::exists(&self.estree_file_path), Ok(false))
    }

    fn run(&mut self) {
        let estree_file_content = fs::read_to_string(&self.estree_file_path).unwrap();

        let estree_units = estree_file_content
            .split("__ESTREE_TEST__")
            .skip(1)
            .map(|s| {
                let s = s.strip_prefix(":PASS:\n```json\n").unwrap();
                s.strip_suffix("\n```\n").unwrap()
            })
            .collect::<Vec<_>>();

        if estree_units.len() != self.base.units.len() {
            // likely a bug in estree-conformance script
            self.base.result = TestResult::GenericError(
                "Unexpected estree file content",
                format!("{} != {}", estree_units.len(), self.base.units.len()),
            );
            return;
        }

        for (unit, estree_json) in self.base.units.iter().zip(estree_units.into_iter()) {
            let source_text = &unit.content;
            let allocator = Allocator::new();
            let options = ParseOptions { preserve_parens: false, ..Default::default() };
            let ret = Parser::new(&allocator, source_text, unit.source_type)
                .with_options(options)
                .parse();

            if ret.panicked || !ret.errors.is_empty() {
                let error = ret
                    .errors
                    .first()
                    .map_or_else(|| "Panicked".to_string(), OxcDiagnostic::to_string);
                self.base.result = TestResult::ParseError(error + "\n", ret.panicked);
                return;
            }

            let mut program = ret.program;
            Utf8ToUtf16::new(source_text).convert_program_with_ascending_order_checks(&mut program);

            let oxc_json = program.to_pretty_estree_ts_json(false);
            if oxc_json == estree_json {
                continue;
            }

            // Mismatch found
            write_diff(self.path(), &oxc_json, estree_json);
            self.base.result = TestResult::Mismatch("Mismatch", oxc_json, estree_json.to_string());
            return;
        }

        self.base.result = TestResult::Passed;
    }
}

/// Write diff to `estree-conformance-diff` directory, unless running on CI.
fn write_diff(path: &Path, oxc_json: &str, expected_json: &str) {
    let is_ci = std::option_env!("CI") == Some("true");
    if is_ci {
        return;
    }

    let diff_path =
        Path::new("./tasks/coverage/estree-conformance-diff").join(path).with_extension("diff");
    fs::create_dir_all(diff_path.parent().unwrap()).unwrap();

    write!(
        fs::File::create(diff_path).unwrap(),
        "{}",
        TextDiff::from_lines(expected_json, oxc_json).unified_diff().missing_newline_hint(false)
    )
    .unwrap();
}
