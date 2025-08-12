use napi_derive::napi;

#[napi(object)]
#[derive(Default)]
pub struct OxcOptions {
    pub run: Option<OxcRunOptions>,
    pub parser: Option<OxcParserOptions>,
    pub linter: Option<OxcLinterOptions>,
    pub transformer: Option<OxcTransformerOptions>,
    pub codegen: Option<OxcCodegenOptions>,
    pub minifier: Option<OxcMinifierOptions>,
    pub formatter: Option<OxcFormatterOptions>,
    pub control_flow: Option<OxcControlFlowOptions>,
}

#[napi(object)]
#[derive(Default)]
pub struct OxcRunOptions {
    pub syntax: Option<bool>,
    pub lint: Option<bool>,
    pub format: Option<bool>,
    pub formatter_format: Option<bool>,
    pub formatter_ir: Option<bool>,
    pub transform: Option<bool>,
    pub type_check: Option<bool>,
    pub scope: Option<bool>,
    pub symbol: Option<bool>,
}

#[napi(object)]
#[derive(Default)]
pub struct OxcParserOptions {
    pub allow_return_outside_function: Option<bool>,
    pub preserve_parens: Option<bool>,
    pub allow_v8_intrinsics: Option<bool>,
    /// Whether to parse regular expressions or not
    pub parse_regular_expression: Option<bool>,
    pub source_type: Option<String>,
    pub source_filename: Option<String>,
}

#[napi(object)]
#[derive(Default)]
pub struct OxcLinterOptions {
    pub config: Option<String>,
}

#[napi(object)]
#[derive(Default)]
pub struct OxcTransformerOptions {
    pub target: Option<String>,
    pub isolated_declarations: Option<bool>,
    /// TypeScript transformation options
    pub typescript: Option<OxcTypeScriptOptions>,
    /// JSX transformation options
    pub jsx: Option<OxcJsxOptions>,
    /// ES2015 transformation options
    pub es2015: Option<OxcES2015Options>,
    /// ES2016 transformation options  
    pub es2016: Option<OxcES2016Options>,
    /// ES2017 transformation options
    pub es2017: Option<OxcES2017Options>,
    /// ES2018 transformation options
    pub es2018: Option<OxcES2018Options>,
    /// ES2019 transformation options
    pub es2019: Option<OxcES2019Options>,
    /// ES2020 transformation options
    pub es2020: Option<OxcES2020Options>,
    /// ES2021 transformation options
    pub es2021: Option<OxcES2021Options>,
    /// ES2022 transformation options
    pub es2022: Option<OxcES2022Options>,
}

#[napi(object)]
#[derive(Default)]
pub struct OxcCodegenOptions {
    pub indentation: Option<u8>,
    pub enable_typescript: Option<bool>,
    pub enable_sourcemap: Option<bool>,
    /// Use single quotes instead of double quotes
    pub single_quote: Option<bool>,
    /// Remove whitespace
    pub minify: Option<bool>,
    /// Indentation character ('tab' or 'space')
    pub indent_char: Option<String>,
    /// Number of characters per indentation level
    pub indent_width: Option<u8>,
    /// Enable comments
    pub comments: Option<OxcCodegenCommentOptions>,
}

#[napi(object)]
#[derive(Default)]
pub struct OxcControlFlowOptions {
    pub verbose: Option<bool>,
}

#[napi(object)]
#[derive(Default)]
pub struct OxcMinifierOptions {
    pub whitespace: Option<bool>,
    pub mangle: Option<bool>,
    pub compress: Option<bool>,
    pub compress_options: Option<OxcCompressOptions>,
}

#[napi(object)]
pub struct OxcCompressOptions {
    /// Set desired EcmaScript standard version for output
    pub target: Option<String>,
    /// Remove `debugger;` statements
    pub drop_debugger: Option<bool>,
    /// Remove `console.*` statements
    pub drop_console: Option<bool>,
    /// Join consecutive var, let and const statements
    pub join_vars: Option<bool>,
    /// Join consecutive simple statements using the comma operator
    pub sequences: Option<bool>,
    /// Drop unreferenced functions and variables ('remove', 'keep_assign', 'keep')
    pub unused: Option<String>,
    /// Keep function / class names
    pub keep_names: Option<OxcCompressKeepNamesOptions>,
    /// Treeshake options
    pub treeshake: Option<OxcTreeShakeOptions>,
    // Legacy options for backward compatibility
    pub booleans: Option<bool>,
    pub evaluate: Option<bool>,
    pub loops: Option<bool>,
    pub typeofs: Option<bool>,
}

// keep same with `oxc_minifier::options::CompressOptions`
impl Default for OxcCompressOptions {
    fn default() -> Self {
        Self {
            target: None,
            drop_debugger: Some(true),
            drop_console: Some(false),
            join_vars: Some(true),
            sequences: Some(true),
            unused: Some("remove".to_string()),
            keep_names: None,
            treeshake: None,
            // Legacy options
            booleans: Some(true),
            evaluate: Some(true),
            loops: Some(true),
            typeofs: Some(true),
        }
    }
}

#[napi(object)]
#[derive(Default)]
pub struct OxcCompressKeepNamesOptions {
    /// Keep function names
    pub function: Option<bool>,
    /// Keep class names
    pub class: Option<bool>,
}

#[napi(object)]
#[derive(Default)]
pub struct OxcTreeShakeOptions {
    /// Whether to respect the pure annotations
    pub annotations: Option<bool>,
    /// Manual pure functions (array of function names)
    pub manual_pure_functions: Option<Vec<String>>,
    /// Property read side effects ('all', 'none', 'only_member')
    pub property_read_side_effects: Option<String>,
    /// Whether accessing a global variable has side effects
    pub unknown_global_side_effects: Option<bool>,
}

#[napi(object)]
#[derive(Default)]
pub struct OxcCodegenCommentOptions {
    /// Print normal comments that do not have special meanings
    pub normal: Option<bool>,
    /// Print jsdoc comments (`/** jsdoc */`)
    pub jsdoc: Option<bool>,
    /// Print annotation comments (pure, webpack, vite, coverage)
    pub annotation: Option<bool>,
    /// Print legal comments ('inline', 'eof', 'none', 'external')
    pub legal: Option<String>,
}

#[napi(object)]
#[derive(Default)]
pub struct OxcFormatterOptions {
    /// The indent style ('tab' or 'space')
    pub indent_style: Option<String>,
    /// The indent width
    pub indent_width: Option<u8>,
    /// The type of line ending ('lf', 'crlf', 'cr')
    pub line_ending: Option<String>,
    /// What's the max width of a line
    pub line_width: Option<u16>,
    /// The style for quotes ('double' or 'single')
    pub quote_style: Option<String>,
    /// The style for JSX quotes ('double' or 'single')
    pub jsx_quote_style: Option<String>,
    /// When properties in objects are quoted ('as-needed' or 'preserve')
    pub quote_properties: Option<String>,
    /// Print trailing commas wherever possible ('all', 'es5', 'none')
    pub trailing_commas: Option<String>,
    /// Whether the formatter prints semicolons ('always' or 'as-needed')
    pub semicolons: Option<String>,
    /// Whether to add non-necessary parentheses to arrow functions ('always' or 'as-needed')
    pub arrow_parentheses: Option<String>,
    /// Whether to insert spaces around brackets in object literals
    pub bracket_spacing: Option<bool>,
    /// Whether to hug the closing bracket of multiline HTML/JSX tags to the end of the last line
    pub bracket_same_line: Option<bool>,
    /// Attribute position style ('auto' or 'multiline')
    pub attribute_position: Option<String>,
    /// Whether to expand object and array literals to multiple lines ('auto', 'always', 'never')
    pub expand: Option<String>,
    /// Controls the position of operators in binary expressions ('start' or 'end')
    pub experimental_operator_position: Option<String>,
}

#[napi(object)]
#[derive(Default)]
pub struct OxcTypeScriptOptions {
    /// Only transform TypeScript syntax, do not downlevel it to earlier JavaScript
    pub only_remove_type_imports: Option<bool>,
    /// Allow declare module syntax
    pub allow_declare_module: Option<bool>,
    /// Allow namespace syntax
    pub allow_namespaces: Option<bool>,
}

#[napi(object)]
#[derive(Default)]
pub struct OxcJsxOptions {
    /// The JSX runtime to use ('automatic' or 'classic')
    pub runtime: Option<String>,
    /// Whether to use React development mode
    pub development: Option<bool>,
    /// The import source for automatic runtime
    pub import_source: Option<String>,
    /// The JSX pragma for classic runtime
    pub pragma: Option<String>,
    /// The JSX pragma fragment for classic runtime  
    pub pragma_frag: Option<String>,
    /// Whether to throw if a XML namespaced tag name is used
    pub throw_if_namespace: Option<bool>,
    /// Whether to use React.createElement for every JSX element
    pub pure: Option<bool>,
}

#[napi(object)]
#[derive(Default)]
pub struct OxcES2015Options {
    /// Transform arrow functions
    pub arrow_function: Option<OxcArrowFunctionOptions>,
}

#[napi(object)]
#[derive(Default)]
pub struct OxcArrowFunctionOptions {
    /// Whether arrow function expressions with parameter destructuring should use assign variable declarations
    pub spec: Option<bool>,
}

#[napi(object)]
#[derive(Default)]
pub struct OxcES2016Options {
    /// Transform exponentiation operator
    pub exponentiation_operator: Option<bool>,
}

#[napi(object)]
#[derive(Default)]
pub struct OxcES2017Options {
    /// Transform async to generator
    pub async_to_generator: Option<bool>,
}

#[napi(object)]
#[derive(Default)]
pub struct OxcES2018Options {
    /// Transform object rest spread
    pub object_rest_spread: Option<OxcObjectRestSpreadOptions>,
    /// Transform async generator functions
    pub async_generator_functions: Option<bool>,
}

#[napi(object)]
#[derive(Default)]
pub struct OxcObjectRestSpreadOptions {
    /// Whether to use Object.assign() instead of spread syntax
    pub loose: Option<bool>,
    /// Whether to use extends helper instead of Object.assign for rest destructuring
    pub use_built_ins: Option<bool>,
    /// Whether to set enumerable false for computed properties
    pub set_spread_properties: Option<bool>,
}

#[napi(object)]
#[derive(Default)]
pub struct OxcES2019Options {
    /// Transform optional catch binding
    pub optional_catch_binding: Option<bool>,
}

#[napi(object)]
#[derive(Default)]
pub struct OxcES2020Options {
    /// Transform optional chaining
    pub optional_chaining: Option<bool>,
    /// Transform nullish coalescing operator
    pub nullish_coalescing_operator: Option<bool>,
    /// Transform BigInt
    pub big_int: Option<bool>,
}

#[napi(object)]
#[derive(Default)]
pub struct OxcES2021Options {
    /// Transform logical assignment operators
    pub logical_assignment_operators: Option<bool>,
}

#[napi(object)]
#[derive(Default)]
pub struct OxcES2022Options {
    /// Transform class static block
    pub class_static_block: Option<bool>,
    /// Transform class properties
    pub class_properties: Option<OxcClassPropertiesOptions>,
}

#[napi(object)]
#[derive(Default)]
pub struct OxcClassPropertiesOptions {
    /// Whether to use loose mode
    pub loose: Option<bool>,
}
