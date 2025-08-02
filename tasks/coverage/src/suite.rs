use std::{
    borrow::Cow,
    io::{Write, stdout},
    panic::UnwindSafe,
    path::{Path, PathBuf},
};

use futures::{StreamExt, stream};
use oxc::{
    diagnostics::{GraphicalReportHandler, GraphicalTheme, NamedSource},
    span::SourceType,
};
use oxc_tasks_common::normalize_path;
use rayon::prelude::*;
use tokio::runtime::Runtime;

use crate::{
    AppArgs, Driver,
    coverage::{CoverageReport, print_diff},
    driver::DriverOptions,
    snapshot_manager::SnapshotManager,
    test_reader::TestReader,
};

#[derive(Debug, PartialEq, Eq)]
pub enum TestResult {
    ToBeRun,
    Passed,
    IncorrectlyPassed,
    Mismatch(/* case */ &'static str, /* actual */ String, /* expected */ String),
    ParseError(String, /* panicked */ bool),
    CorrectError(String, /* panicked */ bool),
    GenericError(/* case */ &'static str, /* error */ String),
}

/// A Test Suite is responsible for reading code from a repository
pub trait Suite<T: Case>: Sync {
    fn run(&mut self, name: &str, args: &AppArgs) {
        self.read_test_cases(name, args);

        if args.debug {
            self.get_test_cases_mut().iter_mut().for_each(|case| {
                println!("{}", case.path().to_string_lossy());
                case.run();
            });
        } else {
            self.get_test_cases_mut().par_iter_mut().for_each(Case::run);
        }

        self.run_coverage(name, args);
    }

    fn run_async(&mut self, args: &AppArgs) {
        self.read_test_cases("runtime", args);
        let cases = self.get_test_cases_mut().iter_mut().map(T::run_async);
        Runtime::new().unwrap().block_on(stream::iter(cases).buffer_unordered(100).count());
        self.run_coverage("runtime", args);
        let _ = oxc_tasks_common::agent().delete("http://localhost:32055").call();
    }

    fn run_coverage(&self, name: &str, args: &AppArgs) {
        let report = CoverageReport::from_test_cases(self.get_test_cases());
        let mut out = stdout();
        report.print(name, args, &mut out).unwrap();
        if args.filter.is_none() {
            SnapshotManager::snapshot_errors(
                name,
                self.get_test_root(),
                self.get_test_cases(),
                &report,
            )
            .unwrap();
        }
    }

    fn get_test_root(&self) -> &Path;

    fn skip_test_path(&self, _path: &Path) -> bool {
        false
    }

    fn save_test_cases(&mut self, cases: Vec<T>);
    fn save_extra_test_cases(&mut self) {}

    fn read_test_cases(&mut self, name: &str, args: &AppArgs) {
        let reader = TestReader::new(self.get_test_root().to_path_buf());
        let cases = reader.read_test_cases(name, args, |path| self.skip_test_path(path));
        self.save_test_cases(cases);
        if args.filter.is_none() {
            self.save_extra_test_cases();
        }
    }

    fn get_test_cases_mut(&mut self) -> &mut Vec<T>;
    fn get_test_cases(&self) -> &Vec<T>;
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

    /// Mark strict mode as always strict
    ///
    /// See <https://github.com/tc39/test262/blob/05c45a4c430ab6fee3e0c7f0d47d8a30d8876a6d/INTERPRETING.md#strict-mode>
    fn always_strict(&self) -> bool {
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
    #[expect(clippy::unused_async)]
    async fn run_async(&mut self) {}

    fn parse(&self, code: &str, source_type: SourceType) -> Result<(), (String, bool)> {
        let path = self.path();

        let mut driver = Driver {
            path: path.to_path_buf(),
            options: DriverOptions {
                allow_return_outside_function: self.allow_return_outside_function(),
                ..DriverOptions::default()
            },
            ..Driver::default()
        };

        let source_text = if self.always_strict() {
            // To run in strict mode, the test contents must be modified prior to execution--
            // a "use strict" directive must be inserted as the initial character sequence of the file,
            // followed by a semicolon (;) and newline character (\n): "use strict";
            Cow::Owned(format!("'use strict';\n{code}"))
        } else {
            Cow::Borrowed(code)
        };

        driver.run(&source_text, source_type);
        let errors = driver.errors();
        if errors.is_empty() {
            Ok(())
        } else {
            let handler =
                GraphicalReportHandler::new().with_theme(GraphicalTheme::unicode_nocolor());
            let mut output = String::new();
            for error in errors {
                let error = error.with_source_code(NamedSource::new(
                    normalize_path(path),
                    source_text.to_string(),
                ));
                handler.render_report(&mut output, error.as_ref()).unwrap();
            }
            Err((output, driver.results.panicked))
        }
    }

    /// Execute the parser once and get the test result
    fn execute(&mut self, source_type: SourceType) -> TestResult {
        let result = self.parse(self.code(), source_type);
        self.evaluate_result(result)
    }

    fn evaluate_result(&self, result: Result<(), (String, bool)>) -> TestResult {
        let should_fail = self.should_fail();
        match result {
            Err((err, panicked)) if should_fail => TestResult::CorrectError(err, panicked),
            Err((err, panicked)) if !should_fail => TestResult::ParseError(err, panicked),
            Ok(()) if should_fail => TestResult::IncorrectlyPassed,
            Ok(()) if !should_fail => TestResult::Passed,
            _ => unreachable!(),
        }
    }

    fn print<W: Write>(&self, args: &AppArgs, writer: &mut W) -> std::io::Result<()> {
        let path = normalize_path(Path::new("tasks/coverage").join(self.path()));
        match self.test_result() {
            TestResult::ParseError(error, _) => {
                writer.write_all(format!("Expect to Parse: {path}\n").as_bytes())?;
                writer.write_all(error.as_bytes())?;
            }
            TestResult::Mismatch(case, ast_string, expected_ast_string) => {
                writer.write_all(format!("{case}: {path}\n",).as_bytes())?;
                if args.diff {
                    print_diff(writer, ast_string.as_str(), expected_ast_string.as_str())?;
                    println!("{case}: {path}");
                }
            }
            TestResult::GenericError(case, error) => {
                writer.write_all(format!("{case} Error: {path}\n",).as_bytes())?;
                writer.write_all(format!("{error}\n").as_bytes())?;
            }
            TestResult::IncorrectlyPassed => {
                writer.write_all(format!("Expect Syntax Error: {path}\n").as_bytes())?;
            }
            TestResult::Passed | TestResult::ToBeRun | TestResult::CorrectError(..) => {}
        }
        writer.write_all(b"\n")?;
        Ok(())
    }
}
