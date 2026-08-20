use oxc_allocator::{Allocator, ArenaVec};
use oxc_diagnostics::OxcDiagnostic;
use oxc_formatter_core::{
    Buffer, Document, EmbeddedIr, Format, FormatSession, FormatState, Formatted, VecBuffer,
    builders::{hard_line_break, text},
    write,
};
use oxc_graphql_parser::{Parser, ast::Document as GraphQLDocument};
use oxc_span::Span;

use crate::{
    context::GraphqlFormatContext,
    options::GraphqlFormatOptions,
    print::{self, GraphqlFormatter, to_span},
};

/// Parse `source_text` as a GraphQL document and build its formatter IR.
///
/// # Errors
/// Returns an [`OxcDiagnostic`] when the parse produces any error.
/// `oxc-graphql-parser` is error-tolerant (it still returns an AST for invalid input),
/// but an AST with errors cannot be formatted faithfully, so a single error is enough to bail out.
pub fn format<'a>(
    allocator: &'a Allocator,
    source_text: &str,
    options: GraphqlFormatOptions,
) -> Result<Formatted<'a, GraphqlFormatContext<'a>>, OxcDiagnostic> {
    let (has_bom, source_text) = oxc_formatter_core::spec::split_bom(source_text);
    let (document, source, comments) = parse_document(allocator, source_text)?;

    let context = GraphqlFormatContext::new(options, source, comments);
    let mut state = FormatState::new(context, allocator);
    // Pre-allocate: measured on 1,241 real-world files (gitlab operations, github/saleor schemas),
    // 0.4x source bytes plus a 1024-element floor for small operation documents avoided reallocation for the entire corpus.
    let capacity = (source.len() * 2 / 5).max(1024);
    let mut buffer = VecBuffer::with_capacity(capacity, &mut state);

    write!(&mut buffer, FormatGraphqlRoot { document, has_bom });

    let elements = buffer.into_vec();
    let context = state.into_context();

    let ir = Document::new(elements, Vec::new());

    Ok(Formatted::new(ir, context))
}

/// Parse `source_text` and build the formatter IR for embedding into another
/// formatter's document (dispatcher path, e.g. graphql-in-js).
///
/// Unlike [`format()`], this:
/// - allocates from the session's shared arena and `GroupId` space,
///   so the IR lives as long as the parent's document
/// - emits neither a BOM nor the trailing newline (the parent owns the layout
///   around the embedded part, matching Prettier's `textToDoc` + `stripTrailingHardline` behavior)
///
/// # Errors
/// Same as [`format()`]: any parse error bails out.
pub fn format_to_ir<'a>(
    session: &FormatSession<'a>,
    source_text: &str,
    options: GraphqlFormatOptions,
) -> Result<EmbeddedIr<'a>, OxcDiagnostic> {
    let allocator = session.allocator();
    let (document, source, comments) = parse_document(allocator, source_text)?;

    let context = GraphqlFormatContext::new(options, source, comments);
    let mut state = FormatState::new_with_session(context, session.clone());
    let mut buffer = VecBuffer::new(&mut state);

    write!(&mut buffer, FormatGraphqlEmbedded { document });

    // GraphQL never collects Tailwind classes.
    Ok(EmbeddedIr { ir: buffer.into_vec(), tailwind_classes: Vec::new() })
}

/// Parse the source into a direct AST and collect comment trivia,
/// bailing out on any parse error.
///
/// Copies the source into the arena so every slice taken from it carries `'a`.
/// Entries own the BOM strip; this layer assumes BOM-free (see [`oxc_formatter_core::spec::split_bom`]).
fn parse_document<'a>(
    allocator: &'a Allocator,
    source_text: &str,
) -> Result<(&'a GraphQLDocument<'a>, &'a str, &'a [Span]), OxcDiagnostic> {
    let source: &'a str = allocator.alloc_str(source_text);

    // Opt-in graphql-js 17 syntax
    let ast = Parser::new(allocator, source).experimental_fragment_arguments(true).parse();
    if let Some(error) = ast.errors().next() {
        let start = u32::try_from(error.index()).unwrap_or(0);
        let len = u32::try_from(error.data().len()).unwrap_or(0);
        return Err(OxcDiagnostic::error(format!("Syntax error: {}", error.message()))
            .with_label(Span::sized(start, len)));
    }

    let comments: &'a [Span] =
        ArenaVec::from_iter_in(ast.comments().iter().copied().map(to_span), &allocator)
            .into_arena_slice();

    let document = allocator.alloc(ast.into_root());

    Ok((document, source, comments))
}

/// Emits the document's definitions followed by any trailing comments,
/// and the final newline.
struct FormatGraphqlRoot<'a> {
    document: &'a GraphQLDocument<'a>,
    has_bom: bool,
}

impl<'a> Format<'a, GraphqlFormatContext<'a>> for FormatGraphqlRoot<'a> {
    fn fmt(&self, f: &mut GraphqlFormatter<'_, 'a>) {
        if self.has_bom {
            write!(f, text("\u{feff}"));
        }

        print::write_document(self.document, f);

        // POSIX convention: every formatted file ends with a newline.
        write!(f, hard_line_break());
    }
}

/// Emits the document's definitions and trailing comments only;
/// no BOM, no final newline (the parent document owns the surrounding layout).
struct FormatGraphqlEmbedded<'a> {
    document: &'a GraphQLDocument<'a>,
}

impl<'a> Format<'a, GraphqlFormatContext<'a>> for FormatGraphqlEmbedded<'a> {
    fn fmt(&self, f: &mut GraphqlFormatter<'_, 'a>) {
        print::write_document(self.document, f);
    }
}
