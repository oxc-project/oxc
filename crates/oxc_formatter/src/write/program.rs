use std::ops::Deref;

use oxc_allocator::{Address, Vec};
use oxc_ast::{ast::*, match_expression};
use oxc_span::GetSpan;
use oxc_syntax::identifier::{ZWNBSP, is_line_terminator};

use crate::{
    Buffer, Format, FormatResult, FormatTrailingCommas, TrailingSeparator, format_args,
    formatter::{prelude::*, separated::FormatSeparatedIter, trivia::FormatTrailingComments},
    generated::ast_nodes::{AstNode, AstNodes},
    utils::{
        call_expression::is_test_call_expression,
        is_long_curried_call,
        member_chain::simple_argument::SimpleArgument,
        string_utils::{FormatLiteralStringToken, StringLiteralParentKind},
    },
    write,
    write::semicolon::OptionalSemicolon,
};

use super::FormatWrite;

impl<'a> FormatWrite<'a> for AstNode<'a, Program<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let format_trailing_comments = format_once(|f| {
            let comments = f.context().comments().comments_before(self.span.end);
            FormatTrailingComments::Comments(comments).fmt(f)
        });

        write!(
            f,
            [
                // BOM
                f.source_text().chars().next().is_some_and(|c| c == ZWNBSP).then_some("\u{feff}"),
                self.hashbang(),
                self.directives(),
                FormatProgramBody(self.body()),
                format_trailing_comments,
                hard_line_break()
            ]
        )
    }
}

struct FormatProgramBody<'a, 'b>(&'b AstNode<'a, Vec<'a, Statement<'a>>>);

impl<'a> Deref for FormatProgramBody<'a, '_> {
    type Target = AstNode<'a, Vec<'a, Statement<'a>>>;
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'a> Format<'a> for FormatProgramBody<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let mut join = f.join_nodes_with_hardline();
        for stmt in
            self.iter().filter(|stmt| !matches!(stmt.as_ref(), Statement::EmptyStatement(_)))
        {
            let span = match stmt.as_ref() {
                // `@decorator export class A {}`
                // Get the span of the decorator.
                Statement::ExportNamedDeclaration(export) => {
                    if let Some(Declaration::ClassDeclaration(decl)) = &export.declaration
                        && let Some(decorator) = decl.decorators.first()
                        && decorator.span().start < export.span.start
                    {
                        decorator.span()
                    } else {
                        export.span
                    }
                }
                // `@decorator export default class A {}`
                // Get the span of the decorator.
                Statement::ExportDefaultDeclaration(export) => {
                    if let ExportDefaultDeclarationKind::ClassDeclaration(decl) =
                        &export.declaration
                        && let Some(decorator) = decl.decorators.first()
                        && decorator.span().start < export.span.start
                    {
                        decorator.span()
                    } else {
                        export.span
                    }
                }
                _ => stmt.span(),
            };

            join.entry(span, stmt);
        }
        join.finish()
    }
}

impl<'a> Format<'a> for AstNode<'a, Vec<'a, Directive<'a>>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let Some(last_directive) = self.last() else {
            // No directives, no extra new line
            return Ok(());
        };

        f.join_nodes_with_hardline().entries(self).finish()?;

        // if next_sibling's first leading_trivia has more than one new_line, we should add an extra empty line at the end of
        // JsDirectiveList, for example:
        //```js
        // "use strict"; <- first leading new_line
        //  			 <- second leading new_line
        // function foo() {

        // }
        //```
        // so we should keep an extra empty line after JsDirectiveList
        let source_text = f.context().source_text();
        let mut count = 0;
        let mut source_text_chars = source_text.slice_from(last_directive.span.end).chars();
        for char in source_text_chars.by_ref() {
            if is_line_terminator(char) {
                count += 1;
            } else if !char.is_whitespace() {
                break;
            }
        }

        // Need an extra empty line if it has the following line and still has non-characters after whitespace.
        let need_extra_empty_line = source_text_chars.next().is_some() && count > 1;
        write!(f, if need_extra_empty_line { empty_line() } else { hard_line_break() })
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, Directive<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(
            f,
            [
                FormatLiteralStringToken::new(
                    f.source_text().text_for(self.expression()),
                    self.expression().span(),
                    /* jsx */
                    false,
                    StringLiteralParentKind::Directive,
                ),
                OptionalSemicolon
            ]
        )
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, Hashbang<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["#!", dynamic_text(self.value().as_str().trim_end())])?;

        if f.source_text().lines_after(self.span.end) > 1 {
            write!(f, [empty_line()])
        } else {
            write!(f, [hard_line_break()])
        }
    }
}
