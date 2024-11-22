use std::{
    fs,
    path::{Path, PathBuf},
};

use cow_utils::CowUtils;
use oxc::parser::ParseOptions;
use oxc::{
    allocator::Allocator,
    codegen::{CodeGenerator, CodegenOptions},
    diagnostics::{NamedSource, OxcDiagnostic},
    parser::Parser,
    span::{SourceType, VALID_EXTENSIONS},
    transformer::{BabelOptions, HelperLoaderMode, TransformOptions},
};
use oxc_tasks_common::{normalize_path, print_diff_in_terminal, project_root};

use crate::{
    constants::{PLUGINS_NOT_SUPPORTED_YET, SKIP_TESTS, SNAPSHOT_TESTS},
    driver::Driver,
    fixture_root, oxc_test_root, packages_root, snap_root, TestRunnerOptions,
};

#[derive(Debug)]
pub struct TestCase {
    pub kind: TestCaseKind,
    pub path: PathBuf,
    options: BabelOptions,
    source_type: SourceType,
    transform_options: Result<TransformOptions, Vec<String>>,
    pub errors: Vec<OxcDiagnostic>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TestCaseKind {
    Conformance,
    Exec,
    Snapshot,
}

impl TestCase {
    pub fn new(cwd: &Path, path: &Path) -> Option<Self> {
        let mut options = BabelOptions::from_test_path(path.parent().unwrap());
        options.cwd.replace(cwd.to_path_buf());
        let transform_options = TransformOptions::try_from(&options);
        let path = path.to_path_buf();
        let errors = vec![];

        // in `exec` directory
        let kind = if path
            .extension()
            .is_some_and(|ext| VALID_EXTENSIONS.contains(&ext.to_str().unwrap()))
            && (path.parent().is_some_and(|path| path.file_name().is_some_and(|n| n == "exec"))
                || path.file_stem().is_some_and(|name| name == "exec"))
        {
            TestCaseKind::Exec
        }
        // named `input.[ext]` or `input.d.ts`
        else if path.file_stem().is_some_and(|name| name == "input" || name == "input.d")
            && path.extension().is_some_and(|ext| VALID_EXTENSIONS.contains(&ext.to_str().unwrap()))
        {
            if path
                .strip_prefix(packages_root())
                .is_ok_and(|p| SNAPSHOT_TESTS.iter().any(|t| p.to_string_lossy().starts_with(t)))
            {
                TestCaseKind::Snapshot
            } else {
                TestCaseKind::Conformance
            }
        } else {
            return None;
        };

        let source_type = Self::source_type(&path, &options);

        Some(Self { kind, path, options, source_type, transform_options, errors })
    }

    fn source_type(path: &Path, options: &BabelOptions) -> SourceType {
        // Some babel test cases have a js extension, but contain typescript code.
        // Therefore, if the typescript plugin exists, enable typescript.
        let mut source_type = SourceType::from_path(path)
            .unwrap()
            .with_script(true)
            .with_jsx(options.plugins.syntax_jsx);
        source_type = match options.source_type.as_deref() {
            Some("unambiguous") => source_type.with_unambiguous(true),
            Some("script") => source_type.with_script(true),
            Some("module") => source_type.with_module(true),
            Some(s) => panic!("Unexpected source type {s}"),
            None => source_type,
        };
        source_type = source_type.with_typescript(
            options.plugins.typescript.is_some() || options.plugins.syntax_typescript.is_some(),
        );
        source_type
    }

    pub fn skip_test_case(&self) -> bool {
        let options = &self.options;

        // Skip plugins we don't support yet
        if PLUGINS_NOT_SUPPORTED_YET
            .iter()
            .any(|plugin| options.plugins.unsupported.iter().any(|p| plugin == p))
        {
            return true;
        }

        if let Some(b) = options.babel_8_breaking {
            if b {
                // Skip deprecated react options
                if self.transform_options.as_ref().is_ok_and(|options| {
                    options.jsx.use_built_ins.is_some() || options.jsx.use_spread.is_some()
                }) {
                    return true;
                }
            } else {
                return true;
            }
        }

        // Legacy decorators is not supported
        if options
            .plugins
            .proposal_decorators
            .as_ref()
            .or(options.plugins.syntax_decorators.as_ref())
            .is_some_and(|o| o.version == "legacy")
        {
            return true;
        }

        // Skip some Babel tests.
        if let Ok(path) = self.path.strip_prefix(packages_root()) {
            // babel skip test cases that in a directory starting with a dot
            // https://github.com/babel/babel/blob/0effd92d886b7135469d23612ceba6414c721673/packages/babel-helper-fixtures/src/index.ts#L223
            if path.components().any(|c| c.as_os_str().to_str().unwrap().starts_with('.')) {
                return true;
            }
            // Skip tests that are known to fail
            let full_path = path.to_string_lossy();
            if SKIP_TESTS.iter().any(|path| full_path.starts_with(path)) {
                return true;
            }
        }

        let dir = self.path.parent().unwrap();
        // Skip custom plugin.js
        if dir.join("plugin.js").exists() {
            return true;
        }

        // Skip custom preset and flow
        if options.presets.unsupported.iter().any(|s| s.starts_with("./") || s == "flow") {
            return true;
        }

        false
    }

    fn transform(&self, mode: HelperLoaderMode) -> Result<String, String> {
        let path = &self.path;
        let transform_options = match &self.transform_options {
            Ok(transform_options) => transform_options,
            Err(json_err) => {
                let error = json_err.iter().map(ToString::to_string).collect::<Vec<_>>().join("\n");
                return Err(error);
            }
        };

        let source_text = fs::read_to_string(path).unwrap();

        let project_root = project_root();
        let mut options = transform_options.clone();
        options.helper_loader.mode = mode;
        let mut driver = Driver::new(false, options).execute(&source_text, self.source_type, path);
        let errors = driver.errors();
        if !errors.is_empty() {
            let source = NamedSource::new(
                path.strip_prefix(project_root).unwrap().to_string_lossy(),
                source_text.to_string(),
            );
            return Err(errors
                .into_iter()
                .map(|err| format!("{:?}", err.with_source_code(source.clone())))
                .collect::<Vec<_>>()
                .join("\n"));
        }
        Ok(driver.printed())
    }

    pub fn test(&mut self, options: &TestRunnerOptions) {
        if options.debug {
            println!("{}", self.path.to_string_lossy());
        }

        let filtered = options.filter.is_some();
        match self.kind {
            TestCaseKind::Conformance => self.test_conformance(filtered),
            TestCaseKind::Exec => {
                if options.exec {
                    self.test_exec(filtered);
                }
            }
            TestCaseKind::Snapshot => self.test_snapshot(filtered),
        }
    }

    /// Test conformance by comparing the parsed babel code and transformed code.
    fn test_conformance(&mut self, filtered: bool) {
        let output_path = self.path.parent().unwrap().read_dir().unwrap().find_map(|entry| {
            let path = entry.ok()?.path();
            let file_stem = path.file_stem()?;
            (file_stem == "output").then_some(path)
        });

        let allocator = Allocator::default();
        let input = fs::read_to_string(&self.path).unwrap();

        if filtered {
            println!("input_path: {:?}", &self.path);
            println!("output_path: {output_path:?}");
        }

        let mut transformed_code = String::new();
        let mut actual_errors = None;
        let mut transform_options = None;

        match self.transform(HelperLoaderMode::External) {
            Err(error) => {
                actual_errors.replace(get_babel_error(&error));
            }
            Ok(code) => {
                transform_options.replace(self.transform_options.as_ref().unwrap().clone());
                transformed_code = code;
            }
        }

        let babel_options = &self.options;

        let output;
        let passed = if let Some(throws) = &babel_options.throws {
            output = throws.cow_replace(" (1:6)", "").into_owned();
            !output.is_empty()
                && actual_errors.as_ref().is_some_and(|errors| errors.contains(&output))
        } else {
            // Get output.js by using our code gen so code comparison can match.
            output = output_path.and_then(|path| fs::read_to_string(path).ok()).map_or_else(
                String::default,
                |output| {
                    // Get expected code by parsing the source text, so we can get the same code generated result.
                    let ret = Parser::new(&allocator, &output, self.source_type)
                        .with_options(ParseOptions {
                            // Related: async to generator, regression
                            allow_return_outside_function: true,
                            ..Default::default()
                        })
                        .parse();

                    CodeGenerator::new()
                        .with_options(CodegenOptions {
                            comments: false,
                            ..CodegenOptions::default()
                        })
                        .build(&ret.program)
                        .code
                },
            );

            if transformed_code == output {
                actual_errors.is_none()
            } else {
                if actual_errors.is_none() {
                    actual_errors.replace("x Output mismatch".to_string());
                }
                false
            }
        };

        if filtered {
            println!("Options:");
            println!("{transform_options:#?}\n");
            println!("Input:\n");
            println!("{input}\n");
            if babel_options.throws.is_some() {
                println!("Expected Errors:\n");
                println!("{output}\n");
                println!("Actual Errors:\n");
                if let Some(actual_errors) = &actual_errors {
                    println!("{actual_errors}\n");
                    if !passed {
                        println!("Diff:\n");
                        print_diff_in_terminal(&output, actual_errors);
                    }
                }
            } else {
                println!("Expected:\n");
                println!("{output}\n");
                println!("Transformed:\n");
                println!("{transformed_code}");
                println!("Errors:\n");
                if let Some(actual_errors) = &actual_errors {
                    println!("{actual_errors}\n");
                }
                if !passed {
                    println!("Diff:\n");
                    print_diff_in_terminal(&output, &transformed_code);
                }
            }

            println!("Passed: {passed}");
        }

        if passed {
            if let Some(options) = transform_options {
                let mismatch_errors =
                    Driver::new(/* check transform mismatch */ true, options)
                        .execute(&input, self.source_type, &self.path)
                        .errors();
                self.errors.extend(mismatch_errors);
            }
        } else if let Some(actual_errors) = actual_errors {
            self.errors.push(OxcDiagnostic::error(actual_errors));
        }
    }

    fn test_exec(&mut self, filtered: bool) {
        if filtered {
            println!("input_path: {:?}", &self.path);
            println!("Input:\n{}\n", fs::read_to_string(&self.path).unwrap());
        }

        let result = match self.transform(HelperLoaderMode::Runtime) {
            Ok(code) => code,
            Err(error) => {
                if filtered {
                    println!("Transform Errors:\n{error:?}\n",);
                }
                return;
            }
        };
        self.write_to_test_files(&result);
    }

    fn write_to_test_files(&self, content: &str) {
        let unprefixed_path = self
            .path
            .strip_prefix(packages_root())
            .or_else(|_| self.path.strip_prefix(oxc_test_root()))
            .unwrap();
        let new_file_name: String =
            normalize_path(unprefixed_path).split('/').collect::<Vec<&str>>().join("-");
        let mut target_path = fixture_root().join(new_file_name);
        target_path.set_extension("test.js");
        let content = Self::template(content);
        fs::write(&target_path, content).unwrap();
    }

    fn template(code: &str) -> String {
        // Move all the import statements to top level.
        let mut codes = vec![];
        let mut imports = vec![];

        for line in code.lines() {
            if line.trim_start().starts_with("import ") {
                imports.push(line);
            } else {
                codes.push(String::from("\t") + line);
            }
        }

        let code = codes.join("\n");
        let imports = imports.join("\n");

        format!(
            r#"import {{expect, test}} from 'vitest';
{imports}
test("exec", () => {{
{code}
}})"#
        )
    }

    fn test_snapshot(&self, filtered: bool) {
        let result = match self.transform(HelperLoaderMode::Runtime) {
            Ok(code) => code,
            Err(error) => error,
        };
        let mut path = snap_root().join(self.path.strip_prefix(packages_root()).unwrap());
        path.set_file_name("output");
        let input_extension = self.path.extension().unwrap().to_str().unwrap();
        let extension =
            input_extension.chars().map(|c| if c == 't' { 'j' } else { c }).collect::<String>();
        path.set_extension(extension);
        if filtered {
            println!("Input path: {:?}", &self.path);
            println!("Output path: {path:?}");
            println!("Input:\n{}\n", fs::read_to_string(&self.path).unwrap());
            println!("Output:\n{result}\n");
        }
        if fs::write(&path, &result).is_err() {
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, &result).unwrap();
        }
    }
}

fn get_babel_error(error: &str) -> String {
    match error {
        "transform-react-jsx: unknown variant `invalidOption`, expected `classic` or `automatic`" => "Runtime must be either \"classic\" or \"automatic\".",
        "Duplicate __self prop found." => "Duplicate __self prop found. You are most likely using the deprecated transform-react-jsx-self Babel plugin. Both __source and __self are automatically set when using the automatic runtime. Please remove transform-react-jsx-source and transform-react-jsx-self from your Babel config.",
        "Duplicate __source prop found." => "Duplicate __source prop found. You are most likely using the deprecated transform-react-jsx-source Babel plugin. Both __source and __self are automatically set when using the automatic runtime. Please remove transform-react-jsx-source and transform-react-jsx-self from your Babel config.",
        "Expected `>` but found `/`" => "Unexpected token, expected \",\"",
        _ => error
    }.to_string()
}
