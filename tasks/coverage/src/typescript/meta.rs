//! <https://github.com/microsoft/TypeScript/blob/6f06eb1b27a6495b209e8be79036f3b2ea92cd0b/src/harness/harnessIO.ts#L1237>

use rustc_hash::FxHashMap;
use std::{fs, path::Path, sync::Arc};

use oxc::{
    allocator::Allocator,
    codegen::CodeGenerator,
    diagnostics::{NamedSource, OxcDiagnostic},
    parser::Parser,
    span::SourceType,
};
use regex::Regex;

use crate::workspace_root;

lazy_static::lazy_static! {
    // Returns a match for a test option. Test options have the form `// @name: value`
    static ref META_OPTIONS: Regex = Regex::new(r"(?m)^/{2}[[:space:]]*@(?P<name>[[:word:]]+)[[:space:]]*:[[:space:]]*(?P<value>[^\r\n]*)").unwrap();
    static ref TEST_BRACES: Regex = Regex::new(r"^[[:space:]]*[{|}][[:space:]]*$").unwrap();
}

#[expect(unused)]
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
    pub fn new(options: &FxHashMap<String, String>) -> Self {
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

#[derive(Debug, Clone)]
pub struct TestUnitData {
    pub name: String,
    pub content: String,
    pub source_type: SourceType,
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
    pub fn make_units_from_test(path: &Path, code: &str) -> Self {
        let mut current_file_options: FxHashMap<String, String> = FxHashMap::default();
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
                        source_type: SourceType::default(),
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
            source_type: SourceType::default(),
        });

        let settings = CompilerSettings::new(&current_file_options);

        let is_module = test_unit_data.len() > 1;
        let test_unit_data = test_unit_data
            .into_iter()
            .filter_map(|mut unit| {
                let mut source_type = Self::get_source_type(Path::new(&unit.name), &settings)?;
                if is_module {
                    source_type = source_type.with_module(true);
                }
                unit.source_type = source_type;
                Some(unit)
            })
            .collect::<Vec<_>>();

        let error_files = Self::get_error_files(path, &settings);
        Self { tests: test_unit_data, settings, error_files }
    }

    fn get_source_type(path: &Path, options: &CompilerSettings) -> Option<SourceType> {
        let source_type = SourceType::from_path(path)
            .ok()?
            .with_jsx(!options.jsx.is_empty())
            .with_unambiguous(true);
        Some(source_type)
    }

    // TypeScript error files can be:
    //   * `filename(module=es2022).errors.txt`
    //   * `filename(target=esnext).errors.txt`
    //   * `filename.errors.txt`
    fn get_error_files(path: &Path, options: &CompilerSettings) -> Vec<String> {
        let file_name = path.file_stem().unwrap().to_string_lossy();
        let root = workspace_root().join("typescript/tests/baselines/reference");
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

#[derive(Debug, Clone, Default)]
pub struct Baseline {
    pub name: String,
    pub original: String,
    pub original_diagnostic: Vec<String>,
    pub oxc_printed: String,
    pub oxc_diagnostics: Vec<OxcDiagnostic>,
}

impl Baseline {
    pub fn print_oxc(&mut self) {
        let allocator = Allocator::default();
        let source_type = SourceType::from_path(Path::new(&self.name)).unwrap();
        let ret = Parser::new(&allocator, &self.original, source_type).parse();
        let printed = CodeGenerator::new().build(&ret.program).code;
        self.oxc_printed = printed;
    }

    fn get_oxc_diagnostic(&self) -> String {
        let source = Arc::new(NamedSource::new(&self.name, self.original.clone()));
        self.oxc_diagnostics
            .iter()
            .map(|d| d.clone().with_source_code(Arc::clone(&source)))
            .map(|error| format!("{error:?}"))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[derive(Debug, Default)]
pub struct BaselineFile {
    pub files: Vec<Baseline>,
}

impl BaselineFile {
    pub fn print(&self) -> String {
        self.files.iter().map(|f| f.oxc_printed.clone()).collect::<Vec<_>>().join("\n")
    }

    pub fn snapshot(&self) -> String {
        self.files
            .iter()
            .map(|f| {
                let printed = f.oxc_printed.clone();
                let diagnostics = f.get_oxc_diagnostic();
                format!("//// [{}] ////\n{}{}", f.name, printed, diagnostics)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn parse(path: &Path) -> Self {
        let Ok(s) = fs::read_to_string(path) else { return Self::default() };

        let mut files: Vec<Baseline> = vec![];
        let mut is_diagnostic = false;

        let mut lines = s.lines().peekable();
        loop {
            let Some(line) = lines.next() else {
                break;
            };
            if let Some(remain) = line.strip_prefix("//// [") {
                is_diagnostic = remain.starts_with("Diagnostics reported]");
                if !is_diagnostic {
                    files.push(Baseline::default());
                    files.last_mut().unwrap().name = remain.trim_end_matches("] ////").to_string();
                }
                continue;
            }
            let last = files.last_mut().unwrap();
            if is_diagnostic {
                // Skip details of the diagnostic
                if line.is_empty() {
                    while lines.peek().is_some_and(|l| l.strip_prefix("//// [").is_none()) {
                        lines.next();
                    }
                    continue;
                }
                last.original_diagnostic.push(line.to_string());
            } else {
                last.original.push_str(line);
                last.original.push_str("\r\n");
            }
        }

        for file in &mut files {
            file.print_oxc();
        }

        Self { files }
    }
}
