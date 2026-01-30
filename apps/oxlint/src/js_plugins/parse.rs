use std::{
    mem::ManuallyDrop,
    ptr::{self, NonNull},
};

use napi::bindgen_prelude::Uint8Array;
use napi_derive::napi;

use oxc_allocator::Allocator;
use oxc_ast_visit::utf8_to_utf16::Utf8ToUtf16;
use oxc_linter::RawTransferMetadata2 as RawTransferMetadata;
use oxc_napi::get_source_type;
use oxc_parser::{ParseOptions, Parser};
use oxc_semantic::SemanticBuilder;

use crate::generated::raw_transfer_constants::{BLOCK_ALIGN as BUFFER_ALIGN, BUFFER_SIZE};

const BUMP_ALIGN: usize = 16;

/// Sentinel value for program offset to indicate parsing failed.
///
/// 0 cannot be a valid offset as it's the start of the buffer, which contains the source text.
/// Allocator bumps downwards, so if source text was empty, the program would be somewhere at end of the buffer.
const PARSE_FAIL_SENTINEL: u32 = 0;

// Parser options
#[napi(object)]
#[derive(Default)]
pub struct ParserOptions {
    /// Treat the source text as `js`, `jsx`, `ts`, `tsx` or `dts`.
    #[napi(ts_type = "'js' | 'jsx' | 'ts' | 'tsx' | 'dts'")]
    pub lang: Option<String>,

    /// Treat the source text as `script` or `module` code.
    #[napi(ts_type = "'script' | 'module' | 'commonjs' | 'unambiguous' | undefined")]
    pub source_type: Option<String>,

    /// Ignore non-fatal parsing errors
    pub ignore_non_fatal_errors: Option<bool>,
}

/// Get offset within a `Uint8Array` which is aligned on `BUFFER_ALIGN`.
///
/// Does not check that the offset is within bounds of `buffer`.
/// To ensure it always is, provide a `Uint8Array` of at least `BUFFER_SIZE + BUFFER_ALIGN` bytes.
#[napi]
#[allow(clippy::needless_pass_by_value, clippy::allow_attributes)]
pub fn get_buffer_offset(buffer: Uint8Array) -> u32 {
    let buffer = &*buffer;
    let offset = (BUFFER_ALIGN - (buffer.as_ptr() as usize % BUFFER_ALIGN)) % BUFFER_ALIGN;
    #[expect(clippy::cast_possible_truncation)]
    return offset as u32;
}

/// Parse AST into provided `Uint8Array` buffer, synchronously.
///
/// Source text must be written into somewhere towards end of the buffer.
/// - `source_start` is position of first byte of source text in buffer
/// - `source_len` is length of source text (in UTF-8 bytes)
///
/// This function will parse the source, and write the AST into the buffer, starting at the end (before the source text).
///
/// It also writes to the very end of the buffer the offset of `Program` within the buffer.
///
/// Caller can deserialize data from the buffer on JS side.
///
/// # SAFETY
///
/// Caller must ensure:
/// * Source text is written into the buffer.
/// * Start of source text is at `source_start` bytes from the start of the buffer.
/// * Source text's UTF-8 byte length is `source_len`.
/// * This section of bytes in the buffer comprises a valid UTF-8 string.
///
/// If source text is originally a JS string on JS side, and converted to a buffer with
/// `Buffer.from(str)` or `new TextEncoder().encode(str)`, this guarantees it's valid UTF-8.
///
/// # Panics
///
/// Panics if source text is too long, or AST takes more memory than is available in the buffer.
#[napi]
#[allow(clippy::needless_pass_by_value, clippy::allow_attributes)]
pub unsafe fn parse_raw_sync(
    filename: String,
    mut buffer: Uint8Array,
    source_start: u32,
    source_len: u32,
    options: Option<ParserOptions>,
) {
    // SAFETY: This function is called synchronously, so buffer cannot be mutated outside this function
    // during the time this `&mut [u8]` exists
    let buffer = unsafe { buffer.as_mut() };

    // SAFETY: `parse_raw_impl` has same safety requirements as this function
    unsafe { parse_raw_impl(&filename, buffer, source_start, source_len, options) };
}

/// Parse AST into buffer.
///
/// # SAFETY
///
/// Caller must ensure:
/// * Source text is written into the buffer.
/// * Start of source text is at `source_start` bytes from the start of the buffer.
/// * Source text's UTF-8 byte length is `source_len`.
/// * This section of bytes in the buffer comprises a valid UTF-8 string.
///
/// If source text is originally a JS string on JS side, and converted to a buffer with
/// `Buffer.from(str)` or `new TextEncoder().encode(str)`, this guarantees it's valid UTF-8.
#[allow(clippy::items_after_statements, clippy::allow_attributes)]
unsafe fn parse_raw_impl(
    filename: &str,
    buffer: &mut [u8],
    source_start: u32,
    source_len: u32,
    options: Option<ParserOptions>,
) {
    // Check buffer has expected size and alignment
    assert_eq!(buffer.len(), BUFFER_SIZE);
    let buffer_ptr = NonNull::from_mut(buffer).cast::<u8>();
    assert!(buffer_ptr.addr().get().is_multiple_of(BUFFER_ALIGN));

    // Get offsets and size of data region to be managed by arena allocator.
    // Leave space for source before it, and space for metadata after it.
    // Metadata actually only takes 5 bytes, but round everything up to multiple of 16,
    // as the arena allocator requires that alignment.
    const RAW_METADATA_SIZE: usize = size_of::<RawTransferMetadata>();
    const {
        assert!(RAW_METADATA_SIZE >= BUMP_ALIGN);
        assert!(RAW_METADATA_SIZE.is_multiple_of(BUMP_ALIGN));
    };
    const RAW_METADATA_OFFSET: usize = BUFFER_SIZE - RAW_METADATA_SIZE;
    const _: () = assert!(RAW_METADATA_OFFSET.is_multiple_of(BUMP_ALIGN));

    // Create `Allocator`.
    // Wrap in `ManuallyDrop` so the allocation doesn't get freed at end of function, or if panic.
    // SAFETY: `buffer_ptr` and `RAW_METADATA_OFFSET` outline a section of the memory in `buffer`.
    // `buffer_ptr` and `RAW_METADATA_OFFSET` are multiples of 16.
    // `RAW_METADATA_OFFSET` is greater than `Allocator::MIN_SIZE`.
    let allocator = unsafe { Allocator::from_raw_parts(buffer_ptr, RAW_METADATA_OFFSET) };
    let allocator = ManuallyDrop::new(allocator);

    // Set cursor to before start of source text. AST will be written into the buffer before the source text.
    // Round down the pointer, so it's aligned on `BUMP_ALIGN`.
    // SAFETY: Caller guarantees that source text starts at `source_start` bytes from start of buffer.
    unsafe {
        let cursor_pos = source_start as usize & !(BUMP_ALIGN - 1);
        let cursor_ptr = buffer_ptr.add(cursor_pos);
        allocator.set_cursor_ptr(cursor_ptr);
    }

    // Get source type
    let options = options.unwrap_or_default();
    let source_type =
        get_source_type(filename, options.lang.as_deref(), options.source_type.as_deref());
    let ignore_non_fatal_errors = options.ignore_non_fatal_errors.unwrap_or(false);

    // Parse source.
    // Enclose parsing logic in a scope to make 100% sure no references to within `Allocator` exist after this.
    let (program_offset, has_bom) = {
        // SAFETY: Caller guarantees source occupies this region of the buffer and is valid UTF-8
        let source_text = unsafe {
            let source_end = source_start as usize + source_len as usize;
            let source_bytes = buffer.get_unchecked(source_start as usize..source_end);
            str::from_utf8_unchecked(source_bytes)
        };

        // Parse with same options as linter
        let parser_ret = Parser::new(&allocator, source_text, source_type)
            .with_options(ParseOptions {
                parse_regular_expression: true,
                allow_return_outside_function: true,
                ..ParseOptions::default()
            })
            .parse();
        let program = allocator.alloc(parser_ret.program);

        let mut parsing_failed =
            parser_ret.panicked || (!parser_ret.errors.is_empty() && !ignore_non_fatal_errors);

        // Check for semantic errors.
        // If `ignore_non_fatal_errors` is `true`, skip running semantic, as any errors will be ignored anyway.
        if !parsing_failed && !ignore_non_fatal_errors {
            let semantic_ret = SemanticBuilder::new().with_check_syntax_error(true).build(program);
            parsing_failed = !semantic_ret.errors.is_empty();
        }

        if parsing_failed {
            // Use sentinel value for program offset to indicate that parsing failed
            (PARSE_FAIL_SENTINEL, false)
        } else {
            // If has BOM, remove it
            const BOM: &str = "\u{feff}";
            const BOM_LEN: usize = BOM.len();

            let mut source_text = program.source_text;
            let has_bom = source_text.starts_with(BOM);
            if has_bom {
                source_text = &source_text[BOM_LEN..];
                program.source_text = source_text;
            }

            // Convert spans to UTF-16.
            // If source starts with BOM, create converter which ignores the BOM.
            let span_converter = if has_bom {
                #[expect(clippy::cast_possible_truncation)]
                Utf8ToUtf16::new_with_offset(source_text, BOM_LEN as u32)
            } else {
                Utf8ToUtf16::new(source_text)
            };

            span_converter.convert_program(program);
            span_converter.convert_comments(&mut program.comments);

            // Return offset of `Program` within buffer (bottom 32 bits of pointer)
            let program_offset = ptr::from_ref(program) as u32;

            (program_offset, has_bom)
        }
    };

    // Write metadata into end of buffer
    #[allow(clippy::cast_possible_truncation)]
    let metadata = RawTransferMetadata::new(
        program_offset,
        source_type.is_typescript(),
        source_type.is_jsx(),
        has_bom,
    );
    // SAFETY: `RAW_METADATA_OFFSET` is less than length of `buffer`.
    // `RAW_METADATA_OFFSET` is aligned on 16.
    unsafe {
        buffer_ptr.add(RAW_METADATA_OFFSET).cast::<RawTransferMetadata>().write(metadata);
    }
}
