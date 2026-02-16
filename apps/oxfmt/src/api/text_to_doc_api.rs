use std::{env, path::PathBuf};

use serde_json::Value;

use oxc_napi::OxcError;
use oxc_span::SourceType;

use crate::core::{
    ExternalFormatter, FormatFileStrategy, FormatResult, JsFormatEmbeddedCb, JsFormatFileCb,
    JsInitExternalFormatterCb, JsSortTailwindClassesCb, SourceFormatter,
    resolve_options_from_value,
};

/// `js_text_to_doc()` implementation for NAPI API.
///
/// # Panics
/// Panics if the current working directory cannot be determined.
pub fn run(
    filename: &str,
    source_text: &str,
    oxfmt_plugin_options_json: &str,
    parent_context: &str,
    init_external_formatter_cb: JsInitExternalFormatterCb,
    format_embedded_cb: JsFormatEmbeddedCb,
    format_file_cb: JsFormatFileCb,
    sort_tailwind_classes_cb: JsSortTailwindClassesCb,
) -> Result<String, Vec<OxcError>> {
    // NOTE: Relative path resolution for external options depends on cwd.
    let cwd = env::current_dir().expect("Failed to get current working directory");
    let num_of_threads = 1;

    let Ok(source_type) = SourceType::from_path(filename) else {
        return Err(vec![OxcError::new(format!("Unsupported file type: {filename}"))]);
    };

    let options: Value = match serde_json::from_str(oxfmt_plugin_options_json) {
        Ok(Value::Object(mut obj)) => {
            // Embedded snippet output must not include final newline.
            obj.insert("insertFinalNewline".to_string(), false.into());
            Value::Object(obj)
        }
        Ok(_) => {
            return Err(vec![OxcError::new(
                "Failed to parse oxfmt plugin options JSON: expected JSON object".to_string(),
            )]);
        }
        Err(err) => {
            return Err(vec![OxcError::new(format!(
                "Failed to parse oxfmt plugin options JSON: {err}"
            ))]);
        }
    };

    let external_formatter = ExternalFormatter::new(
        init_external_formatter_cb,
        format_embedded_cb,
        format_file_cb,
        sort_tailwind_classes_cb,
    );

    // Use `block_in_place()` to avoid nested async runtime access
    match tokio::task::block_in_place(|| external_formatter.init(num_of_threads)) {
        // TODO: Plugins support
        Ok(_) => {}
        Err(err) => {
            external_formatter.cleanup();
            return Err(vec![OxcError::new(format!("Failed to setup external formatter: {err}"))]);
        }
    }

    let strategy = FormatFileStrategy::OxcFormatter { path: PathBuf::from(filename), source_type }
        .resolve_relative_path(&cwd);

    let resolved_options = match resolve_options_from_value(&cwd, options, &strategy) {
        Ok(options) => options,
        Err(err) => {
            external_formatter.cleanup();
            return Err(vec![OxcError::new(format!("Failed to parse configuration: {err}"))]);
        }
    };

    let formatter = SourceFormatter::new(num_of_threads)
        .with_external_formatter(Some(external_formatter.clone()));

    let mut code = match tokio::task::block_in_place(|| {
        formatter.format(&strategy, source_text, resolved_options)
    }) {
        FormatResult::Success { code, .. } => code,
        FormatResult::Error(diagnostics) => {
            external_formatter.cleanup();
            return Err(OxcError::from_diagnostics(filename, source_text, diagnostics));
        }
    };

    if parent_context.starts_with("markdown") || parent_context.starts_with("mdx") {
        // TODO: Handle this with `oxc_formatter` option
        normalize_single_jsx_semicolon_for_markdown(&mut code);
    }

    external_formatter.cleanup();
    Ok(code)
}

fn normalize_single_jsx_semicolon_for_markdown(code: &mut String) {
    if code.starts_with('<') && code.ends_with(">;") {
        code.pop();
    }
    if code.starts_with(";<") && code.ends_with('>') {
        code.remove(0);
    }
}
