// Silence erroneous warnings from Rust Analyser for `#[derive(Tsify)]`
#![allow(non_snake_case)]
#![allow(clippy::needless_pass_by_value)]

use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

use oxc::{allocator::Allocator, parser::Parser, span::SourceType};

#[derive(Debug, Default, Clone, Deserialize, Tsify)]
#[tsify(from_wasm_abi)]
pub struct ParserOptions {
    #[serde(rename = "sourceType")]
    #[tsify(optional, type = "\"script\" | \"module\"")]
    pub source_type: Option<String>,

    /// "module" and "jsx" will be inferred from `sourceFilename`.
    #[serde(rename = "sourceFilename")]
    #[tsify(optional)]
    pub source_filename: Option<String>,
}

#[derive(Default, Tsify)]
#[wasm_bindgen(getter_with_clone)]
pub struct ParseResult {
    #[wasm_bindgen(readonly, skip_typescript)]
    #[tsify(type = "Program")]
    pub program: JsValue,

    #[wasm_bindgen(readonly, skip_typescript)]
    #[tsify(type = "Diagnostic[]")]
    pub errors: Vec<JsValue>,
}

#[derive(Debug, Default, Serialize, Tsify)]
pub struct Diagnostic {
    pub start: usize,
    pub end: usize,
    pub severity: String,
    pub message: String,
}

/// # Errors
///
/// * wasm bindgen serialization failed
///
/// # Panics
///
/// * File extension is invalid
/// * Serde JSON serialization
#[wasm_bindgen(js_name = parseSync)]
pub fn parse_sync(
    source_text: String,
    options: Option<ParserOptions>,
) -> Result<ParseResult, serde_wasm_bindgen::Error> {
    let options = options.unwrap_or_default();

    let allocator = Allocator::default();

    let source_type = options
        .source_filename
        .as_ref()
        .map(|name| SourceType::from_path(name).unwrap())
        .unwrap_or_default();

    let source_type = match options.source_type.as_deref() {
        Some("script") => source_type.with_script(true),
        Some("module") => source_type.with_module(true),
        _ => source_type,
    };

    let ret = Parser::new(&allocator, &source_text, source_type).parse();

    let serializer = serde_wasm_bindgen::Serializer::json_compatible();

    let program = ret.program.serialize(&serializer)?;

    let errors = if ret.errors.is_empty() {
        vec![]
    } else {
        ret.errors
            .iter()
            .flat_map(|error| {
                let Some(labels) = error.labels() else { return vec![] };
                labels
                    .map(|label| {
                        Diagnostic {
                            start: label.offset(),
                            end: label.offset() + label.len(),
                            severity: format!("{:?}", error.severity().unwrap_or_default()),
                            message: format!("{error}"),
                        }
                        .serialize(&serializer)
                        .unwrap()
                    })
                    .collect::<Vec<JsValue>>()
            })
            .collect::<Vec<JsValue>>()
    };

    Ok(ParseResult { program, errors })
}
