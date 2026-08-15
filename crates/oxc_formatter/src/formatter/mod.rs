//! Infrastructure for code formatting
//!
//! This module defines [FormatElement], an IR to format code documents and provides a mean to print
//! such a document to a string. Objects that know how to format themselves implement the [Format] trait.
//!
//! ## Formatting Traits
//!
//! * [Format]: Implemented by objects that can be formatted.
//!
//! ## Formatting Macros
//!
//! This crate defines two macros to construct the IR. These are inspired by Rust's `fmt` macros
//! * [`format!`]: Formats a formattable object
//! * [`format_args!`]: Concatenates a sequence of Format objects.
//! * [`write!`]: Writes a sequence of formattable objects into an output buffer.

// FIXME
#![allow(rustdoc::broken_intra_doc_links)]

mod builders;
pub mod comments;
mod context;
pub mod formatter_js;
pub mod jsdoc;
pub mod prelude;
pub mod separated;
pub mod token;
pub mod trivia;

pub use self::builders::JoinBuilderJsExt;
pub use self::comments::Comments;
pub use self::{
    context::{JsFormatContext, TailwindContextEntry},
    formatter_js::{JsFormatter, JsFormatterExt},
};
use oxc_formatter_core::{
    Arguments, Buffer as _, Document, FormatSession, FormatState, Formatted, VecBuffer,
};

/// The `format` function takes an [`Arguments`] struct and returns the resulting formatting IR.
///
/// The [`Arguments`] instance can be created with the [`format_args!`].
pub fn format<'ast>(
    context: JsFormatContext<'ast>,
    session: &FormatSession<'ast>,
    arguments: Arguments<'_, 'ast, JsFormatContext<'ast>>,
) -> Formatted<'ast, JsFormatContext<'ast>> {
    // Pre-allocate buffer at 40% of source length (source_len * 2 / 5).
    // Analysis of 4,891 VSCode files shows FormatElement buffer length is typically 19% of source (median),
    // with 95th percentile at 30-38% across all file sizes. This 0.4x multiplier avoids reallocation for 95%+ of files.
    let capacity = (context.source_text().len() * 2) / 5;

    let mut state = FormatState::new_with_session(context, session.clone());
    let mut buffer = VecBuffer::with_capacity(capacity, &mut state);

    buffer.write_fmt(arguments);

    let elements = buffer.into_vec();
    let mut context = state.into_context();

    let tailwind_classes = context.take_tailwind_classes();
    let sorted_tailwind_classes = session.sort_tailwind_classes(tailwind_classes);

    let ir = Document::new(elements, sorted_tailwind_classes);

    Formatted::new(ir, context)
}
