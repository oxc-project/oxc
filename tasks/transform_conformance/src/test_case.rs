use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::de::DeserializeOwned;
use serde_json::Value;

use oxc_allocator::Allocator;
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_diagnostics::{miette::miette, Error};
use oxc_parser::Parser;
use oxc_span::{SourceType, VALID_EXTENSIONS};
use oxc_tasks_common::{normalize_path, print_diff_in_terminal, BabelOptions, TestOs};
use oxc_transformer::{
    ES2015Options, ReactOptions, TransformOptions, Transformer, TypeScriptOptions,
};

use crate::{fixture_root, packages_root, TestRunnerEnv, PLUGINS_NOT_SUPPORTED_YET};

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

        // named `input.[ext]``
        if path.file_stem().is_some_and(|name| name == "input")
            && path.extension().is_some_and(|ext| VALID_EXTENSIONS.contains(&ext.to_str().unwrap()))
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

fn transform_options(options: &BabelOptions) -> serde_json::Result<TransformOptions> {
    fn get_options<T: Default + DeserializeOwned>(value: Option<Value>) -> serde_json::Result<T> {
        match value {
            Some(v) => serde_json::from_value::<T>(v),
            None => Ok(T::default()),
        }
    }

    let react = if let Some(options) = options.get_preset("react") {
        get_options::<ReactOptions>(options)?
    } else {
        let jsx_plugin = options.get_plugin("transform-react-jsx");
        let has_jsx_plugin = jsx_plugin.as_ref().is_some();
        let mut react_options =
            jsx_plugin.map(get_options::<ReactOptions>).transpose()?.unwrap_or_default();
        react_options.development = options.get_plugin("transform-react-jsx-development").is_some();
        react_options.jsx_plugin = has_jsx_plugin;
        react_options.display_name_plugin =
            options.get_plugin("transform-react-display-name").is_some();
        react_options.jsx_self_plugin = options.get_plugin("transform-react-jsx-self").is_some();
        react_options.jsx_source_plugin =
            options.get_plugin("transform-react-jsx-source").is_some();
        react_options
    };

    let es2015 = ES2015Options {
        arrow_function: options
            .get_plugin("transform-arrow-functions")
            .map(get_options)
            .transpose()?,
    };

    Ok(TransformOptions {
        cwd: options.cwd.clone().unwrap(),
        assumptions: serde_json::from_value(options.assumptions.clone()).unwrap_or_default(),
        typescript: options
            .get_plugin("transform-typescript")
            .map(get_options::<TypeScriptOptions>)
            .transpose()?
            .unwrap_or_default(),
        react,
        es2015,
    })
}

pub trait TestCase {
    fn new(cwd: &Path, path: &Path) -> Self;

    fn options(&self) -> &BabelOptions;

    fn transform_options(&self) -> &serde_json::Result<TransformOptions>;

    fn test(&self, filtered: bool) -> bool;

    fn path(&self) -> &Path;

    fn skip_test_case(&self) -> bool {
        let options = self.options();

        // Skip windows
        if options.os.as_ref().is_some_and(|os| os.iter().any(TestOs::is_windows)) {
            return true;
        }

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

        false
    }

    fn transform(&self, path: &Path) -> Result<String, Vec<Error>> {
        let transform_options = match self.transform_options() {
            Ok(transform_options) => transform_options,
            Err(json_err) => {
                return Err(vec![miette!(format!("{json_err:?}"))]);
            }
        };

        let allocator = Allocator::default();
        let source_text = fs::read_to_string(path).unwrap();

        let source_type = SourceType::from_path(path).unwrap().with_typescript(
            // Some babel test cases have a js extension, but contain typescript code.
            // Therefore, if the typescript plugin exists, enable the typescript.
            self.options().get_plugin("transform-typescript").is_some(),
        );

        let ret = Parser::new(&allocator, &source_text, source_type).parse();
        let mut program = ret.program;
        let result = Transformer::new(
            &allocator,
            path,
            source_type,
            &source_text,
            &ret.trivias,
            transform_options.clone(),
        )
        .build(&mut program);

        result.map(|()| {
            Codegen::<false>::new("", &source_text, CodegenOptions::default())
                .build(&program)
                .source_text
        })
    }
}

#[derive(Debug)]
pub struct ConformanceTestCase {
    path: PathBuf,
    options: BabelOptions,
    transform_options: serde_json::Result<TransformOptions>,
}

impl TestCase for ConformanceTestCase {
    fn new(cwd: &Path, path: &Path) -> Self {
        let mut options = BabelOptions::from_path(path.parent().unwrap());
        options.cwd.replace(cwd.to_path_buf());
        let transform_options = transform_options(&options);
        Self { path: path.to_path_buf(), options, transform_options }
    }

    fn options(&self) -> &BabelOptions {
        &self.options
    }

    fn transform_options(&self) -> &serde_json::Result<TransformOptions> {
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

        let source_type = SourceType::from_path(&self.path)
            .unwrap()
            .with_script(if self.options.source_type.is_some() {
                !self.options.is_module()
            } else {
                input_is_js && output_is_js
            })
            .with_typescript(self.options.get_plugin("transform-typescript").is_some());

        if filtered {
            println!("input_path: {:?}", &self.path);
            println!("output_path: {output_path:?}");
        }

        let codegen_options = CodegenOptions::default();
        let mut transformed_code = String::new();
        let mut actual_errors = String::new();

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
                        &ret.trivias,
                        transform_options.clone(),
                    );
                    let result = transformer.build(&mut program);
                    if result.is_ok() {
                        transformed_code =
                            Codegen::<false>::new("", &input, codegen_options.clone())
                                .build(&program)
                                .source_text;
                    } else {
                        let error = result
                            .err()
                            .unwrap()
                            .iter()
                            .map(ToString::to_string)
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
                let error = json_err.to_string();
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
                let program = Parser::new(&allocator, &output, source_type).parse().program;
                Codegen::<false>::new("", &output, codegen_options.clone())
                    .build(&program)
                    .source_text
            },
        );

        let passed =
            transformed_code == output || (!output.is_empty() && actual_errors.contains(&output));
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
            println!("Passed: {passed}");
        }
        passed
    }
}

#[derive(Debug)]
pub struct ExecTestCase {
    path: PathBuf,
    options: BabelOptions,
    transform_options: serde_json::Result<TransformOptions>,
}

impl ExecTestCase {
    fn run_test(path: &Path) -> bool {
        TestRunnerEnv::run_test(path)
    }

    fn write_to_test_files(&self, content: &str) -> PathBuf {
        let allocator = Allocator::default();
        let new_file_name: String =
            normalize_path(self.path.strip_prefix(&packages_root()).unwrap())
                .split('/')
                .collect::<Vec<&str>>()
                .join("-");

        let mut target_path = fixture_root().join(new_file_name);
        target_path.set_extension("test.js");
        let content = TestRunnerEnv::template(content);
        fs::write(&target_path, content).unwrap();
        let source_text = fs::read_to_string(&target_path).unwrap();
        let source_type = SourceType::from_path(&target_path).unwrap();
        let transformed_program =
            Parser::new(&allocator, &source_text, source_type).parse().program;
        let result = Codegen::<false>::new("", &source_text, CodegenOptions::default())
            .build(&transformed_program)
            .source_text;

        fs::write(&target_path, result).unwrap();

        target_path
    }
}

impl TestCase for ExecTestCase {
    fn new(cwd: &Path, path: &Path) -> Self {
        let mut options = BabelOptions::from_path(path.parent().unwrap());
        options.cwd.replace(cwd.to_path_buf());
        let transform_options = transform_options(&options);
        Self { path: path.to_path_buf(), options, transform_options }
    }

    fn options(&self) -> &BabelOptions {
        &self.options
    }

    fn transform_options(&self) -> &serde_json::Result<TransformOptions> {
        &self.transform_options
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn test(&self, filtered: bool) -> bool {
        let result = self.transform(&self.path).expect("Transform failed");
        let target_path = self.write_to_test_files(&result);
        let passed = Self::run_test(&target_path);
        if filtered {
            println!("input_path: {:?}", &self.path);
            println!("Input:\n{}\n", fs::read_to_string(&self.path).unwrap());
            println!("Transformed:\n{result}\n");
            println!("Test Result:\n{}\n", TestRunnerEnv::get_test_result(&target_path));
        }

        passed
    }
}

fn get_babel_error(error: &str) -> String {
    match error {
        "unknown variant `invalidOption`, expected `classic` or `automatic`" => "Runtime must be either \"classic\" or \"automatic\".",
        "Duplicate __self prop found." => "Duplicate __self prop found. You are most likely using the deprecated transform-react-jsx-self Babel plugin. Both __source and __self are automatically set when using the automatic runtime. Please remove transform-react-jsx-source and transform-react-jsx-self from your Babel config.",
        "Duplicate __source prop found." => "Duplicate __source prop found. You are most likely using the deprecated transform-react-jsx-source Babel plugin. Both __source and __self are automatically set when using the automatic runtime. Please remove transform-react-jsx-source and transform-react-jsx-self from your Babel config.",
        "Expected `>` but found `/`" => "Unexpected token, expected \",\"",
        _ => error
    }.to_string()
}
