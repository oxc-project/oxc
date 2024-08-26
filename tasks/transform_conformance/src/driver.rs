use std::{mem, ops::ControlFlow, path::Path};

use oxc::{
    ast::ast::Program,
    diagnostics::OxcDiagnostic,
    semantic::post_transform_checker::check_semantic_after_transform,
    span::SourceType,
    transformer::{TransformOptions, TransformerReturn},
    CompilerInterface,
};

pub struct Driver {
    options: TransformOptions,
    printed: String,
    errors: Vec<OxcDiagnostic>,
}

impl CompilerInterface for Driver {
    fn transform_options(&self) -> Option<TransformOptions> {
        Some(self.options.clone())
    }

    fn check_semantic_error(&self) -> bool {
        false
    }

    fn handle_errors(&mut self, errors: Vec<OxcDiagnostic>) {
        self.errors.extend(errors);
    }

    fn after_codegen(&mut self, printed: String) {
        self.printed = printed;
    }

    fn after_transform(
        &mut self,
        program: &mut Program<'_>,
        transformer_return: &mut TransformerReturn,
    ) -> ControlFlow<()> {
        if let Some(errors) = check_semantic_after_transform(
            &transformer_return.symbols,
            &transformer_return.scopes,
            program,
        ) {
            self.errors.extend(errors);
        }
        ControlFlow::Continue(())
    }
}

impl Driver {
    pub fn new(options: TransformOptions) -> Self {
        Self { options, printed: String::new(), errors: vec![] }
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
