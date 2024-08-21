use std::{collections::HashSet, ops::ControlFlow, path::PathBuf};

use oxc::CompilerInterface;

#[allow(clippy::wildcard_imports)]
use oxc::ast::{ast::*, Trivias};
use oxc::codegen::CodegenOptions;
use oxc::diagnostics::OxcDiagnostic;
use oxc::minifier::CompressOptions;
use oxc::parser::{ParseOptions, ParserReturn};
use oxc::semantic::{post_transform_checker::PostTransformChecker, SemanticBuilderReturn};
use oxc::span::{SourceType, Span};
use oxc::transformer::{TransformOptions, TransformerReturn};

use crate::suite::TestResult;

#[allow(clippy::struct_excessive_bools)]
#[derive(Default)]
pub struct Driver {
    pub path: PathBuf,
    // options
    pub transform: Option<TransformOptions>,
    pub compress: bool,
    pub remove_whitespace: bool,
    pub codegen: bool,
    pub check_semantic: bool,
    pub allow_return_outside_function: bool,
    // results
    pub panicked: bool,
    pub errors: Vec<OxcDiagnostic>,
    pub printed: String,
    // states
    pub checker: PostTransformChecker,
}

impl CompilerInterface for Driver {
    fn parse_options(&self) -> ParseOptions {
        ParseOptions {
            allow_return_outside_function: self.allow_return_outside_function,
            ..ParseOptions::default()
        }
    }

    fn transform_options(&self) -> Option<TransformOptions> {
        self.transform.clone()
    }

    fn compress_options(&self) -> Option<CompressOptions> {
        self.compress.then(CompressOptions::all_true)
    }

    fn codegen_options(&self) -> Option<CodegenOptions> {
        self.codegen
            .then(|| CodegenOptions { minify: self.remove_whitespace, ..CodegenOptions::default() })
    }

    fn handle_errors(&mut self, errors: Vec<OxcDiagnostic>) {
        self.errors.extend(errors);
    }

    fn after_parse(&mut self, parser_return: &mut ParserReturn) -> ControlFlow<()> {
        let ParserReturn { program, trivias, panicked, .. } = parser_return;
        self.panicked = *panicked;
        if self.check_comments(trivias) {
            return ControlFlow::Break(());
        }
        // Make sure serialization doesn't crash; also for code coverage.
        let _serializer = program.serializer();
        ControlFlow::Continue(())
    }

    fn after_semantic(
        &mut self,
        program: &mut Program<'_>,
        _semantic_return: &mut SemanticBuilderReturn,
    ) -> ControlFlow<()> {
        if self.check_semantic {
            if let Some(errors) = self.checker.before_transform(program) {
                self.errors.extend(errors);
                return ControlFlow::Break(());
            }
        };
        ControlFlow::Continue(())
    }

    fn after_transform(
        &mut self,
        program: &mut Program<'_>,
        transformer_return: &mut TransformerReturn,
    ) -> ControlFlow<()> {
        if self.check_semantic {
            if let Some(errors) = self.checker.after_transform(
                &transformer_return.symbols,
                &transformer_return.scopes,
                program,
            ) {
                self.errors.extend(errors);
                return ControlFlow::Break(());
            }
        }
        ControlFlow::Continue(())
    }

    fn after_codegen(&mut self, printed: String) {
        self.printed = printed;
    }
}

impl Driver {
    pub fn errors(&mut self) -> Vec<OxcDiagnostic> {
        std::mem::take(&mut self.errors)
    }

    pub fn idempotency(
        mut self,
        case: &'static str,
        source_text: &str,
        source_type: SourceType,
    ) -> TestResult {
        self.run(source_text, source_type);
        let printed1 = self.printed.clone();
        self.run(&printed1, source_type);
        let printed2 = self.printed.clone();
        if printed1 == printed2 {
            TestResult::Passed
        } else {
            TestResult::Mismatch(case, printed1, printed2)
        }
    }

    pub fn run(&mut self, source_text: &str, source_type: SourceType) {
        let path = self.path.clone();
        self.compile(source_text, source_type, &path);
    }

    fn check_comments(&mut self, trivias: &Trivias) -> bool {
        let mut uniq: HashSet<Span> = HashSet::new();
        for comment in trivias.comments() {
            if !uniq.insert(comment.span) {
                self.errors
                    .push(OxcDiagnostic::error("Duplicate Comment").with_label(comment.span));
                return true;
            }
        }
        false
    }
}
