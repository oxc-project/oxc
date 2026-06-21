use std::path::Path;

use napi_derive::napi;

use oxc::{
    allocator::Allocator,
    codegen::{Codegen, CodegenOptions},
    parser::Parser,
    span::SourceType,
};
use oxc_napi::OxcError;
use oxc_sourcemap::napi::SourceMap;

#[napi(object)]
pub struct ReactCompilerResult {
    /// The compiled code.
    ///
    /// When the compiler makes no changes (the file has no React component or
    /// hook, or compilation bails out), this is the **original source unchanged**
    /// — mirroring `babel-plugin-react-compiler`, which leaves untouched
    /// functions as-is.
    pub code: String,
    pub map: Option<SourceMap>,
    /// `true` if the compiler memoized at least one function (i.e. `code` is the
    /// recompiled program rather than the original source).
    pub changed: bool,
    pub errors: Vec<OxcError>,
}

#[napi(object)]
#[derive(Default)]
pub struct ReactCompilerOptions {
    pub sourcemap: Option<bool>,
    /// How the compiler decides which functions to compile.
    /// One of `"infer"` (default), `"syntax"`, `"annotation"`, `"all"`.
    pub compilation_mode: Option<String>,
    /// When the compiler should throw on an error rather than skip the function.
    /// One of `"none"` (default), `"critical_errors"`, `"all_errors"`.
    pub panic_threshold: Option<String>,
}

fn react_compiler_impl(
    filename: &str,
    source_text: &str,
    options: Option<ReactCompilerOptions>,
) -> ReactCompilerResult {
    let source_path = Path::new(filename);
    let source_type = SourceType::from_path(source_path).unwrap_or_else(|_| SourceType::tsx());
    let allocator = Allocator::default();
    let options = options.unwrap_or_default();

    let parsed = Parser::new(&allocator, source_text, source_type).parse();

    let mut plugin_options = oxc_react_compiler::default_plugin_options();
    plugin_options.filename = Some(filename.to_string());
    if let Some(mode) = options.compilation_mode {
        plugin_options.compilation_mode = mode;
    }
    if let Some(threshold) = options.panic_threshold {
        plugin_options.panic_threshold = threshold;
    }

    let result = oxc_react_compiler::transform(&parsed.program, &allocator, plugin_options);

    let diagnostics = parsed.diagnostics.into_iter().chain(result.diagnostics).collect::<Vec<_>>();
    let errors = OxcError::from_diagnostics(filename, source_text, diagnostics);

    match result.program {
        Some(compiled) => {
            let source_map_path = match options.sourcemap {
                Some(true) => Some(source_path.to_path_buf()),
                _ => None,
            };
            let codegen_ret = Codegen::new()
                .with_options(CodegenOptions { source_map_path, ..CodegenOptions::default() })
                .build(&compiled);
            ReactCompilerResult {
                code: codegen_ret.code,
                map: codegen_ret.map.map(SourceMap::from),
                changed: true,
                errors,
            }
        }
        None => {
            ReactCompilerResult { code: source_text.to_string(), map: None, changed: false, errors }
        }
    }
}

/// Run the React Compiler (oxc's native Rust port) on a single file.
///
/// Parses `source_text`, applies the compiler, and returns the recompiled code.
/// Intended to mirror `babel-plugin-react-compiler` for a single-file transform.
#[allow(clippy::needless_pass_by_value, clippy::allow_attributes)]
#[napi]
pub fn react_compiler_sync(
    filename: String,
    source_text: String,
    options: Option<ReactCompilerOptions>,
) -> ReactCompilerResult {
    react_compiler_impl(&filename, &source_text, options)
}
