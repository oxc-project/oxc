// NOTE:
// - `inline_always`: Intentional on hot-path closure/dispatch helpers
// - `must_use_candidate`: Left for incremental cleanup
#![allow(clippy::inline_always, clippy::must_use_candidate)]

//! Language-agnostic formatting infrastructure.
//!
//! This crate provides the core IR and printing infrastructure used by all language-specific
//! formatters in the oxc ecosystem (`oxc_formatter` for JS/TS and future formatters for CSS,
//! JSON, etc.).
//!
//! See `formatter-core-plan.md` for the migration plan from `oxc_formatter`.

mod context;
mod envelope;
pub mod format_element;
mod printer;
mod session;
mod source;
pub mod spec;
mod write;

// `builders` is the crate's public building-block namespace
// (consumers call `builders::token(..)` etc. and re-export it wholesale).
pub use write::builders;

pub use context::options::{
    CoreFormatOptions, IndentStyle, IndentWidth, IndentWidthFromIntError, LineEnding, LineWidth,
    LineWidthFromIntError,
};
pub(crate) use context::simple::SimpleFormatContext;
pub use context::{FormatContext, FormatOptions};
pub use envelope::write_front_matter;
pub use format_element::debug::DisplayDocument;
pub use format_element::document::Document;
pub use format_element::formatted::Formatted;
pub use format_element::group_id::{GroupId, UniqueGroupIdBuilder};
pub use format_element::tag::{
    self, Align, Condition, DedentMode, Group, GroupMode, Label, LabelId, Tag, TagKind,
};
pub use format_element::{
    BestFittingElement, FormatElement, FormatElements, Interned, LINE_TERMINATORS, LineMode,
    PrintMode, TextWidth, normalize_newlines,
};
pub use printer::error::{ActualStart, InvalidDocumentError, PrintError};
pub use printer::{PrintResult, PrintWidth, Printed, PrinterOptions};
// `Printer` stays crate-internal: `Document::print` / `print_with_indent` are the only print entries,
// which is what makes print-without-finalize unrepresentable outside core.
pub(crate) use printer::Printer;
pub use session::embedded::{
    DispatchPayload, DispatchRequest, DispatchResponse, EmbeddedIr, FormatDispatcher,
    TailwindCollector, dispatch_fragment_ir,
};
pub use session::{FormatSession, InputKind, SessionServices, StringEmbedder, TailwindSorter};
pub use source::{SourceText, SpanCursor};
pub use write::arguments::{Argument, Arguments};
pub use write::buffer::{
    AccumulatorBuffer, Buffer, BufferExtensions, HeapVecBuffer, Recorded, Recording,
    RemoveSoftLinesBuffer, ScratchBuffer, VecBuffer,
};
pub use write::extensions::{MemoizeFormat, Memoized};
pub use write::formatter::{Formatter, arena_cow_str};
pub use write::state::FormatState;
pub use write::{Format, write};
