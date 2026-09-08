use oxc_allocator::Allocator;
use oxc_diagnostics::OxcDiagnostic;
use oxc_formatter_core::{
    Buffer, Document, EmbeddedIr, FormatSession, FormatState, Formatted, Formatter, VecBuffer,
    builders::{hard_line_break, text},
    write,
};
use oxc_markdown_parser::{Parser, Span, ast::Root};

use crate::{context::MarkdownFormatContext, options::MarkdownFormatOptions, print};

/// Parse `source_text` as a Markdown document and build its formatter IR.
///
/// # Errors
/// Returns an [`OxcDiagnostic`] when the parser reports a diagnostic.
/// CommonMark has no syntax errors, so markdown mode only fails on input-level limits;
/// MDX mode (not wired yet) will add real syntax errors.
pub fn format<'a>(
    allocator: &'a Allocator,
    source_text: &str,
    options: MarkdownFormatOptions,
) -> Result<Formatted<'a, MarkdownFormatContext<'a>>, OxcDiagnostic> {
    let parsed = parse_for_format(allocator, source_text)?;
    let context = MarkdownFormatContext::new(options, parsed.source, parsed.blanks);
    let mut state = FormatState::new(context, allocator);
    let mut buffer = VecBuffer::new(&mut state);
    let f = &mut Formatter::new(&mut buffer);
    if parsed.has_bom {
        write!(f, text("\u{feff}"));
    }
    // Prettier prints an empty document as the empty string, not `\n`.
    if !parsed.root.children.is_empty() {
        print::write_root(parsed.root, f);
        // POSIX convention: every formatted file ends with a newline.
        write!(f, hard_line_break());
    }
    let elements = buffer.into_vec();
    Ok(Formatted::new(Document::new(elements, Vec::new()), state.into_context()))
}

/// [`parse_for_format`] output: the AST, plus the envelope [`format()`] prints around it.
pub struct ParsedMarkdown<'a> {
    pub root: &'a Root<'a>,
    /// Every logical blank line, in source order.
    pub blanks: &'a [Span],
    /// Normalized arena source every span indexes into.
    pub source: &'a str,
    has_bom: bool,
}

/// Parse `source_text` the way the formatter does, for callers that inspect the AST
/// (e.g. a harness comparing what the source held against the formatted output).
///
/// [`format()`] goes through this too, so what a caller sees is exactly what gets formatted.
/// Owns the BOM split and `\r` normalization.
///
/// # Errors
/// Same as [`format()`].
pub fn parse_for_format<'a>(
    allocator: &'a Allocator,
    source_text: &str,
) -> Result<ParsedMarkdown<'a>, OxcDiagnostic> {
    let (has_bom, source_text) = oxc_formatter_core::spec::split_bom(source_text);
    let (root, source, blanks) = parse_root(allocator, source_text)?;
    Ok(ParsedMarkdown { root, blanks, source, has_bom })
}

/// Parse `source_text` and build the formatter IR for embedding into another
/// formatter's document (dispatcher path, e.g. a ` ```md ` fence inside a JSDoc comment).
///
/// Unlike [`format()`], this:
/// - allocates from the session's shared arena and `GroupId` space, so the IR lives as long as the parent's document
/// - emits neither a BOM nor the trailing newline
///
/// # Errors
/// Same as [`format()`].
pub fn format_to_ir<'a>(
    session: &FormatSession<'a>,
    source_text: &str,
    options: MarkdownFormatOptions,
) -> Result<EmbeddedIr<'a>, OxcDiagnostic> {
    let allocator = session.allocator();
    let (root, source, blanks) = parse_root(allocator, source_text)?;

    let context = MarkdownFormatContext::new(options, source, blanks);
    let mut state = FormatState::new_with_session(context, session.clone());
    let mut buffer = VecBuffer::new(&mut state);
    // No BOM, no final newline: the parent document owns the surrounding layout.
    print::write_root(root, &mut Formatter::new(&mut buffer));

    // Markdown never collects Tailwind classes.
    Ok(EmbeddedIr { ir: buffer.into_vec(), tailwind_classes: Vec::new() })
}

/// Parse the source into the AST, bailing out on any diagnostic.
///
/// Copies the source into the arena so every slice taken from it carries `'a`.
/// Entries own the BOM strip; this layer assumes BOM-free input (see [`oxc_formatter_core::spec::split_bom`]).
fn parse_root<'a>(
    allocator: &'a Allocator,
    source_text: &str,
) -> Result<(&'a Root<'a>, &'a str, &'a [Span]), OxcDiagnostic> {
    // Normalize line endings BEFORE parsing:
    // the printer slices verbatim text from the source in many places (html, code, liquid, math),
    // and the IR forbids `\r`.
    let source_text = oxc_formatter_core::normalize_newlines(source_text, ['\r']);
    let source: &'a str = allocator.alloc_str(&source_text);

    let ret = Parser::new(allocator, source).parse();
    if let Some(diagnostic) = ret.diagnostics.first() {
        return Err(OxcDiagnostic::error(format!("Syntax error: {diagnostic}"))
            .with_label(oxc_span::Span::new(diagnostic.span.start, diagnostic.span.end)));
    }

    let root = allocator.alloc(ret.root);
    let blanks: &'a [Span] = allocator.alloc_slice_copy(&ret.blanks);

    Ok((root, source, blanks))
}
