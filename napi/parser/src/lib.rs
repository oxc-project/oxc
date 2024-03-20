#![allow(clippy::trailing_empty_array)]

mod module_lexer;

use std::{alloc::Layout, str, sync::Arc};

use bumpalo::Bump;
use flexbuffers::FlexbufferSerializer;
use napi::bindgen_prelude::{Buffer, Uint8Array};
use napi_derive::napi;
use serde::Serialize;

use oxc_allocator::Allocator;
pub use oxc_ast::ast::Program;
use oxc_ast::CommentKind;
use oxc_diagnostics::miette::NamedSource;
use oxc_parser::{Parser, ParserReturn};
use oxc_span::SourceType;

pub use crate::module_lexer::*;

/// Babel Parser Options
///
/// <https://github.com/babel/babel/blob/main/packages/babel-parser/typings/babel-parser.d.ts>
#[napi(object)]
#[derive(Default)]
pub struct ParserOptions {
    #[napi(ts_type = "'script' | 'module' | 'unambiguous' | undefined")]
    pub source_type: Option<String>,
    pub source_filename: Option<String>,
    /// Emit `ParenthesizedExpression` in AST.
    ///
    /// If this option is true, parenthesized expressions are represented by
    /// (non-standard) `ParenthesizedExpression` nodes that have a single `expression` property
    /// containing the expression inside parentheses.
    ///
    /// Default: true
    pub preserve_parens: Option<bool>,
}

#[napi(object)]
pub struct ParseResult {
    pub program: String,
    pub comments: Vec<Comment>,
    pub errors: Vec<String>,
}

#[napi(object)]
pub struct Comment {
    pub r#type: &'static str,
    #[napi(ts_type = "'Line' | 'Block'")]
    pub value: String,
    pub start: u32,
    pub end: u32,
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
    Parser::new(allocator, source_text, source_type)
        .preserve_parens(options.preserve_parens.unwrap_or(true))
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

    let comments = ret
        .trivias
        .comments()
        .map(|(kind, span)| Comment {
            r#type: match kind {
                CommentKind::SingleLine => "Line",
                CommentKind::MultiLine => "Block",
            },
            value: span.source_text(&source_text).to_string(),
            start: span.start,
            end: span.end,
        })
        .collect::<Vec<Comment>>();

    ParseResult { program, comments, errors }
}

/// Returns a binary AST in flexbuffers format.
/// This is a POC API. Error handling is not done yet.
///
/// # Panics
///
/// * File extension is invalid
/// * FlexbufferSerializer serialization error
#[allow(clippy::needless_pass_by_value)]
#[napi]
pub fn parse_sync_buffer(source_text: String, options: Option<ParserOptions>) -> Buffer {
    let options = options.unwrap_or_default();
    let allocator = Allocator::default();
    let ret = parse(&allocator, &source_text, &options);
    let mut serializer = FlexbufferSerializer::new();
    ret.program.serialize(&mut serializer).unwrap();
    serializer.take_buffer().into()
}

/// Returns schema for AST types
///
/// # Panics
/// Panics if type definitions cannot be converted to JSON.
#[napi]
pub fn get_schema() -> String {
    let types = layout_inspect::inspect::<Program>();
    serde_json::to_string(&types).unwrap()
}

/// Returns AST as raw bytes from Rust's memory.
/// # Panics
/// Panics if AST takes more memory than expected.
#[napi]
#[allow(clippy::needless_pass_by_value)]
pub fn parse_sync_raw(
    source: Uint8Array,
    options: Option<ParserOptions>,
    bump_size: u32,
) -> Uint8Array {
    // Create allocator with enough capacity for entire AST to be in 1 chunk.
    // Keep requesting allocations until get one where allocation won't straddle 32 bit boundary.
    let mut rejected_bumps = vec![];
    let bump = loop {
        let bump = Bump::with_capacity(bump_size as usize);
        // SAFETY: No allocations from this arena are performed while the iterator is alive.
        // Arena is empty, so no mutable references to data in the arena exist.
        let (ptr, ..) = unsafe { bump.iter_allocated_chunks_raw().next().unwrap() };
        #[allow(clippy::cast_possible_truncation)]
        if ptr as u32 > bump_size {
            break bump;
        }
        // This bump is unsuitable. Store it, so don't get given same allocation again on next attempt.
        // println!("Unsuitable bump {:?}", ptr);
        rejected_bumps.push(bump);
    };
    drop(rejected_bumps);

    let allocator: Allocator = bump.into();

    // Parse + allocate space for metadata in chunk
    let source_text = str::from_utf8(&source).unwrap();
    let options = options.unwrap_or_default();
    let (program_addr, metadata_ptr) = {
        let ret = parse(&allocator, source_text, &options);
        let program = allocator.alloc(ret.program);
        let program_addr = program as *mut _ as usize;
        let metadata_ptr = allocator.alloc_layout(Layout::new::<[usize; 4]>()).as_ptr();
        (program_addr, metadata_ptr)
    };

    // Get pointer to Bump's memory, and check there's only 1 chunk
    let bump = allocator.into_bump();
    let (chunk_ptr, chunk_len) = {
        // SAFETY: No allocations from this arena are performed while the returned iterator is alive.
        // No mutable references to previously allocated data exist.
        let mut chunks_iter = unsafe { bump.iter_allocated_chunks_raw() };
        let (chunk_ptr, chunk_len) = chunks_iter.next().unwrap();
        assert!(chunks_iter.next().is_none());
        (chunk_ptr, chunk_len)
    };
    assert!(chunk_ptr == metadata_ptr);
    let chunk_addr = chunk_ptr as usize;
    let chunk_end = chunk_addr + chunk_len;

    // Write at start of bump:
    // * Offset of program
    // * Memory address of start of bump
    // * Memory address of end of bump
    // * Memory address of start of source
    let program_offset = program_addr - chunk_addr;
    #[allow(clippy::borrow_as_ptr, clippy::ptr_as_ptr)]
    let source_addr = &*source as *const _ as *const u8 as usize;
    // SAFETY: `chunk_ptr` is valid for writes, and we allocated space for `[usize; 4] at that address.
    // LACK OF SAFETY: This may be unsound due to breaking aliasing rules. Or maybe not.
    // TODO: Ensure this is sound.
    unsafe {
        #[allow(clippy::ptr_as_ptr, clippy::cast_ptr_alignment)]
        (chunk_ptr as *mut [usize; 4]).write([program_offset, chunk_addr, chunk_end, source_addr]);
    };

    // Convert to NAPI `Uint8Array`.
    // SAFETY: `chunk_ptr` is valid for reading `len` bytes.
    // LACK OF SAFETY: This block of memory contains uninitialized bytes. I *think* that's OK.
    // TODO: Ensure this is sound.
    unsafe { Uint8Array::with_external_data(chunk_ptr, chunk_len, move |_ptr, _len| drop(bump)) }
}

/// # Panics
///
/// * Tokio crashes
#[allow(clippy::needless_pass_by_value)]
#[napi]
pub async fn parse_async(source_text: String, options: Option<ParserOptions>) -> ParseResult {
    tokio::spawn(async move { parse_sync(source_text, options) }).await.unwrap()
}
