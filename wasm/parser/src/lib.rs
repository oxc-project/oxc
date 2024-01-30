#![allow(clippy::needless_pass_by_value)]

use serde::Serialize;
use tsify::Tsify;
use wasm_bindgen::prelude::*;

use oxc::{allocator::Allocator, parser::Parser, span::SourceType};

#[wasm_bindgen(start)]
pub fn main() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// Babel Parser Options
///
/// <https://github.com/babel/babel/blob/main/packages/babel-parser/typings/babel-parser.d.ts>
#[wasm_bindgen(getter_with_clone)]
#[derive(Default, Tsify)]
pub struct ParserOptions {
    #[wasm_bindgen(js_name = sourceType)]
    pub source_type: Option<String>,
    #[wasm_bindgen]
    pub filename: Option<String>,
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Default, Tsify)]
pub struct ParseResult {
    #[wasm_bindgen(readonly, skip_typescript)]
    #[tsify(type = "Program")]
    pub program: JsValue,
    #[wasm_bindgen(readonly, skip_typescript)]
    #[tsify(type = "OxcDiagnostic[]")]
    pub errors: Vec<JsValue>,
}

#[derive(Default, Tsify, Serialize)]
pub struct OxcDiagnostic {
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
        .filename
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
                        OxcDiagnostic {
                            start: label.offset(),
                            end: label.offset() + label.len(),
                            severity: format!("{:?}", error.severity().unwrap_or_default()),
                            message: format!("{error}"),
                        }
                        .serialize(&serializer)
                        .unwrap()
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
    };

    Ok(ParseResult { program, errors })
}
