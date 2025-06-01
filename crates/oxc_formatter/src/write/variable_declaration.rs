use oxc_allocator::Vec;
use oxc_ast::{AstKind, ast::*};

use crate::{
    format_args,
    formatter::{
        Buffer, Format, FormatError, FormatResult, Formatter, prelude::*,
        separated::FormatSeparatedIter,
    },
    options::TrailingSeparator,
    write,
};

use super::FormatWrite;

impl<'a> FormatWrite<'a> for VariableDeclaration<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, group(&format_args!(self.kind.as_str(), space(), self.declarations)))
    }
}

impl<'a> Format<'a> for Vec<'a, VariableDeclarator<'a>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let length = self.len();

        let is_parent_for_loop = matches!(
            f.parent_parent_kind(),
            Some(
                AstKind::ForStatement(_) | AstKind::ForInStatement(_) | AstKind::ForOfStatement(_)
            )
        );

        let has_any_initializer = self.iter().any(|declarator| declarator.init.is_some());

        let format_separator = format_with(|f| {
            if !is_parent_for_loop && has_any_initializer {
                write!(f, hard_line_break())
            } else {
                write!(f, soft_line_break_or_space())
            }
        });

        let mut declarators = self.iter().zip(
            FormatSeparatedIter::new(self.iter(), ",")
                .with_trailing_separator(TrailingSeparator::Disallowed),
        );

        let (first_declarator_span, format_first_declarator) = match declarators.next() {
            Some((decl, format_first_declarator)) => (decl.span, format_first_declarator),
            None => return Err(FormatError::SyntaxError),
        };

        if length == 1 && !f.comments().has_leading_comments(first_declarator_span.start) {
            return write!(f, format_first_declarator);
        }

        write!(
            f,
            indent(&format_once(|f| {
                write!(f, format_first_declarator)?;

                if length > 1 {
                    write!(f, format_separator)?;
                }

                f.join_with(&format_separator)
                    .entries(declarators.map(|(_, format)| format))
                    .finish()
            }))
        )
    }
}

impl<'a> FormatWrite<'a> for VariableDeclarator<'a> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, self.id)?;
        if let Some(init) = &self.init {
            write!(f, [space(), "=", space(), init])?;
        }
        Ok(())
    }
}
