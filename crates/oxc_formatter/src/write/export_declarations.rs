use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{
    FormatTrailingCommas,
    ast_nodes::{AstNode, AstNodes},
    formatter::{
        Formatter,
        prelude::*,
        separated::FormatSeparatedIter,
        trivia::{FormatLeadingComments, FormatTrailingComments},
    },
    write,
    write::{
        import_declaration::format_import_and_export_source_with_clause,
        semicolon::OptionalSemicolon,
    },
};

use super::FormatWrite;

fn format_export_keyword_with_class_decorators<'a>(
    span: Span,
    keyword: &'static str,
    declaration: &AstNodes<'a>,
    f: &mut Formatter<'_, 'a>,
) {
    // `@decorator export class Cls {}`
    //            ^ print leading comments here
    let format_leading_comments = format_with(|f| {
        let comments = f.context().comments().comments_before(span.start);
        FormatLeadingComments::Comments(comments).fmt(f);
    });

    if let AstNodes::Class(class) = declaration
        && !class.decorators.is_empty()
        && !class.is_expression()
    {
        // `@decorator export class Cls {}`
        // decorators are placed before the export keyword
        if class.decorators[0].span.end < span.start {
            write!(
                f,
                [class.decorators(), hard_line_break(), format_leading_comments, keyword, space()]
            );
        } else {
            // `export @decorator class Cls {}`
            // decorators are placed after the export keyword
            write!(
                f,
                [
                    format_leading_comments,
                    keyword,
                    hard_line_break(),
                    class.decorators(),
                    hard_line_break()
                ]
            );
        }
    } else {
        write!(f, [format_leading_comments, keyword, space()]);
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ExportDefaultDeclaration<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) {
        format_export_keyword_with_class_decorators(
            self.span,
            "export default",
            self.declaration().as_ast_nodes(),
            f,
        );

        write!(f, self.declaration());
        if self.declaration().is_expression() {
            write!(f, OptionalSemicolon);
        }

        self.format_trailing_comments(f);
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ExportAllDeclaration<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) {
        write!(f, ["export", space(), self.export_kind(), "*", space()]);
        if let Some(name) = &self.exported() {
            write!(f, ["as", space(), name, space()]);
        }
        write!(f, ["from", space()]);

        format_import_and_export_source_with_clause(self.source(), self.with_clause(), f);
        write!(f, [OptionalSemicolon]);
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ExportNamedDeclaration<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) {
        let declaration = self.declaration();
        let export_kind = self.export_kind();
        let specifiers = self.specifiers();
        let source = self.source();

        if let Some(decl) = declaration {
            format_export_keyword_with_class_decorators(
                self.span,
                "export",
                decl.as_ast_nodes(),
                f,
            );
            write!(f, decl);
        } else {
            self.format_leading_comments(f);
            write!(f, ["export", space()]);

            let needs_space = f.options().bracket_spacing.value();
            if specifiers.is_empty() {
                let comments =
                    f.context().comments().comments_before_character(self.span.start, b'{');
                let has_line_comment = comments.iter().any(|c| c.is_line());
                // Block comment example:
                // Input:  `export /* comment */ {}`
                // Output: `export /* comment */ {}`
                //
                // Line comment example:
                // Input:  `export // comment
                //         {}`
                // Output: `export // comment
                //          {}`
                if !comments.is_empty() {
                    write!(
                        f,
                        [
                            FormatTrailingComments::Comments(comments),
                            has_line_comment.then_some(soft_line_break()),
                            " "
                        ]
                    );
                }
                write!(
                    f,
                    [export_kind, "{", format_dangling_comments(self.span).with_block_indent()]
                );
            } else if specifiers.len() == 1
                && f.comments().comments_before_character(self.span.start, b'}').is_empty()
            {
                let space = maybe_space(needs_space).memoized();
                write!(f, [export_kind, "{", space, specifiers.first(), space]);
            } else {
                write!(
                    f,
                    [
                        export_kind,
                        "{",
                        group(&soft_block_indent_with_maybe_space(specifiers, needs_space))
                    ]
                );
            }
            write!(f, "}");

            let with_clause = self.with_clause();
            if let Some(source) = source {
                write!(f, [space(), "from", space()]);
                format_import_and_export_source_with_clause(source, with_clause, f);
            }
        }

        if declaration.is_none_or(|d| matches!(d.as_ref(), Declaration::VariableDeclaration(_))) {
            write!(f, OptionalSemicolon);
        }

        self.format_trailing_comments(f);
    }
}

impl<'a> Format<'a> for AstNode<'a, Vec<'a, ExportSpecifier<'a>>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let trailing_separator = FormatTrailingCommas::ES5.trailing_separator(f.options());
        f.join_with(soft_line_break_or_space()).entries(
            FormatSeparatedIter::new(self.iter(), ",")
                .with_trailing_separator(trailing_separator)
                .map(|specifier| {
                    format_with(move |f| {
                        // Should add empty line before the specifier if there are comments before it.
                        let specifier_span = specifier.span();
                        if f.context().comments().has_comment_before(specifier_span.start)
                            && f.source_text().get_lines_before(specifier_span, f.comments()) > 1
                        {
                            write!(f, [empty_line()]);
                        }

                        write!(f, specifier);
                    })
                }),
        );
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ExportSpecifier<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) {
        let comments = f.context().comments().line_comments_before(self.exported.span().end);
        write!(f, [FormatLeadingComments::Comments(comments)]);

        write!(f, [self.export_kind()]);
        if self.local.span() == self.exported.span() {
            write!(f, self.exported());
        } else {
            write!(f, [self.local(), space(), "as", space(), self.exported()]);
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSExportAssignment<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) {
        write!(f, ["export = ", self.expression(), OptionalSemicolon]);
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSNamespaceExportDeclaration<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) {
        write!(f, ["export as namespace ", self.id(), OptionalSemicolon]);
    }
}
