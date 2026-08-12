use oxc_css_parser::{ParserBuilder, ParserOptions, TemplatePlaceholder, ast::Stylesheet};

use oxc_allocator::{Allocator, ArenaVec};
use oxc_diagnostics::OxcDiagnostic;
use oxc_formatter_core::{
    Buffer, Document, EmbeddedIr, Format, FormatSession, FormatState, Formatted, InputKind,
    VecBuffer,
    builders::{empty_line, hard_line_break, text},
    spec::{FrontMatter, blank_front_matter, parse_front_matter},
    write,
};
use oxc_span::Span;

use crate::{
    TEMPLATE_PLACEHOLDER_PREFIX, TEMPLATE_PLACEHOLDER_SUFFIX,
    comments::CssComment,
    context::CssFormatContext,
    options::{CssFormatOptions, CssVariant},
    print::{self, CssFormatter},
};

/// Parse `source_text` as a stylesheet and build its formatter IR.
///
/// # Errors
/// Returns an [`OxcDiagnostic`] when the parse produces any error, including recoverable ones.
/// `oxc-css-parser` can recover from some syntax errors, but a tree with errors cannot be formatted faithfully,
/// so a single error is enough to bail out.
pub fn format<'a>(
    allocator: &'a Allocator,
    source_text: &str,
    options: CssFormatOptions,
) -> Result<Formatted<'a, CssFormatContext<'a>>, OxcDiagnostic> {
    // NOTE: this wrapper labels the run `PhysicalFile` with NO services:
    // front matter is detected but its body degrades to verbatim (`PreserveOriginal`),
    // and `@apply` Tailwind classes print unsorted.
    // Hosts that want them use `format_with_session` with the services installed.
    format_with_session(
        &FormatSession::new(allocator, InputKind::PhysicalFile),
        source_text,
        options,
    )
}

/// Like [`format()`], but on a caller-supplied [`FormatSession`].
///
/// The session's dispatcher formats this document's front matter body
/// (YAML through oxfmt's native registry;
/// see `write_front_matter` for the routing and every verbatim degradation),
/// and its Tailwind sorter orders the collected `@apply` classes at finalize.
///
/// # Errors
/// Same as [`format()`].
pub fn format_with_session<'a>(
    session: &FormatSession<'a>,
    source_text: &str,
    options: CssFormatOptions,
) -> Result<Formatted<'a, CssFormatContext<'a>>, OxcDiagnostic> {
    // The envelope matrix has one decision input, the session's `InputKind`:
    // this entry is the physical-root half (owns BOM + front matter);
    // every embedded kind goes through `format_to_ir`.
    debug_assert!(
        session.input_kind() == InputKind::PhysicalFile,
        "format_with_session is the physical-root entry; embedded inputs go through format_to_ir"
    );
    let allocator = session.allocator();
    let (has_bom, source_text) = oxc_formatter_core::spec::split_bom(source_text);

    // A document root owns front matter;
    // the session's dispatcher decides whether its body actually formats (`write_front_matter`).
    let PreparedSource { source, parse_source, front_matter } =
        prepare_source(allocator, source_text);
    let (stylesheet, comments) =
        parse_stylesheet(allocator, parse_source, options, /* tolerate_placeholders */ false)?;

    let context =
        CssFormatContext::new(options, source, comments, /* template_placeholders */ false);
    let mut state = FormatState::new_with_session(context, session.clone());
    // Pre-allocate: measured on 616 real-world files (bootstrap, vscode, saleor; css/scss/less),
    // 0.5x source bytes plus a 1024-element floor for tiny-file spikes avoids reallocation for 98% of the corpus.
    let capacity = (source.len() / 2).max(1024);
    let mut buffer = VecBuffer::with_capacity(capacity, &mut state);

    write!(&mut buffer, FormatCssRoot { stylesheet: &stylesheet, has_bom, front_matter });

    let elements = buffer.into_vec();
    let mut context = state.into_context();

    let tailwind_classes = context.take_tailwind_classes();
    let sorted_tailwind_classes = session.sort_tailwind_classes(tailwind_classes);

    let ir = Document::new(elements, sorted_tailwind_classes);

    Ok(Formatted::new(ir, context))
}

/// Parse `source_text` and build the formatter IR for embedding into another
/// formatter's document (dispatcher path, e.g. css-in-js).
///
/// Unlike [`format()`], this:
/// - allocates from the session's shared arena and `GroupId` space
/// - emits neither a BOM nor the trailing newline
/// - `template_placeholders` enables the css-in-js parse mode
///   (`` `PLACEHOLDER-N` `` markers + top-level declarations);
///   JSDoc-style whole-stylesheet fragments pass `false`
///
/// The returned [`EmbeddedIr`] also carries the pre-sort `@apply` Tailwind
/// classes the IR's `TailwindClass(index)` elements refer to (empty unless
/// [`CssFormatOptions::sort_tailwindcss`] is on).
/// The parent document owns the batch sort,
/// so the caller must re-index the elements into the parent's class space
/// (`DispatchPayload::into_doc`).
///
/// # Errors
/// Same as [`format()`].
pub fn format_to_ir<'a>(
    session: &FormatSession<'a>,
    source_text: &str,
    options: CssFormatOptions,
    template_placeholders: bool,
) -> Result<EmbeddedIr<'a>, OxcDiagnostic> {
    let allocator = session.allocator();
    let input_kind = session.input_kind();

    let prepared = prepare_source(allocator, source_text);
    if prepared.front_matter.is_some() && !input_kind.owns_front_matter() {
        // A fragment (css-in-js, JSDoc fence) never acquires file envelope semantics:
        // refuse the whole child instead of partially treating its head as front matter.
        return Err(OxcDiagnostic::error(
            "Front matter in a CSS fragment; the part is preserved as-is",
        ));
    }
    let (stylesheet, comments) = parse_stylesheet(
        allocator,
        prepared.parse_source,
        options,
        /* tolerate_placeholders */ template_placeholders,
    )?;

    let context = CssFormatContext::new(options, prepared.source, comments, template_placeholders);
    let mut state = FormatState::new_with_session(context, session.clone());
    let mut buffer = VecBuffer::new(&mut state);

    write!(
        &mut buffer,
        FormatCssEmbedded { stylesheet: &stylesheet, front_matter: prepared.front_matter }
    );

    let elements = buffer.into_vec();
    let tailwind_classes = state.context_mut().take_tailwind_classes();

    Ok(EmbeddedIr { ir: elements, tailwind_classes })
}

/// Normalized arena source, its front matter (when present),
/// and the copy the CSS parser actually sees
/// (front matter blanked byte-preservingly so every span, comment, and source-gap scan aligns with `source`).
struct PreparedSource<'a> {
    source: &'a str,
    parse_source: &'a str,
    front_matter: Option<FrontMatter<'a>>,
}

fn prepare_source<'a>(allocator: &'a Allocator, source_text: &str) -> PreparedSource<'a> {
    // NOTE: Normalize line endings BEFORE parsing like Prettier, unlike other `oxc_formatter_xxx`.
    // For CSS formatter, the printer slices verbatim text from the source in many places.
    // (comments, progid, custom properties, ...etc)
    // And a raw `\r` reaching the core `text()` builder panics.
    // Spans stay consistent because parse and print both use the normalized copy.
    let normalized = oxc_formatter_core::normalize_newlines(source_text, ['\r']);
    let source: &'a str = allocator.alloc_str(&normalized);

    // Front matter is not CSS:
    // blank it out for the parser and keep the detection for the print side
    // (verbatim or dispatched, `write_front_matter`).
    let front_matter = parse_front_matter(source);
    let parse_source: &'a str = match &front_matter {
        Some(fm) => allocator.alloc_str(&blank_front_matter(source, fm.raw.len())),
        None => source,
    };

    PreparedSource { source, parse_source, front_matter }
}

/// Parse the (already normalized, front-matter-blanked) source into an AST
/// and collect comments, bailing out on any error.
///
/// `parse_source` comes from [`prepare_source`]; a leading `\u{feff}` can never
/// arrive here (physical entries split the whole BOM run, `FormatSession::dispatch`
/// preserves BOM-headed embedded inputs), and this parser needs that guarantee:
/// `oxc-css-parser` strips a leading `\u{feff}` from the source itself, which
/// would desync every span from the arena copy the printer slices.
fn parse_stylesheet<'a>(
    allocator: &'a Allocator,
    parse_source: &'a str,
    options: CssFormatOptions,
    tolerate_placeholders: bool,
) -> Result<(Stylesheet<'a>, &'a [CssComment]), OxcDiagnostic> {
    debug_assert!(!parse_source.starts_with('\u{feff}'), "callers must never pass a leading BOM");

    let mut parser = ParserBuilder::new(allocator, parse_source)
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
            // `.css` files in real projects routinely flow through `postcss-simple-vars`
            // (Mantine, Vite/Next templates, etc), so accept its `$var` syntax in `CssVariant::Css`.
            // Scss/Less already handle `$variable` natively.
            allow_postcss_simple_vars: matches!(options.variant, CssVariant::Css),
        })
        .comments()
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

    let comments: &'a [CssComment] = ArenaVec::from_iter_in(
        parser.comments().iter().map(|c| CssComment {
            span: to_span(&c.span),
            inline: matches!(c.kind, oxc_css_parser::token::CommentKind::Line),
        }),
        &allocator,
    )
    .into_arena_slice();

    Ok((stylesheet, comments))
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

fn write_front_matter<'a>(fm: &FrontMatter<'a>, f: &mut CssFormatter<'_, 'a>) {
    // NOTE: TOML currently has no IR-capable formatter, so it degrades to verbatim through `PreserveOriginal`.
    // Still need to specify here since blank TOML frontmatter will be normalized.
    oxc_formatter_core::write_front_matter(fm, &["yaml", "toml"], f);
}

/// Emits the stylesheet followed by any trailing comments, and the final newline.
struct FormatCssRoot<'b, 'a> {
    stylesheet: &'b Stylesheet<'a>,
    has_bom: bool,
    front_matter: Option<FrontMatter<'a>>,
}

/// Whether anything (statements or comments) follows the envelope,
/// deciding the front matter/body gap and the final newline.
fn has_content(stylesheet: &Stylesheet, f: &CssFormatter<'_, '_>) -> bool {
    !stylesheet.statements.is_empty() || f.context().comments().peek().is_some()
}

impl<'a> Format<'a, CssFormatContext<'a>> for FormatCssRoot<'_, 'a> {
    fn fmt(&self, f: &mut CssFormatter<'_, 'a>) {
        if self.has_bom {
            write!(f, text("\u{feff}"));
        }

        let has_content = has_content(self.stylesheet, f);
        if let Some(fm) = &self.front_matter {
            write_front_matter(fm, f);
            if has_content {
                // Always exactly one blank line between the block and the body
                // (regardless of the source gap; measured Prettier behavior).
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

/// Emits the stylesheet only; no BOM, no final newline
/// (front matter still prints for a `VirtualDocument`, whose complete-document envelope the host hands over;
/// the surrounding layout stays the host's).
struct FormatCssEmbedded<'b, 'a> {
    stylesheet: &'b Stylesheet<'a>,
    front_matter: Option<FrontMatter<'a>>,
}

impl<'a> Format<'a, CssFormatContext<'a>> for FormatCssEmbedded<'_, 'a> {
    fn fmt(&self, f: &mut CssFormatter<'_, 'a>) {
        if let Some(fm) = &self.front_matter {
            let has_content = has_content(self.stylesheet, f);
            write_front_matter(fm, f);
            if has_content {
                write!(f, empty_line());
            }
        }

        print::write_stylesheet(self.stylesheet, f);
    }
}
