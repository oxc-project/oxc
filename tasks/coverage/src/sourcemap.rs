use crate::suite::{Case, Suite, TestResult};
use base64::{prelude::BASE64_STANDARD, Engine};
use oxc_span::SourceType;
use oxc_tasks_common::project_root;
use std::io::Write;
use std::{
    fs::File,
    path::{Path, PathBuf},
};

static FIXTURES_PATH: &str =
    "tasks/coverage/babel/packages/babel-generator/test/fixtures/sourcemaps";

pub struct SourcemapSuite<T: Case> {
    test_root: PathBuf,
    test_cases: Vec<T>,
}

impl<T: Case> SourcemapSuite<T> {
    pub fn new() -> Self {
        Self { test_root: project_root().join(FIXTURES_PATH), test_cases: vec![] }
    }
}

impl<T: Case> Suite<T> for SourcemapSuite<T> {
    fn get_test_root(&self) -> &Path {
        &self.test_root
    }

    fn save_test_cases(&mut self, tests: Vec<T>) {
        self.test_cases = tests;
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
        let path = project_root().join(format!("tasks/coverage/{}.snap", name));
        let mut file = File::create(path).unwrap();

        let mut tests = self.get_test_cases().iter().collect::<Vec<_>>();
        tests.sort_by_key(|case| case.path());

        for case in tests {
            let result = case.test_result();
            let path = case.path().to_string_lossy();
            let result = match result {
                TestResult::Snapshot(snapshot) => {
                    let snapshot = snapshot.trim();
                    let snapshot = snapshot.replace("\r\n", "\n");
                    snapshot
                }
                TestResult::ParseError(error, _) => {
                    let error = error.trim();
                    error.to_string()
                }
                _ => {
                    unreachable!()
                }
            };
            writeln!(file, "- {}", path).unwrap();
            writeln!(file, "{}\n\n", result).unwrap();
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

    fn create_visualizer_url(code: &str, map: &str) -> String {
        let hash =
            BASE64_STANDARD.encode(format!("{}\0{}{}\0{}", code.len(), code, map.len(), map));
        format!("https://evanw.github.io/source-map-visualization/#{}", hash)
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
        let allocator = oxc_allocator::Allocator::default();
        let ret = oxc_parser::Parser::new(&allocator, source_text, source_type).parse();

        if !ret.errors.is_empty() {
            for error in ret.errors {
                let error = error.with_source_code(source_text.to_string());
                return TestResult::ParseError(error.to_string(), false);
            }
        }

        let codegen_options = oxc_codegen::CodegenOptions::default();
        let (content, map) = oxc_codegen::Codegen::<false>::new(source_text.len(), codegen_options)
            .build_with_sourcemap(&ret.program, source_text, "");
        let mut buff = vec![];
        map.to_writer(&mut buff).unwrap();

        let mut result = String::from_utf8(buff).unwrap();
        result.push_str("\n");
        result.push_str(Self::create_visualizer_url(&content, &result).as_str());

        TestResult::Snapshot(result)
    }
}
