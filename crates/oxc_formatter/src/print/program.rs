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

        let mut join = f.join_nodes_with_hardline();

        let mut stmts_iter =
            self.iter().filter(|stmt| !matches!(stmt.as_ref(), Statement::EmptyStatement(_)));
        while let Some(mut stmt) = stmts_iter.next() {
            // If import sort is enabled, and current statement is an `ImportDeclaration`,
            // collect consecutive `ImportDeclaration`s starting with `stmt`, sort them, and output them
            if import_sort_enabled && matches!(stmt.as_ref(), Statement::ImportDeclaration(_)) {
                let next_stmt = format_import_decls_with_sort(stmt, &mut stmts_iter, &mut join);
                match next_stmt {
                    Some(next_stmt) => stmt = next_stmt,
                    None => break,
                }
            }

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
    }
}

/// Collect a run of consecutive `ImportDeclaration`s from `stmts_iter`, format them using `join`,
/// then sort them in place.
///
/// Returns the next statement after the run of `ImportDeclaration`s, or `None` if there are no more statements.
///
/// The caller must already have verified that `sort_imports` option is enabled.
///
/// # Panics
/// Panics if `sort_imports` option is not enabled.
//
// `#[cold]` because most statements aren't `ImportDeclaration`s.
// Also, when there *are* lots of `ImportDeclaration`s, they tend to all be grouped together.
// This function consumes the whole run, so is unlikely to be called more than once, even in files with lots of imports.
#[cold]
fn format_import_decls_with_sort<'a, 'iter>(
    stmt: &AstNode<'a, Statement<'a>>,
    stmts_iter: &mut impl Iterator<Item = &'iter AstNode<'a, Statement<'a>>>,
    join: &mut JoinNodesBuilder<'_, '_, 'a, Line>,
) -> Option<&'iter AstNode<'a, Statement<'a>>> {
    // Output inter-statement separator separately, so `chunk_start` points
    // to start of IR for the `ImportDeclaration` itself
    join.separator_no_entry(stmt.span());
    let chunk_start = join.fmt().elements().len();

    // Output first `ImportDeclaration`
    join.entry_no_separator(stmt);

    // Output all following `ImportDeclaration`s.
    // The first import was already written above, so start the count at 1.
    let mut count = 1;
    let mut next_stmt = None;
    for stmt in stmts_iter {
        if let Statement::ImportDeclaration(decl) = stmt.as_ref() {
            join.entry(decl.span, stmt);
            count += 1;
        } else {
            // Some other statement
            next_stmt = Some(stmt);
            break;
        }
    }

    // Sort the run of `ImportDeclaration`s.
    // A single-import run is already in order, so skip the transform.
    if count >= 2 {
        sort_imports_chunk(join.fmt_mut(), chunk_start);
    }

    // Return the next statement (which isn't an `ImportDeclaration`)
    next_stmt
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
