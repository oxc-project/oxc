use napi::Either;
use napi_derive::napi;
use rustc_hash::FxHashMap;

#[napi(object)]
#[derive(Default)]
pub struct OxcOptions {
    pub run: OxcRunOptions,
    pub parser: OxcParserOptions,
    pub linter: Option<OxcLinterOptions>,
    pub formatter: Option<OxcFormatterOptions>,
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
    pub transform: bool,
    pub isolated_declarations: bool,
    pub whitespace: bool,
    pub compress: bool,
    pub mangle: bool,
    pub scope: bool,
    pub symbol: bool,
    pub cfg: bool,
}

#[napi(object)]
#[derive(Default, Clone)]
pub struct OxcParserOptions {
    pub extension: String,
    pub allow_return_outside_function: bool,
    pub preserve_parens: bool,
    pub allow_v8_intrinsics: bool,
    pub semantic_errors: bool,
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

#[napi(object)]
#[derive(Default, Clone)]
pub struct OxcFormatterOptions {
    /// Use tabs instead of spaces (default: false)
    pub use_tabs: Option<bool>,
    /// Number of spaces per indentation level (default: 2)
    pub tab_width: Option<u8>,
    /// Line ending type: "lf" | "crlf" | "cr" (default: "lf")
    pub end_of_line: Option<String>,
    /// Maximum line width (default: 80)
    pub print_width: Option<u16>,
    /// Use single quotes instead of double quotes (default: false)
    pub single_quote: Option<bool>,
    /// Use single quotes in JSX (default: false)
    pub jsx_single_quote: Option<bool>,
    /// When to add quotes around object properties: "as-needed" | "consistent" | "preserve" (default: "as-needed")
    pub quote_props: Option<String>,
    /// Print trailing commas: "all" | "es5" | "none" (default: "all")
    pub trailing_comma: Option<String>,
    /// Print semicolons (default: true)
    pub semi: Option<bool>,
    /// Include parentheses around arrow function parameters: "always" | "avoid" (default: "always")
    pub arrow_parens: Option<String>,
    /// Print spaces between brackets in object literals (default: true)
    pub bracket_spacing: Option<bool>,
    /// Put > of multi-line elements at the end of the last line (default: false)
    pub bracket_same_line: Option<bool>,
    /// Object wrapping style: "preserve" | "collapse" | "always" (default: "preserve")
    pub object_wrap: Option<String>,
    /// Put each attribute on its own line (default: false)
    pub single_attribute_per_line: Option<bool>,
    /// Sort imports configuration (default: None)
    pub experimental_sort_imports: Option<OxcSortImportsOptions>,
}

#[napi(object)]
#[derive(Default, Clone)]
pub struct OxcSortImportsOptions {
    /// Partition imports by newlines (default: false)
    pub partition_by_newline: Option<bool>,
    /// Partition imports by comments (default: false)
    pub partition_by_comment: Option<bool>,
    /// Sort side effects imports (default: false)
    pub sort_side_effects: Option<bool>,
    /// Sort order: "asc" | "desc" (default: "asc")
    pub order: Option<String>,
    /// Ignore case when sorting (default: true)
    pub ignore_case: Option<bool>,
    /// Add newlines between import groups (default: true)
    pub newlines_between: Option<bool>,
    /// Pattern prefixes for internal imports
    pub internal_pattern: Option<Vec<String>>,
    /// Custom groups of imports
    pub groups: Option<Vec<Vec<String>>>,
}
