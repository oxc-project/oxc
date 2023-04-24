use std::{cell::RefCell, rc::Rc};

use oxc_allocator::Allocator;
use oxc_ast::SourceType;
use oxc_diagnostics::Error;
use oxc_linter::Linter;
use oxc_minifier::{Minifier, MinifierOptions};
use oxc_parser::Parser;
use oxc_printer::{Printer, PrinterOptions};
use oxc_semantic::SemanticBuilder;
use serde::ser::Serialize;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
#[derive(Default)]
pub struct Oxc {
    source_text: String,

    options: OxcOptions,

    ast: JsValue,

    printed_text: String,

    diagnostics: RefCell<Vec<Error>>,

    serializer: serde_wasm_bindgen::Serializer,
}

#[wasm_bindgen]
#[derive(Default, Clone, Copy)]
pub struct OxcOptions {
    pub parser: Option<OxcParserOptions>,
    pub linter: Option<OxcLinterOptions>,
    pub minifier: Option<OxcMinifierOptions>,
    pub printer: Option<OxcPrinterOptions>,
}

#[wasm_bindgen]
#[derive(Default, Clone, Copy)]
pub struct OxcParserOptions {
    #[wasm_bindgen(js_name = allowReturnOutsideFunction)]
    pub allow_return_outside_function: bool,
}

#[wasm_bindgen]
#[derive(Default, Clone, Copy)]
pub struct OxcLinterOptions;

#[wasm_bindgen]
#[derive(Default, Clone, Copy)]
pub struct OxcMinifierOptions;

#[wasm_bindgen]
#[derive(Default, Clone, Copy)]
pub struct OxcPrinterOptions {
    mangle: bool,
    #[wasm_bindgen(js_name = minifyWhitespace)]
    pub minify_whitespace: bool,
    pub indentation: u8,
}

#[wasm_bindgen]
impl Oxc {
    #[wasm_bindgen(constructor)]
    #[must_use]
    pub fn new() -> Self {
        Self { serializer: serde_wasm_bindgen::Serializer::json_compatible(), ..Self::default() }
    }

    #[wasm_bindgen(js_name = setOptions)]
    pub fn set_options(&mut self, options: OxcOptions) {
        self.diagnostics = RefCell::default();
        self.options = options;
    }

    #[wasm_bindgen(js_name = setSourceText)]
    pub fn set_source_text(&mut self, text: String) {
        self.diagnostics = RefCell::default();
        self.source_text = text;
    }

    /// Returns AST in JSON
    #[wasm_bindgen(js_name = getAst)]
    pub fn get_ast(&self) -> JsValue {
        self.ast.clone()
    }

    #[wasm_bindgen(js_name = getPrintedText)]
    pub fn get_printed_text(&self) -> String {
        self.printed_text.clone()
    }

    /// Returns Array of String
    #[wasm_bindgen(js_name = getDiagnostics)]
    pub fn get_diagnostics(&self) -> Result<Vec<JsValue>, serde_wasm_bindgen::Error> {
        self.diagnostics
            .borrow()
            .iter()
            .map(|error| {
                let s = format!("{error:?}");
                s.serialize(&self.serializer)
            })
            .collect()
    }

    /// # Errors
    /// Serde serialization error
    #[wasm_bindgen]
    pub fn run(&mut self) -> Result<(), serde_wasm_bindgen::Error> {
        let Some(parser_options) = &self.options.parser else {
            return Ok(())
        };

        let allocator = Allocator::default();
        let source_text = &self.source_text;
        let source_type = SourceType::from_path("test.tsx").unwrap_or_default();

        let ret = Parser::new(&allocator, source_text, source_type)
            .allow_return_outside_function(parser_options.allow_return_outside_function)
            .parse();
        self.save_diagnostics(ret.errors);

        let program = allocator.alloc(ret.program);

        if self.options.minifier.is_some() {
            let minifier_options = MinifierOptions::default();
            Minifier::new(&allocator, minifier_options).build(program);
        }

        self.ast = program.serialize(&self.serializer)?;

        let semantic_ret = SemanticBuilder::new(source_text, source_type, &ret.trivias)
            .with_module_record_builder(true)
            .with_check_syntax_error(true)
            .build(program);
        let semantic = Rc::new(semantic_ret.semantic);
        self.save_diagnostics(semantic_ret.errors);

        if self.options.linter.is_some() {
            let linter_ret = Linter::new().run(&semantic);
            let diagnostics = linter_ret.into_iter().map(|e| e.error).collect();
            self.save_diagnostics(diagnostics);
        }

        if let Some(o) = &self.options.printer {
            let printer_options = PrinterOptions {
                minify_whitespace: o.minify_whitespace,
                indentation: o.indentation,
            };
            let printed = Printer::new(source_text.len(), printer_options)
                .with_symbol_table(&semantic.symbols(), o.mangle)
                .build(program);
            self.printed_text = printed;
        }

        Ok(())
    }

    fn save_diagnostics(&self, diagnostics: Vec<Error>) {
        self.diagnostics.borrow_mut().extend(
            diagnostics
                .into_iter()
                .map(|diagnostic| diagnostic.with_source_code(self.source_text.clone())),
        );
    }
}
