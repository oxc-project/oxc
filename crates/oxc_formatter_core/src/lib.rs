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
mod embedded;
mod envelope;
pub mod format;
pub mod format_element;
mod format_extensions;
mod formatted;
mod formatter;
mod group_id;
mod macros;
mod options;
mod printer;
mod session;
mod simple_context;
mod source;
pub mod spec;
mod state;
mod traits;

pub use arguments::{Argument, Arguments};
pub use buffer::{
    AccumulatorBuffer, Buffer, BufferExtensions, HeapVecBuffer, Inspect, PreambleBuffer, Recorded,
    Recording, RemoveSoftLinesBuffer, ScratchBuffer, VecBuffer,
};
pub use diagnostics::{ActualStart, FormatError, InvalidDocumentError, PrintError};
pub use embedded::{
    DispatchPayload, DispatchRequest, DispatchResponse, EmbeddedIr, FormatDispatcher,
    TailwindCollector, dispatch_fragment_ir,
};
pub use envelope::write_front_matter;
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
pub use formatter::{Formatter, arena_cow_str};
pub use group_id::{GroupId, UniqueGroupIdBuilder};
pub use options::{
    CoreFormatOptions, IndentStyle, IndentWidth, IndentWidthFromIntError, LineEnding, LineWidth,
    LineWidthFromIntError, ParseFormatNumberError,
};
pub use printer::{PrintResult, PrintWidth, Printed, PrinterOptions};
// `Printer` stays crate-internal: `Document::print` / `print_with_indent` are the only print entries,
// which is what makes print-without-finalize unrepresentable outside core.
pub(crate) use printer::Printer;
pub use session::{FormatSession, InputKind, SessionServices, StringEmbedder, TailwindSorter};
pub(crate) use simple_context::SimpleFormatContext;
pub use source::{SourceText, SpanCursor};
pub use state::FormatState;
pub use traits::{FormatContext, FormatOptions};

/// Public return type of the formatter
pub type FormatResult<F> = Result<F, FormatError>;
