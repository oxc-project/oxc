//! Data structures used across other oxc crates.

#![warn(missing_docs)]

#[cfg(feature = "assert_unchecked")]
mod assert_unchecked;

#[cfg(feature = "code_buffer")]
pub mod code_buffer;

#[cfg(feature = "inline_string")]
pub mod inline_string;

#[cfg(feature = "rope")]
pub mod rope;

#[cfg(feature = "stack")]
pub mod stack;
