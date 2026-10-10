use std::{
    alloc::Layout,
    mem::{self, ManuallyDrop},
    ptr::{self, NonNull},
    str,
};

use napi::{
    Task,
    bindgen_prelude::{AsyncTask, Uint8Array},
};
use napi_derive::napi;

use oxc::{
    allocator::{Allocator, ArenaVec, FromIn},
    ast_visit::utf8_to_utf16::Utf8ToUtf16,
    semantic::SemanticBuilder,
};
#[cfg(feature = "tokens")]
use oxc_estree_tokens::{ESTreeTokenOptions, update_tokens};
use oxc_napi::get_source_type;

use crate::{
    AstType, ParserOptions, get_ast_type, parse_impl,
    raw_transfer_constants::{ACTIVE_SIZE, BLOCK_ALIGN, BLOCK_SIZE, CURSOR_MIN_ALIGN},
    raw_transfer_types::{EcmaScriptModule, Error, RawTransferData, RawTransferMetadata},
};

// For raw transfer, use a buffer 2 GiB in size. The buffer can be at any address
// (it only needs to be aligned on `Allocator::RAW_MIN_ALIGN`).
//
// JS reads only the lower 32 bits of each 64-bit pointer, and converts it to an offset within the buffer
// by subtracting the lower 32 bits of the buffer's start address, wrapping to 32 bits:
// `offset = (lo32(ptr) - lo32(buffer_start)) | 0`.
// `lo32(x) == x mod 2^32`, so this equals `(ptr - buffer_start) mod 2^32`, which is the real offset,
// because the offset is always less than buffer size (< 2^31). This holds even if the buffer
// crosses a 4 GiB boundary.
//
// Metadata at end of buffer (`RawTransferMetadata`) stores offsets which are already relative
// to start of buffer, so JS doesn't need to convert them.
//
// Buffer size only 2 GiB so 32-bit offsets don't have the highest bit set.
// This is advantageous for 2 reasons:
//
// 1. V8 stores small integers ("SMI"s) inline, rather than on heap, which is more performant.
//    But 31 bits is the max positive integer considered an SMI.
//
// 2. JS bitwise operators work only on signed 32-bit integers, with 32nd bit as sign bit.
//    So avoiding the 32nd bit being set enables using `>>` bitshift operator,
//    which is cheaper than `>>>`, and does not risk offsets being interpreted as negative.
//
// Previously the buffer was aligned on 4 GiB (`BLOCK_ALIGN`), so lower 32 bits of every pointer
// *was* the offset, with no subtraction. That required JS to allocate a 6 GiB `ArrayBuffer`
// to find an aligned 2 GiB region within it. Buffers aligned on 4 GiB are still accepted
// (subtracting a base of 0 is a no-op).

/// Layout describing the JS-owned buffer (`BLOCK_SIZE` bytes, aligned on `Allocator::RAW_MIN_ALIGN`).
const BLOCK_LAYOUT: Layout = match Layout::from_size_align(BLOCK_SIZE, Allocator::RAW_MIN_ALIGN) {
    Ok(layout) => layout,
    Err(_) => unreachable!(),
};

/// Get lower 32 bits of address of start of a `Uint8Array`, as a signed 32-bit integer.
///
/// JS side uses this to convert pointers in the buffer to offsets within the buffer.
/// Also, the low bits are used to align the buffer on `Allocator::RAW_MIN_ALIGN`.
#[napi(skip_typescript)]
#[allow(clippy::needless_pass_by_value, clippy::allow_attributes)]
pub fn get_buffer_base(buffer: Uint8Array) -> i32 {
    let buffer = &*buffer;
    #[expect(clippy::cast_possible_truncation)]
    return (buffer.as_ptr().addr() as u32).cast_signed();
}

/// Get offset within a `Uint8Array` which is aligned on `BLOCK_ALIGN`.
///
/// Not used by JS side any more, now that buffer doesn't need to be aligned on 4 GiB.
/// Retained only so that benchmarks can compare against buffers aligned on 4 GiB.
///
/// Does not check that the offset is within bounds of `buffer`.
/// To ensure it always is, provide a `Uint8Array` of at least `BLOCK_SIZE + BLOCK_ALIGN` bytes.
#[napi(skip_typescript)]
#[allow(clippy::needless_pass_by_value, clippy::allow_attributes)]
pub fn get_buffer_offset(buffer: Uint8Array) -> u32 {
    let buffer = &*buffer;
    // The final `% BLOCK_ALIGN` is to handle where `buffer` is already aligned on `BLOCK_ALIGN`.
    // In that case, `buffer.as_ptr().addr() % BLOCK_ALIGN == 0`, so without the final `% BLOCK_ALIGN`,
    // `offset` would be `BLOCK_ALIGN`. The final `% BLOCK_ALIGN` reduces it to `0`.
    let offset = (BLOCK_ALIGN - (buffer.as_ptr().addr() % BLOCK_ALIGN)) % BLOCK_ALIGN;
    #[expect(clippy::cast_possible_truncation)]
    return offset as u32;
}

/// Parse AST into provided `Uint8Array` buffer, synchronously.
///
/// Source text must be written into the buffer, starting at offset `source_start`,
/// and its length (in UTF-8 bytes) provided as `source_len`.
///
/// This function will parse the source, and write the AST into the buffer, starting at the end.
///
/// It also writes to the very end of the buffer the offset of `Program` within the buffer.
///
/// Caller can deserialize data from the buffer on JS side.
///
/// # SAFETY
///
/// Caller must ensure:
/// * Source text is written into buffer starting at offset `source_start`.
/// * Source text's UTF-8 byte length is `source_len`.
/// * The bytes comprising the source text form a valid UTF-8 string.
///
/// If source text is originally a JS string on JS side, and converted to a buffer with
/// `Buffer.from(str)` or `new TextEncoder().encode(str)`, this guarantees it's valid UTF-8.
///
/// # Panics
///
/// Panics if source text is too long, or AST takes more memory than is available in the buffer.
#[napi(skip_typescript)]
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

/// Parse AST into provided `Uint8Array` buffer, asynchronously.
///
/// Note: This function can be slower than `parseRawSync` due to the overhead of spawning a thread.
///
/// Source text must be written into the buffer, starting at offset `source_start`,
/// and its length (in UTF-8 bytes) provided as `source_len`.
///
/// This function will parse the source, and write the AST into the buffer, starting at the end.
///
/// It also writes to the very end of the buffer the offset of `Program` within the buffer.
///
/// Caller can deserialize data from the buffer on JS side.
///
/// # SAFETY
///
/// Caller must ensure:
/// * Source text is written into buffer starting at offset `source_start`.
/// * Source text's UTF-8 byte length is `source_len`.
/// * The bytes comprising the source text form a valid UTF-8 string.
/// * Contents of buffer must not be mutated by caller until the `AsyncTask` returned by this
///   function resolves.
///
/// If source text is originally a JS string on JS side, and converted to a buffer with
/// `Buffer.from(str)` or `new TextEncoder().encode(str)`, this guarantees it's valid UTF-8.
///
/// # Panics
///
/// Panics if source text is too long, or AST takes more memory than is available in the buffer.
#[napi(skip_typescript)]
pub fn parse_raw(
    filename: String,
    buffer: Uint8Array,
    source_start: u32,
    source_len: u32,
    options: Option<ParserOptions>,
) -> AsyncTask<ResolveTask> {
    AsyncTask::new(ResolveTask { filename, buffer, source_start, source_len, options })
}

pub struct ResolveTask {
    filename: String,
    buffer: Uint8Array,
    source_start: u32,
    source_len: u32,
    options: Option<ParserOptions>,
}

#[napi]
impl Task for ResolveTask {
    type JsValue = ();
    type Output = ();

    fn compute(&mut self) -> napi::Result<()> {
        // SAFETY: Caller of `parse_async` guarantees not to mutate the contents of buffer
        // between calling `parse_async` and the `AsyncTask` it returns resolving.
        // Therefore, this is a valid exclusive `&mut [u8]`.
        let buffer = unsafe { self.buffer.as_mut() };
        // SAFETY: Caller of `parse_async` guarantees to uphold invariants of `parse_raw_impl`
        unsafe {
            parse_raw_impl(
                &self.filename,
                buffer,
                self.source_start,
                self.source_len,
                self.options.take(),
            );
        }
        Ok(())
    }

    fn resolve(&mut self, _: napi::Env, _result: ()) -> napi::Result<()> {
        Ok(())
    }
}

/// Parse AST into buffer.
///
/// # SAFETY
///
/// Caller must ensure:
/// * Source text is written into buffer starting at offset `source_start`.
/// * Source text's UTF-8 byte length is `source_len`.
/// * The bytes comprising the source text form a valid UTF-8 string.
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
    // Check buffer has expected size and alignment.
    // Buffer does not need to be aligned on `BLOCK_ALIGN` (see comment at top of this file).
    assert_eq!(buffer.len(), BLOCK_SIZE);
    let buffer_ptr = NonNull::from_mut(buffer).cast::<u8>();
    assert!(buffer_ptr.addr().get().is_multiple_of(Allocator::RAW_MIN_ALIGN));

    const _: () = {
        assert!(BLOCK_SIZE.is_multiple_of(Allocator::RAW_MIN_ALIGN));
        assert!(BLOCK_SIZE >= Allocator::RAW_MIN_SIZE);
        assert!(BLOCK_ALIGN.is_multiple_of(Allocator::RAW_MIN_ALIGN));
    };

    // Create `Allocator`.
    //
    // Wrap in `ManuallyDrop` so the allocation doesn't get freed at end of function, or if panic.
    // The buffer is owned by JS, so Rust must not free it - hence `ManuallyDrop`.
    // The `backing_alloc_ptr` and `layout` we pass to `from_raw_parts` aren't used (the `Allocator` is never dropped),
    // but the safety contract requires the chunk region to lie within them, so we describe the buffer itself.
    //
    // SAFETY: `buffer_ptr` and `BLOCK_SIZE` outline the entirety of `buffer`.
    // `buffer_ptr` and `BLOCK_SIZE` are multiples of `Allocator::RAW_MIN_ALIGN`.
    // `BLOCK_SIZE` is `>= Allocator::RAW_MIN_SIZE`.
    // `buffer_ptr` is derived from a `&mut [u8]` slice, so has permission for writes.
    let allocator =
        unsafe { Allocator::from_raw_parts(buffer_ptr, BLOCK_SIZE, buffer_ptr, BLOCK_LAYOUT) };
    let allocator = ManuallyDrop::new(allocator);
    let allocator = &*allocator;

    // Check source text is in bounds of active data region of buffer.
    // Caller guarantees it is, but as this is critical to avoid reading/writing out of bounds,
    // we add this defensive runtime check.
    let source_start = source_start as usize;
    let source_end = source_start + (source_len as usize);
    assert!(source_end <= ACTIVE_SIZE);

    // Set cursor to before start of source text. AST will be written into the buffer before the source text.
    // Round down the pointer, so it's aligned on `CURSOR_MIN_ALIGN`.
    // SAFETY: Caller guarantees that source text starts at `source_start` bytes from start of buffer.
    unsafe {
        let cursor_pos = source_start & !(CURSOR_MIN_ALIGN - 1);
        debug_assert!(cursor_pos <= ACTIVE_SIZE);
        let cursor_ptr = buffer_ptr.add(cursor_pos);
        allocator.set_cursor_ptr(cursor_ptr);
    }

    // Parse source.
    // Enclose parsing logic in a scope to make 100% sure no references to within `Allocator`
    // exist after this.
    let options = options.unwrap_or_default();
    let source_type =
        get_source_type(filename, options.lang.as_deref(), options.source_type.as_deref());
    let is_ts = get_ast_type(source_type, &options) == AstType::TypeScript;

    let (data_offset, tokens_offset, tokens_len) = {
        // Get source text from buffer.
        // Use zero-cost unchecked conversion to `&str` in release builds, full UTF-8 validation in debug builds.
        let source_text = if cfg!(debug_assertions) {
            let source_bytes = &buffer[source_start..source_end];
            str::from_utf8(source_bytes).expect("Source text is not valid UTF-8")
        } else {
            // SAFETY: Caller guarantees source occupies this region of the buffer and is valid UTF-8
            unsafe {
                let source_bytes = buffer.get_unchecked(source_start..source_end);
                str::from_utf8_unchecked(source_bytes)
            }
        };

        // Parse
        let ret = parse_impl(allocator, source_type, source_text, &options);
        let mut program = ret.program;
        let mut module_record = ret.module_record;

        // Convert errors.
        // Run `SemanticBuilder` if requested.
        //
        // Note: Avoid calling `Error::from_diagnostics_in` unless there are some errors,
        // because it's fairly expensive (it copies whole of source text into a `String`).
        let mut errors = if options.show_semantic_errors == Some(true) {
            let semantic_ret = SemanticBuilder::new_compiler().build(&program);

            if !ret.diagnostics.is_empty() || !semantic_ret.diagnostics.is_empty() {
                Error::from_diagnostics_in(
                    ret.diagnostics.into_iter().chain(semantic_ret.diagnostics),
                    source_text,
                    filename,
                    allocator,
                )
            } else {
                ArenaVec::new_in(&allocator)
            }
        } else if !ret.diagnostics.is_empty() {
            Error::from_diagnostics_in(ret.diagnostics, source_text, filename, allocator)
        } else {
            ArenaVec::new_in(&allocator)
        };

        let span_converter = Utf8ToUtf16::new(source_text);

        // Convert tokens.
        // `experimentalTokens` option is only honored when `tokens` Cargo feature is enabled.
        // Otherwise, parser doesn't collect tokens, and `tokens_offset` / `tokens_len` are 0.
        #[cfg(feature = "tokens")]
        let (tokens_offset, tokens_len) = if options.tokens == Some(true) {
            let mut tokens = ret.tokens;
            update_tokens(&mut tokens, &program, &span_converter, ESTreeTokenOptions::new(is_ts));

            let tokens_offset = offset_in_buffer(tokens.as_ptr(), buffer_ptr);
            #[expect(clippy::cast_possible_truncation)]
            let tokens_len = tokens.len() as u32;
            (tokens_offset, tokens_len)
        } else {
            (0, 0)
        };
        #[cfg(not(feature = "tokens"))]
        let (tokens_offset, tokens_len) = (0, 0);

        // Convert spans to UTF-16
        span_converter.convert_program_and_comments(&mut program);
        span_converter.convert_module_record(&mut module_record);
        if let Some(mut converter) = span_converter.converter() {
            for error in &mut errors {
                for label in &mut error.labels {
                    converter.convert_span(&mut label.span);
                }
            }
        }

        let comments = mem::replace(&mut program.comments, ArenaVec::new_in(&allocator));

        // Convert module record
        let module = EcmaScriptModule::from_in(module_record, allocator);

        // Write `RawTransferData` to arena, and return pointer to it
        let data = RawTransferData { program, comments, module, errors };
        let data = allocator.alloc(data);
        let data_offset = offset_in_buffer(ptr::from_ref(data), buffer_ptr);

        (data_offset, tokens_offset, tokens_len)
    };

    // Write metadata into end of buffer
    let metadata = RawTransferMetadata::new(data_offset, is_ts, tokens_offset, tokens_len);
    const RAW_METADATA_OFFSET: usize = ACTIVE_SIZE;
    // SAFETY: `RAW_METADATA_OFFSET` is less than length of `buffer`, and aligned for `RawTransferMetadata`
    unsafe {
        let metadata_ptr = buffer_ptr.add(RAW_METADATA_OFFSET).cast::<RawTransferMetadata>();
        debug_assert!(metadata_ptr.addr().get().is_multiple_of(align_of::<RawTransferMetadata>()));
        metadata_ptr.write(metadata);
    }
}

/// Get offset of `ptr` relative to start of buffer.
///
/// `ptr` must point within the buffer, which is less than 2 GiB in size, so offset always fits in a `u32`.
#[expect(clippy::cast_possible_truncation)]
fn offset_in_buffer<T>(ptr: *const T, buffer_ptr: NonNull<u8>) -> u32 {
    let offset = ptr.addr() - buffer_ptr.addr().get();
    debug_assert!(offset < BLOCK_SIZE);
    offset as u32
}
