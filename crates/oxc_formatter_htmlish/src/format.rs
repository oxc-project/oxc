use oxc_allocator::Allocator;
use oxc_diagnostics::OxcDiagnostic;
use oxc_formatter_core::{
    Document, EmbeddedIr, Format, FormatSession, FormatState, Formatted, Formatter, InputKind,
    builders::{empty_line, hard_line_break, indent, text},
    spec::{FrontMatter, blank_front_matter, parse_front_matter, split_bom},
    write, write_front_matter,
};
use oxc_htmlish_parser::{
    Dialect, Parser,
    ast::{NodeId, Root},
};

use crate::{
    context::HtmlishFormatContext,
    options::HtmlishFormatOptions,
    print::{self, HtmlishFormatter, format_with},
    tree::Tree,
};

/// Parse `source_text` as `options.variant` and build its formatter IR.
///
/// # Errors
/// Returns an [`OxcDiagnostic`] when the parser reports broken structure.
pub fn format<'a>(
    allocator: &'a Allocator,
    source_text: &str,
    options: HtmlishFormatOptions,
) -> Result<Formatted<'a, HtmlishFormatContext<'a>>, OxcDiagnostic> {
    // A service-less session: front matter stays as-is.
    // Hosts that install services (oxfmt) use [`format_with_session`].
    format_with_session(
        &FormatSession::new(allocator, InputKind::PhysicalFile),
        source_text,
        options,
    )
}

/// Like [`format()`], but on a caller-supplied [`FormatSession`]:
/// the root whose session carries the host's `SessionServices` (front matter dispatch, Tailwind sorting).
///
/// # Errors
/// Same as [`format()`].
pub fn format_with_session<'a>(
    session: &FormatSession<'a>,
    source_text: &str,
    options: HtmlishFormatOptions,
) -> Result<Formatted<'a, HtmlishFormatContext<'a>>, OxcDiagnostic> {
    debug_assert!(
        session.input_kind() == InputKind::PhysicalFile,
        "format_with_session is the physical-root entry; embedded inputs go through format_to_ir"
    );
    let parsed = parse_for_format(session.allocator(), source_text, &options)?;

    let context = build_context(session.allocator(), &parsed, options, true, None);
    // A root `Document` owns a fresh Tailwind class scope
    let session = session.with_new_tailwind_scope();
    let mut state = FormatState::new_with_session(context, session.clone());
    // Pre-allocate: measured on 771 HTML files (markup_fmt, biome, deno, Prettier tests, large spec / wikipedia pages),
    // 0.75x source bytes plus a 1024-element floor avoids reallocation for 97.8% of the corpus.
    let capacity = (parsed.source.len() * 3 / 4).max(1024);
    let mut buffer = Formatter::with_capacity(capacity, &mut state);

    write!(&mut buffer, FormatHtmlishRoot { parsed: &parsed });

    let elements = buffer.into_vec();
    let context = state.into_context();

    let ir = Document::new(elements, session.take_sorted_tailwind_classes());

    Ok(Formatted::new(ir, context))
}

/// [`parse_for_format`] output: the AST, plus the envelope [`format()`] prints around it.
pub struct ParsedHtmlish<'a> {
    pub root: &'a Root<'a>,
    /// The arena source every span indexes into, `\r` normalized away and the front matter blanked.
    pub source: &'a str,
    /// The leading `---` / `+++` block (it is not markup), sliced from the unblanked source.
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
    options: &HtmlishFormatOptions,
) -> Result<ParsedHtmlish<'a>, OxcDiagnostic> {
    let (has_bom, stripped) = split_bom(source_text);
    parse_root(allocator, stripped, has_bom, options.variant.to_dialect()).map_err(|mut err| {
        oxc_formatter_core::rebase_labels(&mut err.labels, source_text, stripped);
        err
    })
}

/// Parse `source_text` and build the formatter IR for embedding into another
/// formatter's document (dispatcher path).
///
/// Unlike [`format()`], this:
/// - allocates from the session's shared arena and `GroupId` space, so the IR lives as long as the parent's document
/// - emits neither a BOM nor the trailing newline
///
/// # Errors
/// Same as [`format()`], and when a fragment starts with front matter.
pub fn format_to_ir<'a>(
    session: &FormatSession<'a>,
    source_text: &str,
    options: HtmlishFormatOptions,
    embedded_in: Option<HtmlishEmbeddedIn>,
) -> Result<EmbeddedIr<'a>, OxcDiagnostic> {
    let allocator = session.allocator();
    // `FormatSession::dispatch` never hands a BOM-headed input to an embedded part
    let parsed = parse_root(allocator, source_text, false, options.variant.to_dialect())?;
    if parsed.front_matter.is_some() && !session.input_kind().owns_front_matter() {
        // A fragment never acquires file envelope semantics
        return Err(OxcDiagnostic::error(
            "Front matter in an HTML fragment; the part is preserved as-is",
        ));
    }

    let context = build_context(allocator, &parsed, options, false, embedded_in);
    let mut state = FormatState::new_with_session(context, session.clone());
    let mut buffer = Formatter::new(&mut state);

    write!(&mut buffer, FormatHtmlishEmbedded { parsed: &parsed });

    Ok(EmbeddedIr { ir: buffer.into_vec() })
}

/// What the HTML is embedded in, for the print rules that depend on it.
///
/// The host translates the parent's request into this;
/// `None` is a part formatted on its own (e.g. a JSDoc code block).
/// Embedded HTML leaves `class` and `allow` values as written (as Prettier does with a `parentParser`).
#[derive(Clone, Copy, Debug, Default)]
pub struct HtmlishEmbeddedIn {
    /// Indent the content when more than one node prints at the root
    /// (html-in-js, whose backticks hug the content).
    pub indent_multiple_roots: bool,
    /// The parent's placeholder for an expression (html-in-js's `${}`), as the prefix and suffix around its ASCII digits and `_`:
    /// a value that is one stays unquoted (`attr=${value}` passes a value, not a string),
    /// and a value holding one is no class list to sort.
    pub placeholder: Option<(&'static str, &'static str)>,
}

impl HtmlishEmbeddedIn {
    /// `value` is the parent's placeholder as a whole.
    pub(crate) fn is_placeholder(&self, value: &str) -> bool {
        self.placeholder.is_some_and(|(prefix, suffix)| {
            value.strip_prefix(prefix).and_then(|id| id.strip_suffix(suffix)).is_some_and(|id| {
                !id.is_empty() && id.bytes().all(|b| b.is_ascii_digit() || b == b'_')
            })
        })
    }

    /// `value` holds the parent's placeholder.
    pub(crate) fn has_placeholder(&self, value: &str) -> bool {
        self.placeholder.is_some_and(|(prefix, _)| value.contains(prefix))
    }
}

/// Parse the source into the AST, bailing out on any diagnostic.
///
/// Copies the source into the arena so every slice taken from it carries `'a`.
/// Entries own the BOM strip; this layer assumes BOM-free input (see [`split_bom`]).
fn parse_root<'a>(
    allocator: &'a Allocator,
    source_text: &str,
    has_bom: bool,
    dialect: Dialect,
) -> Result<ParsedHtmlish<'a>, OxcDiagnostic> {
    // The printer slices verbatim text from the source, and the IR forbids `\r`.
    let unblanked: &'a str =
        allocator.alloc_str(&oxc_formatter_core::normalize_newlines(source_text, ['\r']));
    // Blanked byte for byte, so every span still indexes the same source.
    let front_matter = parse_front_matter(unblanked);
    let source: &'a str = match &front_matter {
        Some(fm) => allocator.alloc_str(&blank_front_matter(unblanked, fm.raw.len())),
        None => unblanked,
    };
    let root = Parser::new(allocator, source, dialect).parse().map_err(|diagnostic| {
        let mut err = OxcDiagnostic::error(format!("Syntax error: {diagnostic}"))
            .with_label(oxc_span::Span::new(diagnostic.span.start, diagnostic.span.end));
        oxc_formatter_core::restore_crlf_labels(&mut err.labels, source_text);
        err
    })?;
    Ok(ParsedHtmlish { root: allocator.alloc(root), source, front_matter, has_bom })
}

/// The context over the preprocess table [`Tree::build`] computes from the AST.
fn build_context<'a>(
    allocator: &'a Allocator,
    parsed: &ParsedHtmlish<'a>,
    options: HtmlishFormatOptions,
    standalone: bool,
    embedded_in: Option<HtmlishEmbeddedIn>,
) -> HtmlishFormatContext<'a> {
    let tree = allocator.alloc(Tree::build(
        allocator,
        parsed.root,
        parsed.source,
        options.whitespace_sensitivity,
        options.variant,
        standalone,
    ));
    HtmlishFormatContext::new(options, tree, embedded_in)
}

/// Emits the front matter and the nodes with the final newline.
struct FormatHtmlishRoot<'b, 'a> {
    parsed: &'b ParsedHtmlish<'a>,
}

impl<'a> Format<'a, HtmlishFormatContext<'a>> for FormatHtmlishRoot<'_, 'a> {
    fn fmt(&self, f: &mut HtmlishFormatter<'_, 'a>) {
        if self.parsed.has_bom {
            write!(f, text("\u{feff}"));
        }

        write_document(self.parsed, f);

        // Prettier prints an empty document as the empty string, not `\n`.
        let has_body = f.context().tree()[NodeId::DOCUMENT].first_child.is_some();
        if self.parsed.front_matter.is_some() || has_body {
            write!(f, hard_line_break());
        }
    }
}

/// Emits the front matter and the nodes only;
/// no BOM, no final newline (the parent document owns the surrounding layout).
struct FormatHtmlishEmbedded<'b, 'a> {
    parsed: &'b ParsedHtmlish<'a>,
}

impl<'a> Format<'a, HtmlishFormatContext<'a>> for FormatHtmlishEmbedded<'_, 'a> {
    fn fmt(&self, f: &mut HtmlishFormatter<'_, 'a>) {
        write_document(self.parsed, f);
    }
}

/// The front matter (a blank line after it when a body follows) and the nodes,
/// indented when the parent asks for it and more than one prints at the root.
fn write_document<'a>(parsed: &ParsedHtmlish<'a>, f: &mut HtmlishFormatter<'_, 'a>) {
    let document = &f.context().tree()[NodeId::DOCUMENT];
    let has_body = document.first_child.is_some();
    if let Some(fm) = &parsed.front_matter {
        write_front_matter(fm, &["yaml", "toml"], f);
        if has_body {
            write!(f, empty_line());
        }
    }
    if !has_body {
        return;
    }
    let indent_body = f.context().embedded_in().is_some_and(|e| e.indent_multiple_roots)
        && document.only_child().is_none();
    if indent_body {
        write!(f, indent(&format_with(print::write_root)));
    } else {
        print::write_root(f);
    }
}
