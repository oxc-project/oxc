use oxc_allocator::Allocator;
use oxc_ast::ast::Expression;
use oxc_diagnostics::OxcDiagnostic;
use oxc_formatter_core::{
    Buffer, Document, EmbeddedIr, Format, FormatContext, FormatSession, FormatState, Formatted,
    VecBuffer,
    builders::{hard_line_break, text},
    write,
};
use oxc_span::GetSpan;

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
    let (has_bom, source_text) = oxc_formatter_core::spec::split_bom(source_text);
    let parsed = parse_json(allocator, source_text, options.variant)?;

    let context = JsonFormatContext::new(
        options,
        parsed.wrapped_source,
        parsed.comments,
        parsed.source_offset,
    );
    let mut state = FormatState::new(context, allocator);
    // Pre-allocate: measured on 1,447 real-world files (vscode, saleor, bootstrap),
    // 0.3x source bytes plus a 1024-element floor for tiny-file spikes (`{}` is 3 elements)
    // avoids reallocation for 99.5% of the corpus.
    let capacity = (source_text.len() * 3 / 10).max(1024);
    let mut buffer = VecBuffer::with_capacity(capacity, &mut state);

    write!(&mut buffer, FormatJsonRoot { expression: parsed.expression, has_bom });

    let elements = buffer.into_vec();
    let context = state.into_context();

    if let Some(err) = context.take_error() {
        return Err(err);
    }

    let document = Document::new(elements, Vec::new());

    Ok(Formatted::new(document, context))
}

/// Parse `source_text` and build the formatter IR for embedding into another
/// formatter's document (dispatcher path, e.g. a fenced block in JSDoc/markdown).
///
/// Unlike [`format()`], this:
/// - allocates from the session's shared arena and `GroupId` space,
///   so the IR lives as long as the parent's document
/// - emits neither a BOM nor the trailing newline (the parent owns the surrounding layout)
///
/// # Errors
/// Same as [`format()`]: any parse error bails out.
pub fn format_to_ir<'a>(
    session: &FormatSession<'a>,
    source_text: &str,
    options: JsonFormatOptions,
) -> Result<EmbeddedIr<'a>, OxcDiagnostic> {
    let allocator = session.allocator();
    let parsed = parse_json(allocator, source_text, options.variant)?;

    let context = JsonFormatContext::new(
        options,
        parsed.wrapped_source,
        parsed.comments,
        parsed.source_offset,
    );
    let mut state = FormatState::new_with_session(context, session.clone());
    let mut buffer = VecBuffer::new(&mut state);

    write!(&mut buffer, FormatJsonEmbedded { expression: parsed.expression });

    let elements = buffer.into_vec();
    let context = state.into_context();

    if let Some(err) = context.take_error() {
        return Err(err);
    }

    // JSON never collects Tailwind classes
    Ok(EmbeddedIr { ir: elements, tailwind_classes: Vec::new() })
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

        write_json_content(self.expression, f);

        // POSIX convention: every formatted file ends with a newline.
        // Prettier does the same for all parsers.
        write!(f, hard_line_break());
    }
}

/// Emits the root expression and trailing comments only;
/// no BOM, no final newline (the parent document owns the surrounding layout).
struct FormatJsonEmbedded<'a, 'b> {
    expression: Option<&'b Expression<'a>>,
}

impl<'a> Format<'a, JsonFormatContext<'a>> for FormatJsonEmbedded<'a, '_> {
    fn fmt(&self, f: &mut JsonFormatter<'_, 'a>) {
        write_json_content(self.expression, f);
    }
}

/// The shared middle of both roots:
/// the value followed by any trailing comments at the end of the document.
fn write_json_content<'a>(expression: Option<&Expression<'a>>, f: &mut JsonFormatter<'_, 'a>) {
    let trailing_anchor = if let Some(expression) = expression {
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
}
