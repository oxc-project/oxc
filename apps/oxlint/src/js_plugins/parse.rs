use std::ptr;

use napi::bindgen_prelude::Uint8Array;
use napi_derive::napi;

use oxc_allocator::{Allocator, AllocatorPool};
use oxc_ast::ast::{Comment, CommentContent, CommentKind};
use oxc_ast_visit::utf8_to_utf16::Utf8ToUtf16;
use oxc_estree_tokens::update_tokens_as_js;
use oxc_linter::RawTransferMetadata2 as RawTransferMetadata;
use oxc_napi::get_source_type;
use oxc_parser::{ParseOptions, Parser, ParserReturn, config::RuntimeParserConfig};
use oxc_semantic::SemanticBuilder;

use super::external_linter::fixed_size_chunk_ptr;
use crate::generated::raw_transfer_constants::BUFFER_SIZE;

thread_local! {
    /// Pool holding the single fixed-size `Allocator` that `parse_raw_sync` parses into.
    ///
    /// Rust owns the buffer for the life of the thread, and JS reads it through views obtained
    /// from `get_raw_transfer_buffer`. JS never allocates or writes the buffer itself. This avoids
    /// the 6 GiB `ArrayBuffer` JS would otherwise need to obtain a 4 GiB-aligned 2 GiB region,
    /// which Bun (4 GiB `ArrayBuffer` limit) cannot provide, and lets Windows commit the block lazily.
    ///
    /// Thread-local because `Allocator` is not `Sync`, and test runners may execute `RuleTester`
    /// on several threads, each with its own JS realm and its own `buffers` array.
    static ALLOCATOR_POOL: AllocatorPool = AllocatorPool::new_fixed_size(1);
}

/// Sentinel value for program offset to indicate parsing failed.
///
/// 0 cannot be a valid offset. The allocator bumps downwards from the end of the buffer,
/// so `Program` always sits at a non-zero offset.
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

/// Get a `Uint8Array` view of this thread's raw transfer buffer.
///
/// The view covers the allocatable region plus `RawTransferMetadata`, the same region the linter
/// shares with JS plugins. JS reads the AST from it; the only bytes JS writes are the per-comment
/// and per-token "deserialized" flags, which Rust rewrites on every parse.
///
/// Rust keeps ownership of the memory. The view has no finalizer, and the `is_double_owned` flag
/// used by the linter's `get_buffer` is never set, so the block is freed only when the thread's
/// `ALLOCATOR_POOL` is dropped at thread exit, after the thread's JS realm is gone. JS may call this
/// more than once: a test runner that resets its module registry obtains a fresh view of the same
/// memory. Callers should keep one view per module instance.
///
/// # Panics
///
/// Panics on the thread's first call if the fixed-size allocation cannot be made
/// (see `AllocatorPool::new_fixed_size`).
#[napi]
pub fn get_raw_transfer_buffer() -> Uint8Array {
    ALLOCATOR_POOL.with(|pool| {
        let allocator = pool.get();

        // SAFETY: `ALLOCATOR_POOL` is a fixed-size pool, so `allocator` has a `FixedSizeAllocatorMetadata`
        let chunk_ptr = unsafe {
            let metadata_ptr = allocator.fixed_size_metadata_ptr();
            fixed_size_chunk_ptr(metadata_ptr)
        };

        // SAFETY: Range of memory starting at `chunk_ptr` and encompassing `BUFFER_SIZE` is all within
        // the allocation backing the `Allocator`, which lives as long as `ALLOCATOR_POOL` (thread lifetime).
        // The no-op finalizer leaves ownership with Rust; see doc comment above for the aliasing contract.
        unsafe { Uint8Array::with_external_data(chunk_ptr.as_ptr(), BUFFER_SIZE, |_ptr, _len| {}) }
    })
}

/// Parse source text into this thread's raw transfer buffer, synchronously.
///
/// The source text is copied into the buffer, and the AST is written after it.
/// The offset of `Program` within the buffer is written into the buffer's `RawTransferMetadata` slot.
///
/// Caller can deserialize data from the buffer on JS side, via the view from `getRawTransferBuffer`.
///
/// The buffer's contents remain valid until the next call to `parse_raw_sync` on the same thread.
///
/// Returns the ID of the buffer the AST was written into.
///
/// # Panics
///
/// Panics if source text and AST take more memory than is available in the buffer.
#[napi]
#[allow(clippy::needless_pass_by_value, clippy::allow_attributes)]
pub fn parse_raw_sync(
    filename: String,
    source_text: String,
    options: Option<ParserOptions>,
) -> u32 {
    ALLOCATOR_POOL.with(|pool| {
        let allocator = pool.get();
        parse_raw_impl(&filename, &allocator, &source_text, options);
        // SAFETY: `allocator` was obtained from a fixed-size `AllocatorPool`, so it has a
        // `FixedSizeAllocatorMetadata`. Only an immutable reference is created.
        unsafe { allocator.fixed_size_metadata_ptr().as_ref().id }
    })
}

/// Parse source text into `allocator`, and write `RawTransferMetadata` into its metadata slot.
///
/// `allocator` must have been obtained from a fixed-size `AllocatorPool`.
fn parse_raw_impl(
    filename: &str,
    allocator: &Allocator,
    source_text: &str,
    options: Option<ParserOptions>,
) {
    // Copy source text into the buffer, same as the linter does for JS plugins
    let source_text = allocator.alloc_str(source_text);

    // Get source type
    let options = options.unwrap_or_default();
    let source_type =
        get_source_type(filename, options.lang.as_deref(), options.source_type.as_deref());
    let ignore_non_fatal_errors = options.ignore_non_fatal_errors.unwrap_or(false);

    // Parse source.
    // Enclose parsing logic in a scope to make 100% sure no references to within `Allocator` exist after this.
    let (program_offset, has_bom, tokens_offset, tokens_len) = {
        // Parse with same options as linter.
        // We use `RuntimeParserConfig` even though we always pass `true` here, to avoid compiling the parser twice.
        // The linter itself uses `RuntimeParserConfig`.
        let parser_ret = Parser::new(allocator, source_text, source_type)
            .with_options(ParseOptions {
                parse_regular_expression: true,
                allow_return_outside_function: true,
                ..ParseOptions::default()
            })
            .with_config(RuntimeParserConfig::new(true))
            .parse();
        let ParserReturn { program: parsed_program, diagnostics, mut tokens, fatal_error, .. } =
            parser_ret;
        let program = allocator.alloc(parsed_program);

        let mut parsing_failed =
            fatal_error || (!diagnostics.is_empty() && !ignore_non_fatal_errors);

        // Check for semantic errors.
        // If `ignore_non_fatal_errors` is `true`, skip running semantic, as any errors will be ignored anyway.
        if !parsing_failed && !ignore_non_fatal_errors {
            let semantic_ret = SemanticBuilder::new_compiler().build(program);
            parsing_failed = !semantic_ret.diagnostics.is_empty();
        }

        if parsing_failed {
            // Use sentinel value for program offset to indicate that parsing failed
            (PARSE_FAIL_SENTINEL, false, 0, 0)
        } else {
            // If has BOM, remove it
            const BOM: &str = "\u{feff}";
            const BOM_LEN: usize = BOM.len();

            let original_source_text = program.source_text;
            let mut source_text = original_source_text;
            let has_bom = source_text.starts_with(BOM);
            if has_bom {
                source_text = &source_text[BOM_LEN..];
                program.source_text = source_text;
            }

            // If file has a hashbang, add it to comments.
            // It will be converted to a `Shebang` comment on JS side.
            if let Some(hashbang) = &program.hashbang {
                program.comments.insert(
                    0,
                    Comment::new(hashbang.span.start, hashbang.span.end, CommentKind::Line),
                );
            }

            // Create span converter.
            // If source starts with BOM, create converter which ignores the BOM.
            let span_converter = if has_bom {
                #[expect(clippy::cast_possible_truncation)]
                Utf8ToUtf16::new_with_offset(source_text, BOM_LEN as u32)
            } else {
                Utf8ToUtf16::new(source_text)
            };

            // Convert token spans to UTF-16 and update token kinds
            update_tokens_as_js(&mut tokens, program, &span_converter);

            // Convert AST spans to UTF-16
            span_converter.convert_program(program);

            // Convert comment spans to UTF-16.
            // Also set the `content` field (byte 15) of each comment to `None` (0).
            // JS side uses this byte as a "deserialized" flag for tracking lazy deserialization.
            if let Some(mut converter) = span_converter.converter() {
                for comment in &mut program.comments {
                    converter.convert_span(&mut comment.span);
                    comment.content = CommentContent::None;
                }
            } else {
                for comment in &mut program.comments {
                    comment.content = CommentContent::None;
                }
            }

            let tokens_offset = tokens.as_ptr() as u32;
            #[expect(clippy::cast_possible_truncation)]
            let tokens_len = tokens.len() as u32;

            // Return offset of `Program` within buffer (bottom 32 bits of pointer)
            let program_offset = ptr::from_ref(program) as u32;

            (program_offset, has_bom, tokens_offset, tokens_len)
        }
    };

    // Write metadata into the buffer's `RawTransferMetadata` slot, which sits immediately before
    // `FixedSizeAllocatorMetadata` at the end of the buffer. Same as the linter does for JS plugins.
    let metadata = RawTransferMetadata::new(
        program_offset,
        source_type.is_typescript(),
        source_type.is_jsx(),
        has_bom,
        tokens_offset,
        tokens_len,
    );

    // SAFETY: Caller guarantees `allocator` came from a fixed-size `AllocatorPool`, so it has
    // a `FixedSizeAllocatorMetadata`, and a `RawTransferMetadata` slot immediately before it.
    unsafe {
        let metadata_ptr = allocator
            .fixed_size_metadata_ptr()
            .cast::<u8>()
            .sub(size_of::<RawTransferMetadata>())
            .cast::<RawTransferMetadata>();
        debug_assert!(metadata_ptr.addr().get().is_multiple_of(align_of::<RawTransferMetadata>()));
        metadata_ptr.write(metadata);
    }
}
