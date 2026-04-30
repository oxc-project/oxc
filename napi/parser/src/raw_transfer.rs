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
    allocator::{Allocator, FromIn, Vec as ArenaVec},
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

// For raw transfer, use a buffer 2 GiB in size, with 4 GiB alignment.
// This ensures that all 64-bit pointers have the same value in upper 32 bits,
// so JS only needs to read the lower 32 bits to get an offset into the buffer.
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

const ARENA_ALIGN: usize = Allocator::RAW_MIN_ALIGN;

/// Layout describing the JS-owned buffer (`BLOCK_SIZE` bytes, aligned on `BLOCK_ALIGN`).
const BLOCK_LAYOUT: Layout = match Layout::from_size_align(BLOCK_SIZE, BLOCK_ALIGN) {
    Ok(layout) => layout,
    Err(_) => unreachable!(),
};

/// Get offset within a `Uint8Array` which is aligned on `BLOCK_ALIGN`.
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
/// Source text must be written into the start of the buffer, and its length (in UTF-8 bytes)
/// provided as `source_len`.
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
/// * Source text is written into start of the buffer.
/// * Source text's UTF-8 byte length is `source_len`.
/// * The 1st `source_len` bytes of the buffer comprises a valid UTF-8 string.
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
    source_len: u32,
    options: Option<ParserOptions>,
) {
    // SAFETY: This function is called synchronously, so buffer cannot be mutated outside this function
    // during the time this `&mut [u8]` exists
    let buffer = unsafe { buffer.as_mut() };

    // SAFETY: `parse_raw_impl` has same safety requirements as this function
    unsafe { parse_raw_impl(&filename, buffer, source_len, options) };
}

/// Parse AST into provided `Uint8Array` buffer, asynchronously.
///
/// Note: This function can be slower than `parseRawSync` due to the overhead of spawning a thread.
///
/// Source text must be written into the start of the buffer, and its length (in UTF-8 bytes)
/// provided as `source_len`.
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
/// * Source text is written into start of the buffer.
/// * Source text's UTF-8 byte length is `source_len`.
/// * The 1st `source_len` bytes of the buffer comprises a valid UTF-8 string.
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
    source_len: u32,
    options: Option<ParserOptions>,
) -> AsyncTask<ResolveTask> {
    AsyncTask::new(ResolveTask { filename, buffer, source_len, options })
}

pub struct ResolveTask {
    filename: String,
    buffer: Uint8Array,
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
        unsafe { parse_raw_impl(&self.filename, buffer, self.source_len, self.options.take()) };
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
/// * Source text is written into start of the buffer.
/// * Source text's UTF-8 byte length is `source_len`.
/// * The 1st `source_len` bytes of the buffer comprises a valid UTF-8 string.
///
/// If source text is originally a JS string on JS side, and converted to a buffer with
/// `Buffer.from(str)` or `new TextEncoder().encode(str)`, this guarantees it's valid UTF-8.
#[allow(clippy::items_after_statements, clippy::allow_attributes)]
unsafe fn parse_raw_impl(
    filename: &str,
    buffer: &mut [u8],
    source_len: u32,
    options: Option<ParserOptions>,
) {
    // Check buffer has expected size and alignment
    assert_eq!(buffer.len(), BLOCK_SIZE);
    let buffer_ptr = NonNull::from_mut(buffer).cast::<u8>();
    assert!(buffer_ptr.addr().get().is_multiple_of(BLOCK_ALIGN));

    // Get offsets and size of data region to be managed by arena allocator.
    //
    // After the source text, the allocator's chunk fills the rest of the buffer from `data_ptr` to the end.
    // After the allocatable region (size `ACTIVE_SIZE`) sits:
    // * `RawTransferMetadata`
    // * A reserved slot for `FixedSizeAllocatorMetadata` (unused in `napi/parser`)
    // * `ChunkFooter`
    // The end of `ChunkFooter` is the end of the buffer.
    //
    // We set the cursor to before `RawTransferMetadata` so allocations don't overwrite the metadata regions.
    //
    // Round up `data_offset` to next multiple of `ARENA_ALIGN`, as required by `Allocator`.
    // Check that there's enough space for `RawTransferMetadata`, `FixedSizeAllocatorMetadata`, and `ChunkFooter`.
    let source_len = source_len as usize;
    let data_offset = source_len.next_multiple_of(ARENA_ALIGN);
    assert!(data_offset <= ACTIVE_SIZE, "Source text is too long");

    // Calculate size of allocator chunk (including metadata and `ChunkFooter`)
    let data_size = BLOCK_SIZE - data_offset;

    // Create `Allocator`.
    // Wrap in `ManuallyDrop` so the allocation doesn't get freed at end of function, or if panic.
    // The buffer is owned by JS, so Rust must not free it - hence `ManuallyDrop`. The
    // `backing_alloc_ptr` and `layout` we pass to `from_raw_parts` are never used (the `Allocator`
    // is never dropped), but the safety contract requires the chunk region to lie within them,
    // so we describe the buffer itself.
    // SAFETY: `data_offset` is less than `buffer.len()`, so `.add(data_offset)` cannot wrap
    // or be out of bounds.
    let data_ptr = unsafe { buffer_ptr.add(data_offset) };
    debug_assert!(data_ptr.addr().get().is_multiple_of(ARENA_ALIGN));
    debug_assert!(data_size.is_multiple_of(ARENA_ALIGN));

    // SAFETY: `data_ptr` and `data_size` outline a section of the memory in `buffer`.
    // `data_ptr` and `data_size` are multiples of `ARENA_ALIGN`.
    // `data_size` is greater than `Allocator::RAW_MIN_SIZE`.
    // The chunk region (`data_ptr..data_ptr + data_size`) lies entirely within the buffer.
    // `buffer_ptr` was derived from a `&mut [u8]` slice, so has permission for writes.
    // `data_ptr` was derived from `buffer_ptr`, so inherits that permission.
    let allocator =
        unsafe { Allocator::from_raw_parts(data_ptr, data_size, buffer_ptr, BLOCK_LAYOUT) };
    let allocator = ManuallyDrop::new(allocator);

    const _: () = assert!(ACTIVE_SIZE.is_multiple_of(CURSOR_MIN_ALIGN));

    // Set cursor to before `RawTransferMetadata` so allocations don't overwrite the metadata regions.
    // `RawTransferMetadata` starts at offset `ACTIVE_SIZE` within the buffer.
    // SAFETY: `ACTIVE_SIZE` is within the chunk (after `data_ptr`, before the `ChunkFooter`).
    // `ACTIVE_SIZE` is aligned on `Arena::MIN_ALIGN`.
    unsafe {
        let cursor_ptr = buffer_ptr.add(ACTIVE_SIZE);
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
        // SAFETY: We checked above that `source_len` does not exceed length of buffer
        let source_text = unsafe { buffer.get_unchecked(..source_len) };
        // SAFETY: Caller guarantees source occupies this region of the buffer and is valid UTF-8
        let source_text = unsafe { str::from_utf8_unchecked(source_text) };

        let ret = parse_impl(&allocator, source_type, source_text, &options);
        let mut program = ret.program;
        let mut comments = mem::replace(&mut program.comments, ArenaVec::new_in(&allocator));
        let mut module_record = ret.module_record;

        // Convert errors.
        // Run `SemanticBuilder` if requested.
        //
        // Note: Avoid calling `Error::from_diagnostics_in` unless there are some errors,
        // because it's fairly expensive (it copies whole of source text into a `String`).
        let mut errors = if options.show_semantic_errors == Some(true) {
            let semantic_ret = SemanticBuilder::new().with_check_syntax_error(true).build(&program);

            if !ret.errors.is_empty() || !semantic_ret.errors.is_empty() {
                Error::from_diagnostics_in(
                    ret.errors.into_iter().chain(semantic_ret.errors),
                    source_text,
                    filename,
                    &allocator,
                )
            } else {
                ArenaVec::new_in(&allocator)
            }
        } else if !ret.errors.is_empty() {
            Error::from_diagnostics_in(ret.errors, source_text, filename, &allocator)
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

            let tokens_offset = tokens.as_ptr() as u32;
            #[expect(clippy::cast_possible_truncation)]
            let tokens_len = tokens.len() as u32;
            (tokens_offset, tokens_len)
        } else {
            (0, 0)
        };
        #[cfg(not(feature = "tokens"))]
        let (tokens_offset, tokens_len) = (0, 0);

        // Convert spans to UTF-16
        span_converter.convert_program(&mut program);
        span_converter.convert_comments(&mut comments);
        span_converter.convert_module_record(&mut module_record);
        if let Some(mut converter) = span_converter.converter() {
            for error in &mut errors {
                for label in &mut error.labels {
                    converter.convert_span(&mut label.span);
                }
            }
        }

        // Convert module record
        let module = EcmaScriptModule::from_in(module_record, &allocator);

        // Write `RawTransferData` to arena, and return pointer to it
        let data = RawTransferData { program, comments, module, errors };
        let data = allocator.alloc(data);
        let data_offset = ptr::from_ref(data).cast::<u8>() as u32;

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
