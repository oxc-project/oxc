use std::{
    cell::{OnceCell, Ref, RefCell, RefMut},
    path::Path,
    sync::Arc,
};

use oxc_allocator::Allocator;
use oxc_ast::{ast::Program, Trivias};
use oxc_codegen::Codegen;
use oxc_diagnostics::{Error, NamedSource, OxcDiagnostic};
use oxc_parser::{Parser, ParserReturn};
use oxc_span::SourceType;

use crate::TransformOptions;

#[must_use]
pub(crate) struct TransformContext<'a> {
    pub allocator: &'a Allocator,
    program: RefCell<Program<'a>>,
    pub trivias: Trivias,

    /// Will be initialized if provided to constructor or first accessed.
    /// Prevents allocations for codegen tasks that don't require options.
    options: OnceCell<oxc_transformer::TransformOptions>,
    /// Generate source maps?
    source_map: bool,
    /// Generate `.d.ts` files?
    ///
    /// Used by [`crate::transform`].
    declarations: bool,

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
        options: Option<TransformOptions>,
    ) -> Self {
        let ParserReturn { errors, program, trivias, .. } =
            Parser::new(allocator, source_text, source_type).parse();

        // Options that are added by this napi crates and don't exist in
        // oxc_transformer.
        let source_map = options.as_ref().and_then(|o| o.sourcemap).unwrap_or_default();
        let declarations = options
            .as_ref()
            .and_then(|o| o.typescript.as_ref())
            .and_then(|t| t.declaration)
            .unwrap_or_default();

        // Insert options into the cell if provided. Otherwise they will be
        // initialized to default when first accessed.
        let options_cell = OnceCell::new();
        if let Some(options) = options {
            options_cell.set(options.into()).unwrap();
        }

        Self {
            allocator,
            program: RefCell::new(program),
            trivias,

            options: options_cell,
            source_map,
            declarations,

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
    pub fn file_path(&self) -> &'a Path {
        Path::new(self.filename)
    }

    #[inline]
    pub fn source_text(&self) -> &'a str {
        self.source_text
    }

    #[inline]
    pub fn oxc_options(&self) -> oxc_transformer::TransformOptions {
        self.options.get_or_init(Default::default).clone()
    }

    #[inline]
    pub(crate) fn declarations(&self) -> bool {
        self.declarations
    }

    #[inline]
    pub fn source_type(&self) -> SourceType {
        self.source_type
    }

    #[inline]
    pub fn program(&self) -> Ref<'_, Program<'a>> {
        self.program.borrow()
    }

    #[inline]
    pub fn program_mut(&self) -> RefMut<'_, Program<'a>> {
        self.program.borrow_mut()
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
