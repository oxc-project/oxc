use napi::Either;
use napi_derive::napi;
use rustc_hash::FxHashMap;

#[napi(object)]
#[derive(Default)]
pub struct OxcOptions {
    pub run: OxcRunOptions,
    pub parser: OxcParserOptions,
    pub linter: Option<OxcLinterOptions>,
    pub transformer: Option<OxcTransformerOptions>,
    pub isolated_declarations: Option<OxcIsolatedDeclarationsOptions>,
    pub codegen: Option<OxcCodegenOptions>,
    pub compress: Option<OxcCompressOptions>,
    pub mangle: Option<OxcMangleOptions>,
    pub control_flow: Option<OxcControlFlowOptions>,
    pub inject: Option<OxcInjectOptions>,
    pub define: Option<OxcDefineOptions>,
}

#[napi(object)]
#[derive(Default, Clone, Copy)]
pub struct OxcRunOptions {
    pub lint: bool,
    pub formatter: bool,
    pub formatter_ir: bool,
    pub transform: bool,
    pub isolated_declarations: bool,
    pub whitespace: bool,
    pub compress: bool,
    pub mangle: bool,
    pub scope: bool,
    pub symbol: bool,
}

#[napi(object)]
#[derive(Default, Clone)]
pub struct OxcParserOptions {
    pub extension: String,
    pub allow_return_outside_function: bool,
    pub preserve_parens: bool,
    pub allow_v8_intrinsics: bool,
}

#[napi(object)]
#[derive(Default, Clone)]
pub struct OxcLinterOptions {
    pub config: Option<String>,
}

#[napi(object)]
#[derive(Default, Clone)]
pub struct OxcTransformerOptions {
    pub target: Option<String>,
    pub use_define_for_class_fields: bool,
    pub experimental_decorators: bool,
    pub emit_decorator_metadata: bool,
}

#[napi(object)]
#[derive(Default, Clone)]
pub struct OxcInjectOptions {
    /// Map of variable name to module source or [source, specifier]
    #[napi(ts_type = "Record<string, string | [string, string]>")]
    pub inject: FxHashMap<String, Either<String, Vec<String>>>,
}

#[napi(object)]
#[derive(Default, Clone)]
pub struct OxcDefineOptions {
    /// Map of variable name to value for replacement
    #[napi(ts_type = "Record<string, string>")]
    pub define: FxHashMap<String, String>,
}

#[napi(object)]
#[derive(Default, Clone)]
pub struct OxcIsolatedDeclarationsOptions {
    pub strip_internal: bool,
}

#[napi(object)]
#[derive(Clone)]
pub struct OxcCodegenOptions {
    pub normal: bool,
    pub jsdoc: bool,
    pub annotation: bool,
    pub legal: bool,
}

impl Default for OxcCodegenOptions {
    fn default() -> Self {
        Self { normal: true, jsdoc: true, annotation: true, legal: true }
    }
}

#[napi(object)]
#[derive(Default, Clone)]
pub struct OxcControlFlowOptions {
    pub verbose: Option<bool>,
}

#[napi(object)]
#[derive(Clone, Copy)]
pub struct OxcMangleOptions {
    pub top_level: bool,
    pub keep_names: bool,
}

#[napi(object)]
#[derive(Clone, Copy, Default)]
pub struct OxcCompressOptions;
