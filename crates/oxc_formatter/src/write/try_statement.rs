use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{
    ast_nodes::AstNode,
    formatter::{
        Formatter,
        prelude::*,
        trivia::{FormatLeadingComments, FormatTrailingComments},
    },
    write,
};

use super::FormatWrite;

impl<'a> FormatWrite<'a> for AstNode<'a, TryStatement<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) {
        let block = self.block();
        let handler = self.handler();
        let finalizer = self.finalizer();
        write!(f, ["try", space()]);

        // Use `write` rather than `write!` in order to avoid printing leading comments for `block`.
        block.write(f);
        if let Some(handler) = handler {
            write!(f, [space(), handler]);
        }
        if let Some(finalizer) = finalizer {
            write!(f, [space(), "finally", space()]);
            finalizer.write(f);
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, CatchClause<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) {
        let comments = f.context().comments();
        let leading_comments = comments.comments_before(self.span.start);
        let has_line_comment = leading_comments.iter().any(|comment| {
            comment.is_line()
                || comments.is_own_line_comment(comment)
                || comments.is_end_of_line_comment(comment)
        });

        if has_line_comment {
            // `try {} /* comment */\n catch (e) {}`
            // should be formatted like:
            // `try {} catch (e) { /* comment */ }`
            //
            // Comments before the catch clause should be printed in the block statement.
            // We cache them here to avoid the `params` printing them accidentally.
            let printed_comments = f.intern(&FormatLeadingComments::Comments(leading_comments));
            if let Some(comments) = printed_comments {
                f.context_mut().cache_element(&self.span, comments);
            }
        } else if !leading_comments.is_empty() {
            // otherwise, print them before `catch`
            write!(f, [FormatTrailingComments::Comments(leading_comments), space()]);
        }

        write!(f, ["catch", space(), self.param(), space()]);

        // Use `write` rather than `write!` in order to avoid printing leading comments for `block`.
        self.body().write(f);
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, CatchParameter<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) {
        write!(f, "(");

        let span = self.pattern.span();

        let leading_comments = f.context().comments().comments_before(span.start);
        let leading_comment_with_break = leading_comments
            .iter()
            .any(|comment| comment.is_line() || f.source_text().lines_after(comment.span.end) > 0);

        let trailing_comments =
            f.context().comments().comments_before_character(self.span().end, b')');
        let trailing_comment_with_break = trailing_comments.iter().any(|comment| {
            comment.is_line() || f.source_text().get_lines_before(comment.span, f.comments()) > 0
        });

        if leading_comment_with_break || trailing_comment_with_break {
            write!(
                f,
                soft_block_indent(&format_with(|f| {
                    write!(f, [FormatLeadingComments::Comments(leading_comments)]);
                    let printed_len_before_pattern =
                        f.context().comments().printed_comments().len();
                    write!(f, self.pattern());
                    if trailing_comments.is_empty() ||
                    // The `pattern` cannot print comments that are below it, so we need to check whether there
                    // are any trailing comments that haven't been printed yet. If there are, print them.
                    f.context().comments().printed_comments().len() - printed_len_before_pattern
                        == trailing_comments.len()
                    {
                    } else {
                        write!(f, FormatTrailingComments::Comments(trailing_comments));
                    }
                }))
            );
        } else {
            write!(f, self.pattern());
        }

        self.format_trailing_comments(f);

        write!(f, ")");
    }
}
