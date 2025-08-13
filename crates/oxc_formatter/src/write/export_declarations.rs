use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{
    FormatResult, FormatTrailingCommas,
    formatter::{
        Formatter, prelude::*, separated::FormatSeparatedIter, trivia::FormatLeadingComments,
    },
    generated::ast_nodes::{AstNode, AstNodes},
    write,
    write::semicolon::OptionalSemicolon,
};

use super::FormatWrite;

fn format_export_keyword_with_class_decorators<'a>(
    span: Span,
    keyword: &'static str,
    declaration: &AstNodes<'a>,
    f: &mut Formatter<'_, 'a>,
) -> FormatResult<()> {
    // `@decorator export class Cls {}`
    //            ^ print leading comments here
    let format_leading_comments = |f: &mut Formatter<'_, 'a>| -> FormatResult<()> {
        let comments = f.context().comments().comments_before(span.start);
        if !comments.is_empty() {
            FormatLeadingComments::Comments(comments).fmt(f)?;
        }
        Ok(())
    };

    if let AstNodes::Class(class) = declaration
        && !class.decorators.is_empty()
    {
        // `@decorator export class Cls {}`
        // decorators are placed before the export keyword
        if class.decorators[0].span.end < span.start {
            write!(f, [class.decorators(), hard_line_break()])?;
            format_leading_comments(f)?;
            write!(f, [keyword, space()])
        } else {
            // `export @decorator class Cls {}`
            // decorators are placed after the export keyword
            format_leading_comments(f)?;
            write!(f, [keyword, hard_line_break()])?;
            write!(f, [class.decorators(), hard_line_break()])
        }
    } else {
        format_leading_comments(f)?;
        write!(f, [keyword, space()])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ExportDefaultDeclaration<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        format_export_keyword_with_class_decorators(
            self.span,
            "export default",
            self.declaration().as_ast_nodes(),
            f,
        )?;

        write!(f, self.declaration())?;
        if self.declaration().is_expression() {
            write!(f, OptionalSemicolon)?;
        }

        self.format_trailing_comments(f)
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ExportAllDeclaration<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["export", space(), self.export_kind(), "*", space()])?;
        if let Some(name) = &self.exported() {
            write!(f, ["as", space(), name, space()])?;
        }
        write!(f, ["from", space(), self.source(), self.with_clause(), OptionalSemicolon])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ExportNamedDeclaration<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let declaration = self.declaration();
        let export_kind = self.export_kind();
        let specifiers = self.specifiers();
        let source = self.source();
        let with_clause = self.with_clause();

        if let Some(decl) = declaration {
            format_export_keyword_with_class_decorators(
                self.span,
                "export",
                decl.as_ast_nodes(),
                f,
            )?;
            write!(f, decl)?;
        } else {
            self.format_leading_comments(f)?;
            write!(f, ["export", space()])?;

            let comments = f.context().comments().comments_before_character(self.span.start, b'{');
            if !comments.is_empty() {
                write!(f, [FormatLeadingComments::Comments(comments)])?;
            }
            write!(f, [export_kind, "{"])?;
            if specifiers.is_empty() {
                write!(f, [format_dangling_comments(self.span).with_block_indent()])?;
            } else {
                let should_insert_space_around_brackets = f.options().bracket_spacing.value();
                write!(
                    f,
                    group(&soft_block_indent_with_maybe_space(
                        &specifiers,
                        should_insert_space_around_brackets
                    ))
                )?;
            }
            write!(f, [export_kind, "}"])?;

            if let Some(source) = source {
                write!(f, [space(), "from", space(), source])?;
            }

            if let Some(with_clause) = with_clause {
                write!(f, [space(), with_clause])?;
            }
        }

        if declaration.is_none_or(|d| matches!(d.as_ref(), Declaration::VariableDeclaration(_))) {
            write!(f, OptionalSemicolon)?;
        }

        self.format_trailing_comments(f)
    }
}

impl<'a> Format<'a> for AstNode<'a, Vec<'a, ExportSpecifier<'a>>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let trailing_separator = FormatTrailingCommas::ES5.trailing_separator(f.options());
        let mut joiner = f.join_with(soft_line_break_or_space());
        for specifier in
            FormatSeparatedIter::new(self.iter(), ",").with_trailing_separator(trailing_separator)
        {
            joiner.entry(&format_once(|f| {
                // Should add empty line before the specifier if there are comments before it.
                let comments =
                    f.context().comments().comments_before(specifier.element.span().start);
                if !comments.is_empty() {
                    if get_lines_before(comments[0].span, f) > 1 {
                        write!(f, [empty_line()])?;
                    }
                    write!(f, [FormatLeadingComments::Comments(comments)])?;
                }

                write!(f, specifier)
            }));
        }

        joiner.finish()
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, ExportSpecifier<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let comments = f.context().comments().comments_before(self.exported.span().end);
        let mut len = comments.len();
        while len != 0 && comments[len - 1].is_block() {
            len -= 1;
        }
        if len != 0 {
            write!(f, [FormatLeadingComments::Comments(&comments[..len])])?;
        }

        write!(f, [self.export_kind()]);
        if self.local.span() == self.exported.span() {
            write!(f, self.exported())
        } else {
            write!(f, [self.local(), space(), "as", space(), self.exported()])
        }
    }
}
