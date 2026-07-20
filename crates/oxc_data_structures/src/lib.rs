//! Data structures used across other oxc crates.

#[cfg(feature = "assert_unchecked")]
mod assert_unchecked;

#[cfg(feature = "box_macros")]
pub mod box_macros;

#[cfg(feature = "branch_hints")]
pub mod branch_hints;

#[cfg(feature = "code_buffer")]
pub mod code_buffer;

#[cfg(feature = "fieldless_enum")]
pub mod fieldless_enum;

#[cfg(feature = "inline_string")]
pub mod inline_string;

#[cfg(feature = "non_null")]
pub mod non_null;

#[cfg(feature = "rope")]
pub mod rope;

#[cfg(feature = "slice_iter")]
pub mod slice_iter;

#[cfg(feature = "stack")]
pub mod stack;

#[cfg(feature = "str")]
pub mod str;

#[cfg(feature = "string_ext")]
pub mod string_ext;

// Gated on `any(test, ...)` because `implements!` is used in unit tests for `non_null` and `stack`,
// and unit tests cannot enable features.
// This enables:
// * `cargo test -p oxc_data_structures --features non_null`
// * `cargo test -p oxc_data_structures --features stack`
#[cfg(any(test, feature = "types"))]
pub mod types;
