use oxc_allocator::ArenaVec;
use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::print::semicolon::FormatContentWithSemicolon;
use crate::utils::assignment_like::AssignmentLike;
use crate::{
    ast_nodes::{AstNode, AstNodes},
    format_args,
    formatter::{
        Buffer, Format, JsFormatContext, JsFormatter, prelude::*, separated::FormatSeparatedIter,
    },
    options::TrailingSeparator,
    write,
};

use super::FormatWrite;

impl<'a> FormatWrite<'a> for AstNode<'a, VariableDeclaration<'a>> {
    fn write(&self, f: &mut JsFormatter<'_, 'a>) {
        let semicolon = match self.parent() {
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

        let declarations = self.declarations();
        let format_declarations = format_with(|f| {
            if semicolon {
                let declarations_end =
                    declarations.as_ref().last().map_or(self.span().end, |d| d.span().end);
                write!(
                    f,
                    FormatContentWithSemicolon::new(
                        declarations,
                        declarations_end,
                        self.span().end
                    )
                );
            } else {
                write!(f, declarations);
            }
        });
        write!(f, group(&format_args!(self.kind().as_str(), space(), format_declarations)));
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
                write!(f, &first_declarator);
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
