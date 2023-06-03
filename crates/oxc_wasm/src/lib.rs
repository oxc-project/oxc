use std::{cell::RefCell, rc::Rc};

use oxc_allocator::Allocator;
use oxc_ast_lower::AstLower;
use oxc_diagnostics::Error;
use oxc_formatter::{Formatter, FormatterOptions};
use oxc_linter::Linter;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use serde::Serialize;
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
    hir: JsValue,

    formatted_text: String,

    diagnostics: RefCell<Vec<Error>>,

    serializer: serde_wasm_bindgen::Serializer,
}

#[wasm_bindgen]
#[derive(Default, Clone, Copy)]
pub struct OxcOptions {
    pub parser: Option<OxcParserOptions>,
    pub linter: Option<OxcLinterOptions>,
    pub formatter: Option<OxcFormatterOptions>,
}

#[wasm_bindgen]
impl OxcOptions {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            parser: Some(OxcParserOptions { allow_return_outside_function: true }),
            linter: Some(OxcLinterOptions),
            formatter: Some(OxcFormatterOptions { indentation: 8 }),
        }
    }
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
pub struct OxcFormatterOptions {
    pub indentation: u8,
}

// #[wasm_bindgen]
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

    /// Returns the source text
    #[wasm_bindgen(js_name = getSourceText)]
    pub fn get_source_text(&self) -> String {
        self.source_text.clone()
    }

    /// Returns AST in JSON
    #[wasm_bindgen(js_name = getAst)]
    pub fn get_ast(&self) -> JsValue {
        self.ast.clone()
    }

    /// Returns HIR in JSON
    #[wasm_bindgen(js_name = getHir)]
    pub fn get_hir(&self) -> JsValue {
        self.hir.clone()
    }

    #[wasm_bindgen(js_name = getFormattedText)]
    pub fn get_formatted_text(&self) -> String {
        self.formatted_text.clone()
    }

    /// Returns Array of String
    #[wasm_bindgen(js_name = getDiagnostics)]
    pub fn get_diagnostics(&self) -> Result<Vec<JsValue>, serde_wasm_bindgen::Error> {
        Ok(self
            .diagnostics
            .borrow()
            .iter()
            .flat_map(|error| {
                let Some(labels) = error.labels() else {
                    return vec![]
                };
                labels
                    .map(|label| {
                        OxcDiagnostic {
                            start: label.offset(),
                            end: label.offset() + label.len(),
                            severity: format!("{:?}", error.severity().unwrap_or_default()),
                            message: format!("{}", error),
                        }
                        .serialize(&self.serializer)
                        .unwrap()
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>())
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

        if let Some(o) = &self.options.formatter {
            let formatter_options = FormatterOptions { indentation: o.indentation };
            let printed = Formatter::new(source_text.len(), formatter_options).build(program);
            self.formatted_text = printed;
        }

        let ast_lower_ret = AstLower::new(&allocator, source_type).build(program);
        self.hir = ast_lower_ret.program.serialize(&self.serializer)?;

        Ok(())
    }

    fn save_diagnostics(&self, diagnostics: Vec<Error>) {
        self.diagnostics.borrow_mut().extend(diagnostics);
    }
}
