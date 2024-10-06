use napi_derive::napi;

pub use oxc::napi::{
    isolated_declarations::{IsolatedDeclarationsOptions, IsolatedDeclarationsResult},
    transform::{
        ArrowFunctionsOptions, Es2015Options, JsxOptions, ReactRefreshOptions, TransformOptions,
        TransformResult, TypeScriptOptions,
    },
};

#[napi]
pub fn transform(
    filename: String,
    source_text: String,
    options: Option<TransformOptions>,
) -> TransformResult {
    oxc::napi::transform::transform(filename, source_text, options)
}

#[napi]
pub fn isolated_declaration(
    filename: String,
    source_text: String,
    options: Option<IsolatedDeclarationsOptions>,
) -> IsolatedDeclarationsResult {
    oxc::napi::isolated_declarations::isolated_declaration(filename, source_text, options)
}
