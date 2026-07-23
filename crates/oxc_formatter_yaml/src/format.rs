use oxc_allocator::{Allocator, ArenaVec};
use oxc_diagnostics::OxcDiagnostic;
use oxc_formatter_core::{
    Buffer, Document, EmbeddedContext, EmbeddedIr, Format, FormatState, Formatted, VecBuffer,
    builders::{hard_line_break, text},
    write,
};
use oxc_span::Span;
use oxc_yaml_parser::{Parser, ast::Root};

use crate::{
    context::YamlFormatContext,
    options::YamlFormatOptions,
    print::{self, YamlFormatter, to_span},
};

/// Parse `source_text` as a YAML stream and build its formatter IR.
///
/// # Errors
/// Returns an [`OxcDiagnostic`] when the parse fails.
/// `oxc-yaml-parser` is fail-fast (no partial AST), so any syntax error bails out.
pub fn format<'a>(
    allocator: &'a Allocator,
    source_text: &str,
    options: YamlFormatOptions,
) -> Result<Formatted<'a, YamlFormatContext<'a>>, OxcDiagnostic> {
    // Checked against the original input: `parse_root` strips the BOM
    let has_bom = source_text.starts_with('\u{feff}');
    let (root, source, comments) = parse_root(allocator, source_text)?;

    let context =
        YamlFormatContext::new(options, source, comments, print::last_descendant_end(root));
    let mut state = FormatState::new(context, allocator);
    // TODO: Use `with_capacity` for perf, like `oxc_formatter` does
    let mut buffer = VecBuffer::new(&mut state);

    write!(&mut buffer, FormatYamlRoot { root, has_bom });

    let elements = buffer.into_vec();
    let context = state.into_context();

    let ir = Document::new(elements, Vec::new());
    ir.propagate_expand();

    Ok(Formatted::new(ir, context))
}

/// Parse `source_text` and build the formatter IR for embedding into another
/// formatter's document (dispatcher path, e.g. a fenced block in markdown).
///
/// Unlike [`format()`], this:
/// - allocates from the shared arena in `ctx`, so the IR lives as long as the parent's document
/// - emits neither a BOM nor the trailing newline
/// - skips `propagate_expand()`, which the parent runs on the merged document
///
/// # Errors
/// Same as [`format()`]: any parse error bails out.
pub fn format_to_ir<'a>(
    ctx: &EmbeddedContext<'a, '_>,
    source_text: &str,
    options: YamlFormatOptions,
) -> Result<EmbeddedIr<'a>, OxcDiagnostic> {
    let allocator = ctx.allocator;
    let (root, source, comments) = parse_root(allocator, source_text)?;

    let context =
        YamlFormatContext::new(options, source, comments, print::last_descendant_end(root));
    let mut state = FormatState::new(context, allocator);
    let mut buffer = VecBuffer::new(&mut state);

    write!(&mut buffer, FormatYamlEmbedded { root });

    // YAML never collects Tailwind classes.
    Ok(EmbeddedIr { ir: buffer.into_vec(), tailwind_classes: Vec::new() })
}

/// Parse the source into the yaml-unist-shaped AST and bridge comment trivia,
/// bailing out on any parse error.
///
/// Copies the source into the arena so every slice taken from it carries `'a`.
fn parse_root<'a>(
    allocator: &'a Allocator,
    source_text: &str,
) -> Result<(&'a Root<'a>, &'a str, &'a [Span]), OxcDiagnostic> {
    let source_text = source_text.strip_prefix('\u{feff}').unwrap_or(source_text);
    // NOTE: Normalize line endings BEFORE parsing like Prettier, unlike other `oxc_formatter_xxx`.
    // For YAML formatter, the printer slices verbatim text from the source in many places.
    // And a raw `\r` reaching the core `text()` builder panics.
    // Spans stay consistent because parse and print both use the normalized copy.
    let source_text = oxc_formatter_core::normalize_newlines(source_text, ['\r']);
    let source: &'a str = allocator.alloc_str(&source_text);

    let root = Parser::new(allocator, source).parse().map_err(|error| {
        OxcDiagnostic::error(format!("Syntax error: {}", error.kind))
            .with_label(to_span(error.span))
    })?;

    let root = allocator.alloc(root);

    let comments: &'a [Span] =
        ArenaVec::from_iter_in(root.comments.iter().map(|c| to_span(c.span)), &allocator)
            .into_arena_slice();

    Ok((root, source, comments))
}

/// Emits the stream's documents followed by any trailing comments, and the final newline.
struct FormatYamlRoot<'a> {
    root: &'a Root<'a>,
    has_bom: bool,
}

impl<'a> Format<'a, YamlFormatContext<'a>> for FormatYamlRoot<'a> {
    fn fmt(&self, f: &mut YamlFormatter<'_, 'a>) {
        if self.has_bom {
            write!(f, text("\u{feff}"));
        }

        let keep_chomped_tail = print::write_root(self.root, f);

        // POSIX convention: every formatted file ends with a newline.
        //
        // Prettier suppresses this when the stream's last descendant is a keep-chomped (`+`) block scalar,
        // whose verbatim content already carries the trailing newlines.
        // The scalar side of this handoff is `consumed_trailing_newlines` (print/block.rs):
        // a last-descendant block scalar emits NO trailing newlines of its own,
        // deferring the file tail to this write.
        if !keep_chomped_tail {
            write!(f, hard_line_break());
        }
    }
}

/// Emits the stream's documents and trailing comments only;
/// no BOM, no final newline (the parent document owns the surrounding layout).
struct FormatYamlEmbedded<'a> {
    root: &'a Root<'a>,
}

impl<'a> Format<'a, YamlFormatContext<'a>> for FormatYamlEmbedded<'a> {
    fn fmt(&self, f: &mut YamlFormatter<'_, 'a>) {
        print::write_root(self.root, f);
    }
}
