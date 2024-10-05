use std::{
    cell::{Ref, RefCell},
    sync::Arc,
};

use oxc::{
    allocator::Allocator,
    ast::{ast::Program, Trivias},
    codegen::Codegen,
    diagnostics::{Error, NamedSource, OxcDiagnostic},
    parser::{Parser, ParserReturn},
    span::SourceType,
};

#[must_use]
pub(crate) struct TransformContext<'a> {
    pub allocator: &'a Allocator,
    program: RefCell<Program<'a>>,
    pub trivias: Trivias,

    /// Generate source maps?
    source_map: bool,

    /// Path to the file being transformed.
    filename: &'a str,

    /// Source text of the file being transformed.
    source_text: &'a str,
    source_type: SourceType,

    /// Errors that occurred during transformation.
    errors: RefCell<Vec<OxcDiagnostic>>,
}

impl<'a> TransformContext<'a> {
    pub fn new(
        allocator: &'a Allocator,
        filename: &'a str,
        source_text: &'a str,
        source_type: SourceType,
        source_map: Option<bool>,
    ) -> Self {
        let ParserReturn { errors, program, trivias, .. } =
            Parser::new(allocator, source_text, source_type).parse();
        let source_map = source_map.unwrap_or_default();
        Self {
            allocator,
            program: RefCell::new(program),
            trivias,

            source_map,

            filename,
            source_text,
            source_type,
            errors: RefCell::new(errors),
        }
    }

    #[inline]
    pub fn file_name(&self) -> &'a str {
        self.filename
    }

    #[inline]
    pub fn source_text(&self) -> &'a str {
        self.source_text
    }

    #[inline]
    pub fn program(&self) -> Ref<'_, Program<'a>> {
        self.program.borrow()
    }

    pub fn codegen(&self) -> Codegen<'a> {
        let codegen = Codegen::new();
        if self.source_map {
            codegen.enable_source_map(self.file_name(), self.source_text())
        } else {
            codegen
        }
    }

    pub fn add_diagnostics(&self, diagnostics: Vec<OxcDiagnostic>) {
        if diagnostics.is_empty() {
            return;
        }
        self.errors.borrow_mut().extend(diagnostics);
    }

    pub fn take_and_render_reports(&self) -> Vec<String> {
        let diagnostics = std::mem::take(&mut *self.errors.borrow_mut());
        // TODO: make pretty-printed errors configurable
        self.wrap_diagnostics(diagnostics).map(|error| format!("{error:?}")).collect()
    }

    fn wrap_diagnostics<D: IntoIterator<Item = OxcDiagnostic>>(
        &self,
        diagnostics: D,
    ) -> impl Iterator<Item = Error> {
        let source = {
            let lang = match (self.source_type.is_javascript(), self.source_type.is_jsx()) {
                (true, false) => "JavaScript",
                (true, true) => "JSX",
                (false, true) => "TypeScript React",
                (false, false) => {
                    if self.source_type.is_typescript_definition() {
                        "TypeScript Declaration"
                    } else {
                        "TypeScript"
                    }
                }
            };

            let ns = NamedSource::new(self.file_name(), self.source_text().to_string())
                .with_language(lang);
            Arc::new(ns)
        };

        diagnostics
            .into_iter()
            .map(move |diagnostic| Error::from(diagnostic).with_source_code(Arc::clone(&source)))
    }
}
