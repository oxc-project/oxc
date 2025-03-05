//! Stubs for raw transfer functions on unsupported platforms.
//!
//! These exports are required to avoid type-checking errors.

#![expect(unused_variables, unused_mut)]

use napi::bindgen_prelude::Uint8Array;
use napi_derive::napi;

use crate::ParserOptions;

#[napi]
pub fn get_buffer_offset(buffer: Uint8Array) -> u32 {
    0
}

#[napi]
pub fn parse_sync_raw(
    filename: String,
    mut buffer: Uint8Array,
    source_len: u32,
    options: Option<ParserOptions>,
) {
}

#[napi]
pub fn raw_transfer_supported() -> bool {
    false
}
