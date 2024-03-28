mod module_lexer;

use std::{
    alloc::{self, Layout},
    mem::ManuallyDrop,
    ptr::NonNull,
    str,
    sync::Arc,
};

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

// For raw transfer, use a buffer 4 GiB in size, with 4 GiB alignment.
// This ensures that all 64-bit pointers have the same value in upper 32 bits,
// so JS only needs to read the lower 32 bits to get an offset into the buffer.
// However, only use first half of buffer (2 GiB) for the arena, so 32-bit offsets
// don't have the highest bit set. JS bitwise operators interpret the highest bit as sign bit,
// so this enables using `>>` bitshift operator in JS, rather than the more expensive `>>>`,
// without offsets being interpreted as negative.
const TWO_GIB: usize = 1 << 31;
const FOUR_GIB: usize = 1 << 32;
const EIGHT_GIB: usize = 1 << 33;

const RAW_BUMP_SIZE: usize = TWO_GIB;
const RAW_BUMP_ALIGN: usize = FOUR_GIB;

/// Create a buffer for use with `parse_sync_raw`.
/// # Panics
/// Panics if cannot allocate buffer.
#[napi]
#[allow(unsafe_code)]
pub fn create_buffer() -> Uint8Array {
    // 32-bit systems are not supported
    const_assert!(std::mem::size_of::<usize>() >= 8);

    // Attempt to create allocation with required alignment.
    // On some systems (e.g. MacOS), the allocator only supports alignment of up to 2 GiB,
    // so trying to allocate with 4 GiB alignment may fail.
    // If it does, try again with 8 GiB size and low alignment, and can then use 4 GiB in the middle.
    let mut layout = Layout::from_size_align(FOUR_GIB, FOUR_GIB).unwrap();
    // SAFETY: Layout was created safely
    let mut alloc_ptr = unsafe { alloc::alloc(layout) };
    let data_ptr = if alloc_ptr.is_null() {
        layout = Layout::from_size_align(EIGHT_GIB, 16).unwrap();
        // SAFETY: Layout was created safely
        alloc_ptr = unsafe { alloc::alloc(layout) };
        assert!(!alloc_ptr.is_null(), "Failed to allocate buffer");

        let offset = RAW_BUMP_ALIGN - (alloc_ptr as usize % RAW_BUMP_ALIGN);
        // SAFETY: We allocated 8 GiB, and offset is 4 GiB max, so must be within bounds
        unsafe { alloc_ptr.add(offset) }
    } else {
        alloc_ptr
    };
    debug_assert!(data_ptr as usize % RAW_BUMP_ALIGN == 0);

    // Return as NAPI `Uint8Array`, borrowing the allocation's memory.
    // SAFETY: `data_ptr` is valid for reading `RAW_BUMP_SIZE` bytes.
    // TODO: Add comment pointing to Github discussion where NodeJS maintainer said
    // passing uninitialized data is fine
    unsafe {
        Uint8Array::with_external_data(data_ptr, RAW_BUMP_SIZE, move |_ptr, _len| {
            alloc::dealloc(alloc_ptr, layout);
        })
    }
}

/// Returns AST as raw bytes from Rust's memory.
///
/// Caller provides a buffer.
/// Source text must be written into the start of the buffer, and its length provided as `source_len`.
/// This function will parse the source, and write the AST into the buffer, starting at the end.
/// It also writes to the buffer after the source text:
/// * Offset of `Program` in the buffer.
/// * Mask for converting 64-bit pointers to buffer offsets.
///
/// # SAFETY
/// Caller must ensure:
/// * Source text is written into start of the buffer.
/// * Source text's byte length is `source_len`.
/// * Source text is valid UTF-8.
///
/// If source text is originally a JS string on JS side, and converted to a buffer with
/// `Buffer.from(str)` or `new TextEncoder().encode(str)`, this guarantees it's valid UTF-8.
///
/// # Panics
/// Panics if AST takes more memory than expected.
#[napi]
#[allow(
    unsafe_code,
    clippy::needless_pass_by_value,
    clippy::items_after_statements,
    clippy::missing_safety_doc,
    clippy::unnecessary_safety_comment
)]
pub unsafe fn parse_sync_raw(
    mut buff: Uint8Array,
    source_len: u32,
    options: Option<ParserOptions>,
) {
    // 32-bit systems are not supported
    const_assert!(std::mem::size_of::<usize>() >= 8);

    // Check buffer has expected size and alignment
    let buff = &mut *buff;
    let buff_ptr = (buff as *mut [u8]).cast::<u8>();
    assert_eq!(buff.len(), RAW_BUMP_SIZE);
    assert_eq!(buff_ptr as usize % RAW_BUMP_ALIGN, 0);

    // Get offsets and size of data region to be managed by arena allocator.
    // Only use first 2 GiB of buffer.
    // Leave space for source before it, and 16 bytes for metadata after it.
    const METADATA_SIZE: usize = 16;
    let data_offset = (source_len as usize).next_multiple_of(16);
    let data_size = RAW_BUMP_SIZE.saturating_sub(data_offset + METADATA_SIZE);
    assert!(data_size >= Allocator::MIN_SIZE);

    // Create `Allocator`.
    // Wrap in `ManuallyDrop` so the allocation doesn't get freed at end of function, or if panic.
    // SAFETY: `data_offset` is less than `buff.len()`
    let data_ptr = buff_ptr.add(data_offset);
    // SAFETY: `data_ptr` and `data_size` are multiples of 16.
    // `data_size` is greater than `Allocator::MIN_SIZE`.
    // `data_ptr + data_size` is not after end of `buff`.
    let allocator =
        ManuallyDrop::new(Allocator::from_raw_parts(NonNull::new_unchecked(data_ptr), data_size));

    // Parse source
    let options = options.unwrap_or_default();
    let program_ptr = {
        let source = &buff[..source_len as usize];
        // SAFETY: Caller guarantees source occupies this region of the buffer and is valid UTF-8
        let source_text = str::from_utf8_unchecked(source);
        let ret = parse(&allocator, source_text, &options);
        let program = allocator.alloc(ret.program);
        (program as *const Program).cast::<u8>()
    };

    // Write offset of program into end of buffer
    #[allow(clippy::cast_possible_truncation)]
    let program_offset = program_ptr as u32;
    const METADATA_OFFSET: usize = RAW_BUMP_SIZE - METADATA_SIZE;
    // SAFETY: `METADATA_OFFSET` is less than length of `buff`
    #[allow(clippy::cast_ptr_alignment)]
    buff_ptr.add(METADATA_OFFSET).cast::<u32>().write(program_offset);
}

/// # Panics
///
/// * Tokio crashes
#[allow(clippy::needless_pass_by_value)]
#[napi]
pub async fn parse_async(source_text: String, options: Option<ParserOptions>) -> ParseResult {
    tokio::spawn(async move { parse_sync(source_text, options) }).await.unwrap()
}
