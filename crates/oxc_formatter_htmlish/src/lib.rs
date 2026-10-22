//! HTML (and HTML-based template language) formatter built on top of `oxc_formatter_core`.
//!
//! Parses with `oxc-htmlish-parser` and prints Prettier-compatible output.

mod context;
mod format;
mod options;
mod print;
mod rules;
mod tree;

pub use crate::{
    context::HtmlishFormatContext,
    format::{
        HtmlishEmbeddedIn, ParsedHtmlish, format, format_to_ir, format_with_session,
        parse_for_format,
    },
    options::{HtmlishFormatOptions, HtmlishVariant, WhitespaceSensitivity},
    print::{CssInHtmlStyleAttribute, JsInHtmlEventHandler},
};
