//! TypeScript transpiler tests (isolated declarations)
//! <https://github.com/microsoft/TypeScript/blob/v5.6.3/src/testRunner/transpileRunner.ts>

use std::path::Path;

use oxc::{
    allocator::Allocator,
    codegen::{Codegen, CodegenOptions, CommentOptions},
    isolated_declarations::{IsolatedDeclarations, IsolatedDeclarationsOptions},
    parser::Parser,
    span::SourceType,
};
use rayon::prelude::*;
use walkdir::WalkDir;

use super::meta::{Baseline, BaselineFile, TestCaseContent};
use crate::{CoverageResult, TestResult, print_coverage, snapshot_results, workspace_root};

const TESTS_ROOT: &str = "typescript/tests";

pub fn run(filter: Option<&str>, detail: bool) {
    let results = run_transpile(filter);
    print_coverage("transpile", &results, detail);
    if filter.is_none() {
        snapshot_results("transpile", Path::new("typescript/tests/cases/transpile"), &results);
    }
}

fn run_transpile(filter: Option<&str>) -> Vec<CoverageResult> {
    let test_root = workspace_root().join(TESTS_ROOT).join("cases").join("transpile");

    let paths: Vec<_> = WalkDir::new(&test_root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| !e.file_type().is_dir())
        .filter(|e| {
            e.path().extension().is_some_and(|ext| ext == "ts" || ext == "tsx" || ext == "mts")
        })
        .map(|e| e.path().to_owned())
        .filter(|path| filter.is_none_or(|q| path.to_string_lossy().contains(q)))
        .collect();

    paths
        .into_par_iter()
        .filter_map(|path| {
            let code = std::fs::read_to_string(&path).ok()?;
            let content = TestCaseContent::make_units_from_test(&path, &code);

            // Skip tests that don't have declaration flag
            if !content.settings.declaration {
                return None;
            }

            let result = compare_baseline(&path, &content);
            let rel_path = path.strip_prefix(workspace_root()).ok()?.to_owned();
            Some(CoverageResult { path: rel_path, should_fail: false, result })
        })
        .collect()
}

fn compare_baseline(path: &Path, content: &TestCaseContent) -> TestResult {
    // Get expected baseline
    let rel_path = path
        .strip_prefix(workspace_root().join(TESTS_ROOT).join("cases/transpile"))
        .unwrap_or(path);
    let filename = change_extension(rel_path.to_str().unwrap_or_default());
    let baseline_path =
        workspace_root().join(TESTS_ROOT).join("baselines/reference/transpile").join(&filename);

    let expected = BaselineFile::parse(&baseline_path);
    let actual = run_isolated_declarations(path, content);

    let expected_text = expected.print();
    let actual_text = actual.print();

    if expected.files.len() != actual.files.len() {
        return TestResult::Mismatch("Mismatch", actual_text, expected_text);
    }

    for (actual_file, expected_file) in actual.files.iter().zip(expected.files.iter()) {
        if expected_file.original_diagnostic.is_empty() {
            // No diagnostics expected - compare printed output
            if actual_file.oxc_printed != expected_file.oxc_printed {
                return TestResult::Mismatch(
                    "Mismatch",
                    actual_file.oxc_printed.clone(),
                    expected_file.oxc_printed.clone(),
                );
            }
        } else {
            // Diagnostics expected - check if they match
            let matched = actual_file
                .oxc_diagnostics
                .iter()
                .zip(expected_file.original_diagnostic.iter())
                .all(|(actual_diag, expected_diag)| {
                    expected_diag.contains(&actual_diag.to_string())
                });
            if !matched {
                let rel_path = path
                    .strip_prefix(workspace_root())
                    .map_or_else(|_| path.display().to_string(), |p| p.display().to_string());
                let snapshot = format!("\n#### {} ####\n{}", rel_path, actual.snapshot());
                return TestResult::CorrectError(snapshot, false);
            }
        }
    }

    TestResult::Passed
}

fn run_isolated_declarations(path: &Path, content: &TestCaseContent) -> BaselineFile {
    let mut files = vec![];

    // Add source files
    for unit in &content.tests {
        let mut baseline = Baseline {
            name: unit.name.clone(),
            original: unit.content.clone(),
            ..Baseline::default()
        };
        baseline.print_oxc();
        files.push(baseline);
    }

    // Add transpiled files
    for unit in &content.tests {
        let (source_text, errors) = transpile(path, &unit.content);
        let baseline = Baseline {
            name: change_extension(&unit.name),
            original: unit.content.clone(),
            original_diagnostic: Vec::default(),
            oxc_printed: source_text,
            oxc_diagnostics: errors,
        };
        files.push(baseline);
    }

    BaselineFile { files }
}

fn change_extension(name: &str) -> String {
    Path::new(name)
        .with_extension("")
        .with_extension("d.ts")
        .to_str()
        .unwrap_or_default()
        .to_string()
}

fn transpile(path: &Path, source_text: &str) -> (String, Vec<oxc::diagnostics::OxcDiagnostic>) {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).unwrap_or_default();
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    let ret =
        IsolatedDeclarations::new(&allocator, IsolatedDeclarationsOptions { strip_internal: true })
            .build(&ret.program);
    let printed = Codegen::new()
        .with_options(CodegenOptions {
            comments: CommentOptions { jsdoc: true, ..CommentOptions::disabled() },
            ..CodegenOptions::default()
        })
        .build(&ret.program)
        .code;
    (printed, ret.errors)
}
