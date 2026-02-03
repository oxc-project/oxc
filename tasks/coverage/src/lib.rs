#![expect(clippy::print_stdout, clippy::disallowed_methods)]

mod driver;
mod load;
mod runtime;

mod babel;
mod test262;
mod typescript;

mod tools;

use std::{
    fmt::Write,
    path::{Path, PathBuf},
};

use console::Style;
use oxc::{span::SourceType, transformer::BabelOptions};
use oxc_tasks_common::{Snapshot, normalize_path, project_root};
use similar::{ChangeTag, TextDiff};

pub use driver::Driver;
use test262::MetaData as Test262Meta;
use typescript::meta::{CompilerSettings, TestUnitData};

// ================================
// Data Structures
// ================================

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TestResult {
    ToBeRun,
    Passed,
    IncorrectlyPassed,
    Mismatch(&'static str, String, String),
    ParseError(String, bool),
    CorrectError(String, bool),
    GenericError(&'static str, String),
}

pub struct Test262File {
    pub path: PathBuf,
    pub code: String,
    pub meta: Test262Meta,
}

pub struct BabelFile {
    pub path: PathBuf,
    pub code: String,
    pub source_type: SourceType,
    pub options: BabelOptions,
    pub should_fail: bool,
}

pub struct TypeScriptFile {
    pub path: PathBuf,
    pub code: String,
    pub units: Vec<TestUnitData>,
    pub settings: CompilerSettings,
    pub should_fail: bool,
    /// Error codes from TypeScript baseline error files (used by semantic tests to skip files)
    pub error_codes: Vec<String>,
}

pub struct MiscFile {
    pub path: PathBuf,
    pub code: String,
    pub source_type: SourceType,
    pub should_fail: bool,
}

pub struct TestData {
    pub test262: Vec<Test262File>,
    pub babel: Vec<BabelFile>,
    pub typescript: Vec<TypeScriptFile>,
    pub misc: Vec<MiscFile>,
}

// ================================
// Paths
// ================================

pub fn workspace_root() -> PathBuf {
    project_root().join("tasks").join("coverage")
}

fn snap_root() -> PathBuf {
    workspace_root().join("snapshots")
}

// ================================
// Result & Reporting
// ================================

pub struct CoverageResult {
    pub path: PathBuf,
    pub should_fail: bool,
    pub result: TestResult,
}

impl CoverageResult {
    fn passed(&self) -> bool {
        match &self.result {
            TestResult::Passed if !self.should_fail => true,
            // CorrectError counts as passed for both positive tests (transpile) and negative tests
            TestResult::CorrectError(_, _) => true,
            _ => false,
        }
    }

    fn parsed(&self) -> bool {
        match &self.result {
            TestResult::ParseError(_, panicked) | TestResult::CorrectError(_, panicked) => {
                !panicked
            }
            _ => true,
        }
    }
}

struct CoverageStats<'a> {
    positives: Vec<&'a CoverageResult>,
    negatives: Vec<&'a CoverageResult>,
    all_positives: usize,
    parsed_positives: usize,
    passed_positives: usize,
    all_negatives: usize,
    passed_negatives: usize,
}

impl<'a> CoverageStats<'a> {
    fn new(results: &'a [CoverageResult]) -> Self {
        let (positives, negatives): (Vec<_>, Vec<_>) = results.iter().partition(|r| !r.should_fail);
        let all_positives = positives.len();
        let parsed_positives = positives.iter().filter(|r| r.parsed()).count();
        let passed_positives = positives.iter().filter(|r| r.passed()).count();
        let all_negatives = negatives.len();
        let passed_negatives = negatives.iter().filter(|r| r.passed()).count();
        Self {
            positives,
            negatives,
            all_positives,
            parsed_positives,
            passed_positives,
            all_negatives,
            passed_negatives,
        }
    }

    #[expect(clippy::cast_precision_loss)]
    fn write_summary(&self, name: &str, out: &mut String) {
        let parsed_pct = self.parsed_positives as f64 / self.all_positives as f64 * 100.0;
        let positive_pct = self.passed_positives as f64 / self.all_positives as f64 * 100.0;
        writeln!(out, "{name} Summary:").unwrap();
        writeln!(
            out,
            "AST Parsed     : {}/{} ({parsed_pct:.2}%)",
            self.parsed_positives, self.all_positives
        )
        .unwrap();
        writeln!(
            out,
            "Positive Passed: {}/{} ({positive_pct:.2}%)",
            self.passed_positives, self.all_positives
        )
        .unwrap();
        if self.all_negatives > 0 {
            let negative_pct = self.passed_negatives as f64 / self.all_negatives as f64 * 100.0;
            writeln!(
                out,
                "Negative Passed: {}/{} ({negative_pct:.2}%)",
                self.passed_negatives, self.all_negatives
            )
            .unwrap();
        }
    }

    fn print_summary(&self, name: &str) {
        let mut out = String::new();
        self.write_summary(name, &mut out);
        print!("{out}");
    }
}

pub fn print_coverage(name: &str, results: &[CoverageResult], detail: bool) {
    let stats = CoverageStats::new(results);
    stats.print_summary(name);

    if detail {
        for r in results {
            print_result(r);
        }
    }
}

fn print_result(r: &CoverageResult) {
    let path = normalize_path(Path::new("tasks/coverage").join(&r.path));
    match &r.result {
        TestResult::ParseError(error, _) => {
            println!("Expect to Parse: {path}");
            println!("{error}");
        }
        TestResult::Mismatch(case, actual, expected) => {
            println!("{case}: {path}");
            print_diff(actual, expected);
        }
        TestResult::GenericError(case, error) => {
            println!("{case} Error: {path}");
            println!("{error}");
        }
        TestResult::IncorrectlyPassed => {
            println!("Expect Syntax Error: {path}");
        }
        _ => {}
    }
}

fn print_diff(actual: &str, expected: &str) {
    let diff = TextDiff::from_lines(expected, actual);
    for change in diff.iter_all_changes() {
        let (sign, style) = match change.tag() {
            ChangeTag::Delete => ("-", Style::new().red()),
            ChangeTag::Insert => ("+", Style::new().green()),
            ChangeTag::Equal => continue,
        };
        print!("{}{}", style.apply_to(sign).bold(), style.apply_to(change));
    }
}

pub fn snapshot_results(name: &str, test_root: &Path, results: &[CoverageResult]) {
    let snapshot_path = workspace_root().join(test_root);
    let show_commit = !snapshot_path.to_string_lossy().contains("misc");
    let snapshot = Snapshot::new(&snapshot_path, show_commit);

    let mut out = String::new();
    let stats = CoverageStats::new(results);
    stats.write_summary(name, &mut out);

    // Write failed negatives (tests that should fail but didn't, e.g., IncorrectlyPassed)
    let mut failed_negatives: Vec<_> =
        stats.negatives.into_iter().filter(|r| !r.passed()).collect();
    failed_negatives.sort_by_key(|r| &r.path);

    for r in &failed_negatives {
        let path = normalize_path(Path::new("tasks/coverage").join(&r.path));
        if r.result == TestResult::IncorrectlyPassed {
            writeln!(out, "Expect Syntax Error: {path}\n").unwrap();
        }
    }

    // Write failed positive results (detailed errors)
    let mut failed_positives: Vec<_> =
        stats.positives.into_iter().filter(|r| !r.passed()).collect();
    failed_positives.sort_by_key(|r| &r.path);

    for r in &failed_positives {
        let path = normalize_path(Path::new("tasks/coverage").join(&r.path));
        match &r.result {
            TestResult::ParseError(error, panicked) => {
                let label = if *panicked { "Panicked" } else { "Expect to Parse" };
                writeln!(out, "{label}: {path}").unwrap();
                out.push_str(error);
                out.push('\n'); // Blank line after error content
            }
            TestResult::Mismatch(case, _, _) => {
                writeln!(out, "{case}: {path}\n").unwrap();
            }
            TestResult::GenericError(case, error) => {
                writeln!(out, "{case} Error: {path}").unwrap();
                out.push_str(error);
                out.push_str("\n\n"); // Extra blank line between entries
            }
            _ => {}
        }
    }

    // Write correct errors (negative tests that failed as expected)
    let mut correct_errors: Vec<_> =
        results.iter().filter(|r| matches!(r.result, TestResult::CorrectError(_, _))).collect();
    correct_errors.sort_by_key(|r| &r.path);

    for r in correct_errors {
        if let TestResult::CorrectError(error, _) = &r.result {
            out.push_str(error);
        }
    }

    let path = snap_root().join(format!("{}.snap", name.to_lowercase()));
    snapshot.save(&path, &out);
}

// ================================
// App Args & Main Entry
// ================================

#[derive(Debug, Default)]
pub struct AppArgs {
    pub debug: bool,
    pub filter: Option<String>,
    pub detail: bool,
    pub diff: bool,
}

const TEST262_PATH: &str = "test262/test";
const BABEL_PATH: &str = "babel/packages/babel-parser/test/fixtures";
const TYPESCRIPT_PATH: &str = "typescript/tests/cases";
const MISC_PATH: &str = "misc";

impl AppArgs {
    pub fn run_all(&self) {
        let data = TestData::load(self.filter.as_deref());

        self.run_parser(&data);
        self.run_semantic(&data);
        self.run_codegen(&data);
        self.run_formatter(&data);
        self.run_transformer(&data);
        self.run_minifier(&data);
        self.run_estree(&data);
    }

    fn run_tool<T>(
        &self,
        name: &str,
        test_root: &str,
        data: &[T],
        runner: fn(&[T]) -> Vec<CoverageResult>,
    ) {
        let results = runner(data);
        print_coverage(name, &results, self.detail || self.filter.is_some());
        if self.filter.is_none() {
            snapshot_results(name, Path::new(test_root), &results);
        }
    }

    pub fn run_parser(&self, data: &TestData) {
        self.run_tool("parser_test262", TEST262_PATH, &data.test262, tools::run_parser_test262);
        self.run_tool("parser_babel", BABEL_PATH, &data.babel, tools::run_parser_babel);
        self.run_tool(
            "parser_typescript",
            TYPESCRIPT_PATH,
            &data.typescript,
            tools::run_parser_typescript,
        );
        self.run_tool("parser_misc", MISC_PATH, &data.misc, tools::run_parser_misc);

        if self.filter.is_none() {
            typescript::save_reviewed_tsc_diagnostics_codes();
        }
    }

    pub fn run_semantic(&self, data: &TestData) {
        self.run_tool("semantic_test262", TEST262_PATH, &data.test262, tools::run_semantic_test262);
        self.run_tool("semantic_babel", BABEL_PATH, &data.babel, tools::run_semantic_babel);
        self.run_tool(
            "semantic_typescript",
            TYPESCRIPT_PATH,
            &data.typescript,
            tools::run_semantic_typescript,
        );
        self.run_tool("semantic_misc", MISC_PATH, &data.misc, tools::run_semantic_misc);
    }

    pub fn run_codegen(&self, data: &TestData) {
        self.run_tool("codegen_test262", TEST262_PATH, &data.test262, tools::run_codegen_test262);
        self.run_tool("codegen_babel", BABEL_PATH, &data.babel, tools::run_codegen_babel);
        self.run_tool(
            "codegen_typescript",
            TYPESCRIPT_PATH,
            &data.typescript,
            tools::run_codegen_typescript,
        );
        self.run_tool("codegen_misc", MISC_PATH, &data.misc, tools::run_codegen_misc);
    }

    pub fn run_formatter(&self, data: &TestData) {
        self.run_tool(
            "formatter_test262",
            TEST262_PATH,
            &data.test262,
            tools::run_formatter_test262,
        );
        self.run_tool("formatter_babel", BABEL_PATH, &data.babel, tools::run_formatter_babel);
        self.run_tool(
            "formatter_typescript",
            TYPESCRIPT_PATH,
            &data.typescript,
            tools::run_formatter_typescript,
        );
        self.run_tool("formatter_misc", MISC_PATH, &data.misc, tools::run_formatter_misc);
    }

    pub fn run_transformer(&self, data: &TestData) {
        self.run_tool(
            "transformer_test262",
            TEST262_PATH,
            &data.test262,
            tools::run_transformer_test262,
        );
        self.run_tool("transformer_babel", BABEL_PATH, &data.babel, tools::run_transformer_babel);
        self.run_tool(
            "transformer_typescript",
            TYPESCRIPT_PATH,
            &data.typescript,
            tools::run_transformer_typescript,
        );
        self.run_tool("transformer_misc", MISC_PATH, &data.misc, tools::run_transformer_misc);
    }

    pub fn run_minifier(&self, data: &TestData) {
        self.run_tool("minifier_test262", TEST262_PATH, &data.test262, tools::run_minifier_test262);
        self.run_tool("minifier_babel", BABEL_PATH, &data.babel, tools::run_minifier_babel);
    }

    pub fn run_estree(&self, data: &TestData) {
        self.run_tool("estree_test262", TEST262_PATH, &data.test262, tools::run_estree_test262);
        self.run_tool(
            "estree_typescript",
            TYPESCRIPT_PATH,
            &data.typescript,
            tools::run_estree_typescript,
        );
    }

    /// Run transpiler tests (isolated declarations)
    pub fn run_transpiler(&self) {
        typescript::transpile_runner::run(self.filter.as_deref(), self.detail);
    }

    /// Run runtime tests (requires Node.js subprocess)
    pub fn run_runtime(&self) {
        runtime::run(self.filter.as_deref(), self.detail);
    }
}

#[test]
#[cfg(any(coverage, coverage_nightly))]
fn test() {
    AppArgs::default().run_all();
}
