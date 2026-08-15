//! YAML formatter built on top of `oxc_formatter_core`.
//!
//! Parses with [oxc-yaml-parser](https://docs.rs/oxc-yaml-parser)
//! (a YAML 1.2 parser whose AST mirrors `yaml-unist-parser`'s node shapes)
//! and prints Prettier-compatible output.
//!
//! ```ignore
//! use oxc_allocator::Allocator;
//! use oxc_formatter_yaml::{YamlFormatOptions, format};
//!
//! let allocator = Allocator::new();
//! let formatted = format(&allocator, "key:   value", YamlFormatOptions::default()).unwrap();
//! let out = formatted.print().unwrap().into_code();
//! assert_eq!(out, "key: value\n");
//! ```

mod comments;
mod context;
mod format;
mod options;
mod print;

pub use crate::{
    context::YamlFormatContext,
    format::{format, format_to_ir},
    options::{BracketSpacing, ProseWrap, SingleQuote, TrailingCommas, YamlFormatOptions},
};
