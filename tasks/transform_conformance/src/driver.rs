use std::{mem, ops::ControlFlow, path::Path};

use oxc::{
    ast::ast::Program,
    codegen::{CodegenOptions, CodegenReturn},
    diagnostics::OxcDiagnostic,
    parser::ParseOptions,
    span::SourceType,
    transformer::{TransformOptions, TransformerReturn},
    CompilerInterface,
};
use oxc_tasks_transform_checker::check_semantic_after_transform;

pub struct Driver {
    check_semantic: bool,
    allow_return_outside_function: bool,
    options: TransformOptions,
    printed: String,
    errors: Vec<OxcDiagnostic>,
}

impl CompilerInterface for Driver {
    fn parse_options(&self) -> ParseOptions {
        ParseOptions {
            allow_return_outside_function: self.allow_return_outside_function,
            ..Default::default()
        }
    }

    fn transform_options(&self) -> Option<&TransformOptions> {
        Some(&self.options)
    }

    fn codegen_options(&self) -> Option<CodegenOptions> {
        Some(CodegenOptions { comments: false, ..CodegenOptions::default() })
    }

    fn check_semantic_error(&self) -> bool {
        false
    }

    fn semantic_child_scope_ids(&self) -> bool {
        true
    }

    fn handle_errors(&mut self, errors: Vec<OxcDiagnostic>) {
        self.errors.extend(errors);
    }

    fn after_codegen(&mut self, ret: CodegenReturn) {
        self.printed = ret.code;
    }

    fn after_transform(
        &mut self,
        program: &mut Program<'_>,
        transformer_return: &mut TransformerReturn,
    ) -> ControlFlow<()> {
        if self.check_semantic {
            if let Some(errors) = check_semantic_after_transform(
                &transformer_return.symbols,
                &transformer_return.scopes,
                program,
            ) {
                self.errors.extend(errors);
            }
        }
        ControlFlow::Continue(())
    }
}

impl Driver {
    pub fn new(
        check_semantic: bool,
        allow_return_outside_function: bool,
        options: TransformOptions,
    ) -> Self {
        Self {
            check_semantic,
            allow_return_outside_function,
            options,
            printed: String::new(),
            errors: vec![],
        }
    }

    pub fn errors(&mut self) -> Vec<OxcDiagnostic> {
        mem::take(&mut self.errors)
    }

    pub fn printed(&mut self) -> String {
        mem::take(&mut self.printed)
    }

    pub fn execute(
        mut self,
        source_text: &str,
        source_type: SourceType,
        source_path: &Path,
    ) -> Self {
        self.compile(source_text, source_type, source_path);
        self
    }
}
