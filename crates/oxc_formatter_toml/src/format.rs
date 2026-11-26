use oxc_allocator::Allocator;
use oxc_diagnostics::OxcDiagnostic;
use oxc_formatter_core::{
    Document, EmbeddedIr, FormatSession, FormatState, Formatted, Formatter, builders::text,
    spec::split_bom, write,
};
use oxc_span::Span;

use crate::{context::TomlFormatContext, options::TomlFormatOptions, print::write_root};

/// Parse `source_text` as TOML and build its formatter IR.
///
/// # Errors
/// Returns an [`OxcDiagnostic`] when the parse produces any error.
pub fn format<'a>(
    allocator: &'a Allocator,
    source_text: &str,
    options: TomlFormatOptions,
) -> Result<Formatted<'a, TomlFormatContext<'a>>, OxcDiagnostic> {
    let (has_bom, stripped) = split_bom(source_text);
    let source = allocator.alloc_str(stripped);
    let parse = parse_for_format(source).map_err(|mut err| {
        oxc_formatter_core::rebase_labels(&mut err.labels, source_text, stripped);
        err
    })?;

    let mut state = FormatState::new(TomlFormatContext::new(options, source), allocator);
    let mut buffer = Formatter::new(&mut state);
    if has_bom {
        write!(&mut buffer, text("\u{feff}"));
    }
    write_root(&parse.root, true, &mut buffer);
    let elements = buffer.into_vec();

    Ok(Formatted::new(Document::new(elements, Vec::new()), state.into_context()))
}

/// Format `source_text` into IR for embedding into another formatter's document
/// (e.g. TOML front matter, Markdown code blocks).
///
/// Unlike [`format()`], this allocates from the session's shared arena and `GroupId` space,
/// and emits no trailing newline (the parent owns the surrounding layout).
///
/// # Errors
/// Returns an [`OxcDiagnostic`] when the parse produces any error.
pub fn format_to_ir<'a>(
    session: &FormatSession<'a>,
    source_text: &str,
    options: TomlFormatOptions,
) -> Result<EmbeddedIr<'a>, OxcDiagnostic> {
    let source = session.allocator().alloc_str(source_text);
    let parse = parse_for_format(source)?;

    let mut state =
        FormatState::new_with_session(TomlFormatContext::new(options, source), session.clone());
    let mut buffer = Formatter::new(&mut state);
    write_root(&parse.root, false, &mut buffer);

    Ok(EmbeddedIr { ir: buffer.into_vec() })
}

/// `oxc_toml::parse()` is fault-tolerant, but the IR is built for valid TOML only,
/// so bail out on any parse error.
fn parse_for_format(source_text: &str) -> Result<oxc_toml::Parse<'_>, OxcDiagnostic> {
    let parse = oxc_toml::parse(source_text);
    if let Some(error) = parse.errors.first() {
        return Err(OxcDiagnostic::error(format!("Syntax error: {}", error.message))
            .with_label(Span::new(error.range.start, error.range.end)));
    }
    Ok(parse)
}
