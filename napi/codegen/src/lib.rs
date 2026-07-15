#![expect(clippy::needless_pass_by_value)]

#[cfg(all(
    feature = "allocator",
    not(any(
        target_arch = "arm",
        target_os = "android",
        target_os = "freebsd",
        target_os = "windows",
        target_family = "wasm"
    ))
))]
#[global_allocator]
static ALLOC: mimalloc_safe::MiMalloc = mimalloc_safe::MiMalloc;

mod options;

use napi_derive::napi;

use oxc_napi::OxcError;
use oxc_sourcemap::napi::SourceMap;

pub use crate::options::*;

#[derive(Default)]
#[napi(object)]
pub struct PrintResult {
    pub code: String,
    pub map: Option<SourceMap>,
    pub errors: Vec<OxcError>,
}

/// Returns `true` if raw transfer back is supported on this platform.
///
/// Raw transfer back is only supported on 64-bit little-endian systems.
#[napi(skip_typescript)]
pub fn raw_transfer_supported() -> bool {
    cfg!(all(target_pointer_width = "64", target_endian = "little"))
}

/// Buffer geometry for raw transfer back.
///
/// Must be identical to the constants generated into
/// `napi/parser/src/generated/raw_transfer_constants.rs` by `oxc_ast_tools`.
/// The `RawTransferBackGenerator` will emit this package's own generated copy;
/// until then these are maintained by hand.
#[cfg(all(target_pointer_width = "64", target_endian = "little"))]
mod raw_transfer_back {
    use napi::bindgen_prelude::Uint8Array;
    use napi_derive::napi;

    use crate::{PrintOptions, PrintResult};
    use oxc_napi::OxcError;

    /// Alignment of the memory block used for raw transfer back (4 GiB).
    const BLOCK_ALIGN: usize = 1 << 32;

    /// Get offset within a `Uint8Array` which is aligned on `BLOCK_ALIGN`.
    ///
    /// Does not check that the offset is within bounds of `buffer`.
    /// To ensure it always is, provide a `Uint8Array` of at least
    /// `BLOCK_SIZE + BLOCK_ALIGN` bytes.
    #[napi(skip_typescript)]
    #[allow(clippy::needless_pass_by_value, clippy::allow_attributes)]
    pub fn get_buffer_offset(buffer: Uint8Array) -> u32 {
        let buffer = &*buffer;
        // The final `% BLOCK_ALIGN` handles where `buffer` is already aligned on `BLOCK_ALIGN`.
        let offset = (BLOCK_ALIGN - (buffer.as_ptr().addr() % BLOCK_ALIGN)) % BLOCK_ALIGN;
        #[expect(clippy::cast_possible_truncation)]
        return offset as u32;
    }

    /// Print a `Program` which was serialized into `buffer` by the JS-side
    /// `raw_transfer_back` encoder.
    ///
    /// `program_offset` is the offset of the root `Program` struct within `buffer`.
    /// `source_start` / `source_len` describe the source text region within `buffer`
    /// (both `0` when no source text was provided).
    ///
    /// Not intended to be called directly - `print()` in `src-js/index.js` is the API.
    #[napi(skip_typescript)]
    pub fn print_raw_sync(
        buffer: Uint8Array,
        program_offset: u32,
        source_start: u32,
        source_len: u32,
        options: Option<PrintOptions>,
    ) -> PrintResult {
        let (_, _, _, _) = (buffer, program_offset, source_start, source_len);
        let _codegen_options = options.unwrap_or_default().to_codegen_options();
        // TODO(raw_transfer_back): placement-construct the `Allocator` at the buffer's
        // arena slot, form `&Program` at `program_offset`, and run `oxc_codegen::Codegen`
        // with `_codegen_options`.
        PrintResult {
            errors: vec![OxcError::new(
                "raw_transfer_back is not implemented yet: the ESTree -> arena encoder has not landed".to_string(),
            )],
            ..PrintResult::default()
        }
    }
}

#[cfg(all(target_pointer_width = "64", target_endian = "little"))]
pub use raw_transfer_back::{get_buffer_offset, print_raw_sync};
