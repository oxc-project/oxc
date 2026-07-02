use oxc_allocator::ArenaVec;
use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::print::semicolon::OptionalSemicolon;
use crate::utils::assignment_like::AssignmentLike;
use crate::{
    ast_nodes::{AstNode, AstNodes},
    format_args,
    formatter::{
        Buffer, Format, JsFormatContext, JsFormatter, prelude::*, separated::FormatSeparatedIter,
    },
    options::TrailingSeparator,
    utils::format_node_without_trailing_comments::FormatNodeWithoutCommentsAfterPosition,
    write,
};

use super::FormatWrite;

impl<'a> FormatWrite<'a> for AstNode<'a, VariableDeclaration<'a>> {
    fn write(&self, f: &mut JsFormatter<'_, 'a>) {
        let semicolon = match self.parent() {
            AstNodes::ExportNamedDeclaration(_) => false,
            AstNodes::ForStatement(stmt) => {
                stmt.init().is_some_and(|init| init.span() != self.span())
            }
            AstNodes::ForInStatement(stmt) => stmt.left().span() != self.span(),
            AstNodes::ForOfStatement(stmt) => stmt.left().span() != self.span(),
            _ => true,
        };

        if self.declare() {
            write!(f, ["declare", space()]);
        }

        write!(
            f,
            group(&format_args!(
                self.kind().as_str(),
                space(),
                self.declarations(),
                semicolon.then_some(OptionalSemicolon)
            ))
        );
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, ArenaVec<'a, VariableDeclarator<'a>>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        let length = self.len();

        let is_parent_for_loop = matches!(
            self.grand_parent(),
            AstNodes::ForStatement(_) | AstNodes::ForInStatement(_) | AstNodes::ForOfStatement(_)
        );

        let has_any_initializer = self.iter().any(|declarator| declarator.init().is_some());

        let format_separator = format_with(|f| {
            if !is_parent_for_loop && has_any_initializer {
                write!(f, hard_line_break());
            } else {
                write!(f, soft_line_break_or_space());
            }
        });

        let mut declarators = FormatSeparatedIter::new(self.iter(), ",")
            .with_trailing_separator(TrailingSeparator::Disallowed);

        // `VariableDeclaration` always has at least one declarator.
        let first_declarator = declarators.next().unwrap();

        if length == 1 && !f.comments().has_comment_before(first_declarator.span().start) {
            return if first_declarator.init.is_none()
                && f.comments()
                    .has_comment_in_range(first_declarator.span.end, self.parent().span().end)
            {
                write!(f, indent(&first_declarator));
            } else {
                // Prettier lets `VariableDeclaration` print the semicolon after the single
                // declarator, so leave end-trailing comments for the declaration node.
                // https://github.com/prettier/prettier/blob/1c6ba5539141552e0e8e22d401ea620d8fdff468/src/language-js/print/estree.js#L313-L337
                let boundary = single_declarator_trailing_comments_boundary(&first_declarator, f);
                write!(f, FormatNodeWithoutCommentsAfterPosition::new(&first_declarator, boundary));
            };
        }

        write!(
            f,
            indent(&format_once(|f| {
                write!(f, first_declarator);

                if length > 1 {
                    write!(f, format_separator);
                }

                f.join_with(format_separator).entries(declarators);
            }))
        );
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, VariableDeclarator<'a>> {
    fn write(&self, f: &mut JsFormatter<'_, 'a>) {
        AssignmentLike::VariableDeclarator(self).fmt(f);
    }
}

fn single_declarator_trailing_comments_boundary(
    declarator: &AstNode<'_, VariableDeclarator<'_>>,
    f: &JsFormatter<'_, '_>,
) -> u32 {
    let default_boundary = declarator.span().end;

    let Some(init) = declarator.init() else {
        return default_boundary;
    };

    let Expression::ArrayExpression(array) = init.as_ref() else {
        return default_boundary;
    };

    let Some(last_element) = array.elements.last() else {
        return default_boundary;
    };

    if !last_element.is_elision() {
        return default_boundary;
    }

    let last_element_end = last_element.span().end;
    let comments = f.comments().comments_in_range(last_element_end, array.span.end);
    if comments.iter().any(|comment| comment.is_block() && !comment.preceded_by_newline()) {
        last_element_end
    } else {
        default_boundary
    }
}
