use std::{mem, ops::ControlFlow, path::Path};

use oxc::{
    ast::ast::Program,
    diagnostics::OxcDiagnostic,
    semantic::{post_transform_checker::PostTransformChecker, SemanticBuilderReturn},
    span::SourceType,
    transformer::{TransformOptions, TransformerReturn},
    CompilerInterface,
};

pub struct Driver {
    options: TransformOptions,
    printed: String,
    errors: Vec<OxcDiagnostic>,
    check_semantic: bool,
    checker: PostTransformChecker,
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
        }
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
}

impl Driver {
    pub fn new(options: TransformOptions) -> Self {
        Self {
            options,
            printed: String::new(),
            errors: vec![],
            check_semantic: true,
            checker: PostTransformChecker::default(),
        }
    }

    pub fn execute(
        &mut self,
        source_text: &str,
        source_type: SourceType,
        source_path: &Path,
    ) -> Result<String, Vec<OxcDiagnostic>> {
        self.compile(source_text, source_type, source_path);
        if self.errors.is_empty() {
            Ok(mem::take(&mut self.printed))
        } else {
            Err(mem::take(&mut self.errors))
        }
    }
}
