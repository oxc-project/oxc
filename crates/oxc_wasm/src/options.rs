use serde::Deserialize;
use tsify::Tsify;
use wasm_bindgen::prelude::*;

#[derive(Debug, Default, Clone, Deserialize, Tsify)]
#[tsify(from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct OxcOptions {
    #[tsify(optional)]
    pub run: Option<OxcRunOptions>,
    #[tsify(optional)]
    pub parser: Option<OxcParserOptions>,
    #[tsify(optional)]
    pub linter: Option<OxcLinterOptions>,
    #[tsify(optional)]
    pub transformer: Option<OxcTransformerOptions>,
    #[tsify(optional)]
    pub codegen: Option<OxcCodegenOptions>,
    #[tsify(optional)]
    pub minifier: Option<OxcMinifierOptions>,
    #[tsify(optional)]
    pub control_flow: Option<OxcControlFlowOptions>,
}

#[derive(Debug, Default, Clone, Deserialize, Tsify)]
#[tsify(from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct OxcRunOptions {
    #[tsify(optional)]
    pub syntax: Option<bool>,
    #[tsify(optional)]
    pub lint: Option<bool>,
    #[tsify(optional)]
    pub format: Option<bool>,
    #[tsify(optional)]
    pub prettier_format: Option<bool>,
    #[tsify(optional)]
    pub prettier_ir: Option<bool>,
    #[tsify(optional)]
    pub transform: Option<bool>,
    #[tsify(optional)]
    pub type_check: Option<bool>,
    #[tsify(optional)]
    pub scope: Option<bool>,
    #[tsify(optional)]
    pub symbol: Option<bool>,
}

#[derive(Debug, Default, Clone, Deserialize, Tsify)]
#[tsify(from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct OxcParserOptions {
    #[tsify(optional)]
    pub allow_return_outside_function: Option<bool>,
    #[tsify(optional)]
    pub preserve_parens: Option<bool>,
    #[tsify(optional, type = "\"script\" | \"module\"")]
    pub source_type: Option<String>,
    #[tsify(optional)]
    pub source_filename: Option<String>,
}

#[derive(Debug, Default, Clone, Deserialize, Tsify)]
#[tsify(from_wasm_abi)]
#[serde(rename_all = "camelCase")]
// allow empty object for future compatibility
#[allow(clippy::empty_structs_with_brackets)]
pub struct OxcLinterOptions {}

#[derive(Debug, Default, Clone, Deserialize, Tsify)]
#[tsify(from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct OxcTransformerOptions {
    #[tsify(optional)]
    pub target: Option<String>,
}

#[derive(Debug, Default, Clone, Deserialize, Tsify)]
#[tsify(from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct OxcCodegenOptions {
    #[tsify(optional)]
    pub indentation: Option<u8>,
    #[tsify(optional)]
    pub enable_typescript: Option<bool>,
}

#[derive(Debug, Default, Clone, Deserialize, Tsify)]
#[tsify(from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct OxcControlFlowOptions {
    #[tsify(optional)]
    pub verbose: Option<bool>,
}

#[derive(Debug, Default, Clone, Deserialize, Tsify)]
#[tsify(from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct OxcMinifierOptions {
    #[tsify(optional)]
    pub whitespace: Option<bool>,
    #[tsify(optional)]
    pub mangle: Option<bool>,
    #[tsify(optional)]
    pub compress: Option<bool>,
    #[tsify(optional)]
    pub compress_options: Option<OxcCompressOptions>,
}

#[derive(Debug, Clone, Deserialize, Tsify)]
#[tsify(from_wasm_abi)]
pub struct OxcCompressOptions {
    pub booleans: bool,
    pub drop_debugger: bool,
    pub drop_console: bool,
    pub evaluate: bool,
    pub join_vars: bool,
    pub loops: bool,
    pub typeofs: bool,
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
