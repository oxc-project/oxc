// NOTE:
// - `inline_always`: Intentional on hot-path closure/dispatch helpers
// - `must_use_candidate`: Left for incremental cleanup
// - `broken_intra_doc_links`: Inherited from biome docs;
//   - Many stale references to types that no longer exist or were renamed during the oxc port.
#![allow(clippy::inline_always, clippy::must_use_candidate, rustdoc::broken_intra_doc_links)]

//! Language-agnostic formatting infrastructure.
//!
//! This crate provides the core IR and printing infrastructure used by all language-specific
//! formatters in the oxc ecosystem (`oxc_formatter` for JS/TS and future formatters for CSS,
//! JSON, etc.).
//!
//! See `formatter-core-plan.md` for the migration plan from `oxc_formatter`.

mod arguments;
pub mod buffer;
pub mod builders;
mod diagnostics;
pub mod format;
pub mod format_element;
mod format_extensions;
mod formatted;
mod formatter;
mod group_id;
mod macros;
mod options;
pub mod printer;
mod simple_context;
mod state;
mod text_range;
mod traits;

pub use arguments::{Argument, Arguments};
pub use buffer::{
    Buffer, BufferExtensions, Inspect, PreambleBuffer, Recorded, Recording, RemoveSoftLinesBuffer,
    VecBuffer,
};
pub use diagnostics::{ActualStart, FormatError, InvalidDocumentError, PrintError};
pub use format::{Format, write};
pub use format_element::debug::DisplayDocument;
pub use format_element::document::Document;
pub use format_element::tag::{
    self, Align, Condition, DedentMode, Group, GroupMode, Label, LabelId, Tag, TagKind,
};
pub use format_element::{
    BestFittingElement, FormatElement, FormatElements, Interned, LINE_TERMINATORS, LineMode,
    PrintMode, TextWidth, normalize_newlines,
};
pub use format_extensions::{MemoizeFormat, Memoized};
pub use formatted::Formatted;
pub use formatter::Formatter;
pub use group_id::{GroupId, UniqueGroupIdBuilder};
pub use options::{
    IndentStyle, IndentWidth, IndentWidthFromIntError, LineEnding, LineWidth,
    LineWidthFromIntError, ParseFormatNumberError,
};
pub use printer::{PrintResult, PrintWidth, Printed, Printer, PrinterOptions};
pub use simple_context::{SimpleFormatContext, SimpleFormatOptions};
pub use state::FormatState;
pub use text_range::TextRange;
pub use traits::{FormatContext, FormatOptions};

/// Public return type of the formatter
pub type FormatResult<F> = Result<F, FormatError>;
