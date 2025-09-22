use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{
    FormatResult, FormatTrailingCommas,
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
        FormatLeadingComments::Comments(comments).fmt(f)?;
        Ok(())
    };

    if let AstNodes::Class(class) = declaration
        && !class.decorators.is_empty()
        && !class.is_expression()
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
                    )?;
                }
                write!(
                    f,
                    [export_kind, "{", format_dangling_comments(self.span).with_block_indent()]
                )?;
            } else if specifiers.len() == 1
                && f.comments().comments_before_character(self.span.start, b'}').is_empty()
            {
                let space = maybe_space(needs_space).memoized();
                write!(f, [export_kind, "{", space, specifiers.first(), space])?;
            } else {
                write!(
                    f,
                    [
                        export_kind,
                        "{",
                        group(&soft_block_indent_with_maybe_space(&specifiers, needs_space))
                    ]
                )?;
            }
            write!(f, "}")?;

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
        f.join_with(soft_line_break_or_space())
            .entries(
                FormatSeparatedIter::new(self.iter(), ",")
                    .with_trailing_separator(trailing_separator)
                    .map(|specifier| {
                        format_once(move |f| {
                            // Should add empty line before the specifier if there are comments before it.
                            let comments =
                                f.context().comments().comments_before(specifier.span().start);
                            if !comments.is_empty() {
                                if f.source_text().get_lines_before(comments[0].span, f.comments())
                                    > 1
                                {
                                    write!(f, [empty_line()])?;
                                }
                                write!(f, [FormatLeadingComments::Comments(comments)])?;
                            }

                            write!(f, specifier)
                        })
                    }),
            )
            .finish()
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
            write!(f, self.exported())?;
        } else {
            write!(f, [self.local(), space(), "as", space(), self.exported()])?;
        }

        if f.source_text().next_non_whitespace_byte_is(self.span.end, b'}') {
            // `export { a as b /* comment */ } from 'mod'
            //                  ^^^^^^^^^^^^ get comments that before `}` to print
            let comments = f.context().comments().comments_before_character(self.span.end, b'}');
            write!(f, [FormatTrailingComments::Comments(comments)])
        } else {
            self.format_trailing_comments(f)
        }
    }
}
