use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_span::GetSpan;
use oxc_syntax::identifier::is_identifier_name;

use crate::{
    Format, FormatResult, FormatTrailingCommas, JsLabels, QuoteProperties, TrailingSeparator,
    ast_nodes::{AstNode, AstNodes},
    best_fitting, format_args,
    formatter::{
        Formatter,
        prelude::*,
        separated::FormatSeparatedIter,
        trivia::{FormatLeadingComments, FormatTrailingComments},
    },
    utils::format_node_without_trailing_comments::FormatNodeWithoutTrailingComments,
    write,
    write::semicolon::OptionalSemicolon,
};

use super::FormatWrite;

impl<'a> Format<'a> for ImportOrExportKind {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        if self.is_type() { write!(f, ["type", space()]) } else { Ok(()) }
    }
}

pub fn format_import_and_export_source_with_clause<'a>(
    source: &AstNode<'a, StringLiteral>,
    with_clause: Option<&AstNode<'a, WithClause>>,
    f: &mut Formatter<'_, 'a>,
) -> FormatResult<()> {
    FormatNodeWithoutTrailingComments(source).fmt(f)?;

    if let Some(with_clause) = with_clause {
        if f.comments().has_comment_before(with_clause.span.start) {
            write!(f, [space()])?;
        }

        write!(f, [with_clause])?;
    }

    Ok(())
}

impl<'a> FormatWrite<'a> for AstNode<'a, ImportDeclaration<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let decl = &format_once(|f| {
            write!(f, ["import", space(), self.import_kind])?;

            if let Some(specifiers) = self.specifiers() {
                write!(f, [specifiers, space(), "from", space()])?;
            }

            format_import_and_export_source_with_clause(self.source(), self.with_clause(), f)?;

            write!(f, [OptionalSemicolon])
        });

        write!(f, [labelled(LabelId::of(JsLabels::ImportDeclaration), decl)])
    }
}

impl<'a> Format<'a> for AstNode<'a, Vec<'a, ImportDeclarationSpecifier<'a>>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let mut specifiers_iter = self.iter().peekable();

        while let Some(specifier) = specifiers_iter.peek() {
            match specifier.as_ref() {
                ImportDeclarationSpecifier::ImportDefaultSpecifier(_)
                | ImportDeclarationSpecifier::ImportNamespaceSpecifier(_) => {
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
            && f.comments().comments_before_character(self.parent.span().start, b'}').is_empty()
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
                                        let specifier_span = specifier.span();
                                        if f.context()
                                            .comments()
                                            .has_comment_before(specifier_span.start)
                                            && f.source_text()
                                                .get_lines_before(specifier_span, f.comments())
                                                > 1
                                        {
                                            write!(f, [empty_line()])?;
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
            write!(f, [self.local()])
        } else {
            write!(f, [self.imported(), space(), "as", space(), self.local()])
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ImportDefaultSpecifier<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.local()])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ImportNamespaceSpecifier<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["*", space(), "as", space(), self.local()])
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

impl<'a> FormatWrite<'a> for AstNode<'a, TSImportEqualsDeclaration<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(
            f,
            [
                "import",
                space(),
                self.import_kind(),
                self.id(),
                space(),
                "=",
                space(),
                self.module_reference(),
                OptionalSemicolon
            ]
        )
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSExternalModuleReference<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["require(", self.expression(), ")"])
    }
}
