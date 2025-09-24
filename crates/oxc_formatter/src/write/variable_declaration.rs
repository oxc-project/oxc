use oxc_allocator::{Address, Vec};
use oxc_ast::{AstKind, ast::*};
use oxc_span::GetSpan;

use crate::utils::assignment_like::AssignmentLike;
use crate::write::semicolon::MaybeOptionalSemicolon;
use crate::write::{OptionalSemicolon, semicolon};
use crate::{
    format_args,
    formatter::{
        Buffer, Format, FormatError, FormatResult, Formatter, prelude::*,
        separated::FormatSeparatedIter,
    },
    generated::ast_nodes::{AstNode, AstNodes},
    options::TrailingSeparator,
    write,
};

use super::FormatWrite;

impl<'a> FormatWrite<'a> for AstNode<'a, VariableDeclaration<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let semicolon = match self.parent {
            AstNodes::ExportNamedDeclaration(_) => false,
            AstNodes::ForStatement(stmt) => {
                stmt.init().is_some_and(|init| init.span() != self.span())
            }
            // TODO: It would be better if there is a AstNodes which is `left` of `ForInStatement` and `ForOfStatement`.
            AstNodes::ForInStatement(stmt) => stmt.left().span() != self.span(),
            AstNodes::ForOfStatement(stmt) => stmt.left().span() != self.span(),
            _ => true,
        };

        if self.declare() {
            write!(f, ["declare", space()])?;
        }

        write!(
            f,
            group(&format_args!(
                self.kind().as_str(),
                space(),
                self.declarations(),
                MaybeOptionalSemicolon(semicolon)
            ))
        )
    }
}

impl<'a> Format<'a> for AstNode<'a, Vec<'a, VariableDeclarator<'a>>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let length = self.len();

        let is_parent_for_loop = matches!(
            self.parent.parent(),
            AstNodes::ForStatement(_) | AstNodes::ForInStatement(_) | AstNodes::ForOfStatement(_)
        );

        let has_any_initializer = self.iter().any(|declarator| declarator.init().is_some());

        let format_separator = format_with(|f| {
            if !is_parent_for_loop && has_any_initializer {
                write!(f, hard_line_break())
            } else {
                write!(f, soft_line_break_or_space())
            }
        });

        let mut declarators = FormatSeparatedIter::new(self.iter(), ",")
            .with_trailing_separator(TrailingSeparator::Disallowed);

        // `VariableDeclaration` always has at least one declarator.
        let first_declarator = declarators.next().unwrap();

        if length == 1 && !f.comments().has_comment_before(first_declarator.span().start) {
            return write!(f, first_declarator);
        }

        write!(
            f,
            indent(&format_once(|f| {
                write!(f, first_declarator)?;

                if length > 1 {
                    write!(f, format_separator)?;
                }

                f.join_with(format_separator).entries(declarators).finish()
            }))
        )
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, VariableDeclarator<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        AssignmentLike::VariableDeclarator(self).fmt(f)
    }
}
