mod options;

use std::{cell::RefCell, rc::Rc};

use oxc_allocator::Allocator;
use oxc_ast_lower::AstLower;
use oxc_diagnostics::Error;
use oxc_formatter::{Formatter, FormatterOptions};
use oxc_linter::{LintContext, Linter};
use oxc_minifier::{CompressOptions, Compressor, ManglerBuilder, Printer, PrinterOptions};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_type_synthesis::{synthesize_program, Diagnostic as TypeCheckDiagnostic};
use serde::Serialize;
use wasm_bindgen::prelude::*;

use crate::options::{
    OxcFormatterOptions, OxcLinterOptions, OxcMinifierOptions, OxcParserOptions, OxcRunOptions,
    OxcTypeCheckingOptions,
};

#[wasm_bindgen(start)]
pub fn main() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
#[derive(Default)]
pub struct Oxc {
    source_text: String,

    ast: JsValue,
    ir: JsValue,
    hir: JsValue,

    formatted_text: String,
    minified_text: String,

    diagnostics: RefCell<Vec<Error>>,

    type_check_diagnostics: RefCell<Vec<TypeCheckDiagnostic>>,

    serializer: serde_wasm_bindgen::Serializer,
}

#[derive(Default, Clone, Serialize)]
pub struct OxcDiagnostic {
    pub start: usize,
    pub end: usize,
    pub severity: String,
    pub message: String,
}

#[wasm_bindgen]
impl Oxc {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self { serializer: serde_wasm_bindgen::Serializer::json_compatible(), ..Self::default() }
    }

    #[wasm_bindgen(getter = sourceText)]
    pub fn source_text(&self) -> String {
        self.source_text.clone()
    }

    #[wasm_bindgen(setter = sourceText)]
    pub fn set_source_text(&mut self, source_text: String) {
        self.diagnostics = RefCell::default();
        self.source_text = source_text;
    }

    /// Returns AST in JSON
    #[wasm_bindgen(getter)]
    pub fn ast(&self) -> JsValue {
        self.ast.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn ir(&self) -> JsValue {
        self.ir.clone()
    }

    /// Returns HIR in JSON
    #[wasm_bindgen(getter)]
    pub fn hir(&self) -> JsValue {
        self.hir.clone()
    }

    #[wasm_bindgen(getter = formattedText)]
    pub fn formatted_text(&self) -> String {
        self.formatted_text.clone()
    }

    #[wasm_bindgen(getter = minifiedText)]
    pub fn minified_text(&self) -> String {
        self.minified_text.clone()
    }

    /// Returns Array of String
    #[wasm_bindgen(js_name = getDiagnostics)]
    pub fn get_diagnostics(&self) -> Result<Vec<JsValue>, serde_wasm_bindgen::Error> {
        Ok(self
            .diagnostics
            .borrow()
            .iter()
            .flat_map(|error| {
                let Some(labels) = error.labels() else { return vec![] };
                labels
                    .map(|label| {
                        OxcDiagnostic {
                            start: label.offset(),
                            end: label.offset() + label.len(),
                            severity: format!("{:?}", error.severity().unwrap_or_default()),
                            message: format!("{error}"),
                        }
                        .serialize(&self.serializer)
                        .unwrap()
                    })
                    .collect::<Vec<_>>()
            })
            .chain(self.type_check_diagnostics.borrow().iter().filter_map(|diagnostic| {
                match diagnostic {
                    TypeCheckDiagnostic::Global { .. } => None,
                    TypeCheckDiagnostic::PositionWithAdditionLabels {
                        reason,
                        position,
                        kind,
                        labels: _,
                    }
                    | TypeCheckDiagnostic::Position { reason, position, kind } => Some(
                        OxcDiagnostic {
                            start: position.start as usize,
                            end: position.end as usize,
                            severity: format!("{kind:?}"),
                            message: reason.to_string(),
                        }
                        .serialize(&self.serializer)
                        .unwrap(),
                    ),
                }
            }))
            .collect::<Vec<_>>())
    }

    /// # Errors
    /// Serde serialization error
    #[wasm_bindgen]
    pub fn run(
        &mut self,
        run_options: &OxcRunOptions,
        parser_options: &OxcParserOptions,
        _linter_options: &OxcLinterOptions,
        formatter_options: &OxcFormatterOptions,
        minifier_options: &OxcMinifierOptions,
        _type_checking_options: &OxcTypeCheckingOptions,
    ) -> Result<(), serde_wasm_bindgen::Error> {
        self.diagnostics = RefCell::default();

        let allocator = Allocator::default();
        let source_text = &self.source_text;
        let source_type = SourceType::from_path("test.tsx").unwrap_or_default();

        let ret = Parser::new(&allocator, source_text, source_type)
            .allow_return_outside_function(parser_options.allow_return_outside_function)
            .parse();
        self.save_diagnostics(ret.errors);

        self.ast = ret.program.serialize(&self.serializer)?;
        self.ir = format!("{:#?}", ret.program.body).into();
        let program = allocator.alloc(ret.program);

        if run_options.syntax() && !run_options.lint() {
            let semantic_ret = SemanticBuilder::new(source_text, source_type)
                .with_trivias(&ret.trivias)
                .with_check_syntax_error(true)
                .build(program);
            self.save_diagnostics(semantic_ret.errors);
        }

        if run_options.lint() {
            let semantic_ret = SemanticBuilder::new(source_text, source_type)
                .with_trivias(&ret.trivias)
                .with_check_syntax_error(true)
                .build(program);
            self.save_diagnostics(semantic_ret.errors);

            let semantic = Rc::new(semantic_ret.semantic);
            let lint_ctx = LintContext::new(&semantic);
            let linter_ret = Linter::new().run(lint_ctx);
            let diagnostics = linter_ret.into_iter().map(|e| e.error).collect();
            self.save_diagnostics(diagnostics);
        }

        if run_options.format() {
            let formatter_options = FormatterOptions { indentation: formatter_options.indentation };
            let printed = Formatter::new(source_text.len(), formatter_options).build(program);
            self.formatted_text = printed;
        }

        if run_options.hir() && !run_options.minify() {
            let ast_lower_ret = AstLower::new(&allocator, source_text, source_type).build(program);
            self.hir = ast_lower_ret.program.serialize(&self.serializer)?;
        }

        if run_options.minify() {
            let ast_lower_ret = AstLower::new(&allocator, source_text, source_type).build(program);
            let hir = allocator.alloc(ast_lower_ret.program);
            let semantic = ast_lower_ret.semantic;

            let mut printer = Printer::new(self.source_text.len(), PrinterOptions);
            let _semantic =
                Compressor::new(&allocator, semantic, CompressOptions::default()).build(hir);
            if minifier_options.mangle() {
                let mangler = ManglerBuilder::new(source_text, source_type).build(hir);
                printer.with_mangler(mangler);
            }

            self.minified_text = printer.build(hir);
        }

        if run_options.type_check() {
            let (diagnostics, ..) = synthesize_program(program, |_: &std::path::Path| None);

            *self.type_check_diagnostics.borrow_mut() = diagnostics.get_diagnostics();
        }

        Ok(())
    }

    fn save_diagnostics(&self, diagnostics: Vec<Error>) {
        self.diagnostics.borrow_mut().extend(diagnostics);
    }
}
