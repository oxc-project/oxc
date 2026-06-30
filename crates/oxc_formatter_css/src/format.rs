use oxc_css_parser::{ParserBuilder, ParserOptions, TemplatePlaceholder, ast::Stylesheet};

use oxc_allocator::{Allocator, ArenaVec};
use oxc_diagnostics::OxcDiagnostic;
use oxc_formatter_core::{
    Buffer, Document, EmbeddedContext, EmbeddedIr, Format, FormatState, Formatted, VecBuffer,
    builders::{empty_line, hard_line_break, text},
    write,
};
use oxc_span::Span;

use crate::{
    TEMPLATE_PLACEHOLDER_PREFIX, TEMPLATE_PLACEHOLDER_SUFFIX,
    comments::CssComment,
    context::CssFormatContext,
    options::CssFormatOptions,
    print::{self, CssFormatter},
};

/// Host-supplied batch sorter for `@apply` Tailwind classes
/// (one pre-sort string in, one sorted string out, index-aligned).
///
/// The sorter owns: ordering, dedup, whitespace collapse,
/// and skipping `{{...}}` template interpolations (Vue/Angular templates).
pub type TailwindSorter<'s> = &'s dyn Fn(Vec<String>) -> Vec<String>;

/// Parse `source_text` as a stylesheet and build its formatter IR.
///
/// `sort_tailwind_classes` sorts the `@apply` classes collected when
/// [`CssFormatOptions::sort_tailwindcss`] is on; `None` (or an unset option)
/// prints them as-is.
///
/// # Errors
/// Returns an [`OxcDiagnostic`] when the parse produces any error, including recoverable ones.
/// `oxc-css-parser` can recover from some syntax errors, but a tree with errors cannot be formatted faithfully,
/// so a single error is enough to bail out.
pub fn format<'a>(
    allocator: &'a Allocator,
    source_text: &str,
    options: CssFormatOptions,
    sort_tailwind_classes: Option<TailwindSorter<'_>>,
) -> Result<Formatted<'a, CssFormatContext<'a>>, OxcDiagnostic> {
    let has_bom = source_text.starts_with('\u{feff}');

    let (stylesheet, source, comments) =
        parse_stylesheet(allocator, source_text, options, /* tolerate_placeholders */ false)?;
    let front_matter = parse_front_matter(source);

    let context =
        CssFormatContext::new(options, source, comments, /* template_placeholders */ false);
    let mut state = FormatState::new(context, allocator);
    // TODO: Use `with_capacity` for perf, like `oxc_formatter` does
    let mut buffer = VecBuffer::new(&mut state);

    write!(&mut buffer, FormatCssRoot { stylesheet: &stylesheet, has_bom, front_matter });

    let elements = buffer.into_vec();
    let mut context = state.into_context();

    let tailwind_classes = context.take_tailwind_classes();
    let sorted_tailwind_classes = match sort_tailwind_classes {
        Some(sorter) if !tailwind_classes.is_empty() => sorter(tailwind_classes),
        _ => tailwind_classes,
    };

    let ir = Document::new(elements, sorted_tailwind_classes);
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
/// The returned [`EmbeddedIr`] also carries the pre-sort `@apply` Tailwind
/// classes the IR's `TailwindClass(index)` elements refer to (empty unless
/// [`CssFormatOptions::sort_tailwindcss`] is on).
/// The parent document owns the batch sort,
/// so the caller must re-index the elements into the parent's class space
/// (`DispatchResult::remap_tailwind_into`).
///
/// # Errors
/// Same as [`format()`].
pub fn format_to_ir<'a>(
    ctx: &EmbeddedContext<'a, '_>,
    source_text: &str,
    options: CssFormatOptions,
) -> Result<EmbeddedIr<'a>, OxcDiagnostic> {
    let allocator = ctx.allocator;
    // css-in-js: The dispatcher input substitutes `${}` interpolations
    // with `` `PLACEHOLDER-N` `` markers, which may sit in value or selector position.
    let allow_placeholders = true;
    let (stylesheet, source, comments) = parse_stylesheet(
        allocator,
        source_text,
        options,
        /* tolerate_placeholders */ allow_placeholders,
    )?;

    let context = CssFormatContext::new(
        options,
        source,
        comments,
        /* template_placeholders */ allow_placeholders,
    );
    let mut state = FormatState::new(context, allocator);
    let mut buffer = VecBuffer::new(&mut state);

    write!(&mut buffer, FormatCssEmbedded { stylesheet: &stylesheet });

    let elements = buffer.into_vec();
    let tailwind_classes = state.context_mut().take_tailwind_classes();

    Ok(EmbeddedIr { ir: elements, tailwind_classes })
}

/// Parse the source into an AST and collect comments, bailing out on any error.
///
/// Copies the source into the arena (minus any BOM, which `oxc-css-parser` would skip anyway)
/// so every slice taken from it carries `'a`.
fn parse_stylesheet<'a>(
    allocator: &'a Allocator,
    source_text: &str,
    options: CssFormatOptions,
    tolerate_placeholders: bool,
) -> Result<(Stylesheet<'a>, &'a str, &'a [CssComment]), OxcDiagnostic> {
    let source_text = source_text.strip_prefix('\u{feff}').unwrap_or(source_text);
    // NOTE: Normalize line endings BEFORE parsing like Prettier, unlike other `oxc_formatter_xxx`.
    // For CSS formatter, the printer slices verbatim text from the source in many places.
    // (comments, progid, custom properties, ...etc)
    // And a raw `\r` reaching the core `text()` builder panics.
    // Spans stay consistent because parse and print both use the normalized copy.
    let source_text = oxc_formatter_core::normalize_newlines(source_text, ['\r']);
    let source: &'a str = allocator.alloc_str(&source_text);

    // Front matter is not CSS:
    // blank it out (preserving line structure so spans and gaps stay aligned)
    // and print it verbatim from `source`.
    let parse_source: &'a str = match parse_front_matter(source) {
        Some(fm) => {
            let end = fm.raw.len();
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
        .syntax(options.variant.to_css_syntax())
        .options(ParserOptions {
            // Derive the affix from the host sentinel (single source of truth),
            // minus the leading backtick which oxc-css-parser consumes as the placeholder sigil
            // (the closing backtick `TEMPLATE_PLACEHOLDER_SUFFIX` is fixed in oxc-css-parser).
            // Only valid for SCSS; oxc-css-parser asserts that.
            template_placeholder: tolerate_placeholders.then_some(TemplatePlaceholder {
                prefix: TEMPLATE_PLACEHOLDER_PREFIX
                    .strip_prefix(TEMPLATE_PLACEHOLDER_SUFFIX)
                    .expect("placeholder prefix starts with a backtick"),
            }),
            try_parsing_value_in_custom_property: true,
            ..ParserOptions::default()
        })
        .comments(&mut comments)
        .build();

    let stylesheet = parser.parse::<Stylesheet>().map_err(|error| to_diagnostic(&error))?;
    // Top-level declarations are valid only as an embedded css-in-js fragment.
    // A standalone file rejects them like any other recoverable error
    // (they are not valid CSS/SCSS/Less); only the embedded path tolerates them.
    if let Some(error) = parser.recoverable_errors().iter().find(|error| {
        !(tolerate_placeholders
            && matches!(error.kind, oxc_css_parser::error::ErrorKind::TopLevelDeclaration))
    }) {
        return Err(to_diagnostic(error));
    }
    drop(parser);

    let comments: &'a [CssComment] = ArenaVec::from_iter_in(
        comments.iter().map(|c| CssComment {
            span: to_span(&c.span),
            inline: matches!(c.kind, oxc_css_parser::token::CommentKind::Line),
        }),
        &allocator,
    )
    .into_arena_slice();

    Ok((stylesheet, source, comments))
}

fn to_diagnostic(error: &oxc_css_parser::error::Error) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("Syntax error: {}", error.kind)).with_label(to_span(&error.span))
}

pub fn to_span(span: &oxc_css_parser::Span) -> Span {
    Span::new(
        u32::try_from(span.start).unwrap_or(u32::MAX),
        u32::try_from(span.end).unwrap_or(u32::MAX),
    )
}

/// Prettier-style front matter
/// (`---` / `+++` fence starting at offset 0, closed by the same fence on its own line).
///
/// The opening fence may carry an explicit language tag (`---yaml`, `---mycustomparser`)
/// captured in [`Self::language`] (empty if absent).
struct FrontMatter<'a> {
    /// The full verbatim slice from offset 0 to the end of the closing fence.
    raw: &'a str,
    /// The opening delimiter literal — `"---"` or `"+++"`.
    delim: &'static str,
    /// Explicit language tag from the opening fence's first line, trimmed.
    /// Empty when the fence is a bare `---` / `+++`.
    language: &'a str,
}

/// Detects [`FrontMatter`] at the start of `source`
/// (single parse, shared by the parse-side blanking and the print-side dispatch).
fn parse_front_matter(source: &str) -> Option<FrontMatter<'_>> {
    let delim = if source.starts_with("---") {
        "---"
    } else if source.starts_with("+++") {
        "+++"
    } else {
        return None;
    };

    let first_line_end = source.find('\n')?;
    let language = source[delim.len()..first_line_end].trim();
    let mut line_start = first_line_end + 1;
    while line_start < source.len() {
        let line_end = source[line_start..].find('\n').map_or(source.len(), |i| line_start + i);
        let line = source[line_start..line_end].trim_end_matches('\r');
        if line.trim_end() == delim {
            let raw_end = line_start + line.trim_end().len();
            return Some(FrontMatter { raw: &source[..raw_end], delim, language });
        }
        line_start = line_end + 1;
    }
    None
}

/// Best-effort YAML front-matter normalization (Prettier reformats it with its YAML printer):
/// `key: value` spacing and 2-space nesting indents.
/// Returns `None` (verbatim) for any construct beyond plain mappings,
/// sequence items and comments, e.g. block scalars, quoted keys.
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
    front_matter: Option<FrontMatter<'a>>,
}

impl<'a> Format<'a, CssFormatContext<'a>> for FormatCssRoot<'_, 'a> {
    fn fmt(&self, f: &mut CssFormatter<'_, 'a>) {
        if self.has_bom {
            write!(f, text("\u{feff}"));
        }

        let has_content =
            !self.stylesheet.statements.is_empty() || f.context().comments().peek().is_some();
        if let Some(fm) = &self.front_matter {
            // YAML normalization runs only for the default `---` shape (no explicit tag);
            // `+++` (TOML) and explicit non-default languages stay verbatim.
            // Prettier delegates them to dedicated language printers we don't bridge for now.
            let normalized = (fm.delim == "---" && fm.language.is_empty())
                .then(|| try_format_yaml_front_matter(fm.raw))
                .flatten();
            if let Some(s) = normalized {
                write!(f, text(f.allocator().alloc_str(&s)));
            } else {
                write!(f, text(fm.raw));
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
