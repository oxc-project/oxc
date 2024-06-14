//! <https://github.com/microsoft/TypeScript/blob/6f06eb1b27a6495b209e8be79036f3b2ea92cd0b/src/harness/harnessIO.ts#L1237>

use std::{collections::HashMap, fs, path::Path};

use regex::Regex;

use crate::project_root;

use super::TESTS_ROOT;

lazy_static::lazy_static! {
    // Returns a match for a test option. Test options have the form `// @name: value`
    static ref META_OPTIONS: Regex = Regex::new(r"(?m)^/{2}[[:space:]]*@(?P<name>[[:word:]]+)[[:space:]]*:[[:space:]]*(?P<value>[^\r\n]*)").unwrap();
    static ref TEST_BRACES: Regex = Regex::new(r"^[[:space:]]*[{|}][[:space:]]*$").unwrap();
}

#[allow(unused)]
#[derive(Debug)]
pub struct CompilerSettings {
    pub modules: Vec<String>,
    pub targets: Vec<String>,
    pub strict: bool,
    pub jsx: Vec<String>, // 'react', 'preserve'
    pub declaration: bool,
    pub emit_declaration_only: bool,
    pub always_strict: bool, // Ensure 'use strict' is always emitted.
    pub allow_unreachable_code: bool,
    pub allow_unused_labels: bool,
    pub no_fallthrough_cases_in_switch: bool,
}

impl CompilerSettings {
    pub fn new(options: &HashMap<String, String>) -> Self {
        Self {
            modules: Self::split_value_options(options.get("module")),
            targets: Self::split_value_options(options.get("target")),
            strict: Self::value_to_boolean(options.get("strict"), false),
            jsx: Self::split_value_options(options.get("jsx")),
            declaration: Self::value_to_boolean(options.get("declaration"), false),
            emit_declaration_only: Self::value_to_boolean(
                options.get("emitDeclarationOnly"),
                false,
            ),
            always_strict: Self::value_to_boolean(options.get("alwaysstrict"), false),
            allow_unreachable_code: Self::value_to_boolean(
                options.get("allowunreachablecode"),
                true,
            ),
            allow_unused_labels: Self::value_to_boolean(options.get("allowunusedlabels"), true),
            no_fallthrough_cases_in_switch: Self::value_to_boolean(
                options.get("nofallthroughcasesinswitch"),
                false,
            ),
        }
    }

    fn split_value_options(value: Option<&String>) -> Vec<String> {
        value
            .map(|value| value.split(',').map(|s| s.trim().to_lowercase()).collect())
            .unwrap_or_default()
    }

    fn value_to_boolean(value: Option<&String>, default: bool) -> bool {
        match value.map(AsRef::as_ref) {
            Some("true") => true,
            Some("false") => false,
            _ => default,
        }
    }
}

#[derive(Debug)]
pub struct TestUnitData {
    pub name: String,
    pub content: String,
}

#[derive(Debug)]
pub struct TestCaseContent {
    pub tests: Vec<TestUnitData>,
    pub settings: CompilerSettings,
    pub error_files: Vec<String>,
}

impl TestCaseContent {
    /// TypeScript supports multiple files in a single test case.
    /// These files start with `// @<option-name>: <option-value>` and are followed by the file's content.
    /// This function extracts the individual files with their content and drops unsupported files.
    pub fn make_units_from_test(path: &Path, code: &str) -> TestCaseContent {
        let mut current_file_options: HashMap<String, String> = HashMap::default();
        let mut current_file_name: Option<String> = None;
        let mut test_unit_data: Vec<TestUnitData> = vec![];
        let mut current_file_content = String::new();

        for line in code.lines() {
            if let Some(option) = META_OPTIONS.captures(line) {
                let meta_name = option.name("name").unwrap().as_str();
                let meta_name = meta_name.to_lowercase();
                let meta_value = option.name("value").unwrap().as_str();
                let meta_value = meta_value.trim();
                if meta_name != "filename" {
                    current_file_options.insert(meta_name.clone(), meta_value.to_string());
                    continue;
                }
                if let Some(file_name) = current_file_name.take() {
                    test_unit_data.push(TestUnitData {
                        name: file_name,
                        content: std::mem::take(&mut current_file_content),
                    });
                }
                current_file_name = Some(meta_value.to_string());
            } else {
                if !current_file_content.is_empty() {
                    current_file_content.push('\n');
                }
                current_file_content.push_str(line);
            }
        }

        // normalize the fileName for the single file case
        let file_name = if !test_unit_data.is_empty() || current_file_name.is_some() {
            current_file_name.unwrap()
        } else {
            path.file_name().unwrap().to_string_lossy().to_string()
        };

        test_unit_data.push(TestUnitData {
            name: file_name,
            content: std::mem::take(&mut current_file_content),
        });

        let settings = CompilerSettings::new(&current_file_options);
        let error_files = Self::get_error_files(path, &settings);
        Self { tests: test_unit_data, settings, error_files }
    }

    // TypeScript error files can be:
    //   * `filename(module=es2022).errors.txt`
    //   * `filename(target=esnext).errors.txt`
    //   * `filename.errors.txt`
    fn get_error_files(path: &Path, options: &CompilerSettings) -> Vec<String> {
        let file_name = path.file_stem().unwrap().to_string_lossy();
        let root = project_root().join(TESTS_ROOT).join("baselines/reference");
        let mut suffixes = vec![String::new()];
        suffixes.extend(options.modules.iter().map(|module| format!("(module={module})")));
        suffixes.extend(options.targets.iter().map(|target| format!("(target={target})")));
        suffixes.extend(options.jsx.iter().map(|jsx| format!("(jsx={jsx})")));
        let mut error_files = vec![];
        for suffix in suffixes {
            let error_path = root.join(format!("{file_name}{suffix}.errors.txt"));
            if error_path.exists() {
                let error_file = fs::read_to_string(error_path).unwrap();
                error_files.push(error_file);
            }
        }
        error_files
    }
}
