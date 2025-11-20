pub mod codegen;
pub mod estree;
pub mod formatter;
pub mod minifier;
pub mod parser;
pub mod semantic;
pub mod transformer;

use oxc::transformer::{JsxOptions, JsxRuntime, TransformOptions};

/// Get default transformer options with configurable JSX runtime
///
/// Most tools use the default (Automatic) runtime, but semantic analysis
/// uses Classic runtime to allow inline JSX pragmas in test files.
pub fn get_default_transformer_options(runtime: Option<JsxRuntime>) -> TransformOptions {
    TransformOptions {
        jsx: JsxOptions {
            jsx_plugin: true,
            jsx_self_plugin: true,
            jsx_source_plugin: true,
            runtime: runtime.unwrap_or_default(),
            ..Default::default()
        },
        ..TransformOptions::enable_all()
    }
}
