use apollo_parser::{Parser, SyntaxKind, cst, cst::CstNode};
use oxc_allocator::{Allocator, ArenaVec};
use oxc_diagnostics::OxcDiagnostic;
use oxc_formatter_core::{
    Buffer, Document, Format, FormatState, Formatted, VecBuffer,
    builders::{hard_line_break, text},
    write,
};
use oxc_span::Span;

use crate::{
    context::GraphqlFormatContext,
    options::GraphqlFormatOptions,
    print::{self, GraphqlFormatter},
};

/// Parse `source_text` as a GraphQL document and build its formatter IR.
///
/// # Errors
/// Returns an [`OxcDiagnostic`] when the parse produces any error.
/// apollo-parser is error-tolerant (it returns a CST even for invalid input),
/// but a CST with errors cannot be formatted faithfully, so a single error is
/// enough to bail out. The caller (oxfmt) decides what to do next
/// (report, or fall back to Prettier).
pub fn format<'a>(
    allocator: &'a Allocator,
    source_text: &str,
    options: GraphqlFormatOptions,
) -> Result<Formatted<'a, GraphqlFormatContext<'a>>, OxcDiagnostic> {
    // Copy the source into the arena so every slice taken from it carries `'a`.
    let source: &'a str = allocator.alloc_str(source_text);

    let tree = Parser::new(source).parse();
    if let Some(error) = tree.errors().next() {
        let start = u32::try_from(error.index()).unwrap_or(0);
        let len = u32::try_from(error.data().len()).unwrap_or(0);
        return Err(OxcDiagnostic::error(format!("Syntax error: {}", error.message()))
            .with_label(Span::sized(start, len)));
    }

    let document = tree.document();

    // Collect comment trivia in document order. GraphQL comments are `# ...` line comments.
    let comments: &'a [Span] = ArenaVec::from_iter_in(
        document
            .syntax()
            .descendants_with_tokens()
            .filter_map(apollo_parser::SyntaxElement::into_token)
            .filter(|t| t.kind() == SyntaxKind::COMMENT)
            .map(|t| {
                let range = t.text_range();
                Span::new(range.start().into(), range.end().into())
            }),
        &allocator,
    )
    .into_arena_slice();

    let context = GraphqlFormatContext::new(options, source, comments);
    let mut state = FormatState::new(context, allocator);
    let mut buffer = VecBuffer::new(&mut state);

    let has_bom = source.starts_with('\u{feff}');
    write!(&mut buffer, FormatGraphqlRoot { document: &document, has_bom });

    let elements = buffer.into_vec();
    let context = state.into_context();

    let ir = Document::new(elements, Vec::new());
    ir.propagate_expand();

    Ok(Formatted::new(ir, context))
}

/// Emits the document's definitions followed by any trailing comments,
/// and the final newline.
struct FormatGraphqlRoot<'b> {
    document: &'b cst::Document,
    has_bom: bool,
}

impl<'a> Format<'a, GraphqlFormatContext<'a>> for FormatGraphqlRoot<'_> {
    fn fmt(&self, f: &mut GraphqlFormatter<'_, 'a>) {
        if self.has_bom {
            write!(f, text("\u{feff}"));
        }

        print::write_document(self.document, f);

        // POSIX convention: every formatted file ends with a newline.
        write!(f, hard_line_break());
    }
}
