use std::{
    cell::RefCell,
    fs,
    path::{Path, PathBuf},
    rc::Rc,
};

use oxc_allocator::Allocator;
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::{SourceType, VALID_EXTENSIONS};
use oxc_tasks_common::{normalize_path, BabelOptions};
use oxc_transformer::{
    NullishCoalescingOperatorOptions, ReactJsxOptions, TransformOptions, TransformTarget,
    Transformer,
};
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::{fixture_root, root, TestRunnerEnv};

pub enum TestCaseKind {
    Transform(ConformanceTestCase),
    Exec(ExecTestCase),
}

impl TestCaseKind {
    pub fn test(&self, filter: Option<&str>) -> bool {
        match self {
            Self::Transform(test_case) => test_case.test(filter),
            // not support filter yet.
            Self::Exec(test_case) => test_case.test(None),
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

    fn test(&self, filter: Option<&str>) -> bool;

    fn transform_options(&self) -> TransformOptions {
        fn get_options<T: Default + DeserializeOwned>(value: Option<Value>) -> T {
            value.and_then(|v| serde_json::from_value::<T>(v).ok()).unwrap_or_default()
        }

        let options = self.options();
        TransformOptions {
            target: TransformTarget::ESNext,
            react_jsx: Some(ReactJsxOptions::default()),
            assumptions: options.assumptions,
            class_static_block: options.get_plugin("transform-class-static-block").is_some(),
            logical_assignment_operators: options
                .get_plugin("transform-logical-assignment-operators")
                .is_some(),
            nullish_coalescing_operator: options
                .get_plugin("transform-nullish-coalescing-operator")
                .map(get_options::<NullishCoalescingOperatorOptions>),
            optional_catch_binding: options
                .get_plugin("transform-optional-catch-binding")
                .is_some(),
            exponentiation_operator: options
                .get_plugin("transform-exponentiation-operator")
                .is_some(),
            shorthand_properties: options.get_plugin("transform-shorthand-properties").is_some(),
            sticky_regex: options.get_plugin("transform-sticky-regex").is_some(),
        }
    }

    fn skip_test_case(&self) -> bool {
        // Legacy decorators is not supported by the parser
        if self
            .options()
            .get_plugin("syntax-decorators")
            .flatten()
            .as_ref()
            .and_then(|o| o.as_object())
            .and_then(|o| o.get("version"))
            .is_some_and(|s| s == "legacy")
        {
            return true;
        }
        false
    }

    fn transform(&self, path: &Path) -> String {
        let allocator = Allocator::default();
        let source_text = fs::read_to_string(path).unwrap();
        let source_type = SourceType::from_path(path).unwrap();
        let transformed_program =
            Parser::new(&allocator, &source_text, source_type).parse().program;

        let semantic =
            SemanticBuilder::new(&source_text, source_type).build(&transformed_program).semantic;
        let (symbols, scopes) = semantic.into_symbol_table_and_scope_tree();
        let symbols = Rc::new(RefCell::new(symbols));
        let scopes = Rc::new(RefCell::new(scopes));
        let transformed_program = allocator.alloc(transformed_program);

        Transformer::new(&allocator, source_type, &symbols, &scopes, self.transform_options())
            .build(transformed_program);
        Codegen::<false>::new(source_text.len(), CodegenOptions).build(transformed_program)
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

    /// Test conformance by comparing the parsed babel code and transformed code.
    fn test(&self, filter: Option<&str>) -> bool {
        let filtered = filter.is_some_and(|f| self.path.to_string_lossy().as_ref().contains(f));

        let output_path = self.path.parent().unwrap().read_dir().unwrap().find_map(|entry| {
            let path = entry.ok()?.path();
            let file_stem = path.file_stem()?;
            (file_stem == "output").then_some(path)
        });

        let allocator = Allocator::default();
        let input = fs::read_to_string(&self.path).unwrap();
        let source_type = SourceType::from_path(&self.path).unwrap();

        if filtered {
            println!("input_path: {:?}", &self.path);
            println!("output_path: {output_path:?}");
        }

        // Transform input.js
        let program = Parser::new(&allocator, &input, source_type).parse().program;
        let semantic = SemanticBuilder::new(&input, source_type).build(&program).semantic;
        let (symbols, scopes) = semantic.into_symbol_table_and_scope_tree();
        let symbols = Rc::new(RefCell::new(symbols));
        let scopes = Rc::new(RefCell::new(scopes));
        let program = allocator.alloc(program);
        Transformer::new(&allocator, source_type, &symbols, &scopes, self.transform_options())
            .build(program);
        let transformed_code = Codegen::<false>::new(input.len(), CodegenOptions).build(program);

        // Get output.js by using our codeg so code comparison can match.
        let output = output_path.and_then(|path| fs::read_to_string(path).ok()).map_or_else(
            || {
                // The transformation should be equal to input.js If output.js does not exist.
                let program = Parser::new(&allocator, &input, source_type).parse().program;
                Codegen::<false>::new(input.len(), CodegenOptions).build(&program)
            },
            |output| {
                // Get expected code by parsing the source text, so we can get the same code generated result.
                let program = Parser::new(&allocator, &output, source_type).parse().program;
                Codegen::<false>::new(output.len(), CodegenOptions).build(&program)
            },
        );

        let passed = transformed_code == output;
        if filtered {
            println!("Input:\n");
            println!("{input}\n");
            println!("Output:\n");
            println!("{output}\n");
            println!("Transformed:\n");
            println!("{transformed_code}\n");
            println!("Passed: {passed}");
        }
        passed
    }
}

pub struct ExecTestCase {
    path: PathBuf,
    options: BabelOptions,
    test_runner_env: TestRunnerEnv,
}

impl ExecTestCase {
    pub fn with_test_runner_env(mut self, runner_env: TestRunnerEnv) -> Self {
        self.test_runner_env = runner_env;
        self
    }

    fn run_test(&self, path: &Path) -> bool {
        self.test_runner_env.run_test(path)
    }

    fn write_to_test_files(&self, content: &str) -> PathBuf {
        let allocator = Allocator::default();
        let new_file_name: String = normalize_path(self.path.strip_prefix(&root()).unwrap())
            .split('/')
            .collect::<Vec<&str>>()
            .join("-");

        let mut target_path = fixture_root().join(new_file_name);
        target_path.set_extension("test.js");
        let content = self.test_runner_env.template(content);
        fs::write(&target_path, content).unwrap();
        let source_text = fs::read_to_string(&target_path).unwrap();
        let source_type = SourceType::from_path(&target_path).unwrap();
        let transformed_program =
            Parser::new(&allocator, &source_text, source_type).parse().program;
        let result =
            Codegen::<false>::new(source_text.len(), CodegenOptions).build(&transformed_program);

        fs::write(&target_path, result).unwrap();

        target_path
    }
}

impl TestCase for ExecTestCase {
    fn options(&self) -> &BabelOptions {
        &self.options
    }

    fn new<P: Into<PathBuf>>(path: P) -> Self {
        let path = path.into();
        let options = BabelOptions::from_path(path.parent().unwrap());
        Self { path, options, test_runner_env: TestRunnerEnv::NodeJS }
    }

    fn test(&self, _filter: Option<&str>) -> bool {
        let result = self.transform(&self.path);
        let target_path = self.write_to_test_files(&result);
        self.run_test(&target_path)
    }
}
