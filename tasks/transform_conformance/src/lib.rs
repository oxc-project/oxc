use serde::de::DeserializeOwned;
use serde_json::Value;
use std::{
    cell::RefCell,
    fs::{self, File},
    io::Write,
    path::PathBuf,
    rc::Rc,
};
use walkdir::WalkDir;

use oxc_allocator::Allocator;
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::{SourceType, VALID_EXTENSIONS};
use oxc_tasks_common::{normalize_path, project_root, BabelOptions};
use oxc_transformer::{
    NullishCoalescingOperatorOptions, ReactJsxOptions, TransformOptions, TransformTarget,
    Transformer,
};

#[test]
#[cfg(any(coverage, coverage_nightly))]
fn test() {
    TestRunner::new(TestRunnerOptions::default()).run();
}

#[derive(Default)]
pub struct TestRunnerOptions {
    pub filter: Option<String>,
}

/// The test runner which walks the babel repository and searches for transformation tests.
pub struct TestRunner {
    options: TestRunnerOptions,
}

fn root() -> PathBuf {
    project_root().join("tasks/coverage/babel/packages")
}

impl TestRunner {
    pub fn new(options: TestRunnerOptions) -> Self {
        Self { options }
    }

    /// # Panics
    pub fn run(self) {
        let root = root();

        let cases = [
            // ES2024
            "babel-plugin-transform-unicode-sets-regex",
            // ES2022
            "babel-plugin-transform-class-properties",
            "babel-plugin-transform-class-static-block",
            "babel-plugin-transform-private-methods",
            "babel-plugin-transform-private-property-in-object",
            // [Syntax] "babel-plugin-transform-syntax-top-level-await",
            // ES2021
            "babel-plugin-transform-logical-assignment-operators",
            "babel-plugin-transform-numeric-separator",
            // ES2020
            "babel-plugin-transform-export-namespace-from",
            "babel-plugin-transform-dynamic-import",
            "babel-plugin-transform-export-namespace-from",
            "babel-plugin-transform-nullish-coalescing-operator",
            "babel-plugin-transform-optional-chaining",
            // [Syntax] "babel-plugin-transform-syntax-bigint",
            // [Syntax] "babel-plugin-transform-syntax-dynamic-import",
            // [Syntax] "babel-plugin-transform-syntax-import-meta",
            // ES2019
            "babel-plugin-transform-optional-catch-binding",
            "babel-plugin-transform-json-strings",
            // ES2018
            "babel-plugin-transform-async-generator-functions",
            "babel-plugin-transform-object-rest-spread",
            // [Regex] "babel-plugin-transform-unicode-property-regex",
            "babel-plugin-transform-dotall-regex",
            // [Regex] "babel-plugin-transform-named-capturing-groups-regex",
            // ES2017
            "babel-plugin-transform-async-to-generator",
            // ES2016
            "babel-plugin-transform-exponentiation-operator",
            // ES2015
            "babel-plugin-transform-shorthand-properties",
            "babel-plugin-transform-sticky-regex",
            "babel-plugin-transform-unicode-regex",
            // TypeScript
            "babel-plugin-transform-typescript",
            // React
            "babel-plugin-transform-react-jsx",
        ];

        let mut snapshot = String::new();
        let mut total = 0;
        let mut all_passed = 0;

        // Get all fixtures
        for case in cases {
            let root = root.join(case).join("test/fixtures");
            let mut paths = WalkDir::new(&root)
                .into_iter()
                .filter_map(Result::ok)
                .filter(|e| {
                    e.path().file_stem().is_some_and(|name| name == "input")
                        && e.path()
                            .extension()
                            .is_some_and(|ext| VALID_EXTENSIONS.contains(&ext.to_str().unwrap()))
                })
                .map(walkdir::DirEntry::into_path)
                .collect::<Vec<_>>();
            paths.sort_unstable();
            let num_of_tests = paths.len();
            total += num_of_tests;

            // Run the test
            let (passed, failed): (Vec<PathBuf>, Vec<PathBuf>) = paths
                .into_iter()
                .partition(|path| TestCase::new(path).test(self.options.filter.as_deref()));
            all_passed += passed.len();

            // Snapshot
            snapshot.push_str("# ");
            snapshot.push_str(case);
            if failed.is_empty() {
                snapshot.push_str(" (All passed)\n");
            } else {
                snapshot.push_str(&format!(" ({}/{})\n", passed.len(), num_of_tests));
            }
            for path in failed {
                snapshot.push_str("* Failed: ");
                snapshot.push_str(&normalize_path(path.strip_prefix(&root).unwrap()));
                snapshot.push('\n');
            }
            snapshot.push('\n');
        }

        let snapshot = format!("Passed: {all_passed}/{total}\n\n{snapshot}");
        let path = project_root().join("tasks/transform_conformance/babel.snap.md");
        let mut file = File::create(path).unwrap();
        file.write_all(snapshot.as_bytes()).unwrap();
    }
}

struct TestCase {
    path: PathBuf,
    options: BabelOptions,
}

impl TestCase {
    fn new<P: Into<PathBuf>>(path: P) -> Self {
        let path = path.into();
        let options = BabelOptions::from_path(path.parent().unwrap());
        Self { path, options }
    }

    fn transform_options(&self) -> TransformOptions {
        fn get_options<T: Default + DeserializeOwned>(value: Option<Value>) -> T {
            value.and_then(|v| serde_json::from_value::<T>(v).ok()).unwrap_or_default()
        }

        let options = &self.options;
        TransformOptions {
            target: TransformTarget::ESNext,
            react_jsx: Some(ReactJsxOptions::default()),
            assumptions: options.assumptions,
            class_static_block: options.get_plugin("transform-class-static-block").is_some(),
            logical_assignment_operators: options
                .get_plugin("transform-logical-assignment-operators")
                .is_some(),
            nullish_coalescing_operator: self
                .options
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

    /// Test conformance by comparing the parsed babel code and transformed code.
    fn test(&self, filter: Option<&str>) -> bool {
        let output_path = self.path.parent().unwrap().read_dir().unwrap().find_map(|entry| {
            let path = entry.ok()?.path();
            let file_stem = path.file_stem()?;
            (file_stem == "output").then_some(path)
        });
        let source_text = fs::read_to_string(&self.path).unwrap();
        let filtered = filter.is_some_and(|f| self.path.to_string_lossy().as_ref().contains(f));

        if filtered {
            println!("input_path: {:?}", &self.path);
            println!("output_path: {output_path:?}");
        }

        let allocator = Allocator::default();
        let source_type = SourceType::from_path(&self.path).unwrap();

        // Get expected code by parsing the source text, so we can get the same code generated result.
        let expected = output_path.and_then(|path| fs::read_to_string(path).ok());
        let Some(expected) = &expected else { return false };
        let expected_program = Parser::new(&allocator, expected, source_type).parse().program;
        let expected_code =
            Codegen::<false>::new(source_text.len(), CodegenOptions).build(&expected_program);

        // Get transformed text.

        let transformed_program =
            Parser::new(&allocator, &source_text, source_type).parse().program;

        let semantic =
            SemanticBuilder::new(&source_text, source_type).build(&transformed_program).semantic;
        let (symbols, _scope_tree) = semantic.into_symbol_table_and_scope_tree();
        let symbols = Rc::new(RefCell::new(symbols));

        let transformed_program = allocator.alloc(transformed_program);

        Transformer::new(&allocator, source_type, &symbols, self.transform_options())
            .build(transformed_program);
        let transformed_code =
            Codegen::<false>::new(source_text.len(), CodegenOptions).build(transformed_program);

        let passed = transformed_code == expected_code;
        if filtered {
            println!("Expected:\n");
            println!("{expected}\n");
            println!("Expected codegen:\n");
            println!("{expected_code}\n");
            println!("Transformed:\n");
            println!("{transformed_code}\n");
            println!("Passed: {passed}");
        }
        passed
    }
}
