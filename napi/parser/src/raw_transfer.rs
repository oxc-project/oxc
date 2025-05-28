use std::{
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
use oxc_napi::get_source_type;

use crate::{
    AstType, ParserOptions, get_ast_type, parse,
    raw_transfer_types::{EcmaScriptModule, Error, RawTransferData},
};

// Only 64-bit little-endian platforms are supported at present.
const IS_SUPPORTED_PLATFORM: bool =
    cfg!(all(target_pointer_width = "64", target_endian = "little"));

// For raw transfer, use a buffer 4 GiB in size, with 4 GiB alignment.
// This ensures that all 64-bit pointers have the same value in upper 32 bits,
// so JS only needs to read the lower 32 bits to get an offset into the buffer.
// However, only use first half of buffer (2 GiB) for the arena, so 32-bit offsets
// don't have the highest bit set. JS bitwise operators interpret the highest bit as sign bit,
// so this enables using `>>` bitshift operator in JS, rather than the more expensive `>>>`,
// without offsets being interpreted as negative.
const TWO_GIB: usize = 1 << 31;
// `1 << 32`.
// We use `IS_SUPPORTED_PLATFORM as usize * 32` to avoid compilation failure on 32-bit platforms.
const FOUR_GIB: usize = 1 << (IS_SUPPORTED_PLATFORM as usize * 32);

const BUFFER_SIZE: usize = TWO_GIB;
const BUFFER_ALIGN: usize = FOUR_GIB;
const BUMP_ALIGN: usize = 16;

/// Get offset within a `Uint8Array` which is aligned on 4 GiB.
///
/// Does not check that the offset is within bounds of `buffer`.
/// To ensure it always is, provide a `Uint8Array` of at least 4 GiB size.
#[napi]
pub fn get_buffer_offset(buffer: Uint8Array) -> u32 {
    let buffer = &*buffer;
    let buffer_addr32 = buffer.as_ptr() as u32;
    0u32.wrapping_sub(buffer_addr32)
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
#[napi]
pub unsafe fn parse_sync_raw(
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
/// Note: This function can be slower than `parseSyncRaw` due to the overhead of spawning a thread.
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
#[napi]
pub fn parse_async_raw(
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
    assert!(
        IS_SUPPORTED_PLATFORM,
        "Raw transfer is only supported on 64-bit little-endian platforms"
    );

    // Check buffer has expected size and alignment
    assert_eq!(buffer.len(), BUFFER_SIZE);
    let buffer_ptr = ptr::from_mut(buffer).cast::<u8>();
    assert!(is_multiple_of(buffer_ptr as usize, BUFFER_ALIGN));

    // Get offsets and size of data region to be managed by arena allocator.
    // Leave space for source before it, and 16 bytes for metadata after it.
    // Metadata actually only takes 5 bytes, but round everything up to multiple of 16,
    // as `bumpalo` requires that alignment.
    const METADATA_SIZE: usize = 16;
    const {
        assert!(METADATA_SIZE >= BUMP_ALIGN);
        assert!(is_multiple_of(METADATA_SIZE, BUMP_ALIGN));
    };
    let source_len = source_len as usize;
    let data_offset = source_len.next_multiple_of(BUMP_ALIGN);
    let data_size = BUFFER_SIZE.saturating_sub(data_offset + METADATA_SIZE);
    assert!(data_size >= Allocator::RAW_MIN_SIZE, "Source text is too long");

    // Create `Allocator`.
    // Wrap in `ManuallyDrop` so the allocation doesn't get freed at end of function, or if panic.
    // SAFETY: `data_offset` is less than `buffer.len()`, so `.add(data_offset)` cannot wrap
    // or be out of bounds.
    let data_ptr = unsafe { buffer_ptr.add(data_offset) };
    debug_assert!(is_multiple_of(data_ptr as usize, BUMP_ALIGN));
    debug_assert!(is_multiple_of(data_size, BUMP_ALIGN));
    // SAFETY: `data_ptr` and `data_size` outline a section of the memory in `buffer`.
    // `data_ptr` and `data_size` are multiples of 16.
    // `data_size` is greater than `Allocator::MIN_SIZE`.
    let allocator =
        unsafe { Allocator::from_raw_parts(NonNull::new_unchecked(data_ptr), data_size) };
    let allocator = ManuallyDrop::new(allocator);

    // Parse source.
    // Enclose parsing logic in a scope to make 100% sure no references to within `Allocator`
    // exist after this.
    let options = options.unwrap_or_default();
    let source_type =
        get_source_type(filename, options.lang.as_deref(), options.source_type.as_deref());
    let ast_type = get_ast_type(source_type, &options);

    let data_ptr = {
        // SAFETY: We checked above that `source_len` does not exceed length of buffer
        let source_text = unsafe { buffer.get_unchecked(..source_len) };
        // SAFETY: Caller guarantees source occupies this region of the buffer and is valid UTF-8
        let source_text = unsafe { str::from_utf8_unchecked(source_text) };

        let ret = parse(&allocator, source_type, source_text, &options);
        let mut program = ret.program;
        let mut comments = mem::replace(&mut program.comments, ArenaVec::new_in(&allocator));
        let mut module_record = ret.module_record;

        // Convert errors
        let mut errors = ArenaVec::from_iter_in(
            ret.errors.iter().map(|error| Error::from_in(error, &allocator)),
            &allocator,
        );

        // Run `SemanticBuilder` if requested
        if options.show_semantic_errors == Some(true) {
            let semantic_ret = SemanticBuilder::new().with_check_syntax_error(true).build(&program);
            errors
                .extend(semantic_ret.errors.iter().map(|error| Error::from_in(error, &allocator)));
        }

        // Convert spans to UTF-16
        let span_converter = Utf8ToUtf16::new(source_text);
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
        ptr::from_ref(data).cast::<u8>()
    };

    // Write offset of `RawTransferData` and `bool` representing AST type into end of buffer
    #[allow(clippy::cast_possible_truncation)]
    let data_offset = data_ptr as u32;
    const METADATA_OFFSET: usize = BUFFER_SIZE - METADATA_SIZE;
    // SAFETY: `METADATA_OFFSET` is less than length of `buffer`
    #[expect(clippy::cast_ptr_alignment)]
    unsafe {
        buffer_ptr.add(METADATA_OFFSET).cast::<u32>().write(data_offset);
        buffer_ptr.add(METADATA_OFFSET + 4).cast::<bool>().write(ast_type == AstType::TypeScript);
    }
}

/// Returns `true` if raw transfer is supported on this platform.
#[napi]
pub fn raw_transfer_supported() -> bool {
    IS_SUPPORTED_PLATFORM
}

/// Returns `true` if `n` is a multiple of `divisor`.
const fn is_multiple_of(n: usize, divisor: usize) -> bool {
    n % divisor == 0
}
