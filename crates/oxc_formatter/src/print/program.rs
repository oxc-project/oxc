use std::ops::Deref;

use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_span::GetSpan;
use oxc_syntax::identifier::ZWNBSP;

use crate::{
    Buffer, Format,
    ast_nodes::AstNode,
    formatter::{prelude::*, trivia::FormatTrailingComments},
    print::semicolon::OptionalSemicolon,
    utils::string::{FormatLiteralStringToken, StringLiteralParentKind},
    write,
};

use super::FormatWrite;

impl<'a> FormatWrite<'a> for AstNode<'a, Program<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) {
        let format_trailing_comments = format_with(|f| {
            write!(
                f,
                FormatTrailingComments::Comments(f.context().comments().unprinted_comments())
            );
        });

        write!(
            f,
            [
                // BOM
                f.source_text()
                    .chars()
                    .next()
                    .is_some_and(|c| c == ZWNBSP)
                    .then_some(text("\u{feff}")),
                self.hashbang(),
                self.directives(),
                FormatProgramBody(self.body()),
                format_trailing_comments,
                hard_line_break()
            ]
        );
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
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let mut join = f.join_nodes_with_hardline();
        let mut prev_is_function = false;

        for stmt in
            self.iter().filter(|stmt| !matches!(stmt.as_ref(), Statement::EmptyStatement(_)))
        {
            let is_function = is_function_like_declaration(stmt.as_ref());
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

            // Force an empty line between consecutive function declarations
            if prev_is_function && is_function && !join.has_lines_before(span) {
                join.entry_with_forced_empty_line(stmt);
            } else {
                join.entry(span, stmt);
            }

            prev_is_function = is_function;
        }
    }
}

impl<'a> Format<'a> for AstNode<'a, Vec<'a, Directive<'a>>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let Some(last_directive) = self.last() else {
            // No directives, no extra new line
            return;
        };

        f.join_nodes_with_hardline().entries(self);

        // if next_sibling's first leading_trivia has more than one new_line, we should add an extra empty line at the end of
        // the last directive, for example:
        //```js
        // "use strict"; <- first leading new_line
        //  			 <- second leading new_line
        // function foo() {
        //
        // }
        //```
        // so we should keep an extra empty line after the last directive.

        let need_extra_empty_line = f.source_text().lines_after(last_directive.span.end) > 1;
        write!(f, if need_extra_empty_line { empty_line() } else { hard_line_break() });
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, Directive<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) {
        write!(
            f,
            [
                FormatLiteralStringToken::new(
                    f.source_text().text_for(&self.expression),
                    /* jsx */
                    false,
                    StringLiteralParentKind::Directive,
                ),
                OptionalSemicolon
            ]
        );
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, Hashbang<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) {
        write!(f, ["#!", text(self.value().as_str().trim_end())]);

        if f.source_text().lines_after(self.span.end) > 1 {
            write!(f, [empty_line()]);
        } else {
            write!(f, [hard_line_break()]);
        }
    }
}

/// Returns `true` if the statement is a function declaration,
/// including exported function declarations.
pub(crate) fn is_function_like_declaration(stmt: &Statement<'_>) -> bool {
    match stmt {
        Statement::FunctionDeclaration(_) => true,
        Statement::ExportNamedDeclaration(export) => {
            matches!(export.declaration, Some(Declaration::FunctionDeclaration(_)))
        }
        Statement::ExportDefaultDeclaration(export) => {
            matches!(export.declaration, ExportDefaultDeclarationKind::FunctionDeclaration(_))
        }
        _ => false,
    }
}
