//! Allocation of raw transfer buffers on Rust side.
//!
//! Raw transfer requires a buffer of `BLOCK_SIZE` bytes (2 GiB - 16), aligned on `BLOCK_ALIGN` (4 GiB),
//! so that the lower 32 bits of every pointer in the AST equal the pointer's offset within the buffer.
//!
//! JS cannot allocate an `ArrayBuffer` with a specific alignment, so the JS-side fallback allocates
//! `BLOCK_SIZE + BLOCK_ALIGN` (6 GiB) and uses an aligned region within it. That consumes 6 GiB of
//! virtual address space per buffer.
//!
//! [`create_raw_transfer_buffer`] instead allocates an aligned region of 2 GiB directly, using OS APIs.
//! The buffer then only occupies 2 GiB of address space.
//! The memory is passed to JS as an external `ArrayBuffer`, and is freed when that `ArrayBuffer`
//! is garbage collected.

use std::{
    ffi::c_void,
    ptr::{self, NonNull},
};

use napi::{bindgen_prelude::ToNapiValue, sys};
use napi_derive::napi;

use crate::raw_transfer_constants::BLOCK_SIZE;

/// Allocate a buffer for raw transfer: `BLOCK_SIZE` bytes, aligned on `BLOCK_ALIGN` (4 GiB),
/// occupying only 2 GiB of virtual address space.
///
/// Returns an `ArrayBuffer`, or `null` if the buffer could not be allocated, or the platform or runtime
/// does not support it (e.g. Electron, which does not allow external `ArrayBuffer`s).
/// If `null` is returned, JS should fall back to allocating the buffer itself.
#[napi(skip_typescript)]
pub fn create_raw_transfer_buffer() -> RawTransferBuffer {
    RawTransferBuffer { ptr: block::alloc() }
}

/// A buffer allocated by [`create_raw_transfer_buffer`], which converts to a JS `ArrayBuffer`,
/// or to `null` if allocation failed.
pub struct RawTransferBuffer {
    ptr: Option<NonNull<u8>>,
}

impl ToNapiValue for RawTransferBuffer {
    unsafe fn to_napi_value(env: sys::napi_env, mut val: Self) -> napi::Result<sys::napi_value> {
        if let Some(ptr) = val.ptr.take() {
            let mut array_buffer = ptr::null_mut();
            // SAFETY: `ptr` points to an allocation of at least `BLOCK_SIZE` bytes, which is valid
            // until `finalize_buffer` is called
            let status = unsafe {
                sys::napi_create_external_arraybuffer(
                    env,
                    ptr.as_ptr().cast::<c_void>(),
                    BLOCK_SIZE,
                    Some(finalize_buffer),
                    ptr::null_mut(),
                    &raw mut array_buffer,
                )
            };
            match status {
                sys::Status::napi_ok => return Ok(array_buffer),
                // Runtime does not support external `ArrayBuffer`s (e.g. Electron).
                // The finalizer has not been registered, so free the memory.
                // SAFETY: `ptr` was allocated by `block::alloc`, and not passed to JS
                sys::Status::napi_no_external_buffers_allowed => unsafe { block::free(ptr) },
                // Creating the `ArrayBuffer` failed for some other reason. Depending on where it failed,
                // the runtime may already have called the finalizer, or may call it later (e.g. NodeJS's
                // `node::Buffer::New` calls it on some error paths), so freeing the memory here could
                // free it twice. Leak it instead. No pages have been written to, so this only leaks
                // address space.
                _ => {}
            }

            // Return `null`. JS will fall back to allocating the buffer itself.
            // Note: Do not use napi-rs's `Uint8Array::with_external_data`, because when external buffers
            // are not allowed, it copies the data into a new `ArrayBuffer`, which here would be 2 GiB.
        }

        let mut null = ptr::null_mut();
        // SAFETY: `env` is a valid napi env
        let status = unsafe { sys::napi_get_null(env, &raw mut null) };
        if status != sys::Status::napi_ok {
            return Err(napi::Error::new(napi::Status::GenericFailure, "Failed to create `null`"));
        }
        Ok(null)
    }
}

impl Drop for RawTransferBuffer {
    fn drop(&mut self) {
        // If not converted to a JS value, free the memory
        if let Some(ptr) = self.ptr.take() {
            // SAFETY: `ptr` was allocated by `block::alloc`, and not passed to JS
            unsafe { block::free(ptr) };
        }
    }
}

/// Finalizer for external `ArrayBuffer`s created by [`RawTransferBuffer::to_napi_value`].
/// Called when the `ArrayBuffer` is garbage collected.
unsafe extern "C" fn finalize_buffer(_env: sys::napi_env, data: *mut c_void, _hint: *mut c_void) {
    // SAFETY: `data` is the pointer which was passed to `napi_create_external_arraybuffer`,
    // which was allocated by `block::alloc`. JS no longer has access to it.
    unsafe { block::free(NonNull::new_unchecked(data.cast::<u8>())) };
}

/// Placeholder until allocation is implemented for each platform. `alloc` always fails,
/// so `create_raw_transfer_buffer` returns `null`, and JS must allocate the buffer itself.
mod block {
    use std::ptr::NonNull;

    pub fn alloc() -> Option<NonNull<u8>> {
        None
    }

    /// # SAFETY
    /// Never called, as `alloc` never returns a pointer.
    pub unsafe fn free(_ptr: NonNull<u8>) {
        unreachable!();
    }
}
