use oxc_allocator::CloneIn;
use oxc_ast_macros::{ast, ast_meta};
use oxc_estree::ESTree;
use oxc_span::{Atom, ContentEq, GetSpan, GetSpanMut, Span};

#[ast]
#[generate_derive(CloneIn, ContentEq, ESTree, GetSpan, GetSpanMut)]
#[estree(add_fields(value = TokenValue), no_type, no_ts_def, no_parent)]
#[derive(Debug)]
/// Represents a token in the source code.
pub struct Token<'a> {
    /// Span.
    #[span]
    pub span: Span,
    /// Type.
    pub r#type: Atom<'a>,
    /// Flags.
    pub flags: Option<Atom<'a>>,
    /// Pattern.
    pub pattern: Option<Atom<'a>>,
}

/// Custom deserializer for `value` field of `Token`.
#[ast_meta]
#[generate_derive(CloneIn, ContentEq, ESTree)]
#[estree(ts_type = "string", raw_deser = "SOURCE_TEXT.slice(THIS.start, THIS.end)")]
pub struct TokenValue<'a, 'b>(pub &'b Token<'a>);
