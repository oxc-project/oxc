use std::{mem, ops::ControlFlow, path::Path};

use oxc::{
    ast::{ast::Program, Trivias},
    codegen::{CodeGenerator, CodegenOptions, CodegenReturn},
    diagnostics::OxcDiagnostic,
    mangler::Mangler,
    span::SourceType,
    transformer::{TransformOptions, TransformerReturn},
    CompilerInterface,
};
use oxc_tasks_transform_checker::check_semantic_after_transform;

pub struct Driver {
    check_semantic: bool,
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

    // Disable comments
    fn codegen<'a>(
        &self,
        program: &Program<'a>,
        _source_text: &'a str,
        _source_path: &Path,
        _trivias: &Trivias,
        mangler: Option<Mangler>,
        options: CodegenOptions,
    ) -> CodegenReturn {
        CodeGenerator::new().with_options(options).with_mangler(mangler).build(program)
    }
}

impl Driver {
    pub fn new(check_semantic: bool, options: TransformOptions) -> Self {
        Self { check_semantic, options, printed: String::new(), errors: vec![] }
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
