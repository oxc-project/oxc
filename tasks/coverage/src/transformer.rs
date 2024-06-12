use std::path::{Path, PathBuf};

use oxc_allocator::Allocator;
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_parser::Parser;
use oxc_span::SourceType;
use oxc_transformer::{
    ArrowFunctionsOptions, ES2015Options, ReactJsxRuntime, ReactOptions, TransformOptions,
    Transformer, TypeScriptOptions,
};

use crate::{
    babel::BabelCase,
    misc::MiscCase,
    suite::{Case, TestResult},
    test262::{Test262Case, TestFlag},
    typescript::TypeScriptCase,
};

fn get_default_transformer_options() -> TransformOptions {
    TransformOptions {
        typescript: TypeScriptOptions::default(),
        es2015: ES2015Options { arrow_function: Some(ArrowFunctionsOptions::default()) },
        react: ReactOptions {
            jsx_plugin: true,
            jsx_self_plugin: true,
            jsx_source_plugin: true,
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Runs the transformer and make sure it doesn't crash.
fn get_result(
    source_text: &str,
    source_type: SourceType,
    source_path: &Path,
    options: Option<TransformOptions>,
) -> TestResult {
    let allocator = Allocator::default();
    let filename = source_path.file_name().unwrap().to_string_lossy();
    let options = options.unwrap_or_else(get_default_transformer_options);
    let parse_result1 = Parser::new(&allocator, source_text, source_type).parse();
    let mut program = parse_result1.program;
    let transform_result1 = Transformer::new(
        &allocator,
        source_path,
        source_type,
        source_text,
        parse_result1.trivias.clone(),
        options.clone(),
    )
    .build(&mut program);

    let ts_source_text1 = Codegen::<false>::new(
        &filename,
        source_text,
        parse_result1.trivias.clone(),
        CodegenOptions::default().with_typescript(true),
    )
    .build(&program)
    .source_text;

    let source_text1 = Codegen::<false>::new(
        &filename,
        source_text,
        parse_result1.trivias.clone(),
        CodegenOptions::default(),
    )
    .build(&program)
    .source_text;

    if transform_result1.is_ok() && ts_source_text1 != source_text1 {
        return TestResult::Mismatch(ts_source_text1.clone(), source_text1.clone());
    }

    let parse_result2 = Parser::new(&allocator, &ts_source_text1, source_type).parse();
    let mut program = parse_result2.program;

    let transform_result2 = Transformer::new(
        &allocator,
        source_path,
        source_type,
        &source_text1,
        parse_result2.trivias.clone(),
        options,
    )
    .build(&mut program);

    let source_text2 = Codegen::<false>::new(
        &filename,
        &source_text1,
        parse_result2.trivias,
        CodegenOptions::default(),
    )
    .build(&program)
    .source_text;

    if source_text1 == source_text2
        || transform_result1.is_err_and(|err| {
            // If error messages are the same, we consider it as a pass.
            transform_result2
                .map_err(|err| err.iter().map(ToString::to_string).collect::<Vec<_>>().join("\n"))
                .is_err_and(|err_message| {
                    err.iter().map(ToString::to_string).collect::<Vec<_>>().join("\n")
                        == err_message
                })
        })
    {
        TestResult::Passed
    } else {
        TestResult::Mismatch(source_text1.clone(), source_text2)
    }
}

pub struct TransformerTest262Case {
    base: Test262Case,
}

impl Case for TransformerTest262Case {
    fn new(path: PathBuf, code: String) -> Self {
        Self { base: Test262Case::new(path, code) }
    }

    fn code(&self) -> &str {
        self.base.code()
    }

    fn path(&self) -> &Path {
        self.base.path()
    }

    fn test_result(&self) -> &TestResult {
        self.base.test_result()
    }

    fn skip_test_case(&self) -> bool {
        self.base.should_fail()
    }

    fn run(&mut self) {
        let source_text = self.base.code();
        let is_module = self.base.meta().flags.contains(&TestFlag::Module);
        let source_type = SourceType::default().with_module(is_module);
        let result = get_result(source_text, source_type, self.path(), None);
        self.base.set_result(result);
    }
}

pub struct TransformerBabelCase {
    base: BabelCase,
}

impl Case for TransformerBabelCase {
    fn new(path: PathBuf, code: String) -> Self {
        Self { base: BabelCase::new(path, code) }
    }

    fn code(&self) -> &str {
        self.base.code()
    }

    fn path(&self) -> &Path {
        self.base.path()
    }

    fn test_result(&self) -> &TestResult {
        self.base.test_result()
    }

    fn skip_test_case(&self) -> bool {
        self.base.skip_test_case() || self.base.should_fail()
    }

    fn run(&mut self) {
        let source_text = self.base.code();
        let source_type = self.base.source_type();
        let result = get_result(source_text, source_type, self.path(), None);
        self.base.set_result(result);
    }
}

pub struct TransformerTypeScriptCase {
    base: TypeScriptCase,
}

impl Case for TransformerTypeScriptCase {
    fn new(path: PathBuf, code: String) -> Self {
        Self { base: TypeScriptCase::new(path, code) }
    }

    fn code(&self) -> &str {
        self.base.code()
    }

    fn path(&self) -> &Path {
        self.base.path()
    }

    fn test_result(&self) -> &TestResult {
        self.base.test_result()
    }

    fn skip_test_case(&self) -> bool {
        self.base.skip_test_case() || self.base.should_fail()
    }

    fn run(&mut self) {
        let mut options = get_default_transformer_options();
        let mut source_type = self.base.source_type();
        // handle @jsx: react, `react` of behavior is match babel following options
        if self.base.meta().options.jsx.last().is_some_and(|jsx| jsx == "react") {
            source_type = source_type.with_module(true);
            options.react.runtime = ReactJsxRuntime::Classic;
        }
        let result = get_result(self.base.code(), source_type, self.path(), Some(options));
        self.base.set_result(result);
    }
}

pub struct TransformerMiscCase {
    base: MiscCase,
}

impl Case for TransformerMiscCase {
    fn new(path: PathBuf, code: String) -> Self {
        Self { base: MiscCase::new(path, code) }
    }

    fn code(&self) -> &str {
        self.base.code()
    }

    fn path(&self) -> &Path {
        self.base.path()
    }

    fn test_result(&self) -> &TestResult {
        self.base.test_result()
    }

    fn skip_test_case(&self) -> bool {
        self.base.skip_test_case() || self.base.should_fail()
    }

    fn run(&mut self) {
        let result = get_result(self.base.code(), self.base.source_type(), self.path(), None);
        self.base.set_result(result);
    }
}
