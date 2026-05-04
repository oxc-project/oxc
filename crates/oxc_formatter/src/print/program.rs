use std::ops::Deref;

use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_span::GetSpan;
use oxc_syntax::identifier::ZWNBSP;

use crate::{
    Buffer, Format,
    ast_nodes::AstNode,
    formatter::{prelude::*, trivia::FormatTrailingComments},
    ir_transform::sort_imports_chunk,
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
                FormatStatementsWithImports(self.body()),
                format_trailing_comments,
                hard_line_break()
            ]
        );
    }
}

pub(super) struct FormatStatementsWithImports<'a, 'b>(pub &'b AstNode<'a, Vec<'a, Statement<'a>>>);

impl<'a> Deref for FormatStatementsWithImports<'a, '_> {
    type Target = AstNode<'a, Vec<'a, Statement<'a>>>;
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'a> Format<'a> for FormatStatementsWithImports<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let import_sort_enabled = f.options().sort_imports.is_some();
        let mut imports_chunk_start: Option<usize> = None;

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

            if import_sort_enabled {
                if matches!(stmt.as_ref(), Statement::ImportDeclaration(_)) {
                    if imports_chunk_start.is_none() {
                        // First import in a chunk. Output inter-statement separator separately
                        // so `imports_chunk_start` points to start of IR for the `ImportDeclaration` itself.
                        join.separator_no_entry(span);
                        imports_chunk_start = Some(join.fmt().elements().len());
                        join.entry_no_separator(stmt);
                        continue;
                    }
                } else if let Some(chunk_start) = imports_chunk_start.take() {
                    // Any other statement after an `ImportDeclaration`, or a run of them.
                    // Sort the chunk of imports.
                    sort_imports_chunk(join.fmt_mut(), chunk_start);
                }
            }

            join.entry(span, stmt);
        }

        // If last statement was an `ImportDeclaration`, sort the chunk of imports
        if let Some(chunk_start) = imports_chunk_start.take() {
            sort_imports_chunk(join.fmt_mut(), chunk_start);
        }
    }
}

impl<'a> Format<'a> for AstNode<'a, Vec<'a, Directive<'a>>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let Some(last_directive) = self.last() else {
            // No directives, no extra new line
            return;
        };

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

        // If the last directive has a trailing comment, `lines_after` stops at the first
        // non-whitespace character (`/`) and returns 0 before counting any newlines.
        let check_pos = f
            .context()
            .comments()
            .end_of_line_comments_after(last_directive.span.end)
            .last()
            .map_or(last_directive.span.end, |c| c.span.end);
        let need_extra_empty_line = f.source_text().lines_after(check_pos) > 1;

        f.join_nodes_with_hardline().entries(self);

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
