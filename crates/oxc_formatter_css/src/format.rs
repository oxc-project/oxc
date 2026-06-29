use oxc_allocator::{Allocator, ArenaVec};
use oxc_diagnostics::OxcDiagnostic;
use oxc_formatter_core::{
    Buffer, Document, EmbeddedContext, Format, FormatElement, FormatState, Formatted, VecBuffer,
    builders::{empty_line, hard_line_break, text},
    write,
};
use oxc_span::Span;
use raffia::{ParserBuilder, ast::Stylesheet};

use crate::{
    comments::CssComment,
    context::CssFormatContext,
    options::CssFormatOptions,
    print::{self, CssFormatter},
};

/// Parse `source_text` as a stylesheet and build its formatter IR.
///
/// # Errors
/// Returns an [`OxcDiagnostic`] when the parse produces any error, including
/// recoverable ones. raffia can recover from some syntax errors, but a tree
/// with errors cannot be formatted faithfully, so a single error is enough to
/// bail out. The caller (oxfmt) decides what to do next
/// (report, or fall back to Prettier).
pub fn format<'a>(
    allocator: &'a Allocator,
    source_text: &str,
    options: CssFormatOptions,
) -> Result<Formatted<'a, CssFormatContext<'a>>, OxcDiagnostic> {
    let has_bom = source_text.starts_with('\u{feff}');
    let (stylesheet, source, comments) = parse_stylesheet(allocator, source_text, options)?;
    let front_matter = front_matter_end(source).map(|end| &source[..end]);

    let context = CssFormatContext::new(options, source, comments);
    let mut state = FormatState::new(context, allocator);
    let mut buffer = VecBuffer::new(&mut state);

    write!(&mut buffer, FormatCssRoot { stylesheet: &stylesheet, has_bom, front_matter });

    let elements = buffer.into_vec();
    let context = state.into_context();

    let ir = Document::new(elements, Vec::new());
    ir.propagate_expand();

    Ok(Formatted::new(ir, context))
}

/// Parse `source_text` and build the formatter IR for embedding into another
/// formatter's document (dispatcher path, e.g. css-in-js).
///
/// Unlike [`format()`], this:
/// - allocates from the shared arena in `ctx`
/// - emits neither a BOM nor the trailing newline
/// - skips `propagate_expand()`, which the parent runs on the merged document
///
/// # Errors
/// Same as [`format()`].
pub fn format_to_ir<'a>(
    ctx: &EmbeddedContext<'a, '_>,
    source_text: &str,
    options: CssFormatOptions,
) -> Result<ArenaVec<'a, FormatElement<'a>>, OxcDiagnostic> {
    let allocator = ctx.allocator;
    let (stylesheet, source, comments) = parse_stylesheet(allocator, source_text, options)?;

    let context = CssFormatContext::new(options, source, comments);
    let mut state = FormatState::new(context, allocator);
    let mut buffer = VecBuffer::new(&mut state);

    write!(&mut buffer, FormatCssEmbedded { stylesheet: &stylesheet });

    Ok(buffer.into_vec())
}

/// Parse the source into an AST and collect comments, bailing out on any error.
///
/// Copies the source into the arena (minus any BOM, which raffia would skip
/// anyway) so every slice taken from it carries `'a`.
fn parse_stylesheet<'a>(
    allocator: &'a Allocator,
    source_text: &str,
    options: CssFormatOptions,
) -> Result<(Stylesheet<'a>, &'a str, &'a [CssComment]), OxcDiagnostic> {
    let source_text = source_text.strip_prefix('\u{feff}').unwrap_or(source_text);
    let source: &'a str = allocator.alloc_str(source_text);

    // Front matter is not CSS: blank it out (preserving line structure so
    // spans and gaps stay aligned) and print it verbatim from `source`.
    let parse_source: &'a str = match front_matter_end(source) {
        Some(end) => {
            let mut blanked = String::with_capacity(source.len());
            for c in source[..end].chars() {
                blanked.push(if c == '\n' || c == '\r' { c } else { ' ' });
            }
            blanked.push_str(&source[end..]);
            allocator.alloc_str(&blanked)
        }
        None => source,
    };

    let mut comments = vec![];
    let mut parser = ParserBuilder::new(parse_source)
        .syntax(options.variant.to_raffia())
        .comments(&mut comments)
        .build();

    let stylesheet = parser.parse::<Stylesheet>().map_err(|error| to_diagnostic(&error))?;
    if let Some(error) = parser.recoverable_errors().first() {
        return Err(to_diagnostic(error));
    }
    drop(parser);

    let comments: &'a [CssComment] = ArenaVec::from_iter_in(
        comments.iter().map(|c| CssComment {
            span: to_span(&c.span),
            inline: matches!(c.kind, raffia::token::CommentKind::Line),
        }),
        &allocator,
    )
    .into_arena_slice();

    Ok((stylesheet, source, comments))
}

fn to_diagnostic(error: &raffia::error::Error) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("Syntax error: {}", error.kind)).with_label(to_span(&error.span))
}

pub fn to_span(span: &raffia::Span) -> Span {
    Span::new(
        u32::try_from(span.start).unwrap_or(u32::MAX),
        u32::try_from(span.end).unwrap_or(u32::MAX),
    )
}

/// Detects Prettier-style front matter (`---` / `+++` fence starting at
/// offset 0, closed by the same fence on its own line) and returns the end
/// offset of the closing fence.
fn front_matter_end(source: &str) -> Option<usize> {
    let delim = if source.starts_with("---") {
        "---"
    } else if source.starts_with("+++") {
        "+++"
    } else {
        return None;
    };
    let first_line_end = source.find('\n')?;
    if !source[delim.len()..first_line_end].trim().is_empty() {
        return None;
    }
    let mut line_start = first_line_end + 1;
    while line_start < source.len() {
        let line_end = source[line_start..].find('\n').map_or(source.len(), |i| line_start + i);
        let line = source[line_start..line_end].trim_end_matches('\r');
        if line.trim_end() == delim {
            return Some(line_start + line.trim_end().len());
        }
        line_start = line_end + 1;
    }
    None
}

/// Best-effort YAML front-matter normalization (Prettier reformats it with
/// its YAML printer): `key: value` spacing and 2-space nesting indents.
/// Returns `None` (→ verbatim) for any construct beyond plain mappings,
/// sequence items and comments — e.g. block scalars, quoted keys.
fn try_format_yaml_front_matter(front_matter: &str) -> Option<String> {
    let inner = front_matter.strip_prefix("---")?.strip_suffix("---")?;
    let mut out = String::with_capacity(front_matter.len());
    out.push_str("---\n");
    // Source indents of mapping keys awaiting nested content.
    let mut stack: Vec<usize> = vec![];
    for line in inner.lines().skip(1) {
        let content = line.trim();
        if content.is_empty() {
            continue;
        }
        let indent = line.len() - line.trim_start().len();
        while stack.last().is_some_and(|&top| indent <= top) {
            stack.pop();
        }
        let level = stack.len();
        for _ in 0..level {
            out.push_str("  ");
        }
        if let Some(rest) = content.strip_prefix("- ") {
            out.push_str("- ");
            out.push_str(rest.trim());
        } else if content.starts_with('#') {
            out.push_str(content);
        } else if let Some(colon) = content.find(':') {
            let key = &content[..colon];
            if key.is_empty() || key.contains([' ', '\t', '"', '\'', '{', '[', '#']) {
                return None;
            }
            let value = content[colon + 1..].trim();
            if value.is_empty() {
                out.push_str(key);
                out.push(':');
                stack.push(indent);
            } else {
                if value.starts_with(['|', '>', '&', '*', '{', '[']) {
                    return None;
                }
                out.push_str(key);
                out.push_str(": ");
                out.push_str(value);
            }
        } else {
            return None;
        }
        out.push('\n');
    }
    out.push_str("---");
    Some(out)
}

/// Emits the stylesheet followed by any trailing comments, and the final newline.
struct FormatCssRoot<'b, 'a> {
    stylesheet: &'b Stylesheet<'a>,
    has_bom: bool,
    front_matter: Option<&'a str>,
}

impl<'a> Format<'a, CssFormatContext<'a>> for FormatCssRoot<'_, 'a> {
    fn fmt(&self, f: &mut CssFormatter<'_, 'a>) {
        if self.has_bom {
            write!(f, text("\u{feff}"));
        }

        let has_content =
            !self.stylesheet.statements.is_empty() || f.context().comments().peek().is_some();
        if let Some(front_matter) = self.front_matter {
            // Whitespace-only front matter collapses to adjacent fences.
            let collapsed = front_matter
                .strip_prefix("---")
                .and_then(|s| s.strip_suffix("---"))
                .is_some_and(|inner| inner.trim().is_empty());
            if collapsed {
                write!(f, ["---", hard_line_break(), "---"]);
            } else if let Some(formatted) = try_format_yaml_front_matter(front_matter) {
                write!(f, text(f.allocator().alloc_str(&formatted)));
            } else {
                write!(f, text(front_matter));
            }
            if has_content {
                // A hardline plus a blank line (consecutive hardlines collapse).
                write!(f, empty_line());
            } else {
                write!(f, hard_line_break());
            }
        }

        print::write_stylesheet(self.stylesheet, f);

        // POSIX convention: every formatted file ends with a newline.
        if has_content {
            write!(f, hard_line_break());
        }
    }
}

/// Emits the stylesheet only; no BOM, no final newline.
struct FormatCssEmbedded<'b, 'a> {
    stylesheet: &'b Stylesheet<'a>,
}

impl<'a> Format<'a, CssFormatContext<'a>> for FormatCssEmbedded<'_, 'a> {
    fn fmt(&self, f: &mut CssFormatter<'_, 'a>) {
        print::write_stylesheet(self.stylesheet, f);
    }
}
