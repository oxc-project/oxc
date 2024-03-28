use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use oxc_allocator::Allocator;
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_parser::Parser;
use oxc_sourcemap::SourcemapVisualizer;
use oxc_span::SourceType;
use oxc_tasks_common::{project_root, TestFiles};

use crate::suite::{Case, Suite, TestResult};

static FIXTURES_PATH: &str =
    "tasks/coverage/babel/packages/babel-generator/test/fixtures/sourcemaps";

pub struct SourcemapSuite<T: Case> {
    test_root: PathBuf,
    test_cases: Vec<T>,
}

impl<T: Case> SourcemapSuite<T> {
    pub fn new() -> Self {
        Self {
            test_root: project_root().join(FIXTURES_PATH),
            test_cases: TestFiles::new()
                .files()
                .iter()
                .filter(|file| file.file_name.contains("react"))
                .map(|file| T::new(file.file_name.clone().into(), file.source_text.clone()))
                .collect::<Vec<_>>(),
        }
    }
}

impl<T: Case> Suite<T> for SourcemapSuite<T> {
    fn get_test_root(&self) -> &Path {
        &self.test_root
    }

    fn save_test_cases(&mut self, tests: Vec<T>) {
        self.test_cases.extend(tests);
    }

    fn get_test_cases(&self) -> &Vec<T> {
        &self.test_cases
    }

    fn get_test_cases_mut(&mut self) -> &mut Vec<T> {
        &mut self.test_cases
    }

    fn skip_test_path(&self, path: &Path) -> bool {
        let path = path.to_string_lossy();
        !path.contains("input.js")
    }

    fn run_coverage(&self, name: &str, _args: &crate::AppArgs) {
        let path = project_root().join(format!("tasks/coverage/{name}.snap"));
        let mut file = File::create(path).unwrap();

        let mut tests = self.get_test_cases().iter().collect::<Vec<_>>();
        tests.sort_by_key(|case| case.path());

        for case in tests {
            let result = case.test_result();
            let path = case.path().to_string_lossy();
            let result = match result {
                TestResult::Snapshot(snapshot) => snapshot.to_string(),
                TestResult::ParseError(error, _) => format!("- {path}\n{error}"),
                _ => {
                    unreachable!()
                }
            };
            writeln!(file, "{result}\n\n").unwrap();
        }
    }
}

pub struct SourcemapCase {
    path: PathBuf,
    code: String,
    source_type: SourceType,
    result: TestResult,
}

impl SourcemapCase {
    pub fn source_type(&self) -> SourceType {
        self.source_type
    }
}

impl Case for SourcemapCase {
    fn new(path: PathBuf, code: String) -> Self {
        let source_type = SourceType::from_path(&path).unwrap();
        Self { path, code, source_type, result: TestResult::ToBeRun }
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

    fn run(&mut self) {
        let source_type = self.source_type();
        self.result = self.execute(source_type);
    }

    fn execute(&mut self, source_type: SourceType) -> TestResult {
        let source_text = self.code();
        let allocator = Allocator::default();
        let ret = Parser::new(&allocator, source_text, source_type).parse();

        if !ret.errors.is_empty() {
            if let Some(error) = ret.errors.into_iter().next() {
                let error = error.with_source_code(source_text.to_string());
                return TestResult::ParseError(error.to_string(), false);
            }
        }

        let codegen_options =
            CodegenOptions { enable_source_map: true, ..CodegenOptions::default() };
        let codegen_ret = Codegen::<false>::new(
            self.path.to_string_lossy().as_ref(),
            source_text,
            codegen_options,
        )
        .build(&ret.program);

        TestResult::Snapshot(
            SourcemapVisualizer::new(&codegen_ret.source_text, &codegen_ret.source_map.unwrap())
                .into_visualizer_text(),
        )
    }
}
