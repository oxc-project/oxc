mod module_lexer;

use std::sync::Arc;

use napi::{bindgen_prelude::AsyncTask, Task};
use napi_derive::napi;

use oxc::{
    allocator::Allocator,
    ast::CommentKind,
    diagnostics::{Error, NamedSource},
    napi::parse::{Comment, ParseResult, ParserOptions},
    parser::{ParseOptions, Parser, ParserReturn},
    span::SourceType,
};

pub use crate::module_lexer::*;

fn parse<'a>(
    allocator: &'a Allocator,
    source_text: &'a str,
    options: &ParserOptions,
) -> ParserReturn<'a> {
    let source_type = options
        .source_filename
        .as_ref()
        .and_then(|name| SourceType::from_path(name).ok())
        .unwrap_or_default();
    let source_type = match options.source_type.as_deref() {
        Some("script") => source_type.with_script(true),
        Some("module") => source_type.with_module(true),
        _ => source_type,
    };
    Parser::new(allocator, source_text, source_type)
        .with_options(ParseOptions {
            preserve_parens: options.preserve_parens.unwrap_or(true),
            ..ParseOptions::default()
        })
        .parse()
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

#[allow(clippy::needless_lifetimes)]
fn parse_with_return<'a>(source_text: &'a str, options: &ParserOptions) -> ParseResult {
    let allocator = Allocator::default();
    let ret = parse(&allocator, source_text, options);
    let program = serde_json::to_string(&ret.program).unwrap();

    let errors = if ret.errors.is_empty() {
        vec![]
    } else {
        let file_name = options.source_filename.clone().unwrap_or_default();
        let source = Arc::new(NamedSource::new(file_name, source_text.to_string()));
        ret.errors
            .into_iter()
            .map(|diagnostic| Error::from(diagnostic).with_source_code(Arc::clone(&source)))
            .map(|error| format!("{error:?}"))
            .collect()
    };

    let comments = ret
        .program
        .comments
        .iter()
        .map(|comment| Comment {
            r#type: match comment.kind {
                CommentKind::Line => "Line",
                CommentKind::Block => "Block",
            },
            value: comment.span.source_text(source_text).to_string(),
            start: comment.span.start,
            end: comment.span.end,
        })
        .collect::<Vec<Comment>>();

    ParseResult { program, comments, errors }
}

/// # Panics
///
/// * File extension is invalid
/// * Serde JSON serialization
#[allow(clippy::needless_pass_by_value)]
#[napi]
pub fn parse_sync(source_text: String, options: Option<ParserOptions>) -> ParseResult {
    let options = options.unwrap_or_default();
    parse_with_return(&source_text, &options)
}

pub struct ResolveTask {
    source_text: String,
    options: ParserOptions,
}

#[napi]
impl Task for ResolveTask {
    type JsValue = ParseResult;
    type Output = ParseResult;

    fn compute(&mut self) -> napi::Result<Self::Output> {
        Ok(parse_with_return(&self.source_text, &self.options))
    }

    fn resolve(&mut self, _: napi::Env, result: Self::Output) -> napi::Result<Self::JsValue> {
        Ok(result)
    }
}

/// # Panics
///
/// * Tokio crashes
#[allow(clippy::needless_pass_by_value)]
#[napi]
pub fn parse_async(source_text: String, options: Option<ParserOptions>) -> AsyncTask<ResolveTask> {
    let options = options.unwrap_or_default();
    AsyncTask::new(ResolveTask { source_text, options })
}
