//! Language-agnostic formatting infrastructure.
//!
//! This crate provides the core IR and printing infrastructure used by all language-specific
//! formatters in the oxc ecosystem (`oxc_formatter` for JS/TS and future formatters for CSS,
//! JSON, etc.).
//!
//! See `formatter-core-plan.md` for the migration plan from `oxc_formatter`.

mod options;

pub use options::{
    IndentStyle, IndentWidth, IndentWidthFromIntError, LineEnding, LineWidth,
    LineWidthFromIntError, ParseFormatNumberError,
};
