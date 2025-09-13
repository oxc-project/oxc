use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_span::GetSpan;
use oxc_syntax::identifier::is_identifier_name;

use crate::{
    Format, FormatResult, FormatTrailingCommas, QuoteProperties, TrailingSeparator, best_fitting,
    format_args,
    formatter::{
        Formatter,
        prelude::*,
        separated::FormatSeparatedIter,
        trivia::{FormatLeadingComments, FormatTrailingComments},
    },
    generated::ast_nodes::{AstNode, AstNodes},
    write,
    write::semicolon::OptionalSemicolon,
};

use super::FormatWrite;

impl<'a> Format<'a> for ImportOrExportKind {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        if self.is_type() { write!(f, ["type", space()]) } else { Ok(()) }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ImportDeclaration<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["import", space(), self.import_kind])?;

        if let Some(specifiers) = self.specifiers() {
            write!(f, [specifiers, space(), "from", space()])?;
        }

        write!(f, [self.source(), self.with_clause(), OptionalSemicolon])
    }
}

impl<'a> Format<'a> for AstNode<'a, Vec<'a, ImportDeclarationSpecifier<'a>>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let mut specifiers_iter = self.iter().peekable();

        while let Some(specifier) = specifiers_iter.peek() {
            match specifier.as_ref() {
                ImportDeclarationSpecifier::ImportDefaultSpecifier(_) => {
                    write!(f, [specifiers_iter.next().unwrap()])?;
                }
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(_) => {
                    write!(f, [specifiers_iter.next().unwrap()])?;
                }
                ImportDeclarationSpecifier::ImportSpecifier(_) => {
                    break;
                }
            }

            if specifiers_iter.peek().is_some() {
                write!(f, [",", space()])?;
            } else {
                return Ok(());
            }
        }

        let should_insert_space_around_brackets = f.options().bracket_spacing.value();

        if self.is_empty() {
            write!(f, ["{}"])?;
        } else if self.len() == 1
            && let Some(ImportDeclarationSpecifier::ImportSpecifier(specifier)) =
                specifiers_iter.peek().map(AsRef::as_ref)
            && !f.comments().has_comment_before(specifier.local.span.start)
        {
            write!(
                f,
                [
                    "{",
                    maybe_space(should_insert_space_around_brackets),
                    specifiers_iter.next().unwrap(),
                    maybe_space(should_insert_space_around_brackets),
                    "}",
                ]
            )?;
        } else {
            write!(
                f,
                [
                    "{",
                    group(&soft_block_indent_with_maybe_space(
                        &format_once(|f| {
                            let trailing_separator =
                                FormatTrailingCommas::ES5.trailing_separator(f.options());
                            let iter = FormatSeparatedIter::new(specifiers_iter, ",")
                                .with_trailing_separator(trailing_separator)
                                .map(|specifier| {
                                    format_once(move |f| {
                                        // Should add empty line before the specifier if there are comments before it.
                                        let comments = f
                                            .context()
                                            .comments()
                                            .comments_before(specifier.span().start);
                                        if !comments.is_empty() {
                                            if f.source_text()
                                                .get_lines_before(comments[0].span, f.comments())
                                                > 1
                                            {
                                                write!(f, [empty_line()])?;
                                            }
                                            write!(f, [FormatLeadingComments::Comments(comments)])?;
                                        }

                                        write!(f, specifier)
                                    })
                                });
                            f.join_with(soft_line_break_or_space()).entries(iter).finish()
                        }),
                        should_insert_space_around_brackets
                    )),
                    "}"
                ]
            )?;
        }

        Ok(())
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ImportSpecifier<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let comments = f.context().comments().comments_before(self.local.span.end);
        let mut len = comments.len();
        while len != 0 && comments[len - 1].is_block() {
            len -= 1;
        }
        if len != 0 {
            write!(f, [FormatLeadingComments::Comments(&comments[..len])])?;
        }
        write!(f, [self.import_kind()])?;
        if self.local.span == self.imported.span() {
            write!(f, [self.local()])?;
        } else {
            write!(f, [self.imported(), space(), "as", space(), self.local()])?;
        }

        if f.source_text().next_non_whitespace_byte_is(self.span.end, b'}') {
            let comments = f.context().comments().comments_before_character(self.span.end, b'}');
            write!(f, [FormatTrailingComments::Comments(comments)])
        } else {
            self.format_trailing_comments(f)
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ImportDefaultSpecifier<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.local().fmt(f)
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ImportNamespaceSpecifier<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["*", space(), "as", space()])?;
        let local = self.local();
        local.format_leading_comments(f)?;
        local.write(f)?;
        // `import * as all /* comment */ from 'mod'`
        //                  ^^^^^^^^^^^^ get comments that before `from` keyword to print
        // `f` is the first character of `from`
        let comments = f.context().comments().comments_before_character(local.span().start, b'f');
        write!(f, [space(), FormatTrailingComments::Comments(comments)])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, WithClause<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let should_insert_space_around_brackets = f.options().bracket_spacing.value();
        let format_comment = format_with(|f| {
            if self.with_entries().is_empty() {
                let comments = f.context().comments().comments_before(self.span.end);
                write!(f, [space(), FormatLeadingComments::Comments(comments)])?;
            }
            Ok(())
        });
        write!(
            f,
            [
                space(),
                format_comment,
                match self.keyword() {
                    WithClauseKeyword::With => "with",
                    WithClauseKeyword::Assert => "assert",
                },
                space(),
                self.with_entries(),
            ]
        )
    }
}

impl<'a> Format<'a> for AstNode<'a, Vec<'a, ImportAttribute<'a>>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, "{")?;
        if !self.is_empty() {
            let maybe_space = maybe_space(f.options().bracket_spacing.value());
            write!(f, [maybe_space])?;

            f.join_with(space())
                .entries_with_trailing_separator(self.iter(), ",", TrailingSeparator::Disallowed)
                .finish()?;

            write!(f, [maybe_space])?;
        }
        write!(f, "}")
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ImportAttribute<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        if let AstNodes::StringLiteral(s) = self.key().as_ast_nodes() {
            if f.options().quote_properties == QuoteProperties::AsNeeded
                && is_identifier_name(s.value().as_str())
            {
                dynamic_text(s.value().as_str()).fmt(f)?;
            } else {
                s.fmt(f)?;
            }
        } else {
            write!(f, self.key())?;
        }
        write!(f, [":", space(), self.value()])
    }
}
