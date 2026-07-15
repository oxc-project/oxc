use oxc_allocator::Allocator;
use oxc_ast::ast::Expression;
use oxc_diagnostics::OxcDiagnostic;
use oxc_formatter_core::{
    Buffer, Document, Format, FormatContext, FormatState, Formatted, HeapVecBuffer,
    builders::{hard_line_break, text},
    write,
};
use oxc_span::GetSpan;
use oxc_syntax::identifier::ZWNBSP;

use crate::{
    comments::write_trailing_inside_comments,
    context::JsonFormatContext,
    options::{JsonFormatOptions, JsonVariant},
    parse::parse_json,
    print::{FmtJsonStringifyValue, FmtJsonValue, JsonFormatter},
};

/// Parse `source_text` as JSON and build its formatter IR.
///
/// # Errors
/// Returns an [`OxcDiagnostic`] when the parser rejects `source_text`.
pub fn format<'a>(
    allocator: &'a Allocator,
    source_text: &str,
    options: JsonFormatOptions,
) -> Result<Formatted<'a, JsonFormatContext<'a>>, OxcDiagnostic> {
    let parsed = parse_json(allocator, source_text, options.variant)?;

    let context = JsonFormatContext::new(
        options,
        parsed.wrapped_source,
        parsed.comments,
        parsed.source_offset,
    );
    let mut state = FormatState::new(context, allocator);
    // Stage the document on the heap; the arena gets one exactly-sized copy at the end
    // (see `HeapVecBuffer`).
    let mut buffer = HeapVecBuffer::new(&mut state);

    // BOM detection runs on the original `source_text`; `wrapped_source` may prepend `(`.
    let has_bom = source_text.starts_with(ZWNBSP);
    write!(&mut buffer, FormatJsonRoot { expression: parsed.expression, has_bom });

    let elements = buffer.take_into_arena_vec();
    drop(buffer);
    let context = state.into_context();

    if let Some(err) = context.take_error() {
        return Err(err);
    }

    let document = Document::new(elements, Vec::new());
    document.propagate_expand();

    Ok(Formatted::new(document, context))
}

// ---

/// Emits the root expression (when present) followed by any trailing comments
/// at the end of the document.
struct FormatJsonRoot<'a, 'b> {
    expression: Option<&'b Expression<'a>>,
    has_bom: bool,
}

impl<'a> Format<'a, JsonFormatContext<'a>> for FormatJsonRoot<'a, '_> {
    fn fmt(&self, f: &mut JsonFormatter<'_, 'a>) {
        if self.has_bom {
            write!(f, text("\u{feff}"));
        }

        let trailing_anchor = if let Some(expression) = self.expression {
            if f.context().options().variant == JsonVariant::JsonStringify {
                FmtJsonStringifyValue { expression }.fmt(f);
            } else {
                FmtJsonValue { expression }.fmt(f);
            }
            expression.span().end
        } else {
            // Comments-only source: emit pending comments from the start of the source
            0
        };
        let trailing = f.context().comments().take_remaining();
        write_trailing_inside_comments(trailing, trailing_anchor, f);

        // POSIX convention: every formatted file ends with a newline.
        // Prettier does the same for all parsers.
        write!(f, hard_line_break());
    }
}
