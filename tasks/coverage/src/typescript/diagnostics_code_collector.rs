use std::{fs, path::Path};

use oxc_tasks_common::Snapshot;
use rustc_hash::{FxHashMap, FxHashSet};
use walkdir::WalkDir;

use crate::{snap_root, workspace_root};

const TESTS_ROOT: &str = "typescript/tests";

pub fn save_reviewed_tsc_diagnostics_codes() {
    // Collected codes are based on current TypeScript version in submodule.
    let snapshot = Snapshot::new(&workspace_root().join(TESTS_ROOT), true);

    let mut contents = vec![];
    contents.extend([
        "---".to_string(),
        "NOTE: If you are seeing this file with Git diffs, please follow the instructions below to review the changes.".to_string(),
        String::new(),
        "Q. Is it possible for `oxc_parser` to detect and report this error during static parsing?".to_string(),
        String::new(),
        "- Yes: No action required, just commit the change. (Create an issue for that will be helpful.)".to_string(),
        "- No: Please add it to the `NOT_SUPPORTED_ERROR_CODES` list in `constants.rs` with an example error message, and commit the change.".to_string(),
        "---".to_string(),
        String::new(),
    ]);

    let codes = collect_diagnostics_codes();
    contents.extend(codes.iter().map(ToString::to_string));

    snapshot.save(&snap_root().join("tsc_diagnostics_codes.snap"), &contents.join("\n"));
}

fn collect_diagnostics_codes() -> Vec<u32> {
    let ts_repo_dir = workspace_root().join(TESTS_ROOT);
    let baselines_dir = ts_repo_dir.join("baselines/reference");

    // First, collect all `.errors.txt` files from `tests/baselines/reference`.
    // At this point, snapshots for tests other than `compiler` and `conformance` are also included.
    // NOTE: Some `.errors.txt` files are located in subdirectories, but we do not need them
    let all_errors_text_paths = collect_errors_txt_files(&baselines_dir);

    // Each test case refers to a `.errors.txt` file with the same name as the test file.
    // The `.errors.txt` file may be generated multiple times depending on the variations of `@option`.
    let mut errors_map: FxHashMap<&str, Vec<&str>> = FxHashMap::default();
    for errors_text_path in &all_errors_text_paths {
        let test_id = errors_text_path_to_test_id(errors_text_path);
        errors_map.entry(test_id).or_default().push(errors_text_path);
    }

    // Now, collect all test files from `tests/cases/compiler` and `tests/cases/conformance`.
    let all_test_paths = collect_test_files(&ts_repo_dir.join("cases"));

    // If the test is expected to produce an error, a `.errors.txt` file should exist.
    // Keep `.errors.txt` files that have a corresponding test file.
    let mut target_errors_text_paths = FxHashSet::default();
    for test_path in &all_test_paths {
        let test_id = test_path_to_test_id(test_path);

        if let Some(errors_text_paths) = errors_map.get(test_id) {
            for &errors_text_path in errors_text_paths {
                if is_target_errors_text_path(errors_text_path) {
                    target_errors_text_paths.insert(errors_text_path);
                }
            }
        }
    }

    // Finally, extract diagnostic error codes from the target `.errors.txt` files.
    let mut error_codes_set = FxHashSet::default();
    for &errors_text_path in &target_errors_text_paths {
        let full_path = baselines_dir.join(errors_text_path);
        let Ok(errors_text) = fs::read_to_string(&full_path) else {
            unreachable!("Failed to read file: {}", full_path.display());
        };
        let codes = extract_error_codes(&errors_text);
        error_codes_set.extend(codes);
    }

    // Return sorted error codes
    let mut codes: Vec<u32> = error_codes_set.into_iter().collect();
    codes.sort_unstable();

    codes
}

fn collect_errors_txt_files(baselines_dir: &Path) -> Vec<String> {
    let mut files = Vec::new();

    if let Ok(entries) = fs::read_dir(baselines_dir) {
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type()
                && !file_type.is_dir()
                && let Some(file_name) = entry.file_name().to_str()
                && file_name.ends_with(".errors.txt")
            {
                files.push(file_name.to_string());
            }
        }
    }

    files
}

fn collect_test_files(cases_dir: &Path) -> Vec<String> {
    let mut files = Vec::new();

    for dir_name in ["compiler", "conformance"] {
        let dir_path = cases_dir.join(dir_name);
        for entry in WalkDir::new(&dir_path)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| !e.file_type().is_dir())
        {
            if let Ok(relative_path) = entry.path().strip_prefix(cases_dir)
                && let Some(path_str) = relative_path.to_str()
            {
                files.push(path_str.to_string());
            }
        }
    }

    files
}

// Extracts a test ID from a given test path.
// Path can be like:
// - compiler/jsxContainsOnlyTriviaWhiteSpacesNotCountedAsChild.tsx
// - conformance/parser/ecmascript5/Statements/BreakStatements/parser_breakTarget2.ts
// - conformance/es6/decorators/class/property/decoratorOnClassProperty1.es6.ts
fn test_path_to_test_id(test_path: &str) -> &str {
    let path = Path::new(test_path);
    if let Some(file_name) = path.file_name().and_then(|n| n.to_str())
        && let Some(file_stem) = Path::new(file_name).file_stem().and_then(|s| s.to_str())
    {
        return file_stem;
    }

    test_path
}

// Extracts a test ID from a given errors text path.
// Path can be like:
// - importDeferTypeConflict2.errors.txt
// - asyncGeneratorParameterEvaluation(target=es2018).errors.txt
// - project/maprootUrlModuleSimpleSpecifyOutputFile/node/maprootUrlModuleSimpleSpecifyOutputFile.errors.txt
fn errors_text_path_to_test_id(errors_text_path: &str) -> &str {
    let path = Path::new(errors_text_path);
    if let Some(file_name) = path.file_name().and_then(|n| n.to_str())
        && let Some(test_id_part) = file_name.strip_suffix(".errors.txt")
        && let Some(test_id) = test_id_part.split('(').next()
    {
        return test_id;
    }

    errors_text_path
}

// This is synced with `get_error_files()` in `meta.rs`
const SUPPORTED_VARIANTS: &[&str] = &[
    "module=",
    "target=",
    "jsx=",
    "preserveconstenums=",
    "usedefineforclassfields=",
    "experimentaldecorators=",
];
// If path contains variations, we want to keep specific variations only.
fn is_target_errors_text_path(errors_text_path: &str) -> bool {
    let has_variations = errors_text_path.ends_with(").errors.txt");
    if !has_variations {
        return true;
    }

    SUPPORTED_VARIANTS.iter().any(|option| errors_text_path.contains(option))
}

// Extracts error diagnostic codes from the content of `.errors.txt` file.
// The file is expected to contain a summary and details of errors.
//
// The summary contains lines like:
// - ArrowFunction3.ts(1,12): error TS1005: ',' expected.
// - error TS2688: Cannot find type definition file for 'react'.
//
// The details contain lines like:
// - !!! error TS1005: '}' expected.
fn extract_error_codes(errors_text: &str) -> FxHashSet<u32> {
    let mut error_codes = FxHashSet::default();

    // Match pattern: `error TS<code>: `, where <code> is 4-5 digits
    for line in errors_text.lines() {
        if let Some(start) = line.find("error TS") {
            let rest = &line[start + 8..]; // skip "error TS"
            let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
            if (4..=5).contains(&digits.len())
                && let Ok(code) = digits.parse::<u32>()
            {
                error_codes.insert(code);
            }
        }
    }

    error_codes
}
