#![allow(clippy::trailing_empty_array)]

use std::sync::Arc;

use miette::NamedSource;
use napi_derive::napi;
use oxc_allocator::Allocator;
pub use oxc_ast::ast::Program;
use oxc_parser::{Parser, ParserReturn};
use oxc_span::SourceType;

/// Babel Parser Options
///
/// <https://github.com/babel/babel/blob/main/packages/babel-parser/typings/babel-parser.d.ts>
#[napi(object)]
#[derive(Default)]
pub struct ParserOptions {
    #[napi(ts_type = "'script' | 'module' | 'unambiguous' | undefined")]
    pub source_type: Option<String>,
    pub source_filename: Option<String>,
}

#[napi(object)]
pub struct ParseResult {
    pub program: String,
    pub errors: Vec<String>,
}

fn parse<'a>(
    allocator: &'a Allocator,
    source_text: &'a str,
    options: &ParserOptions,
) -> ParserReturn<'a> {
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
    Parser::new(allocator, source_text, source_type).parse()
}

/// Parse without returning anything.
/// This is for benchmark purposes such as measuring napi communication overhead.
///
/// # Panics
///
/// * File extension is invalid
/// * Serde JSON serialization
#[allow(clippy::needless_pass_by_value)]
#[napi]
pub fn parse_without_return(source_text: String, options: Option<ParserOptions>) {
    let options = options.unwrap_or_default();
    let allocator = Allocator::default();
    parse(&allocator, &source_text, &options);
}

/// # Panics
///
/// * File extension is invalid
/// * Serde JSON serialization
#[allow(clippy::needless_pass_by_value)]
#[napi]
pub fn parse_sync(source_text: String, options: Option<ParserOptions>) -> ParseResult {
    let options = options.unwrap_or_default();

    let allocator = Allocator::default();
    let ret = parse(&allocator, &source_text, &options);
    let program = serde_json::to_string(&ret.program).unwrap();

    let errors = if ret.errors.is_empty() {
        vec![]
    } else {
        let file_name = options.source_filename.unwrap_or_default();
        let source = Arc::new(NamedSource::new(file_name, source_text.to_string()));
        ret.errors
            .into_iter()
            .map(|diagnostic| diagnostic.with_source_code(Arc::clone(&source)))
            .map(|error| format!("{error:?}"))
            .collect()
    };

    ParseResult { program, errors }
}

/// # Panics
///
/// * Tokio crashes
#[allow(clippy::needless_pass_by_value)]
#[napi]
pub async fn parse_async(source_text: String, options: Option<ParserOptions>) -> ParseResult {
    tokio::spawn(async move { parse_sync(source_text, options) }).await.unwrap()
}
