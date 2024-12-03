#![allow(clippy::needless_pass_by_value)]

mod module_lexer;

use std::sync::Arc;

use napi::{bindgen_prelude::AsyncTask, Task};
use napi_derive::napi;

use self_cell::self_cell;

use oxc::{
    allocator::Allocator,
    ast::CommentKind,
    diagnostics::{Error, NamedSource},
    napi::parse::{Comment, ParseResult, ParserOptions},
    parser::{ParseOptions, Parser, ParserReturn},
    span::SourceType,
};

pub use crate::module_lexer::*;
use string_wizard::MagicString;

/// A Parser instance that holds a MagicString.
#[napi]
pub struct ParserBuilder {
    cell: ParserBuilderImpl,
}

self_cell!(
    struct ParserBuilderImpl {
        owner: String,

        #[covariant]
        dependent: MagicString,
    }
);

#[napi]
impl ParserBuilder {
    #[napi(constructor)]
    pub fn new(source_text: String) -> Self {
        Self { cell: ParserBuilderImpl::new(source_text, |s| MagicString::new(s)) }
    }

    /// # Panics
    ///
    /// * File extension is invalid
    /// * Serde JSON serialization
    #[napi]
    pub fn parse_sync(&mut self, options: Option<ParserOptions>) -> ParseResult {
        let options = options.unwrap_or_default();
        // TODO: update magic string filename here.
        parse_with_return(self.cell.borrow_owner(), &options)
    }

    #[napi]
    pub fn source_text(&self, start: u32, end: u32) -> &str {
        &self.cell.borrow_owner()[start as usize..end as usize]
    }

    #[napi]
    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        self.cell.borrow_dependent().to_string()
    }

    #[napi]
    #[allow(clippy::len_without_is_empty, clippy::cast_possible_truncation)]
    pub fn len(&self) -> u32 {
        self.cell.borrow_dependent().len() as u32
    }

    // #[napi]
    // pub fn generate_map(&self) -> oxc::napi::source_map::SourceMap {
    // let json = self.cell.borrow_dependent().source_map(SourceMapOptions::default()).to_json();
    // oxc::napi::source_map::SourceMap {
    // file: json.file,
    // mappings: json.mappings,
    // names: json.names,
    // source_root: json.source_root,
    // sources: json.sources,
    // sources_content: json.sources_content.map(|content| {
    // content.into_iter().map(Option::unwrap_or_default).collect::<Vec<_>>()
    // }),
    // version: 3,
    // x_google_ignorelist: None,
    // }
    // }

    #[napi]
    pub fn append(&mut self, source: String) {
        self.cell.with_dependent_mut(|_, s| {
            s.append(source);
        });
    }

    #[napi]
    pub fn append_left(&mut self, text_index: u32, content: String) {
        self.cell.with_dependent_mut(|_, s| {
            s.append_left(text_index as usize, content);
        });
    }

    #[napi]
    pub fn append_right(&mut self, text_index: u32, content: String) {
        self.cell.with_dependent_mut(|_, s| {
            s.append_left(text_index as usize, content);
        });
    }

    #[napi]
    pub fn indent(&mut self) {
        self.cell.with_dependent_mut(|_, s| {
            s.indent();
        });
    }

    // #[napi]
    // pub fn indent_with(&mut self, opts: string_wizard::IndentOptions<'_, '_>) {
    // self.cell.with_dependent_mut(|_, s| {
    // s.indent_with(&content, opts);
    // });
    // }

    #[napi]
    pub fn prepend(&mut self, source: String) {
        self.cell.with_dependent_mut(|_, s| {
            s.prepend(source);
        });
    }

    #[napi]
    pub fn prepend_left(&mut self, text_index: u32, content: String) {
        self.cell.with_dependent_mut(|_, s| {
            s.prepend_left(text_index as usize, content);
        });
    }

    #[napi]
    pub fn relocate(&mut self, start: u32, end: u32, to: u32) {
        self.cell.with_dependent_mut(|_, s| {
            s.relocate(start as usize, end as usize, to as usize);
        });
    }

    #[napi]
    pub fn remove(&mut self, start: u32, end: u32) {
        self.cell.with_dependent_mut(|_, s| {
            s.remove(start as usize, end as usize);
        });
    }

    #[napi]
    pub fn update(&mut self, start: u32, end: u32, content: String) {
        self.cell.with_dependent_mut(|_, s| {
            s.update(start as usize, end as usize, content);
        });
    }

    // #[napi]
    // pub fn update_with(&mut self, start: u32, end: u32, content: String, opts: UpdateOptions) {
    // self.cell.with_dependent_mut(|_, s| {
    // s.update_with(start as usize, end as usize, content, opts);
    // });
    // }
}

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
            value: comment.content_span().source_text(source_text).to_string(),
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
/// * Internal runtime (tokio) crashes
#[napi]
pub fn parse_async(source_text: String, options: Option<ParserOptions>) -> AsyncTask<ResolveTask> {
    let options = options.unwrap_or_default();
    AsyncTask::new(ResolveTask { source_text, options })
}
