//! Markdown formatter built on top of `oxc_formatter_core`.
//!
//! Parses with `oxc-markdown-parser`
//! (CommonMark + GFM + Prettier's extensions, a span-faithful AST whose style facts are first-class fields)
//! and prints Prettier-compatible output.
//!
//! ```ignore
//! use oxc_allocator::Allocator;
//! use oxc_formatter_markdown::{MarkdownFormatOptions, format};
//!
//! let allocator = Allocator::new();
//! let formatted = format(&allocator, "#  Title", MarkdownFormatOptions::default()).unwrap();
//! let out = formatted.print().unwrap().into_code();
//! assert_eq!(out, "# Title\n");
//! ```

mod context;
mod format;
mod options;
mod print;

pub use crate::{
    context::MarkdownFormatContext,
    format::{ParsedMarkdown, format, format_to_ir, parse_for_format},
    options::{MarkdownFormatOptions, ProseWrap, SingleQuote},
};
