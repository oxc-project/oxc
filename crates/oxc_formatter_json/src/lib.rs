//! JSON formatter built on top of `oxc_formatter_core`.
//!
//! ```ignore
//! use oxc_allocator::Allocator;
//! use oxc_formatter_json::{JsonFormatOptions, format};
//!
//! let allocator = Allocator::new();
//! let formatted = format(&allocator, "{\"a\":1}", JsonFormatOptions::default()).unwrap();
//! let out = formatted.print().unwrap().into_code();
//! assert_eq!(out, "{ \"a\": 1 }");
//! ```

mod comments;
mod context;
mod format;
mod options;
mod parse;
mod print;
mod separated;

pub use crate::{
    context::JsonFormatContext,
    format::format,
    options::{
        BracketSpacing, Expand, JsonFormatOptions, JsonVariant, QuoteProps, SingleQuote,
        TrailingCommas,
    },
};
