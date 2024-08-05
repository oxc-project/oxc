use std::{
    fs,
    path::{Path, PathBuf},
};

use oxc_allocator::Allocator;
use oxc_codegen::CodeGenerator;
use oxc_diagnostics::{Error, OxcDiagnostic};
use oxc_parser::Parser;
use oxc_span::{SourceType, VALID_EXTENSIONS};
use oxc_tasks_common::{normalize_path, print_diff_in_terminal};
use oxc_transformer::{BabelOptions, TransformOptions, Transformer};

use crate::{
    constants::{PLUGINS_NOT_SUPPORTED_YET, SKIP_TESTS},
    fixture_root, packages_root,
    semantic::SemanticTester,
    TestRunnerEnv,
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

    pub fn test(&self, filter: bool) -> bool {
        match self {
            Self::Transform(test_case) => test_case.test(filter),
            Self::Exec(test_case) => test_case.test(filter),
        }
    }
}

fn transform_options(options: &BabelOptions) -> Result<TransformOptions, Vec<Error>> {
    TransformOptions::from_babel_options(options)
}

pub trait TestCase {
    fn new(cwd: &Path, path: &Path) -> Self;

    fn options(&self) -> &BabelOptions;

    fn transform_options(&self) -> &Result<TransformOptions, Vec<Error>>;

    fn test(&self, filtered: bool) -> bool;

    fn path(&self) -> &Path;

    fn skip_test_case(&self) -> bool {
        let options = self.options();

        // Skip plugins we don't support yet
        if PLUGINS_NOT_SUPPORTED_YET.iter().any(|plugin| options.get_plugin(plugin).is_some()) {
            return true;
        }

        if let Some(b) = options.babel_8_breaking {
            if b {
                // Skip deprecated react options
                if self.transform_options().as_ref().is_ok_and(|options| {
                    options.react.use_built_ins.is_some() || options.react.use_spread.is_some()
                }) {
                    return true;
                }
            } else {
                return true;
            }
        }

        // Legacy decorators is not supported by the parser
        if options
            .get_plugin("syntax-decorators")
            .flatten()
            .as_ref()
            .and_then(|o| o.as_object())
            .and_then(|o| o.get("version"))
            .is_some_and(|s| s == "legacy")
        {
            return true;
        }

        // babel skip test cases that in a directory starting with a dot
        // https://github.com/babel/babel/blob/0effd92d886b7135469d23612ceba6414c721673/packages/babel-helper-fixtures/src/index.ts#L223
        let dir = self.path().parent().unwrap();
        if dir.file_name().is_some_and(|n| n.to_string_lossy().starts_with('.')) {
            return true;
        }

        // Skip custom plugin.js
        if dir.join("plugin.js").exists() {
            return true;
        }

        // Skip custom preset and flow
        if options.presets.iter().any(|value| value.as_str().is_some_and(|s| s.starts_with("./")))
            || options.get_preset("flow").is_some()
        {
            return true;
        }

        // Skip tests that are known to fail
        let full_path = self.path().to_string_lossy();
        if SKIP_TESTS.iter().any(|path| full_path.ends_with(path)) {
            return true;
        }

        false
    }

    fn transform(&self, path: &Path) -> Result<String, Vec<OxcDiagnostic>> {
        let transform_options = match self.transform_options() {
            Ok(transform_options) => transform_options,
            Err(json_err) => {
                return Err(vec![OxcDiagnostic::error(format!("{json_err:?}"))]);
            }
        };

        let allocator = Allocator::default();
        let source_text = fs::read_to_string(path).unwrap();

        // Some babel test cases have a js extension, but contain typescript code.
        // Therefore, if the typescript plugin exists, enable typescript.
        let mut source_type = SourceType::from_path(path).unwrap();
        if !source_type.is_typescript()
            && (self.options().get_plugin("transform-typescript").is_some()
                || self.options().get_plugin("syntax-typescript").is_some())
        {
            source_type = source_type.with_typescript(true);
        }

        let ret = Parser::new(&allocator, &source_text, source_type).parse();
        let mut program = ret.program;
        let result = Transformer::new(
            &allocator,
            path,
            source_type,
            &source_text,
            ret.trivias.clone(),
            transform_options.clone(),
        )
        .build(&mut program);
        if result.errors.is_empty() {
            Ok(CodeGenerator::new().build(&program).source_text)
        } else {
            Err(result.errors)
        }
    }
}

#[derive(Debug)]
pub struct ConformanceTestCase {
    path: PathBuf,
    options: BabelOptions,
    transform_options: Result<TransformOptions, Vec<Error>>,
}

impl TestCase for ConformanceTestCase {
    fn new(cwd: &Path, path: &Path) -> Self {
        let mut options = BabelOptions::from_test_path(path.parent().unwrap());
        options.cwd.replace(cwd.to_path_buf());
        let transform_options = transform_options(&options);
        Self { path: path.to_path_buf(), options, transform_options }
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

    /// Test conformance by comparing the parsed babel code and transformed code.
    fn test(&self, filtered: bool) -> bool {
        let output_path = self.path.parent().unwrap().read_dir().unwrap().find_map(|entry| {
            let path = entry.ok()?.path();
            let file_stem = path.file_stem()?;
            (file_stem == "output").then_some(path)
        });

        let allocator = Allocator::default();
        let input = fs::read_to_string(&self.path).unwrap();
        let input_is_js = self.path.extension().and_then(std::ffi::OsStr::to_str) == Some("js");
        let output_is_js = output_path
            .as_ref()
            .is_some_and(|path| path.extension().and_then(std::ffi::OsStr::to_str) == Some("js"));

        let mut source_type = SourceType::from_path(&self.path)
            .unwrap()
            .with_script(if self.options.source_type.is_some() {
                !self.options.is_module()
            } else {
                input_is_js && output_is_js
            })
            .with_jsx(self.options.get_plugin("syntax-jsx").is_some());
        if !source_type.is_typescript()
            && (self.options.get_plugin("transform-typescript").is_some()
                || self.options.get_plugin("syntax-typescript").is_some())
        {
            source_type = source_type.with_typescript(true);
        }

        if filtered {
            println!("input_path: {:?}", &self.path);
            println!("output_path: {output_path:?}");
        }

        let mut transformed_code = String::new();
        let mut actual_errors = String::new();
        let mut semantic_errors = Vec::default();

        let transform_options = match self.transform_options() {
            Ok(transform_options) => {
                let ret = Parser::new(&allocator, &input, source_type).parse();
                if ret.errors.is_empty() {
                    let mut program = ret.program;
                    let transformer = Transformer::new(
                        &allocator,
                        &self.path,
                        source_type,
                        &input,
                        ret.trivias.clone(),
                        transform_options.clone(),
                    );
                    let ret = transformer.build(&mut program);

                    semantic_errors = SemanticTester::new(ret.scopes, ret.symbols).test(&program);

                    if ret.errors.is_empty() {
                        transformed_code = CodeGenerator::new().build(&program).source_text;
                    } else {
                        let error = ret
                            .errors
                            .into_iter()
                            .map(|e| Error::from(e).to_string())
                            .collect::<Vec<_>>()
                            .join("\n");
                        actual_errors = get_babel_error(&error);
                    }
                } else {
                    let error = ret
                        .errors
                        .into_iter()
                        .map(|err| err.to_string())
                        .collect::<Vec<_>>()
                        .join("\n");
                    actual_errors = get_babel_error(&error);
                }
                Some(transform_options.clone())
            }
            Err(json_err) => {
                let error = json_err.iter().map(ToString::to_string).collect::<Vec<_>>().join("\n");
                actual_errors = get_babel_error(&error);
                None
            }
        };

        let babel_options = self.options();

        // Get output.js by using our code gen so code comparison can match.
        let output = output_path.and_then(|path| fs::read_to_string(path).ok()).map_or_else(
            || {
                if let Some(throws) = &babel_options.throws {
                    return throws.to_string().replace(" (1:6)", "");
                }
                String::default()
            },
            |output| {
                // Get expected code by parsing the source text, so we can get the same code generated result.
                let ret = Parser::new(&allocator, &output, source_type).parse();
                CodeGenerator::new().build(&ret.program).source_text
            },
        );

        let passed = semantic_errors.is_empty()
            && (transformed_code == output
                || (!output.is_empty() && actual_errors.contains(&output)));

        if filtered {
            println!("Options:");
            println!("{transform_options:#?}\n");
            println!("Input:\n");
            println!("{input}\n");
            if babel_options.throws.is_some() {
                println!("Expected Errors:\n");
                println!("{output}\n");
                println!("Actual Errors:\n");
                println!("{actual_errors}\n");
                if !passed {
                    println!("Diff:\n");
                    print_diff_in_terminal(&output, &actual_errors);
                }
            } else {
                println!("Expected:\n");
                println!("{output}\n");
                println!("Transformed:\n");
                println!("{transformed_code}");
                println!("Errors:\n");
                println!("{actual_errors}\n");
                if !passed {
                    println!("Diff:\n");
                    print_diff_in_terminal(&output, &transformed_code);
                }
            }

            if !semantic_errors.is_empty() {
                println!("\nSemantic Errors:\n\n{}\n", semantic_errors.join("\n"));
            }

            println!("Passed: {passed}");
        }
        passed
    }
}

#[derive(Debug)]
pub struct ExecTestCase {
    path: PathBuf,
    options: BabelOptions,
    transform_options: Result<TransformOptions, Vec<Error>>,
}

impl ExecTestCase {
    fn run_test(path: &Path) -> bool {
        TestRunnerEnv::run_test(path)
    }

    fn write_to_test_files(&self, content: &str) -> PathBuf {
        let allocator = Allocator::default();
        let new_file_name: String =
            normalize_path(self.path.strip_prefix(packages_root()).unwrap())
                .split('/')
                .collect::<Vec<&str>>()
                .join("-");

        let mut target_path = fixture_root().join(new_file_name);
        target_path.set_extension("test.js");
        let content = TestRunnerEnv::template(content);
        fs::write(&target_path, content).unwrap();
        let source_text = fs::read_to_string(&target_path).unwrap();
        let source_type = SourceType::from_path(&target_path).unwrap();
        let transformed_ret = Parser::new(&allocator, &source_text, source_type).parse();
        let result = CodeGenerator::new().build(&transformed_ret.program).source_text;
        fs::write(&target_path, result).unwrap();
        target_path
    }
}

impl TestCase for ExecTestCase {
    fn new(cwd: &Path, path: &Path) -> Self {
        let mut options = BabelOptions::from_test_path(path.parent().unwrap());
        options.cwd.replace(cwd.to_path_buf());
        let transform_options = transform_options(&options);
        Self { path: path.to_path_buf(), options, transform_options }
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

    fn test(&self, filtered: bool) -> bool {
        if filtered {
            println!("input_path: {:?}", &self.path);
            println!("Input:\n{}\n", fs::read_to_string(&self.path).unwrap());
        }

        let result = match self.transform(&self.path) {
            Ok(result) => result,
            Err(error) => {
                if filtered {
                    println!(
                        "Transform Errors:\n{}\n",
                        error.iter().map(ToString::to_string).collect::<Vec<_>>().join("\n")
                    );
                }
                return false;
            }
        };
        let target_path = self.write_to_test_files(&result);
        let passed = Self::run_test(&target_path);

        if filtered {
            println!("Transformed:\n{result}\n");
            println!("Test Result:\n{}\n", TestRunnerEnv::get_test_result(&target_path));
        }

        passed
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
