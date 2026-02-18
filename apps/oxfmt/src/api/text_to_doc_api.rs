use std::{env, path::PathBuf};

use serde_json::Value;

use oxc_formatter::TextToDocMode;
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

    let text_to_doc_mode = parse_text_to_doc_mode(parent_context);

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

    let code = match tokio::task::block_in_place(|| {
        formatter.format(&strategy, source_text, resolved_options)
    }) {
        FormatResult::Success { code, .. } => code,
        FormatResult::Error(diagnostics) => {
            external_formatter.cleanup();
            return Err(OxcError::from_diagnostics(filename, source_text, diagnostics));
        }
    };

    external_formatter.cleanup();

    // Extract the relevant portion for Vue embedded contexts, then build Doc JSON.
    let extracted = extract_for_mode(&code, text_to_doc_mode, source_text);
    Ok(build_doc_json(&extracted))
}

/// Parse the `parent_context` string to determine the `TextToDocMode`.
fn parse_text_to_doc_mode(parent_context: &str) -> TextToDocMode {
    match parent_context {
        "vue:VueForBindingLeft" => TextToDocMode::VueForBindingLeft,
        "vue:VueBindings" => TextToDocMode::VueBindings,
        "vue:VueGeneric" => TextToDocMode::VueGeneric,
        _ => TextToDocMode::Full,
    }
}

/// Extract the relevant portion from formatted code based on the Vue mode.
///
/// For `VueForBindingLeft` / `VueBindings`:
///   Input: formatted `function _(PARAMS) {}`
///   - Extract PARAMS from between `function _(` and `) {}`
///   - For `VueForBindingLeft` with multiple params: wrap in `(...)` to match Prettier's `printHtmlBinding`
///
/// For `VueGeneric`:
///   Input: formatted `type T<PARAMS> = any`
///   - Extract PARAMS from between `<` and `>`
fn extract_for_mode(formatted_code: &str, mode: TextToDocMode, source_text: &str) -> String {
    match mode {
        TextToDocMode::Full => formatted_code.to_string(),
        TextToDocMode::VueForBindingLeft => {
            let params = extract_function_params(formatted_code);
            // Count the number of params in the original source to determine bracket wrapping.
            // The original source is `function _(PARAMS) {}` where PARAMS was extracted from the v-for.
            let needs_parens = count_vue_for_params(source_text) > 1;
            if needs_parens { format!("({params})") } else { params }
        }
        TextToDocMode::VueBindings => extract_function_params(formatted_code),
        TextToDocMode::VueGeneric => extract_type_params(formatted_code),
    }
}

/// Extract function parameters from formatted `function _(PARAMS) {}`.
/// Returns the content between `(` and the last `)` before `{}`.
fn extract_function_params(code: &str) -> String {
    // Find first `(` after `function _`
    let Some(open_paren) = code.find('(') else {
        return code.to_string();
    };

    // Find the matching `)` - search backwards from `) {}`
    let Some(close_paren) = code.rfind(") {}") else {
        return code.to_string();
    };

    if open_paren >= close_paren {
        return code.to_string();
    }

    code[open_paren + 1..close_paren].to_string()
}

/// Extract type parameters from formatted `type T<PARAMS> = any;`.
/// Returns the content between `<` and `>`.
fn extract_type_params(code: &str) -> String {
    let Some(open_angle) = code.find('<') else {
        return code.to_string();
    };

    // Find the matching `>` - search for `> = any`
    let close_angle = match code.rfind("> = any") {
        Some(idx) => idx,
        None => {
            // Fallback: find last `>`
            match code.rfind('>') {
                Some(idx) if idx > open_angle => idx,
                _ => return code.to_string(),
            }
        }
    };

    if open_angle >= close_angle {
        return code.to_string();
    }

    code[open_angle + 1..close_angle].to_string()
}

/// Count the number of top-level parameters in the Vue v-for source text.
/// Source text is `function _(PARAMS) {}`.
fn count_vue_for_params(source_text: &str) -> usize {
    let Some(open_paren) = source_text.find('(') else {
        return 1;
    };
    let Some(close_paren) = source_text.rfind(") {}") else {
        return 1;
    };
    if open_paren >= close_paren {
        return 1;
    }

    let params = &source_text[open_paren + 1..close_paren];

    // Count top-level commas (not inside nested brackets)
    let mut depth = 0i32;
    let mut count = 1usize;
    for ch in params.chars() {
        match ch {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth -= 1,
            ',' if depth == 0 => count += 1,
            _ => {}
        }
    }
    count
}

/// Build a Prettier Doc JSON string from formatted code.
///
/// Single line: `"formatted text"`
/// Multi-line: `["line1", {"type":"line","hard":true}, {"type":"break-parent"}, "line2"]`
fn build_doc_json(code: &str) -> String {
    let lines: Vec<&str> = code.split('\n').collect();

    if lines.len() <= 1 {
        // Single line (or empty): just a JSON string
        serde_json::to_string(code).unwrap_or_default()
    } else {
        // Multiple lines: build array with hardline separators
        let mut parts: Vec<Value> = Vec::new();
        for (i, line) in lines.iter().enumerate() {
            if i > 0 {
                // hardline = [{ type: "line", hard: true }, { type: "break-parent" }]
                parts.push(serde_json::json!({"type": "line", "hard": true}));
                parts.push(serde_json::json!({"type": "break-parent"}));
            }
            parts.push(Value::String((*line).to_string()));
        }
        serde_json::to_string(&parts).unwrap_or_default()
    }
}
