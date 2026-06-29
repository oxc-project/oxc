//! GraphQL formatter built on top of `oxc_formatter_core`.
//!
//! Parses with [apollo-parser](https://docs.rs/apollo-parser) (lossless CST,
//! October 2021 GraphQL spec) and prints Prettier-compatible output.
//!
//! ```ignore
//! use oxc_allocator::Allocator;
//! use oxc_formatter_graphql::{GraphqlFormatOptions, format};
//!
//! let allocator = Allocator::new();
//! let formatted = format(&allocator, "{ hello }", GraphqlFormatOptions::default()).unwrap();
//! let out = formatted.print().unwrap().into_code();
//! assert_eq!(out, "{\n  hello\n}\n");
//! ```

mod comments;
mod context;
mod format;
mod options;
mod print;

pub use crate::{
    context::GraphqlFormatContext,
    format::{format, format_to_ir},
    options::{BracketSpacing, GraphqlFormatOptions},
};
