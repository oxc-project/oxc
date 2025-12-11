use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{
    Format, FormatTrailingCommas, JsLabels, TrailingSeparator,
    ast_nodes::{AstNode, AstNodes},
    format_args,
    formatter::{
        Formatter, prelude::*, separated::FormatSeparatedIter, trivia::FormatLeadingComments,
    },
    utils::string::{
        FormatLiteralStringToken, StringLiteralParentKind, is_identifier_name_patched,
    },
    write,
    write::semicolon::OptionalSemicolon,
};

use super::FormatWrite;

impl<'a> Format<'a> for ImportOrExportKind {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        if self.is_type() {
            write!(f, ["type", space()]);
        }
    }
}

pub fn format_import_and_export_source_with_clause<'a>(
    source: &AstNode<'a, StringLiteral>,
    with_clause: Option<&AstNode<'a, WithClause>>,
    f: &mut Formatter<'_, 'a>,
) {
    source.fmt(f);

    if let Some(with_clause) = with_clause {
        if f.comments().has_comment_before(with_clause.span.start) {
            write!(f, [space()]);
        }

        write!(f, [with_clause]);
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ImportDeclaration<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) {
        let decl = &format_with(|f| {
            write!(f, ["import", space(), self.import_kind]);

            if let Some(specifiers) = self.specifiers() {
                write!(f, [specifiers, space(), "from", space()]);
            }

            format_import_and_export_source_with_clause(self.source(), self.with_clause(), f);

            write!(f, [OptionalSemicolon]);
        });

        write!(f, [labelled(LabelId::of(JsLabels::ImportDeclaration), decl)]);
    }
}

impl<'a> Format<'a> for AstNode<'a, Vec<'a, ImportDeclarationSpecifier<'a>>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let mut specifiers_iter = self.iter().peekable();

        while let Some(specifier) = specifiers_iter.peek() {
            match specifier.as_ref() {
                ImportDeclarationSpecifier::ImportDefaultSpecifier(_)
                | ImportDeclarationSpecifier::ImportNamespaceSpecifier(_) => {
                    write!(f, [specifiers_iter.next().unwrap()]);
                }
                ImportDeclarationSpecifier::ImportSpecifier(_) => {
                    break;
                }
            }

            if specifiers_iter.peek().is_some() {
                write!(f, [",", space()]);
            } else {
                return;
            }
        }

        let should_insert_space_around_brackets = f.options().bracket_spacing.value();

        if self.is_empty() {
            write!(f, ["{}"]);
        } else if self.len() == 1
            && matches!(
                specifiers_iter.peek().map(AsRef::as_ref),
                Some(ImportDeclarationSpecifier::ImportSpecifier(_))
            )
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
            );
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
                                    format_with(move |f| {
                                        // Should add empty line before the specifier if there are comments before it.
                                        let specifier_span = specifier.span();
                                        if f.context()
                                            .comments()
                                            .has_comment_before(specifier_span.start)
                                            && f.source_text()
                                                .get_lines_before(specifier_span, f.comments())
                                                > 1
                                        {
                                            write!(f, [empty_line()]);
                                        }
                                        write!(f, specifier);
                                    })
                                });
                            f.join_with(soft_line_break_or_space()).entries(iter);
                        }),
                        should_insert_space_around_brackets
                    )),
                    "}"
                ]
            );
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ImportSpecifier<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) {
        let comments = f.context().comments().line_comments_before(self.local.span.end);
        write!(f, [FormatLeadingComments::Comments(comments), self.import_kind()]);
        if self.local.span == self.imported.span() {
            write!(f, [self.local()]);
        } else {
            write!(f, [self.imported(), space(), "as", space(), self.local()]);
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ImportDefaultSpecifier<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) {
        write!(f, [self.local()]);
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ImportNamespaceSpecifier<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) {
        write!(f, ["*", space(), "as", space(), self.local()]);
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, WithClause<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) {
        if f.options().quote_properties.is_consistent() {
            let quote_needed = self.with_entries.iter().any(|attribute| {
                matches!(&attribute.key, ImportAttributeKey::StringLiteral(string) if {
                    let quote_less_content = f.source_text().text_for(&string.span.shrink(1));
                    !is_identifier_name_patched(quote_less_content)
                })
            });

            f.context_mut().push_quote_needed(quote_needed);
        }

        let format_comment = format_with(|f| {
            if self.with_entries().is_empty() {
                let comments = f.context().comments().comments_before(self.span.end);
                write!(f, [space(), FormatLeadingComments::Comments(comments)]);
            }
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
                self.with_entries()
            ]
        );

        if f.options().quote_properties.is_consistent() {
            f.context_mut().pop_quote_needed();
        }
    }
}

impl<'a> Format<'a> for AstNode<'a, Vec<'a, ImportAttribute<'a>>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        if self.is_empty() {
            return write!(f, "{}");
        }

        let format_inner = format_with(|f| {
            let should_insert_space_around_brackets = f.options().bracket_spacing.value();

            write!(f, "{");

            if self.len() > 1
                || self.first().is_some_and(|attribute| attribute.key.as_atom().as_str() != "type")
                || f.comments().has_comment_before(self.parent.span().end)
            {
                write!(
                    f,
                    [soft_block_indent_with_maybe_space(
                        &format_with(|f| {
                            let trailing_separator =
                                FormatTrailingCommas::ES5.trailing_separator(f.options());

                            f.join_with(soft_line_break()).entries_with_trailing_separator(
                                self.iter(),
                                ",",
                                trailing_separator,
                            );
                        },),
                        should_insert_space_around_brackets
                    )]
                );
            } else {
                write!(
                    f,
                    [format_with(|f| {
                        let maybe_space = maybe_space(f.options().bracket_spacing.value());
                        write!(f, [maybe_space]);

                        f.join_with(space()).entries_with_trailing_separator(
                            self.iter(),
                            ",",
                            TrailingSeparator::Disallowed,
                        );

                        write!(f, [maybe_space]);
                    })]
                );
            }

            write!(f, "}");
        });

        let first = self.as_ref().first().unwrap();

        write!(
            f,
            group(&format_inner)
                .should_expand(f.source_text().has_newline_before(first.span.start))
        );
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ImportAttribute<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) {
        if let AstNodes::StringLiteral(string) = self.key().as_ast_nodes() {
            let format = FormatLiteralStringToken::new(
                f.source_text().text_for(string),
                false,
                StringLiteralParentKind::ImportAttribute,
            )
            .clean_text(f);

            string.format_leading_comments(f);
            write!(f, format);
            string.format_trailing_comments(f);
        } else {
            write!(f, self.key());
        }
        write!(f, [":", space()]);

        let has_leading_own_line_comment =
            f.comments().has_leading_own_line_comment(self.value.span.start);

        if has_leading_own_line_comment {
            write!(f, [group(&indent(&format_args!(soft_line_break(), self.value())))]);
        } else {
            write!(f, [self.value()]);
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSImportEqualsDeclaration<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) {
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
        );
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSExternalModuleReference<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) {
        write!(f, ["require("]);

        if f.comments().has_comment_in_span(self.span) {
            write!(f, [block_indent(self.expression())]);
        } else {
            write!(f, [self.expression()]);
        }

        write!(f, [")"]);
    }
}
