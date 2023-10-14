use wasm_bindgen::prelude::*;

#[allow(clippy::struct_excessive_bools)]
#[wasm_bindgen]
#[derive(Default, Clone, Copy)]
pub struct OxcRunOptions {
    syntax: bool,
    lint: bool,
    format: bool,
    transform: bool,
    type_check: bool,
}

#[wasm_bindgen]
impl OxcRunOptions {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self { syntax: true, lint: true, ..Self::default() }
    }

    #[wasm_bindgen(getter)]
    pub fn syntax(self) -> bool {
        self.format
    }

    #[wasm_bindgen(setter)]
    pub fn set_syntax(&mut self, yes: bool) {
        self.syntax = yes;
    }

    #[wasm_bindgen(getter)]
    pub fn lint(self) -> bool {
        self.lint
    }

    #[wasm_bindgen(setter)]
    pub fn set_lint(&mut self, yes: bool) {
        self.lint = yes;
    }

    #[wasm_bindgen(getter)]
    pub fn format(self) -> bool {
        self.format
    }

    #[wasm_bindgen(setter)]
    pub fn set_format(&mut self, yes: bool) {
        self.format = yes;
    }

    #[wasm_bindgen(getter)]
    pub fn transform(self) -> bool {
        self.transform
    }

    #[wasm_bindgen(setter)]
    pub fn set_transform(&mut self, yes: bool) {
        self.transform = yes;
    }

    #[wasm_bindgen(getter)]
    pub fn type_check(self) -> bool {
        self.type_check
    }

    #[wasm_bindgen(setter)]
    pub fn set_type_check(&mut self, yes: bool) {
        self.type_check = yes;
    }
}

#[wasm_bindgen]
#[derive(Default, Clone, Copy)]
pub struct OxcParserOptions {
    #[wasm_bindgen(js_name = allowReturnOutsideFunction)]
    pub allow_return_outside_function: bool,
}

#[wasm_bindgen]
impl OxcParserOptions {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::default()
    }
}

#[wasm_bindgen]
#[derive(Default, Clone, Copy)]
pub struct OxcLinterOptions;

#[wasm_bindgen]
impl OxcLinterOptions {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self
    }
}

#[wasm_bindgen]
#[derive(Default, Clone, Copy)]
pub struct OxcFormatterOptions {
    pub indentation: u8,
}

#[wasm_bindgen]
impl OxcFormatterOptions {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::default()
    }
}

#[wasm_bindgen]
#[derive(Default, Clone, Copy)]
pub struct OxcMinifierOptions {
    whitespace: bool,
    mangle: bool,
    compress: bool,
}

#[wasm_bindgen]
impl OxcMinifierOptions {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::default()
    }

    #[wasm_bindgen(getter)]
    pub fn whitespace(self) -> bool {
        self.whitespace
    }

    #[wasm_bindgen(setter)]
    pub fn set_whitespace(&mut self, yes: bool) {
        self.whitespace = yes;
    }

    #[wasm_bindgen(getter)]
    pub fn mangle(self) -> bool {
        self.mangle
    }

    #[wasm_bindgen(setter)]
    pub fn set_mangle(&mut self, yes: bool) {
        self.mangle = yes;
    }

    #[wasm_bindgen(getter)]
    pub fn compress(self) -> bool {
        self.compress
    }

    #[wasm_bindgen(setter)]
    pub fn set_compress(&mut self, yes: bool) {
        self.compress = yes;
    }
}

#[wasm_bindgen]
#[derive(Default, Clone, Copy)]
pub struct OxcTypeCheckingOptions;

#[wasm_bindgen]
impl OxcTypeCheckingOptions {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self
    }
}
