//! Data structures used across other oxc crates.

#![warn(missing_docs)]

#[cfg(feature = "assert_unchecked")]
mod assert_unchecked;

#[cfg(feature = "box_macros")]
pub mod box_macros;

#[cfg(feature = "code_buffer")]
pub mod code_buffer;

#[cfg(feature = "inline_string")]
pub mod inline_string;

#[cfg(feature = "rope")]
pub mod rope;

#[cfg(feature = "slice_iter")]
pub mod slice_iter;

#[cfg(feature = "stack")]
pub mod stack;
