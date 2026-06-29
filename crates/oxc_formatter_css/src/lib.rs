//! CSS/SCSS/Less formatter built on top of `oxc_formatter_core`.
//!
//! Parses with [raffia](https://docs.rs/raffia) and prints Prettier-compatible output.
//!
//! ```ignore
//! use oxc_allocator::Allocator;
//! use oxc_formatter_css::{CssFormatOptions, format};
//!
//! let allocator = Allocator::new();
//! let formatted = format(&allocator, "a { color: red }", CssFormatOptions::default()).unwrap();
//! let out = formatted.print().unwrap().into_code();
//! assert_eq!(out, "a {\n  color: red;\n}\n");
//! ```

mod comments;
mod context;
mod format;
mod options;
mod print;

pub use crate::{
    context::CssFormatContext,
    format::{format, format_to_ir},
    options::{CssFormatOptions, CssVariant, SingleQuote, TrailingCommas},
};
