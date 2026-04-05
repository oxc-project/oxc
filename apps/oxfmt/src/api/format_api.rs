use std::{env, path::PathBuf};

use serde_json::Value;

use oxc_napi::OxcError;

use crate::core::{
    ExternalFormatter, FormatFileStrategy, FormatResult, JsFormatEmbeddedCb, JsFormatEmbeddedDocCb,
    JsFormatFileCb, JsInitExternalFormatterCb, JsSortTailwindClassesCb, SourceFormatter,
    extract_external_plugin_specs, resolve_options_from_value,
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

    let raw_options = options.unwrap_or_default();
    let plugin_specs = extract_external_plugin_specs(&raw_options, Some(&cwd));

    // Use `block_in_place()` to avoid nested async runtime access
    let external_plugin_support = match tokio::task::block_in_place(|| {
        external_formatter.init(num_of_threads, &plugin_specs)
    }) {
        Ok(languages) => crate::core::ExternalPluginSupport::from_language_json_strings(&languages),
        Err(err) => {
            external_formatter.cleanup();
            return ApiFormatResult {
                code: source_text,
                errors: vec![OxcError::new(format!("Failed to setup external formatter: {err}"))],
            };
        }
    };

    // Determine format strategy from file path
    let Ok(strategy) = FormatFileStrategy::from_path_with_external_support(
        PathBuf::from(filename),
        &external_plugin_support,
    )
    .map(|s| s.resolve_relative_path(&cwd)) else {
        external_formatter.cleanup();
        return ApiFormatResult {
            code: source_text,
            errors: vec![OxcError::new(format!("Unsupported file type: {filename}"))],
        };
    };

    // Resolve format options directly from the provided options
    let resolved_options = match resolve_options_from_value(raw_options, &strategy, Some(&cwd)) {
        Ok(options) => options,
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
    let result = match tokio::task::block_in_place(|| {
        formatter.format(&strategy, &source_text, resolved_options)
    }) {
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
