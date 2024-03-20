mod module_lexer;

use std::{alloc::Layout, str, sync::Arc};

use bumpalo::Bump;
use napi::bindgen_prelude::Uint8Array;
use napi_derive::napi;
use static_assertions::const_assert;

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
#[allow(unsafe_code, clippy::needless_pass_by_value, clippy::items_after_statements)]
pub fn parse_sync_raw(
    source: Uint8Array,
    options: Option<ParserOptions>,
    bump_size: u32,
) -> Uint8Array {
    // 32-bit systems are not supported
    const_assert!(std::mem::size_of::<usize>() >= 8);

    // Limit AST size to 2 GiB.
    // This allows using faster `>>` bitshift operations in JS, instead of `>>>`.
    const MAX_BUMP_SIZE: u32 = 1u32 << 31;
    assert!(bump_size <= MAX_BUMP_SIZE, "AST cannot be larger than 2 GiB ({MAX_BUMP_SIZE} bytes)");

    // Round up bump size to a power of 2
    let bump_size = (bump_size as usize).next_power_of_two();

    // Create allocator with enough capacity for entire AST to be in 1 chunk.
    // We want to send a buffer to JS which is `bump_size` long, and aligned on `bump_size`.
    // Then can convert a 64-bit pointer to offset in that buffer with `ptr & (bump_size - 1)`.
    // Bumpalo doesn't allow requesting an allocation with a specific alignment,
    // so allocate twice as much as required, so that can use just the chunk in the middle
    // which is aligned to `bump_size`.
    let bump = Bump::with_capacity(bump_size * 2);

    // Prevent Bumpalo creating any further chunks.
    // Attempts to use more memory than requested will result in an OOM error.
    bump.set_allocation_limit(Some(0));

    // Get pointer to end of allocation.
    // SAFETY: No allocations from this arena are performed while the iterator is alive.
    // Arena is empty, so no mutable references to data in the arena exist.
    let (mut end_ptr, ..) = unsafe { bump.iter_allocated_chunks_raw().next().unwrap() };

    // Consume space at end of the bump, so end of the chunk we'll send to JS is aligned on `bump_size`.
    // As that chunk will be `bump_size` in length, start of the chunk will also be aligned on `bump_size`.
    let padding = end_ptr as usize & (bump_size - 1);
    if padding != 0 {
        // SAFETY: `align` is 1 which is non-zero and a power of 2.
        // `size` must be less than `isize::MAX` as it's less than `bump_size` which is `u32`
        // and we've checked above this is a 64-bit system.
        let padding_layout = unsafe { Layout::from_size_align_unchecked(padding, 1) };
        end_ptr = bump.alloc_layout(padding_layout).as_ptr();
    }
    debug_assert_eq!(end_ptr as usize & (bump_size - 1), 0);
    // SAFETY: We allocated `bump_size * 2` bytes.
    // Bumpalo's pointer is initially at end of the allocation, so this must be within the allocation.
    let start_ptr = unsafe { end_ptr.sub(bump_size) };

    // Copy source into arena + parse.
    // Reason for copying source into arena is to simplify calculations on JS side.
    // All string data (whether slices of source, or escaped strings) are in the same buffer.
    let allocator: Allocator = bump.into();
    let options = options.unwrap_or_default();
    let (program_ptr, source_ptr) = {
        let source_text = simdutf8::basic::from_utf8(&source).unwrap();
        let source_text = allocator.alloc_str(source_text);
        let ret = parse(&allocator, source_text, &options);
        let program = allocator.alloc(ret.program);
        ((program as *const Program).cast::<u8>(), (source_text as *const str).cast::<u8>())
    };
    let bump = allocator.into_bump();

    // Consume space between program and where metadata will go.
    // Once metadata is written before the padding, metadata will be at `start_ptr`
    // (i.e. start of buffer being passed to JS)
    type Metadata = [u32; 3];
    const METADATA_SIZE: usize = std::mem::size_of::<Metadata>();

    #[allow(clippy::cast_possible_wrap)]
    // SAFETY: `program_ptr` and `start_ptr` are part of same allocation
    let padding = unsafe { program_ptr.offset_from(start_ptr) } - METADATA_SIZE as isize;
    assert!(padding >= 0, "AST is larger than requested size {bump_size}");
    if padding > 0 {
        // SAFETY: `align` is 1 which is non-zero and a power of 2.
        // `size` must be less than `isize::MAX` as it's less than `bump_size` and check above
        // ensures `bump_size` cannot exceed `isize::MAX`.
        #[allow(clippy::cast_sign_loss)]
        let padding_layout = unsafe { Layout::from_size_align_unchecked(padding as usize, 1) };
        bump.alloc_layout(padding_layout);
    }

    // Write metadata
    #[allow(clippy::cast_possible_truncation)]
    let ptr_mask = (bump_size - 1) as u32;
    let program_offset = program_ptr as u32 & ptr_mask;
    let source_offset = source_ptr as u32 & ptr_mask;
    let metadata: Metadata = [program_offset, source_offset, ptr_mask];
    let metadata = bump.alloc(metadata);
    debug_assert!((metadata as *const Metadata).cast::<u8>() == start_ptr);

    // Convert slice of allocation between `start_ptr` and `end_ptr` (length `bump_size`)
    // to NAPI `Uint8Array`. This buffer is aligned on `bump_size`, and with the metadata at the start.
    // SAFETY: `start_ptr` is valid for reading `bump_size` bytes.
    // TODO: Add comment pointing to Github discussion where NodeJS maintainer said
    // passing uninitialized data is fine
    unsafe { Uint8Array::with_external_data(start_ptr, bump_size, move |_ptr, _len| drop(bump)) }
}

/// # Panics
///
/// * Tokio crashes
#[allow(clippy::needless_pass_by_value)]
#[napi]
pub async fn parse_async(source_text: String, options: Option<ParserOptions>) -> ParseResult {
    tokio::spawn(async move { parse_sync(source_text, options) }).await.unwrap()
}
