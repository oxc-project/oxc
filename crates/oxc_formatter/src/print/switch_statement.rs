use oxc_allocator::ArenaVec;
use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{
    Format,
    ast_nodes::{AstNode, AstNodes},
    format_args,
    formatter::{
        JsFormatter,
        prelude::*,
        trivia::{DanglingIndentMode, FormatDanglingComments},
    },
    utils::is_dropped_statement,
    write,
};

use super::FormatWrite;

impl<'a> FormatWrite<'a> for AstNode<'a, SwitchStatement<'a>> {
    fn write(&self, f: &mut JsFormatter<'_, 'a>) {
        let discriminant = self.discriminant();
        let cases = self.cases();
        let format_cases = format_with(|f| {
            if cases.is_empty() {
                // Comments inside empty braces (`switch (a) { /* comment */ }`)
                // would otherwise leak behind the closing brace.
                if f.context().comments().has_comment_before(self.span.end) {
                    format_dangling_comments(self.span).fmt(f);
                } else {
                    hard_line_break().fmt(f);
                }
            } else {
                cases.fmt(f);
            }
        });
        write!(
            f,
            [
                "switch",
                space(),
                "(",
                group(&soft_block_indent(&discriminant)),
                ")",
                space(),
                "{",
                block_indent(&format_cases),
                "}"
            ]
        );
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for AstNode<'a, ArenaVec<'a, SwitchCase<'a>>> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        f.join_nodes_with_hardline().entries(self);
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, SwitchCase<'a>> {
    fn write(&self, f: &mut JsFormatter<'_, 'a>) {
        let is_default = if let Some(test) = self.test() {
            write!(f, ["case", space(), test, ":"]);
            false
        } else {
            write!(f, ["default", ":"]);
            true
        };

        let consequent = self.consequent();
        // When the case block is empty, the case becomes a fallthrough, so it
        // is collapsed directly on top of the next case (just a single
        // hardline).
        // When the block is a single statement _and_ it's a block statement,
        // then the opening brace of the block can hug the same line as the
        // case. But, if there's more than one statement, then the block
        // _cannot_ hug. This distinction helps clarify that the case continues
        // past the end of the block statement, despite the braces making it
        // seem like it might end.
        // Lastly, the default case is just to break and indent the body.
        //
        // switch (key) {
        //   case fallthrough: // trailing comment
        //   case normalBody:
        //     someWork();
        //     break;
        //
        //   case blockBody: {
        //     const a = 1;
        //     break;
        //   }
        //
        //   case separateBlockBody:
        //     {
        //       breakIsNotInsideTheBlock();
        //     }
        //     break;
        //
        //   default:
        //     break;
        // }
        if consequent.is_empty() {
            // Print nothing to ensure that trailing comments on the same line
            // are printed on the same line. The parent list formatter takes
            // care of inserting a hard line break between cases.
            return;
        }

        // The first statement in the clause when it is a `BlockStatement`
        // and there are no other non-empty statements.
        // Empties may show up when parsing depending on if the input code includes certain newlines.
        let first_statement = consequent.first().unwrap();
        let single_block_statement = match first_statement.as_ast_nodes() {
            AstNodes::BlockStatement(block)
                if consequent
                    .iter()
                    .skip(1)
                    .all(|statement| is_dropped_statement(statement.as_ref())) =>
            {
                Some(block)
            }
            _ => None,
        };
        let is_single_block_statement = single_block_statement.is_some();

        // Comments between the `:` and the clause body:
        // block comments before a single-block body print outside its `{` for every clause;
        // the end-of-line handling stays default-only
        // (after `case a:` they belong to the consequent's leading pass instead).
        let comments = f.context().comments();
        let comments = if is_single_block_statement {
            comments.block_comments_before(first_statement.span().start)
        } else if is_default {
            #[expect(clippy::cast_possible_truncation)]
            const DEFAULT_LEN: u32 = "default".len() as u32;
            comments.end_of_line_comments_after(self.span.start + DEFAULT_LEN)
        } else {
            &[]
        };

        if !comments.is_empty() {
            write!(
                f,
                [
                    space(),
                    FormatDanglingComments::Comments { comments, indent: DanglingIndentMode::None },
                ]
            );
        }

        if let Some(block) = single_block_statement {
            // The clause pulls pending line comments between the `:` and the `{`
            // INSIDE the block (`default: // c` + `{` -> `default: { // c`);
            // its block comments are printed outside above.
            // Use `write` to skip the block's leading-comments pass.
            write!(f, [space()]);
            block.write(f);
        } else {
            // no line break needed after because it is added by the indent in the switch statement
            write!(f, indent(&format_args!(hard_line_break(), consequent)));
        }
    }
}
