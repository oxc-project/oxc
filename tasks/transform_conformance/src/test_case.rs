use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::de::DeserializeOwned;
use serde_json::Value;

use oxc_allocator::Allocator;
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_diagnostics::Error;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::{SourceType, VALID_EXTENSIONS};
use oxc_tasks_common::{normalize_path, print_diff_in_terminal, BabelOptions};
use oxc_transformer::{
    DecoratorsOptions, ReactDisplayNameOptions, ReactJsxOptions, ReactJsxSelfOptions,
    ReactJsxSourceOptions, TransformOptions, Transformer, TypeScriptOptions,
};

use crate::{fixture_root, root, TestRunnerEnv};

pub enum TestCaseKind {
    Transform(ConformanceTestCase),
    Exec(ExecTestCase),
}

impl TestCaseKind {
    pub fn test(&self, filter: bool) -> bool {
        match self {
            Self::Transform(test_case) => test_case.test(filter),
            Self::Exec(test_case) => test_case.test(filter),
        }
    }

    pub fn from_path(path: &Path) -> Option<Self> {
        // in `exec` directory
        if path.parent().is_some_and(|path| path.file_name().is_some_and(|n| n == "exec"))
            && path.extension().is_some_and(|ext| VALID_EXTENSIONS.contains(&ext.to_str().unwrap()))
        {
            return Some(Self::Exec(ExecTestCase::new(path)));
        }
        // named `exec.[ext]`
        if path.file_stem().is_some_and(|name| name == "exec")
            && path.extension().is_some_and(|ext| VALID_EXTENSIONS.contains(&ext.to_str().unwrap()))
        {
            return Some(Self::Exec(ExecTestCase::new(path)));
        }

        // named `input.[ext]``
        if path.file_stem().is_some_and(|name| name == "input")
            && path.extension().is_some_and(|ext| VALID_EXTENSIONS.contains(&ext.to_str().unwrap()))
        {
            return Some(Self::Transform(ConformanceTestCase::new(path)));
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
}

pub trait TestCase {
    fn new<P: Into<PathBuf>>(path: P) -> Self;

    fn options(&self) -> &BabelOptions;

    fn test(&self, filtered: bool) -> bool;

    fn path(&self) -> &Path;

    fn transform_options(&self) -> TransformOptions {
        fn get_options<T: Default + DeserializeOwned>(value: Option<Value>) -> T {
            value.and_then(|v| serde_json::from_value::<T>(v).ok()).unwrap_or_default()
        }
        let options = self.options();
        TransformOptions {
            assumptions: serde_json::from_value(options.assumptions.clone()).unwrap_or_default(),
            decorators: options
                .get_plugin("proposal-decorators")
                .map(get_options::<DecoratorsOptions>)
                .unwrap_or_default(),
            typescript: options
                .get_plugin("transform-typescript")
                .map(get_options::<TypeScriptOptions>)
                .unwrap_or_default(),
            react_display_name: options
                .get_plugin("transform-react-display-name")
                .map(get_options::<ReactDisplayNameOptions>)
                .unwrap_or_default(),
            react_jsx: options
                .get_plugin("transform-react-jsx")
                .map(get_options::<ReactJsxOptions>)
                .unwrap_or_default(),
            react_jsx_self: options
                .get_plugin("transform-react-jsx-self")
                .map(get_options::<ReactJsxSelfOptions>)
                .unwrap_or_default(),
            react_jsx_source: options
                .get_plugin("transform-react-jsx-source")
                .map(get_options::<ReactJsxSourceOptions>)
                .unwrap_or_default(),
        }
    }

    fn skip_test_case(&self) -> bool {
        let options = self.options();

        // Skip test cases that are not supported by babel 8
        if let Some(b) = options.babel_8_breaking {
            return !b;
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
        if self.path().parent().is_some_and(|p| {
            p.file_name().is_some_and(|n| n.to_str().map_or(false, |s| s.starts_with('.')))
        }) {
            return true;
        }
        false
    }

    fn transform(&self, path: &Path) -> Result<String, Vec<Error>> {
        let allocator = Allocator::default();
        let source_text = fs::read_to_string(path).unwrap();

        let source_type = SourceType::from_path(path).unwrap().with_typescript(false);

        let ret = Parser::new(&allocator, &source_text, source_type).parse();

        let semantic = SemanticBuilder::new(&source_text, source_type)
            .with_trivias(ret.trivias)
            .build_module_record(PathBuf::new(), &ret.program)
            .build(&ret.program)
            .semantic;

        let transformed_program = allocator.alloc(ret.program);

        let result = Transformer::new(&allocator, source_type, semantic, self.transform_options())
            .build(transformed_program);

        result.map(|()| {
            Codegen::<false>::new("", &source_text, CodegenOptions::default())
                .build(transformed_program)
                .source_text
        })
    }
}

pub struct ConformanceTestCase {
    path: PathBuf,
    options: BabelOptions,
}

impl TestCase for ConformanceTestCase {
    fn new<P: Into<PathBuf>>(path: P) -> Self {
        let path = path.into();
        let options = BabelOptions::from_path(path.parent().unwrap());
        Self { path, options }
    }

    fn options(&self) -> &BabelOptions {
        &self.options
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

        let transform_options = self.transform_options();

        let source_type = SourceType::from_path(&self.path)
            .unwrap()
            .with_script(if self.options.source_type.is_some() {
                !self.options.is_module()
            } else {
                input_is_js && output_is_js
            })
            .with_typescript(false);

        if filtered {
            println!("input_path: {:?}", &self.path);
            println!("output_path: {output_path:?}");
        }

        // Transform input.js
        let ret = Parser::new(&allocator, &input, source_type).parse();
        let semantic = SemanticBuilder::new(&input, source_type)
            .with_trivias(ret.trivias)
            .build_module_record(PathBuf::new(), &ret.program)
            .build(&ret.program)
            .semantic;
        let program = allocator.alloc(ret.program);
        let transformer =
            Transformer::new(&allocator, source_type, semantic, transform_options.clone());

        let codegen_options = CodegenOptions::default();
        let mut transformed_code = String::new();
        let mut actual_errors = String::new();
        let result = transformer.build(program);
        if result.is_ok() {
            transformed_code = Codegen::<false>::new("", &input, codegen_options.clone())
                .build(program)
                .source_text;
        } else {
            actual_errors = result.err().unwrap().iter().map(ToString::to_string).collect();
        }

        let babel_options = self.options();

        // Get output.js by using our codeg so code comparison can match.
        let output = output_path.and_then(|path| fs::read_to_string(path).ok()).map_or_else(
            || {
                if let Some(throws) = &babel_options.throws {
                    return throws.to_string();
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
            println!("Input:\n");
            println!("{input}\n");
            println!("Options:");
            println!("{transform_options:#?}\n");
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

pub struct ExecTestCase {
    path: PathBuf,
    options: BabelOptions,
}

impl ExecTestCase {
    fn run_test(path: &Path) -> bool {
        TestRunnerEnv::run_test(path)
    }

    fn write_to_test_files(&self, content: &str) -> PathBuf {
        let allocator = Allocator::default();
        let new_file_name: String = normalize_path(self.path.strip_prefix(&root()).unwrap())
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
    fn options(&self) -> &BabelOptions {
        &self.options
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn new<P: Into<PathBuf>>(path: P) -> Self {
        let path = path.into();
        let options = BabelOptions::from_path(path.parent().unwrap());
        Self { path, options }
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
