use oxc_allocator::ArenaVec;
use oxc_ast::ast::*;

use crate::{
    Format,
    ast_nodes::AstNode,
    format_args,
    formatter::{
        JsFormatter,
        prelude::*,
        trivia::{DanglingIndentMode, FormatDanglingComments},
    },
    utils::{
        is_dropped_statement,
        statement_body::{FormatStatementBody, write_node_with_trailing_comments_before},
    },
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
        // Whether the first statement in the clause is a `BlockStatement`
        // and there are no other non-empty statements.
        // Empties may show up when parsing depending on if the input code includes certain newlines.
        let is_single_block_statement =
            matches!(self.consequent.first(), Some(Statement::BlockStatement(_)))
                && self.consequent.iter().skip(1).all(is_dropped_statement);

        let is_default = if let Some(test) = self.test() {
            write!(f, ["case", space()]);
            if is_single_block_statement {
                // For non-block consequents the generic pass's `line_suffix` flush lands
                // before their line break, keeping the comment on the clause line,
                // so the bound is only needed before a block's `{`.
                write_node_with_trailing_comments_before(test, b':', f);
            } else {
                write!(f, test);
            }
            write!(f, ":");
            false
        } else {
            write!(f, ["default", ":"]);
            true
        };

        // When the case block is empty, the case becomes a fallthrough, so it
        // is collapsed directly on top of the next case (just a single hardline).
        // When the block is a single statement _and_ it's a block statement,
        // then the opening brace of the block can hug the same line as the case.
        // But, if there's more than one statement, then the block _cannot_ hug.
        // This distinction helps clarify that the case continues past the end of the block statement,
        // despite the braces making it seem like it might end.
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
        if self.consequent.is_empty() {
            // Print nothing to ensure that trailing comments on the same line
            // are printed on the same line. The parent list formatter takes
            // care of inserting a hard line break between cases.
            return;
        }

        let consequent = self.consequent();
        if is_single_block_statement {
            // `unwrap` is safe: the empty consequent returned above.
            write!(f, FormatStatementBody::new(consequent.first().unwrap()));
            return;
        }

        if is_default {
            // No test node to carry a trailing comment,
            // so synthesize the head's end from the keyword (escapes are illegal in keywords).
            // NOTE: relies on `:` being in `end_of_line_comments_after`'s trivia allowlist.
            #[expect(clippy::cast_possible_truncation)]
            const DEFAULT_LEN: u32 = "default".len() as u32;
            let comments =
                f.context().comments().end_of_line_comments_after(self.span.start + DEFAULT_LEN);
            if !comments.is_empty() {
                write!(
                    f,
                    [
                        space(),
                        FormatDanglingComments::Comments {
                            comments,
                            indent: DanglingIndentMode::None
                        },
                    ]
                );
            }
        }
        // No line break needed after because it is added by the indent in the switch statement
        write!(f, indent(&format_args!(hard_line_break(), consequent)));
    }
}
