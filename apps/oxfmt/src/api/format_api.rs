use std::{env, path::Path, sync::Arc};

use serde_json::Value;

use oxc_napi::OxcError;

use crate::core::{
    ExternalFormatter, FormatResult, JsFormatEmbeddedCb, JsFormatEmbeddedDocCb, JsFormatFileCb,
    JsInitExternalFormatterCb, JsSortTailwindClassesCb, SourceFormatter, classify_file_kind,
    resolve, utils,
};

pub struct ApiFormatResult {
    pub code: String,
    pub errors: Vec<OxcError>,
}

/// `format()` implementation for NAPI API.
///
/// # Panics
/// Panics if the current working directory cannot be determined.
pub fn run(
    filename: &str,
    source_text: String,
    options: Option<Value>,
    init_external_formatter_cb: JsInitExternalFormatterCb,
    format_file_cb: JsFormatFileCb,
    format_embedded_cb: JsFormatEmbeddedCb,
    format_embedded_doc_cb: JsFormatEmbeddedDocCb,
    sort_tailwind_classes_cb: JsSortTailwindClassesCb,
) -> ApiFormatResult {
    // NOTE: In NAPI context, we don't have a config file path, since options are passed directly as a JSON.
    // However, relative -> absolute path conversion is needed for Tailwind plugin to work correctly,
    // use current working directory as the base.
    let cwd = env::current_dir().expect("Failed to get current working directory");
    let num_of_threads = 1;

    let external_formatter = ExternalFormatter::new(
        init_external_formatter_cb,
        format_file_cb,
        format_embedded_cb,
        format_embedded_doc_cb,
        sort_tailwind_classes_cb,
    );

    // Use `block_in_place()` to avoid nested async runtime access
    match tokio::task::block_in_place(|| external_formatter.init(num_of_threads)) {
        // TODO: Plugins support
        Ok(_) => {}
        Err(err) => {
            external_formatter.cleanup();
            return ApiFormatResult {
                code: source_text,
                errors: vec![OxcError::new(format!("Failed to setup external formatter: {err}"))],
            };
        }
    }

    let filepath = utils::normalize_relative_path(&cwd, Path::new(filename));
    let Some(kind) = classify_file_kind(Arc::from(filepath)) else {
        external_formatter.cleanup();
        return ApiFormatResult {
            code: source_text,
            errors: vec![OxcError::new(format!("Unsupported file type: {filename}"))],
        };
    };
    let strategy = match resolve(options.unwrap_or_default(), kind, Some(&cwd)) {
        Ok(strategy) => strategy,
        Err(err) => {
            external_formatter.cleanup();
            return ApiFormatResult {
                code: source_text,
                errors: vec![OxcError::new(format!("Failed to parse configuration: {err}"))],
            };
        }
    };

    // Create formatter and format
    let formatter = SourceFormatter::new(num_of_threads)
        .with_external_formatter(Some(external_formatter.clone()));

    // Use `block_in_place()` to avoid nested async runtime access
    let result = match tokio::task::block_in_place(|| formatter.format(&source_text, strategy)) {
        FormatResult::Success { code, .. } => ApiFormatResult { code, errors: vec![] },
        FormatResult::Error(diagnostics) => {
            let errors = OxcError::from_diagnostics(filename, &source_text, diagnostics);
            ApiFormatResult { code: source_text, errors }
        }
    };

    // Explicitly drop ThreadsafeFunctions before returning to prevent
    // use-after-free during V8 cleanup (Node.js issue with TSFN cleanup timing)
    external_formatter.cleanup();

    result
}
