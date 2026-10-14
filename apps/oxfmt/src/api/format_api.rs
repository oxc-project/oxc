use std::{env, path::Path};

use serde_json::Value;

use oxc_napi::OxcError;

use crate::core::{
    ExternalServices, FormatResult, JsFormatEmbeddedCb, JsFormatEmbeddedDocCb, JsFormatFileCb,
    JsSortTailwindClassesCb, ResolveOutcome, SourceFormatter, classify_file, resolve_for_api,
    utils,
};

pub struct ApiFormatResult {
    pub code: String,
    pub errors: Vec<OxcError>,
}

/// `format()` implementation for the NAPI direct-document API.
///
/// This path formats the caller-supplied document and options directly.
/// It does not discover source files or search for project config or ignore files.
///
/// # Panics
/// Panics if the current working directory cannot be determined.
pub fn run(
    filename: &str,
    source_text: String,
    options: Option<Value>,
    format_file_cb: JsFormatFileCb,
    format_embedded_cb: JsFormatEmbeddedCb,
    format_embedded_doc_cb: JsFormatEmbeddedDocCb,
    sort_tailwind_classes_cb: JsSortTailwindClassesCb,
) -> ApiFormatResult {
    // NOTE: In NAPI context, we don't have a config file path, since options are passed directly as a JSON.
    // However, relative -> absolute path conversion is needed for Tailwind plugin to work correctly,
    // use current working directory as the base.
    // (Otherwise, the plugin resolves them against the Prettier config file, see `resolve_tailwind_paths()`.)
    //
    // `cwd` is intentionally not an API parameter, same as Prettier's `format()`.
    // Options are expected to be resolved by the caller, relative paths just fall back to `process.cwd()`.
    // To resolve against another base, resolve options beforehand (e.g. from the config file dir).
    //
    // Normalizing `filename` is not strictly required, since downstream consumers resolve it against `process.cwd()` too.
    // It only keeps paths absolute and consistent inside, e.g. for error messages.
    let cwd = env::current_dir().expect("Failed to get current working directory");
    let num_of_threads = 1;

    let external_services = ExternalServices::new(
        format_file_cb,
        format_embedded_cb,
        format_embedded_doc_cb,
        sort_tailwind_classes_cb,
    );
    let _cleanup = external_services.cleanup_guard();

    let filepath = utils::normalize_relative_path(&cwd, Path::new(filename));
    let Some(strategy) = classify_file(&filepath) else {
        return ApiFormatResult {
            code: source_text,
            errors: vec![OxcError::new(format!("Unsupported file type: {filename}"))],
        };
    };
    let plan = match resolve_for_api(options.unwrap_or_default(), &filepath, strategy, &cwd) {
        Ok(ResolveOutcome::Format(plan)) => plan,
        Ok(ResolveOutcome::MissingPlugin(plugin)) => {
            return ApiFormatResult {
                code: source_text,
                errors: vec![OxcError::new(format!(
                    "Cannot format `.{plugin}`: `{plugin}` plugin is not enabled in resolved config: {filename}"
                ))],
            };
        }
        Err(err) => {
            return ApiFormatResult {
                code: source_text,
                errors: vec![OxcError::new(format!("Failed to parse configuration: {err}"))],
            };
        }
    };

    // Create formatter and format
    let formatter =
        SourceFormatter::new(num_of_threads).with_external_services(Some(external_services));

    // Use `block_in_place()` to avoid nested async runtime access
    match tokio::task::block_in_place(|| formatter.format(&source_text, plan)) {
        FormatResult::Success { code, .. } => ApiFormatResult { code, errors: vec![] },
        FormatResult::Error(diagnostics) => {
            let errors = OxcError::from_diagnostics(filename, &source_text, diagnostics);
            ApiFormatResult { code: source_text, errors }
        }
    }
}
