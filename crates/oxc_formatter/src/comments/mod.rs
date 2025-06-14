use std::{cell::Cell, ops::ControlFlow};

use oxc_ast::{Comment, ast};
use oxc_span::Span;

use crate::{
    Format, FormatResult, SourceComment, SyntaxTriviaPieceComments,
    formatter::{
        Formatter,
        comments::CommentKind,
        trivia::{format_leading_comments_with_comments, format_trailing_comments_with_comments},
    },
    generated::ast_nodes::FollowingNode,
};

#[inline]
pub fn is_new_line(char: char) -> ControlFlow<bool> {
    if char == ' ' || char == '\t' {
        ControlFlow::Continue(())
    } else if char == '\n' || char == '\r' || char == '\u{2028}' || char == '\u{2029}' {
        ControlFlow::Break(true)
    } else {
        ControlFlow::Break(false)
    }
}

pub fn has_new_line_backward(text: &str) -> bool {
    let mut chars = text.chars().rev();

    for char in chars {
        match is_new_line(char) {
            ControlFlow::Continue(()) => continue,
            ControlFlow::Break(true) => return true,
            ControlFlow::Break(false) => return false,
        }
    }

    false
}

pub fn has_new_line_forward(text: &str) -> bool {
    let mut chars = text.chars();

    for char in chars {
        match is_new_line(char) {
            ControlFlow::Continue(()) => continue,
            ControlFlow::Break(true) => return true,
            ControlFlow::Break(false) => return false,
        }
    }

    false
}

pub fn is_own_line_comment(comment: &Comment, source_text: &str) -> bool {
    let start = comment.span.start;
    if start == 0 {
        return false;
    }

    has_new_line_backward(Span::sized(0, comment.span.start).source_text(source_text))
}

pub fn is_end_of_line_comment(comment: &Comment, source_text: &str) -> bool {
    let end = comment.span.end;
    has_new_line_forward(&source_text[(end as usize)..])
}

pub fn print_leading_comments(
    span: Span,
    f: &mut Formatter<'_, '_>,
    allocator: &oxc_allocator::Allocator,
) -> FormatResult<()> {
    let source_text = f.context().source_text();
    let leading_comments = f
        .context()
        .comments_raw()
        .iter()
        .take_while(|comment| {
            comment.span.end <= span.start
                && (is_own_line_comment(comment, source_text)
                    || is_end_of_line_comment(comment, source_text)
                    || true)
        })
        .map(|comment| SourceComment::from_comment(comment, source_text));

    let leading_comments = oxc_allocator::Vec::from_iter_in(leading_comments, allocator);

    if leading_comments.is_empty() {
        return Ok(());
    }

    f.context_mut().set_printed_comment_index(leading_comments.len());
    format_leading_comments_with_comments(leading_comments.as_slice()).fmt(f)
}

pub fn print_trailing_comments(
    span: Span,
    following_node: Option<FollowingNode<'_>>,
    f: &mut Formatter<'_, '_>,
    allocator: &oxc_allocator::Allocator,
) -> FormatResult<()> {
    let following_span = following_node.map(|node| node.span());
    println!("{:?}", following_span);
    let source_text = f.context().source_text();
    let comments_raw = f.context().comments_raw();
    let last_comment_index = comments_raw
        .iter()
        .enumerate()
        .take_while(|(index, comment)| {
            // Trailing comments
            span.end <= comment.span.start
                // Skip comments that are after the following node
                && following_span.is_none_or(|following_span| comment.span.end <= following_span.start)
                && (is_own_line_comment(comment, source_text)
                    || is_end_of_line_comment(comment, source_text)
                    || true)
        })
        .last();

    if let Some((mut last_comment_index, _)) = last_comment_index {
        if let Some(following_span) = following_span {
            let mut gap_end = following_span.start;
            for cur_index in last_comment_index..0 {
                let comment = &comments_raw[last_comment_index];
                let gap_str = Span::new(comment.span.end, gap_end).source_text(source_text);
                if gap_str.as_bytes().iter().all(|&b| matches!(b, b' ' | b'(')) {
                    gap_end = comment.span.start;
                } else {
                    // If there is a non-whitespace character, we stop here
                    break;
                }
            }
        };

        let trailing_comments = oxc_allocator::Vec::from_iter_in(
            comments_raw[..last_comment_index]
                .iter()
                .map(|comment| SourceComment::from_comment(comment, source_text)),
            allocator,
        );

        if trailing_comments.is_empty() {
            return Ok(());
        }

        f.context_mut().set_printed_comment_index(trailing_comments.len());
        dbg!(&trailing_comments);

        format_trailing_comments_with_comments(&trailing_comments).fmt(f)
    } else {
        Ok(())
    }
}

impl SourceComment {
    fn from_comment(comment: &Comment, source_text: &str) -> Self {
        SourceComment {
            span: comment.span,
            lines_before: source_text[..comment.span.start as usize]
                .bytes()
                .rev()
                .take_while(|&b| b == b'\n')
                .count() as u32,
            lines_after: source_text[comment.span.end as usize..]
                .bytes()
                .take_while(|&b| b == b'\n')
                .count() as u32,
            // FIXME
            piece: SyntaxTriviaPieceComments,
            kind: match comment.kind {
                ast::CommentKind::Line => CommentKind::Line,
                ast::CommentKind::Block => CommentKind::Block, // TODO: missing CommentKind::InlineBlock
            },
            formatted: Cell::new(false),
        }
    }
}

// let following_span = match self.parent {
//         AstNodes::VariableDeclarator(declarator) => {
//             declarator.init.as_ref().map(|init| init.span())
//         }
//         _ => None,
//     };
