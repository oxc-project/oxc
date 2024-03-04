use std::{
    fs::{self, File},
    io::{stdout, Read, Write},
    panic::UnwindSafe,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    result::Result,
};

use console::Style;
use encoding_rs::UTF_16LE;
use encoding_rs_io::DecodeReaderBytesBuilder;
use futures::future::join_all;
use oxc_allocator::Allocator;
use oxc_diagnostics::{miette::NamedSource, GraphicalReportHandler, GraphicalTheme};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_tasks_common::normalize_path;
use rayon::prelude::*;
use similar::{ChangeTag, TextDiff};
use tokio::runtime::Runtime;
use walkdir::WalkDir;

use crate::{project_root, AppArgs};

#[derive(Debug)]
pub enum TestResult {
    ToBeRun,
    Passed,
    IncorrectlyPassed,
    #[allow(unused)]
    Mismatch(String, String),
    ParseError(String, /* panicked */ bool),
    CorrectError(String, /* panicked */ bool),
    RuntimeError(String),
    CodegenError(/* reason */ &'static str),
    Snapshot(String),
}

pub struct CoverageReport<'a, T> {
    failed_positives: Vec<&'a T>,
    failed_negatives: Vec<&'a T>,
    parsed_positives: usize,
    passed_positives: usize,
    passed_negatives: usize,
    all_positives: usize,
    all_negatives: usize,
}

/// A Test Suite is responsible for reading code from a repository
pub trait Suite<T: Case> {
    fn run(&mut self, name: &str, args: &AppArgs) {
        self.read_test_cases(name, args);
        let cases = self.get_test_cases_mut();
        for case in cases {
            case.run();
        }
        self.run_coverage(name, args);
    }

    fn run_async(&mut self, name: &str, args: &AppArgs) {
        let rt = Runtime::new().unwrap();
        self.read_test_cases(name, args);

        let cases = self.get_test_cases_mut();

        rt.block_on(async {
            join_all(cases.iter_mut().map(T::run_async)).await;
        });

        self.run_coverage(name, args);
    }

    fn run_coverage(&self, name: &str, args: &AppArgs) {
        let report = self.coverage_report();

        let mut out = stdout();

        self.print_coverage(name, args, &report, &mut out).unwrap();

        if args.filter.is_none() {
            self.snapshot_errors(name, &report).unwrap();
        }
    }

    fn get_test_root(&self) -> &Path;

    fn skip_test_path(&self, _path: &Path) -> bool {
        false
    }

    fn save_test_cases(&mut self, cases: Vec<T>);

    fn read_test_cases(&mut self, name: &str, args: &AppArgs) {
        let test_root = self.get_test_root();

        let get_paths = || {
            let filter = args.filter.as_ref();
            WalkDir::new(test_root)
                .into_iter()
                .filter_map(Result::ok)
                .filter(|e| !e.file_type().is_dir())
                .map(|e| e.path().to_owned())
                .filter(|path| !self.skip_test_path(path))
                .filter(|path| filter.map_or(true, |query| path.to_string_lossy().contains(query)))
                .collect::<Vec<_>>()
        };

        let mut paths = get_paths();

        // Initialize git submodule if it is empty and no filter is provided
        if paths.is_empty() && args.filter.is_none() {
            println!("-------------------------------------------------------");
            println!("git submodule is empty for {name}");
            println!("Running `just submodules` to clone the submodules");
            println!("This may take a while.");
            println!("-------------------------------------------------------");
            Command::new("just")
                .args(["submodules"])
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .output()
                .expect("failed to execute `just submodules`");
            paths = get_paths();
        }

        // read all files, run the tests and save them
        let cases = paths
            .into_par_iter()
            .map(|path| {
                let code = fs::read_to_string(&path).unwrap_or_else(|_| {
                    // TypeScript tests may contain utf_16 encoding files
                    let file = fs::File::open(&path).unwrap();
                    let mut content = String::new();
                    DecodeReaderBytesBuilder::new()
                        .encoding(Some(UTF_16LE))
                        .build(file)
                        .read_to_string(&mut content)
                        .unwrap();
                    content
                });

                let path = path.strip_prefix(test_root).unwrap().to_owned();
                // remove the Byte Order Mark in some of the TypeScript files
                let code = code.trim_start_matches(|c| c == '\u{feff}').to_string();
                T::new(path, code)
            })
            .filter(|case| !case.skip_test_case())
            .collect::<Vec<_>>();

        self.save_test_cases(cases);
    }

    fn get_test_cases_mut(&mut self) -> &mut Vec<T>;
    fn get_test_cases(&self) -> &Vec<T>;

    fn coverage_report(&self) -> CoverageReport<T> {
        let tests = self.get_test_cases();

        let (negatives, positives): (Vec<_>, Vec<_>) =
            tests.iter().partition(|case| case.should_fail());

        let all_positives = positives.len();
        let parsed_positives = positives.iter().filter(|case| case.test_parsed()).count();

        let mut failed_positives =
            positives.into_iter().filter(|case| !case.test_passed()).collect::<Vec<_>>();

        failed_positives.sort_by_key(|case| case.path());

        let passed_positives = all_positives - failed_positives.len();

        let all_negatives = negatives.len();
        let mut failed_negatives =
            negatives.into_iter().filter(|case| !case.test_passed()).collect::<Vec<_>>();
        failed_negatives.sort_by_key(|case| case.path());

        let passed_negatives = all_negatives - failed_negatives.len();

        CoverageReport {
            failed_positives,
            failed_negatives,
            parsed_positives,
            passed_positives,
            passed_negatives,
            all_positives,
            all_negatives,
        }
    }

    /// # Errors
    #[allow(clippy::cast_precision_loss)]
    fn print_coverage<W: Write>(
        &self,
        name: &str,
        args: &AppArgs,
        report: &CoverageReport<T>,
        writer: &mut W,
    ) -> std::io::Result<()> {
        let CoverageReport {
            parsed_positives,
            passed_positives,
            passed_negatives,
            all_positives,
            all_negatives,
            ..
        } = report;

        let parsed_diff = (*parsed_positives as f64 / *all_positives as f64) * 100.0;
        let positive_diff = (*passed_positives as f64 / *all_positives as f64) * 100.0;
        let negative_diff = (*passed_negatives as f64 / *all_negatives as f64) * 100.0;
        writer.write_all(format!("{name} Summary:\n").as_bytes())?;
        let msg =
            format!("AST Parsed     : {parsed_positives}/{all_positives} ({parsed_diff:.2}%)\n");
        writer.write_all(msg.as_bytes())?;
        let msg =
            format!("Positive Passed: {passed_positives}/{all_positives} ({positive_diff:.2}%)\n");
        writer.write_all(msg.as_bytes())?;
        if *all_negatives > 0 {
            let msg = format!(
                "Negative Passed: {passed_negatives}/{all_negatives} ({negative_diff:.2}%)\n"
            );
            writer.write_all(msg.as_bytes())?;
        }

        if args.should_print_detail() {
            for case in &report.failed_negatives {
                case.print(args, writer)?;
            }
            for case in &report.failed_positives {
                case.print(args, writer)?;
            }
        }
        writer.flush()?;
        Ok(())
    }

    /// # Errors
    fn snapshot_errors(&self, name: &str, report: &CoverageReport<T>) -> std::io::Result<()> {
        let path = project_root().join(format!("tasks/coverage/{}.snap", name.to_lowercase()));
        let mut file = File::create(path).unwrap();

        let mut tests = self
            .get_test_cases()
            .iter()
            .filter(|case| matches!(case.test_result(), TestResult::CorrectError(_, _)))
            .collect::<Vec<_>>();

        tests.sort_by_key(|case| case.path());

        let args = AppArgs { detail: true, ..AppArgs::default() };
        self.print_coverage(name, &args, report, &mut file)?;

        let mut out = String::new();
        for case in &tests {
            if let TestResult::CorrectError(error, _) = &case.test_result() {
                out.push_str(error);
            }
        }

        file.write_all(out.as_bytes())?;
        file.flush()?;
        Ok(())
    }
}

/// A Test Case is responsible for interpreting the contents of a file
pub trait Case: Sized + Sync + Send + UnwindSafe {
    fn new(path: PathBuf, code: String) -> Self;

    fn code(&self) -> &str;
    fn path(&self) -> &Path;
    fn allow_return_outside_function(&self) -> bool {
        false
    }
    fn test_result(&self) -> &TestResult;

    fn should_fail(&self) -> bool {
        false
    }

    fn skip_test_case(&self) -> bool {
        false
    }

    fn test_passed(&self) -> bool {
        let result = self.test_result();
        assert!(!matches!(result, TestResult::ToBeRun), "test should be run");
        matches!(result, TestResult::Passed | TestResult::CorrectError(_, _))
    }

    fn test_parsed(&self) -> bool {
        let result = self.test_result();
        assert!(!matches!(result, TestResult::ToBeRun), "test should be run");
        match result {
            TestResult::ParseError(_, panicked) | TestResult::CorrectError(_, panicked) => {
                !panicked
            }
            _ => true,
        }
    }

    /// Run a single test case, this is responsible for saving the test result
    fn run(&mut self);

    /// Async version of run
    #[allow(clippy::unused_async)]
    async fn run_async(&mut self) {}

    /// Execute the parser once and get the test result
    fn execute(&mut self, source_type: SourceType) -> TestResult {
        let allocator = Allocator::default();
        let source_text = self.code();
        let parser_ret = Parser::new(&allocator, source_text, source_type)
            .allow_return_outside_function(self.allow_return_outside_function())
            .parse();

        // Make sure serialization doesn't crash for ast and hir, also for code coverage.
        let _json = parser_ret.program.to_json();

        let program = allocator.alloc(parser_ret.program);
        let semantic_ret = SemanticBuilder::new(source_text, source_type)
            .with_trivias(parser_ret.trivias)
            .with_check_syntax_error(true)
            .build_module_record(PathBuf::new(), program)
            .build(program);
        if let Some(res) = self.check_semantic(&semantic_ret.semantic) {
            return res;
        }
        let errors = parser_ret.errors.into_iter().chain(semantic_ret.errors).collect::<Vec<_>>();

        let result = if errors.is_empty() {
            Ok(String::new())
        } else {
            let handler =
                GraphicalReportHandler::new().with_theme(GraphicalTheme::unicode_nocolor());
            let mut output = String::new();
            for error in errors {
                let error = error.with_source_code(NamedSource::new(
                    normalize_path(self.path()),
                    source_text.to_string(),
                ));
                handler.render_report(&mut output, error.as_ref()).unwrap();
            }
            Err(output)
        };

        let should_fail = self.should_fail();
        match result {
            Err(err) if should_fail => TestResult::CorrectError(err, parser_ret.panicked),
            Err(err) if !should_fail => TestResult::ParseError(err, parser_ret.panicked),
            Ok(_) if should_fail => TestResult::IncorrectlyPassed,
            Ok(_) if !should_fail => TestResult::Passed,
            _ => unreachable!(),
        }
    }

    fn print<W: Write>(&self, args: &AppArgs, writer: &mut W) -> std::io::Result<()> {
        match self.test_result() {
            TestResult::ParseError(error, _) => {
                writer.write_all(
                    format!("Expect to Parse: {:?}\n", normalize_path(self.path())).as_bytes(),
                )?;
                writer.write_all(error.as_bytes())?;
            }
            TestResult::Mismatch(ast_string, expected_ast_string) => {
                if args.diff {
                    self.print_diff(writer, ast_string.as_str(), expected_ast_string.as_str())?;
                    println!("Mismatch: {:?}", normalize_path(self.path()));
                }
            }
            TestResult::RuntimeError(error) => {
                writer.write_all(
                    format!("Expect to run correctly: {:?}\n", normalize_path(self.path()))
                        .as_bytes(),
                )?;
                writer.write_all(format!("But got a runtime error: {error}\n\n").as_bytes())?;
            }
            TestResult::IncorrectlyPassed => {
                writer.write_all(
                    format!("Expect Syntax Error: {:?}\n", normalize_path(self.path())).as_bytes(),
                )?;
            }
            TestResult::CodegenError(reason) => {
                writer.write_all(
                    format!("{reason} failed: {:?}\n", normalize_path(self.path())).as_bytes(),
                )?;
            }
            TestResult::Snapshot(snapshot) => {
                writer.write_all(snapshot.as_bytes())?;
            }
            TestResult::Passed | TestResult::ToBeRun | TestResult::CorrectError(..) => {}
        }
        Ok(())
    }

    fn print_diff<W: Write>(
        &self,
        writer: &mut W,
        origin_string: &str,
        expected_string: &str,
    ) -> std::io::Result<()> {
        let diff = TextDiff::from_lines(origin_string, expected_string);
        for change in diff.iter_all_changes() {
            let (sign, style) = match change.tag() {
                ChangeTag::Delete => ("-", Style::new().red()),
                ChangeTag::Insert => ("+", Style::new().green()),
                ChangeTag::Equal => continue, // (" ", Style::new()),
            };
            writer.write_all(
                format!("{}{}", style.apply_to(sign).bold(), style.apply_to(change)).as_bytes(),
            )?;
        }
        Ok(())
    }

    fn check_semantic(&self, _semantic: &oxc_semantic::Semantic<'_>) -> Option<TestResult> {
        None
    }
}
