//! CST position utilities over oxc-graphql-parser's lossless tree.
//!
//! `oxc-graphql-parser` attaches pending trivia (whitespace, comments, insignificant commas)
//! to whichever node is open when the next significant token is consumed,
//! so a node's raw `text_range()` may start or end on trivia.
//! Every layout decision therefore uses significant-token positions computed here.

use oxc_graphql_parser::{SyntaxKind, SyntaxNode, SyntaxToken};

use oxc_span::Span;

use super::GraphqlFormatter;

/// Whether `kind` is a trivia token. In GraphQL, commas are trivia too.
fn is_trivia(kind: SyntaxKind) -> bool {
    matches!(kind, SyntaxKind::WHITESPACE | SyntaxKind::COMMENT | SyntaxKind::COMMA)
}

/// Start offset of the first significant (non-trivia) token within `node`.
///
/// oxc-graphql-parser attaches pending trivia to the node that is open when the next
/// significant token is consumed, so `node.text_range().start()` may point at a
/// comment that logically precedes the node. All layout decisions use significant
/// token positions instead.
pub fn sig_start(node: &SyntaxNode) -> u32 {
    let node_end = u32::from(node.text_range().end());
    let mut tok = node.first_token();
    while let Some(t) = tok {
        if u32::from(t.text_range().start()) >= node_end {
            break;
        }
        if !is_trivia(t.kind()) {
            return t.text_range().start().into();
        }
        tok = t.next_token();
    }
    node.text_range().start().into()
}

/// End offset of the last significant (non-trivia) token within `node`.
pub fn sig_end(node: &SyntaxNode) -> u32 {
    let node_start = u32::from(node.text_range().start());
    let mut tok = node.last_token();
    while let Some(t) = tok {
        if u32::from(t.text_range().end()) <= node_start {
            break;
        }
        if !is_trivia(t.kind()) {
            return t.text_range().end().into();
        }
        tok = t.prev_token();
    }
    node.text_range().end().into()
}

/// Significant-token span of `node`.
fn sig_span(node: &SyntaxNode) -> Span {
    Span::new(sig_start(node), sig_end(node))
}

/// Start offset of a closing delimiter: the token's start when present,
/// the container's significant end otherwise (error-resilient fallback).
pub fn closing_token_start(token: Option<SyntaxToken>, container: &SyntaxNode) -> u32 {
    token.map_or_else(|| sig_end(container), |t| t.text_range().start().into())
}

/// Source slice of `node`'s significant span, carrying the arena lifetime.
pub fn node_text<'a>(f: &GraphqlFormatter<'_, 'a>, node: &SyntaxNode) -> &'a str {
    let span = sig_span(node);
    f.context().source_text().slice_range(span.start, span.end)
}
