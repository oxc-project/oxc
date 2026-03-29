use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{
    Format,
    ast_nodes::AstNode,
    format_args,
    formatter::{
        Formatter,
        prelude::*,
        trivia::{DanglingIndentMode, FormatDanglingComments},
    },
    utils::statement_body::FormatStatementBody,
    write,
};

use super::FormatWrite;

impl<'a> FormatWrite<'a> for AstNode<'a, SwitchStatement<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) {
        let discriminant = self.discriminant();
        let cases = self.cases();
        let format_cases =
            format_with(|f| if cases.is_empty() { hard_line_break().fmt(f) } else { cases.fmt(f) });
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

impl<'a> Format<'a> for AstNode<'a, Vec<'a, SwitchCase<'a>>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        f.join_nodes_with_hardline().entries(self);
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, SwitchCase<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) {
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

        // Whether the first statement in the clause is a BlockStatement, and
        // there are no other non-empty statements. Empties may show up when
        // parsing depending on if the input code includes certain newlines.
        let first_statement = consequent.first().unwrap();
        let is_single_block_statement =
            matches!(first_statement.as_ref(), Statement::BlockStatement(_))
                && consequent
                    .iter()
                    .skip(1)
                    .all(|statement| matches!(statement.as_ref(), Statement::EmptyStatement(_)));

        // Format dangling comments before default case body.
        if is_default {
            let comments = f.context().comments();
            let comments = if is_single_block_statement {
                comments.block_comments_before(first_statement.span().start)
            } else {
                #[expect(clippy::cast_possible_truncation)]
                const DEFAULT_LEN: u32 = "default".len() as u32;
                comments.end_of_line_comments_after(self.span.start + DEFAULT_LEN)
            };

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

        if is_single_block_statement {
            write!(f, [FormatStatementBody::new(first_statement)]);
        } else {
            // no line break needed after because it is added by the indent in the switch statement
            write!(f, indent(&format_args!(hard_line_break(), consequent)));
        }
    }
}
