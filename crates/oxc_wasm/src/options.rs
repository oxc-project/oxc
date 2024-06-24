use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Default, Clone, Copy)]
pub struct OxcRunOptions {
    syntax: bool,
    lint: bool,
    format: bool,
    prettier_format: bool,
    prettier_ir: bool,
    transform: bool,
    type_check: bool,
    scope: bool,
    symbol: bool,
}

#[wasm_bindgen]
impl OxcRunOptions {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self { syntax: true, lint: true, ..Self::default() }
    }

    #[wasm_bindgen(getter)]
    pub fn syntax(&self) -> bool {
        self.syntax
    }

    #[wasm_bindgen(setter)]
    pub fn set_syntax(&mut self, yes: bool) {
        self.syntax = yes;
    }

    #[wasm_bindgen(getter)]
    pub fn lint(&self) -> bool {
        self.lint
    }

    #[wasm_bindgen(setter)]
    pub fn set_lint(&mut self, yes: bool) {
        self.lint = yes;
    }

    #[wasm_bindgen(getter)]
    pub fn format(&self) -> bool {
        self.format
    }

    #[wasm_bindgen(setter)]
    pub fn set_format(&mut self, yes: bool) {
        self.format = yes;
    }

    #[wasm_bindgen(getter)]
    pub fn prettier_format(&self) -> bool {
        self.prettier_format
    }

    #[wasm_bindgen(setter)]
    pub fn set_prettier_format(&mut self, yes: bool) {
        self.prettier_format = yes;
    }

    #[wasm_bindgen(getter)]
    pub fn prettier_ir(&self) -> bool {
        self.prettier_ir
    }

    #[wasm_bindgen(setter)]
    pub fn set_prettier_ir(&mut self, yes: bool) {
        self.prettier_ir = yes;
    }

    #[wasm_bindgen(getter)]
    pub fn transform(&self) -> bool {
        self.transform
    }

    #[wasm_bindgen(setter)]
    pub fn set_transform(&mut self, yes: bool) {
        self.transform = yes;
    }

    #[wasm_bindgen(getter)]
    pub fn type_check(&self) -> bool {
        self.type_check
    }

    #[wasm_bindgen(setter)]
    pub fn set_type_check(&mut self, yes: bool) {
        self.type_check = yes;
    }

    #[wasm_bindgen(getter)]
    pub fn scope(&self) -> bool {
        self.scope
    }

    #[wasm_bindgen(setter)]
    pub fn set_scope(&mut self, yes: bool) {
        self.scope = yes;
    }

    #[wasm_bindgen(getter)]
    pub fn symbol(&self) -> bool {
        self.symbol
    }

    #[wasm_bindgen(setter)]
    pub fn set_symbol(&mut self, yes: bool) {
        self.symbol = yes;
    }
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Default, Clone)]
pub struct OxcParserOptions {
    #[wasm_bindgen(js_name = allowReturnOutsideFunction)]
    pub allow_return_outside_function: bool,

    #[wasm_bindgen(js_name = sourceFilename)]
    pub source_filename: Option<String>,
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
pub struct OxcCodegenOptions {
    pub indentation: u8,
    #[wasm_bindgen(js_name = enableTypescript)]
    pub enable_typescript: bool,
}

#[wasm_bindgen]
impl OxcCodegenOptions {
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
    compress_options: OxcCompressOptions,
}

#[wasm_bindgen]
#[allow(clippy::trivially_copy_pass_by_ref)]
impl OxcMinifierOptions {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::default()
    }

    #[wasm_bindgen(getter)]
    pub fn whitespace(&self) -> bool {
        self.whitespace
    }

    #[wasm_bindgen(setter)]
    pub fn set_whitespace(&mut self, yes: bool) {
        self.whitespace = yes;
    }

    #[wasm_bindgen(getter)]
    pub fn mangle(&self) -> bool {
        self.mangle
    }

    #[wasm_bindgen(setter)]
    pub fn set_mangle(&mut self, yes: bool) {
        self.mangle = yes;
    }

    #[wasm_bindgen(getter)]
    pub fn compress(&self) -> bool {
        self.compress
    }

    #[wasm_bindgen(setter)]
    pub fn set_compress(&mut self, yes: bool) {
        self.compress = yes;
    }

    #[wasm_bindgen(getter = compressOptions)]
    pub fn compress_options(&self) -> OxcCompressOptions {
        self.compress_options
    }

    #[wasm_bindgen(setter = compressOptions)]
    pub fn set_compress_options(&mut self, options: OxcCompressOptions) {
        self.compress_options = options;
    }
}

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct OxcCompressOptions {
    booleans: bool,
    drop_debugger: bool,
    drop_console: bool,
    evaluate: bool,
    join_vars: bool,
    loops: bool,
    typeofs: bool,
}

// keep same with `oxc_minifier::options::CompressOptions`
impl Default for OxcCompressOptions {
    fn default() -> Self {
        Self {
            booleans: true,
            drop_debugger: true,
            drop_console: false,
            evaluate: true,
            join_vars: true,
            loops: true,
            typeofs: true,
        }
    }
}

#[wasm_bindgen]
impl OxcCompressOptions {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::default()
    }

    #[wasm_bindgen(getter)]
    pub fn booleans(self) -> bool {
        self.booleans
    }

    #[wasm_bindgen(setter)]
    pub fn set_booleans(&mut self, yes: bool) {
        self.booleans = yes;
    }

    #[wasm_bindgen(getter = dropDebugger)]
    pub fn drop_debugger(self) -> bool {
        self.drop_debugger
    }

    #[wasm_bindgen(setter = dropDebugger)]
    pub fn set_drop_debugger(&mut self, yes: bool) {
        self.drop_debugger = yes;
    }

    #[wasm_bindgen(getter = dropConsole)]
    pub fn drop_console(self) -> bool {
        self.drop_console
    }

    #[wasm_bindgen(setter = dropConsole)]
    pub fn set_drop_console(&mut self, yes: bool) {
        self.drop_console = yes;
    }

    #[wasm_bindgen(getter)]
    pub fn evaluate(self) -> bool {
        self.evaluate
    }

    #[wasm_bindgen(setter)]
    pub fn set_evaluate(&mut self, yes: bool) {
        self.evaluate = yes;
    }

    #[wasm_bindgen(getter = joinVars)]
    pub fn join_vars(self) -> bool {
        self.join_vars
    }

    #[wasm_bindgen(setter = joinVars)]
    pub fn set_join_vars(&mut self, yes: bool) {
        self.join_vars = yes;
    }

    #[wasm_bindgen(getter)]
    pub fn loops(self) -> bool {
        self.loops
    }

    #[wasm_bindgen(setter)]
    pub fn set_loops(&mut self, yes: bool) {
        self.loops = yes;
    }

    #[wasm_bindgen(getter)]
    pub fn typeofs(self) -> bool {
        self.typeofs
    }

    #[wasm_bindgen(setter)]
    pub fn set_typeofs(&mut self, yes: bool) {
        self.typeofs = yes;
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
