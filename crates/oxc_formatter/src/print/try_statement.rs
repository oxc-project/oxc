use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{
    ast_nodes::AstNode,
    formatter::{
        JsFormatter,
        prelude::*,
        trivia::{FormatLeadingComments, FormatTrailingComments},
    },
    write,
};

use super::FormatWrite;
use crate::utils::{
    format_node_without_trailing_comments::FormatNodeWithoutTrailingComments,
    statement_body::{write_comments_between_blocks, write_head_body_separator},
};

impl<'a> FormatWrite<'a> for AstNode<'a, TryStatement<'a>> {
    fn write(&self, f: &mut JsFormatter<'_, 'a>) {
        let block = self.block();
        let handler = self.handler();
        let finalizer = self.finalizer();

        write!(f, "try");
        write_head_body_separator(block.span.start, f);
        write_block_before_keyword(block, handler.is_some() || finalizer.is_some(), f);

        if let Some(handler) = handler {
            write!(f, space());
            write_block_before_keyword(handler, finalizer.is_some(), f);
        }
        if let Some(finalizer) = finalizer {
            // Lexical scan for the keyword's first byte: the gap holds only trivia and `finally`.
            let previous_end = handler.map_or(block.span().end, |handler| handler.span().end);
            let before_keyword =
                f.context().comments().comments_before_character(previous_end, b'f');
            // The pending space is dropped at a line start and coalesced otherwise
            write_comments_between_blocks(before_keyword, f);
            write!(f, [space(), "finally"]);
            write_head_body_separator(finalizer.span.start, f);
            write!(f, finalizer);
        }
    }
}

/// Prints `node`, suppressing its generic trailing pass when a keyword (`catch`/`finally`) follows,
/// so it cannot claim a line comment past that keyword; the keyword site splits the comments instead.
fn write_block_before_keyword<'a, T>(node: &T, keyword_follows: bool, f: &mut JsFormatter<'_, 'a>)
where
    T: Format<'a, JsFormatContext<'a>> + GetSpan,
{
    if keyword_follows {
        FormatNodeWithoutTrailingComments(node).fmt(f);
    } else {
        node.fmt(f);
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, CatchClause<'a>> {
    fn write(&self, f: &mut JsFormatter<'_, 'a>) {
        let leading_comments = f.context().comments().comments_before(self.span.start);
        if write_comments_between_blocks(leading_comments, f) {
            write!(f, space());
        }

        write!(f, ["catch", space(), self.param()]);

        let block = self.body();
        write_head_body_separator(block.span.start, f);
        write!(f, block);
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, CatchParameter<'a>> {
    fn write(&self, f: &mut JsFormatter<'_, 'a>) {
        write!(f, "(");

        let span = self.pattern.span();

        let leading_comments = f.context().comments().comments_before(span.start);
        let leading_comment_with_break = leading_comments
            .iter()
            .any(|comment| comment.is_line() || comment.followed_by_newline());

        let trailing_comments =
            f.context().comments().comments_before_character(self.span().end, b')');
        let trailing_comment_with_break = trailing_comments
            .iter()
            .any(|comment| comment.is_line() || comment.preceded_by_newline());

        if leading_comment_with_break || trailing_comment_with_break {
            write!(
                f,
                soft_block_indent(&format_with(|f| {
                    write!(f, [FormatLeadingComments::Comments(leading_comments)]);
                    write!(f, self.pattern());
                    write!(f, self.type_annotation());
                    // Re-queried after the cursor advanced:
                    // the pattern may have printed part of the earlier `trailing_comments` slice already
                    let remaining =
                        f.context().comments().comments_before_character(self.span().end, b')');
                    write!(f, FormatTrailingComments::Comments(remaining));
                }))
            );
        } else {
            write!(f, self.pattern());
            write!(f, self.type_annotation());
        }

        // Bound the trailing print at the `)`:
        // the generic pass would claim an end-of-line comment past it and flush it beyond the catch body's `{`.
        // (Re-queried again: the branch above advances the cursor.)
        let remaining = f.context().comments().comments_before_character(self.span().end, b')');
        write!(f, FormatTrailingComments::Comments(remaining));

        write!(f, ")");
    }
}
