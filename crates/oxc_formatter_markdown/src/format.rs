use oxc_allocator::Allocator;
use oxc_diagnostics::OxcDiagnostic;
use oxc_formatter_core::{
    Buffer, Document, EmbeddedIr, Format, FormatSession, FormatState, Formatted, VecBuffer,
    builders::{empty_line, hard_line_break, text},
    spec::{FrontMatter, blank_front_matter, parse_front_matter},
    write, write_front_matter,
};
use oxc_markdown_parser::{Parser, Span, ast::Root};

use crate::{
    context::MarkdownFormatContext,
    options::MarkdownFormatOptions,
    print::{self, MarkdownFormatter},
};

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
    // TODO: Pre-allocate
    let mut buffer = VecBuffer::new(&mut state);

    write!(&mut buffer, FormatMarkdownRoot { parsed: &parsed });

    let elements = buffer.into_vec();
    let context = state.into_context();

    let ir = Document::new(elements, Vec::new());

    Ok(Formatted::new(ir, context))
}

/// [`parse_for_format`] output: the AST, plus the envelope [`format()`] prints around it.
pub struct ParsedMarkdown<'a> {
    pub root: &'a Root<'a>,
    /// Every logical blank line, in source order.
    pub blanks: &'a [Span],
    /// Normalized arena source every span indexes into, the front matter blanked
    /// (what the parser saw; a verbatim slice overlapping it prints the blanks, as Prettier's does).
    pub source: &'a str,
    /// The leading `---` / `+++` block (it is not markdown), sliced from the unblanked source.
    pub front_matter: Option<FrontMatter<'a>>,
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
    parse_root(allocator, source_text, has_bom)
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
    // `FormatSession::dispatch` never hands a BOM-headed input to an embedded part
    let parsed = parse_root(allocator, source_text, false)?;
    if parsed.front_matter.is_some() && !session.input_kind().owns_front_matter() {
        // A fragment (a JSDoc fence) never acquires file envelope semantics:
        // refuse the whole child instead of partially treating its head as front matter.
        return Err(OxcDiagnostic::error(
            "Front matter in a Markdown fragment; the part is preserved as-is",
        ));
    }

    let context = MarkdownFormatContext::new(options, parsed.source, parsed.blanks);
    let mut state = FormatState::new_with_session(context, session.clone());
    let mut buffer = VecBuffer::new(&mut state);

    write!(&mut buffer, FormatMarkdownEmbedded { parsed: &parsed });

    // No child of Markdown collects Tailwind classes yet (see `TailwindCollector` in `context.rs`)
    Ok(EmbeddedIr { ir: buffer.into_vec(), tailwind_classes: Vec::new() })
}

/// Parse the source into the AST, bailing out on any diagnostic.
///
/// Copies the source into the arena so every slice taken from it carries `'a`.
/// Entries own the BOM strip; this layer assumes BOM-free input (see [`oxc_formatter_core::spec::split_bom`]).
fn parse_root<'a>(
    allocator: &'a Allocator,
    source_text: &str,
    has_bom: bool,
) -> Result<ParsedMarkdown<'a>, OxcDiagnostic> {
    // Normalize line endings BEFORE parsing:
    // the printer slices verbatim text from the source in many places (html, code, liquid, math),
    // and the IR forbids `\r`.
    let source_text = oxc_formatter_core::normalize_newlines(source_text, ['\r']);
    let unblanked: &'a str = allocator.alloc_str(&source_text);

    // Front matter is not markdown (`---` would be a thematic break, its body a setext heading):
    // blanked, byte for byte, for the parser and the printer alike, and printed from `fm.raw` by `write_document`.
    // Blanked rather than cut off because the body may start on the closing line (`---<div>`),
    // where its column is syntax (an HTML block's indent, indented code),
    // and because every span, `blanks` and `fm.raw` then share one offset space.
    // The printer slices the blanked copy too, so that such a body line prints as the parser read it (`   <div>`),
    // not as the delimiter again.
    let front_matter = parse_front_matter(unblanked);
    let source: &'a str = match &front_matter {
        Some(fm) => allocator.alloc_str(&blank_front_matter(unblanked, fm.raw.len())),
        None => unblanked,
    };

    let ret = Parser::new(allocator, source).parse();
    if let Some(diagnostic) = ret.diagnostics.first() {
        return Err(OxcDiagnostic::error(format!("Syntax error: {diagnostic}"))
            .with_label(oxc_span::Span::new(diagnostic.span.start, diagnostic.span.end)));
    }

    let root = allocator.alloc(ret.root);
    let blanks: &'a [Span] = allocator.alloc_slice_copy(&ret.blanks);

    Ok(ParsedMarkdown { root, blanks, source, front_matter, has_bom })
}

/// Emits the front matter and the blocks with the final newline.
struct FormatMarkdownRoot<'b, 'a> {
    parsed: &'b ParsedMarkdown<'a>,
}

impl<'a> Format<'a, MarkdownFormatContext<'a>> for FormatMarkdownRoot<'_, 'a> {
    fn fmt(&self, f: &mut MarkdownFormatter<'_, 'a>) {
        if self.parsed.has_bom {
            write!(f, text("\u{feff}"));
        }

        write_document(self.parsed, f);

        // Prettier prints an empty document as the empty string, not `\n`.
        // POSIX convention otherwise: every formatted file ends with a newline.
        if self.parsed.front_matter.is_some() || !self.parsed.root.children.is_empty() {
            write!(f, hard_line_break());
        }
    }
}

/// Emits the front matter and the blocks only;
/// no BOM, no final newline (the parent document owns the surrounding layout).
struct FormatMarkdownEmbedded<'b, 'a> {
    parsed: &'b ParsedMarkdown<'a>,
}

impl<'a> Format<'a, MarkdownFormatContext<'a>> for FormatMarkdownEmbedded<'_, 'a> {
    fn fmt(&self, f: &mut MarkdownFormatter<'_, 'a>) {
        write_document(self.parsed, f);
    }
}

/// The front matter (a blank line after it when a body follows) and the blocks.
fn write_document<'a>(parsed: &ParsedMarkdown<'a>, f: &mut MarkdownFormatter<'_, 'a>) {
    if let Some(fm) = &parsed.front_matter {
        write_front_matter(fm, &["yaml", "toml"], f);
        if !parsed.root.children.is_empty() {
            write!(f, empty_line());
        }
    }
    print::write_root(parsed.root, f);
}
