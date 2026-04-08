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

        // If the last directive has a trailing comment, `lines_after` stops at the first
        // non-whitespace character (`/`) and returns 0 before counting any newlines.
        // Skip past trailing `//` and `/* */` comments (including multi-line block comments)
        // to find the position where blank lines should be counted from.
        let after = f.source_text().slice_from(last_directive.span.end);
        let check_pos =
            last_directive.span.end + u32::try_from(skip_trailing_comments(after)).unwrap_or(0);
        let need_extra_empty_line = f.source_text().lines_after(check_pos) > 1;
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

/// Returns the byte offset past any trailing `//` or `/* */` comments (including multi-line block
/// comments) in `s`, stopping at the first newline or non-comment, non-whitespace character.
/// Used to position `lines_after` correctly when a directive has a trailing comment.
fn skip_trailing_comments(s: &str) -> usize {
    let bytes = s.as_bytes();
    let mut i = 0;
    loop {
        // Skip single-line whitespace (space, tab).
        while i < bytes.len() && matches!(bytes[i], b' ' | b'\t') {
            i += 1;
        }
        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'/' {
            // Line comment: advance to end of line so `lines_after` counts from the newline.
            while i < bytes.len() && !matches!(bytes[i], b'\n' | b'\r') {
                i += 1;
            }
            return i;
        }
        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            // Block comment: skip past the closing `*/` then continue (handles multi-line).
            i += 2;
            while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                i += 1;
            }
            i += 2; // skip `*/`
            continue;
        }
        // Newline or non-comment content — stop here.
        return i;
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
