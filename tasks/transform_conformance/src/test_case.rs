use std::{
    fs,
    path::{Path, PathBuf},
};

use cow_utils::CowUtils;
use oxc::parser::ParseOptions;
use oxc::{
    allocator::Allocator,
    codegen::{CodeGenerator, CodegenOptions},
    diagnostics::{Error, NamedSource, OxcDiagnostic},
    parser::Parser,
    span::{SourceType, VALID_EXTENSIONS},
    transformer::{BabelOptions, HelperLoaderMode, TransformOptions},
};
use oxc_tasks_common::{normalize_path, print_diff_in_terminal, project_root};

use crate::{
    constants::{PLUGINS_NOT_SUPPORTED_YET, SKIP_TESTS},
    driver::Driver,
    fixture_root, oxc_test_root, packages_root,
};

#[derive(Debug)]
pub enum TestCaseKind {
    Transform(ConformanceTestCase),
    Exec(ExecTestCase),
}

impl TestCaseKind {
    pub fn new(cwd: &Path, path: &Path) -> Option<Self> {
        // in `exec` directory
        if path.parent().is_some_and(|path| path.file_name().is_some_and(|n| n == "exec"))
            && path.extension().is_some_and(|ext| VALID_EXTENSIONS.contains(&ext.to_str().unwrap()))
        {
            return Some(Self::Exec(ExecTestCase::new(cwd, path)));
        }
        // named `exec.[ext]`
        if path.file_stem().is_some_and(|name| name == "exec")
            && path.extension().is_some_and(|ext| VALID_EXTENSIONS.contains(&ext.to_str().unwrap()))
        {
            return Some(Self::Exec(ExecTestCase::new(cwd, path)));
        }

        // named `input.[ext]` or `input.d.ts`
        if (path.file_stem().is_some_and(|name| name == "input")
            && path
                .extension()
                .is_some_and(|ext| VALID_EXTENSIONS.contains(&ext.to_str().unwrap())))
            || path.file_name().is_some_and(|name| name == "input.d.ts")
        {
            return Some(Self::Transform(ConformanceTestCase::new(cwd, path)));
        }

        None
    }

    pub fn skip_test_case(&self) -> bool {
        match self {
            Self::Transform(test_case) => test_case.skip_test_case(),
            Self::Exec(exec_case) => exec_case.skip_test_case(),
        }
    }

    pub fn path(&self) -> &Path {
        match self {
            Self::Transform(test_case) => &test_case.path,
            Self::Exec(exec_case) => &exec_case.path,
        }
    }

    pub fn test(&mut self, filter: bool) {
        match self {
            Self::Transform(test_case) => test_case.test(filter),
            Self::Exec(test_case) => test_case.test(filter),
        }
    }

    pub fn errors(&self) -> &Vec<OxcDiagnostic> {
        match self {
            Self::Transform(test_case) => test_case.errors(),
            Self::Exec(test_case) => test_case.errors(),
        }
    }
}

pub trait TestCase {
    fn new(cwd: &Path, path: &Path) -> Self;

    fn options(&self) -> &BabelOptions;

    fn transform_options(&self) -> &Result<TransformOptions, Vec<Error>>;

    fn test(&mut self, filtered: bool);

    fn errors(&self) -> &Vec<OxcDiagnostic>;

    fn path(&self) -> &Path;

    fn skip_test_case(&self) -> bool {
        let options = self.options();

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
                if self.transform_options().as_ref().is_ok_and(|options| {
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
        if let Ok(path) = self.path().strip_prefix(packages_root()) {
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

        let dir = self.path().parent().unwrap();
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

    fn transform(&self, path: &Path) -> Result<Driver, OxcDiagnostic> {
        let transform_options = match self.transform_options() {
            Ok(transform_options) => transform_options,
            Err(json_err) => {
                return Err(OxcDiagnostic::error(format!("{json_err:?}")));
            }
        };

        let source_text = fs::read_to_string(path).unwrap();

        // Some babel test cases have a js extension, but contain typescript code.
        // Therefore, if the typescript plugin exists, enable typescript.
        let source_type = SourceType::from_path(path).unwrap().with_typescript(
            self.options().plugins.syntax_typescript.is_some()
                || self.options().plugins.typescript.is_some(),
        );

        let mut options = transform_options.clone();
        options.helper_loader.mode = HelperLoaderMode::Runtime;
        let driver = Driver::new(false, options).execute(&source_text, source_type, path);
        Ok(driver)
    }
}

#[derive(Debug)]
pub struct ConformanceTestCase {
    path: PathBuf,
    options: BabelOptions,
    transform_options: Result<TransformOptions, Vec<Error>>,
    errors: Vec<OxcDiagnostic>,
}

impl TestCase for ConformanceTestCase {
    fn new(cwd: &Path, path: &Path) -> Self {
        let mut options = BabelOptions::from_test_path(path.parent().unwrap());
        options.cwd.replace(cwd.to_path_buf());
        let transform_options = TransformOptions::try_from(&options);
        Self { path: path.to_path_buf(), options, transform_options, errors: vec![] }
    }

    fn options(&self) -> &BabelOptions {
        &self.options
    }

    fn transform_options(&self) -> &Result<TransformOptions, Vec<Error>> {
        &self.transform_options
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn errors(&self) -> &Vec<OxcDiagnostic> {
        &self.errors
    }

    /// Test conformance by comparing the parsed babel code and transformed code.
    fn test(&mut self, filtered: bool) {
        let output_path = self.path.parent().unwrap().read_dir().unwrap().find_map(|entry| {
            let path = entry.ok()?.path();
            let file_stem = path.file_stem()?;
            (file_stem == "output").then_some(path)
        });

        let allocator = Allocator::default();
        let input = fs::read_to_string(&self.path).unwrap();

        let source_type = {
            let mut source_type = SourceType::from_path(&self.path)
                .unwrap()
                .with_script(true)
                .with_jsx(self.options.plugins.syntax_jsx);

            source_type = match self.options.source_type.as_deref() {
                Some("unambiguous") => source_type.with_unambiguous(true),
                Some("script") => source_type.with_script(true),
                Some("module") => source_type.with_module(true),
                Some(s) => panic!("Unexpected source type {s}"),
                None => source_type,
            };

            source_type = source_type.with_typescript(
                self.options.plugins.typescript.is_some()
                    || self.options.plugins.syntax_typescript.is_some(),
            );

            source_type
        };

        if filtered {
            println!("input_path: {:?}", &self.path);
            println!("output_path: {output_path:?}");
        }

        let project_root = project_root();
        let mut transformed_code = String::new();
        let mut actual_errors = None;
        let mut transform_options = None;

        match self.transform_options() {
            Err(json_err) => {
                let error = json_err.iter().map(ToString::to_string).collect::<Vec<_>>().join("\n");
                actual_errors.replace(get_babel_error(&error));
            }
            Ok(options) => {
                transform_options.replace(options.clone());
                let mut driver =
                    Driver::new(false, options.clone()).execute(&input, source_type, &self.path);
                transformed_code = driver.printed();
                let errors = driver.errors();
                if !errors.is_empty() {
                    let source = NamedSource::new(
                        self.path.strip_prefix(project_root).unwrap().to_string_lossy(),
                        input.to_string(),
                    );
                    let error = errors
                        .into_iter()
                        .map(|err| format!("{:?}", err.with_source_code(source.clone())))
                        .collect::<Vec<_>>()
                        .join("\n");
                    actual_errors.replace(get_babel_error(&error));
                }
            }
        }

        let babel_options = self.options();

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
                    let ret = Parser::new(&allocator, &output, source_type)
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
                        .execute(&input, source_type, &self.path)
                        .errors();
                self.errors.extend(mismatch_errors);
            }
        } else if let Some(actual_errors) = actual_errors {
            self.errors.push(OxcDiagnostic::error(actual_errors));
        }
    }
}

#[derive(Debug)]
pub struct ExecTestCase {
    path: PathBuf,
    options: BabelOptions,
    transform_options: Result<TransformOptions, Vec<Error>>,
    errors: Vec<OxcDiagnostic>,
}

impl ExecTestCase {
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
}

impl TestCase for ExecTestCase {
    fn new(cwd: &Path, path: &Path) -> Self {
        let mut options = BabelOptions::from_test_path(path.parent().unwrap());
        options.cwd.replace(cwd.to_path_buf());
        let transform_options = TransformOptions::try_from(&options);
        Self { path: path.to_path_buf(), options, transform_options, errors: vec![] }
    }

    fn options(&self) -> &BabelOptions {
        &self.options
    }

    fn transform_options(&self) -> &Result<TransformOptions, Vec<Error>> {
        &self.transform_options
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn errors(&self) -> &Vec<OxcDiagnostic> {
        &self.errors
    }

    fn test(&mut self, filtered: bool) {
        if filtered {
            println!("input_path: {:?}", &self.path);
            println!("Input:\n{}\n", fs::read_to_string(&self.path).unwrap());
        }

        let result = match self.transform(&self.path) {
            Ok(mut driver) => driver.printed(),
            Err(error) => {
                if filtered {
                    println!("Transform Errors:\n{error:?}\n",);
                }
                return;
            }
        };
        self.write_to_test_files(&result);
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
