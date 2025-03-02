//! Data structures used across other oxc crates.

#![warn(missing_docs)]

#[cfg(feature = "code_buffer")]
pub mod code_buffer;
#[cfg(feature = "inline_string")]
pub mod inline_string;
#[cfg(feature = "num")]
pub mod num;
#[cfg(feature = "rope")]
pub mod rope;
#[cfg(feature = "stack")]
pub mod stack;
