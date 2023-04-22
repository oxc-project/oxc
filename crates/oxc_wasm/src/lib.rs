use oxc_allocator::Allocator;
use oxc_ast::SourceType;
use oxc_diagnostics::Error;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub struct Oxc {
    diagnostics: Vec<Error>,
}

#[wasm_bindgen]
impl Oxc {
    #[wasm_bindgen(constructor)]
    #[allow(clippy::new_without_default)]
    #[must_use]
    pub fn new() -> Self {
        Self { diagnostics: vec![] }
    }

    #[wasm_bindgen]
    #[must_use]
    pub fn diagnostics(&self) -> JsValue {
        let diagnostics = self
            .diagnostics
            .iter()
            .map(|error| format!("{error:?}"))
            .collect::<Vec<String>>()
            .join("\n");
        JsValue::from_str(&diagnostics)
    }

    /// # Panics
    #[wasm_bindgen]
    #[must_use]
    pub fn parse(&mut self, source_text: &str) -> JsValue {
        self.diagnostics = vec![];

        let allocator = Allocator::default();
        let source_type = SourceType::from_path("test.tsx").unwrap_or_default();

        let ret = Parser::new(&allocator, source_text, source_type)
            .allow_return_outside_function(true)
            .parse();
        self.diagnostics.extend(ret.errors);

        let program = allocator.alloc(ret.program);
        let semantic_ret = SemanticBuilder::new(source_text, source_type, &ret.trivias)
            .with_module_record_builder(true)
            .with_check_syntax_error(true)
            .build(program);
        self.diagnostics.extend(semantic_ret.errors);

        serde_wasm_bindgen::to_value(program).unwrap()
    }
}
