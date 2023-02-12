use std::{
    fs::{self, File},
    io::{stdout, Read, Write},
    panic::{catch_unwind, UnwindSafe},
    path::{Path, PathBuf},
    result::Result,
};

use console::Style;
use encoding_rs::UTF_16LE;
use encoding_rs_io::DecodeReaderBytesBuilder;
use oxc_allocator::Allocator;
use oxc_ast::SourceType;
use oxc_parser::Parser;
use rayon::prelude::*;
use similar::{ChangeTag, TextDiff};
use walkdir::WalkDir;

use crate::{project_root, AppArgs};

#[derive(Debug)]
pub enum TestResult {
    ToBeRun,
    Passed,
    IncorrectlyPassed,
    ParseError(String),
    Mismatch(String, String),
    CorrectError(String),
}

#[allow(unused)]
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
        self.read_test_cases(args);
        let report = self.coverage_report();

        let mut out = stdout();

        self.print_coverage(name, args, &report, &mut out).unwrap();

        if args.filter.is_none() {
            self.snapshot_errors(name, &report).unwrap();
        }
    }

    fn get_test_root(&self) -> &Path;

    fn skip_test_path(&self, path: &Path) -> bool;

    fn save_test_cases(&mut self, cases: Vec<T>);

    fn read_test_cases(&mut self, args: &AppArgs) {
        let filter = args.filter.as_ref();

        let test_root = self.get_test_root();
        // get all files paths
        let paths = WalkDir::new(test_root)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
            .map(|e| e.path().to_owned())
            .filter(|path| !self.skip_test_path(path))
            .filter(|path| filter.map_or(true, |query| path.to_string_lossy().contains(query)))
            .collect::<Vec<_>>();

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
            .filter_map(|mut case| {
                let path = case.path().to_path_buf();
                catch_unwind(move || {
                    case.run();
                    Some(case)
                })
                .unwrap_or_else(|_| {
                    println!("panic: {path:?}");
                    None
                })
            })
            .collect::<Vec<_>>();

        self.save_test_cases(cases);
    }

    fn get_test_cases(&self) -> &Vec<T>;

    fn coverage_report(&self) -> CoverageReport<T> {
        let tests = self.get_test_cases();

        let (negatives, positives): (Vec<_>, Vec<_>) =
            tests.iter().partition(|case| case.should_fail());

        let all_positives = positives.len();
        let not_parsed_positives = positives.iter().filter(|case| !case.test_parsed()).count();

        let mut failed_positives =
            positives.into_iter().filter(|case| !case.test_passed()).collect::<Vec<_>>();

        failed_positives.sort_by_key(|case| case.path().to_string_lossy().to_string());

        let passed_positives = all_positives - failed_positives.len();
        let parsed_positives = all_positives - not_parsed_positives;

        let all_negatives = negatives.len();
        let mut failed_negatives =
            negatives.into_iter().filter(|case| !case.test_passed()).collect::<Vec<_>>();
        failed_negatives.sort_by_key(|case| case.path().to_string_lossy().to_string());

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

            // passed_positives,
            // passed_negatives,
            all_positives,
            // all_negatives,
            ..
        } = report;

        let parsed_diff = (*parsed_positives as f64 / *all_positives as f64) * 100.0;
        // let positive_diff = (*passed_positives as f64 / *all_positives as f64) * 100.0;
        // let negative_diff = (*passed_negatives as f64 / *all_negatives as f64) * 100.0;
        writer.write_all(format!("{name} Summary:\n").as_bytes())?;
        let msg =
            format!("AST Parsed     : {parsed_positives}/{all_positives} ({parsed_diff:.2}%)\n");
        writer.write_all(msg.as_bytes())?;
        // let msg =
        // format!("Positive Passed: {passed_positives}/{all_positives} ({positive_diff:.2}%)\n");
        // writer.write_all(msg.as_bytes())?;
        // let msg =
        // format!("Negative Passed: {passed_negatives}/{all_negatives} ({negative_diff:.2}%)\n");
        // writer.write_all(msg.as_bytes())?;

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
            .filter(|case| matches!(case.test_result(), TestResult::CorrectError(_)))
            .collect::<Vec<_>>();

        tests.sort_by_key(|case| case.path().to_string_lossy().to_string());

        let args = AppArgs { detail: true, ..AppArgs::default() };
        self.print_coverage(name, &args, report, &mut file)?;

        let mut out = String::new();
        for case in &tests {
            if let TestResult::CorrectError(error) = &case.test_result() {
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
        matches!(result, TestResult::Passed | TestResult::CorrectError(_))
    }

    fn test_parsed(&self) -> bool {
        let result = self.test_result();
        assert!(!matches!(result, TestResult::ToBeRun), "test should be run");
        matches!(result, TestResult::Passed | TestResult::Mismatch(_, _))
    }

    /// Run a single test case, this is responsible for saving the test result
    fn run(&mut self);

    /// Execute the parser once and get the test result
    fn execute(&mut self, source_type: SourceType) -> TestResult {
        let allocator = Allocator::default();
        let source = self.code();
        let ret = Parser::new(&allocator, source, source_type).parse();
        let passed = ret.errors.is_empty();
        let result = if passed { Ok(String::new()) } else { Err(String::new()) };
        self.parser_return_to_test_result(result)
    }

    fn parser_return_to_test_result(&self, result: Result<String, String>) -> TestResult {
        let should_fail = self.should_fail();
        match result {
            Err(err) if should_fail => TestResult::CorrectError(err),
            Err(err) if !should_fail => TestResult::ParseError(err),
            Ok(_) if should_fail => TestResult::IncorrectlyPassed,
            Ok(_) if !should_fail => TestResult::Passed,
            _ => unreachable!(),
        }
    }

    fn print<W: Write>(&self, args: &AppArgs, writer: &mut W) -> std::io::Result<()> {
        match self.test_result() {
            TestResult::ParseError(error) => {
                writer.write_all(format!("Expect to Parse: {:?}\n", self.path()).as_bytes())?;
                writer.write_all(error.as_bytes())?;
            }
            TestResult::Mismatch(ast_string, expected_ast_string) => {
                if args.diff {
                    self.print_diff(writer, ast_string.as_str(), expected_ast_string.as_str())?;
                    println!("Mismatch: {:?}", self.path());
                }
            }
            // TestResult::IncorrectlyPassed => {
            // writer.write_all(format!("Expect Syntax Error: {:?}\n", self.path()).as_bytes())?;
            // }
            _ => {}
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
}
