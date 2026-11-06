//! [`JsRoot::Program`](crate::JsRoot::Program): statements, a `<script>`'s, a Markdown code block's,
//! an event handler attribute's (`onclick="a(); b()"`).

use oxc_ast::ast::*;

use crate::{
    ProgramLayout, ProgramSemicolons,
    ast_nodes::AstNode,
    formatter::{JsFormatter, TopLevelSemicolons, prelude::*, trivia::format_dangling_comments},
    write,
};

use super::super::{block_statement::is_empty_block, program::FormatProgramBody};

/// Writes `program`'s statements, ending as `semicolons` says, laid out as `layout` says.
pub(super) fn write_program<'a>(
    program: &AstNode<'_, 'a, Program<'a>>,
    semicolons: ProgramSemicolons,
    layout: ProgramLayout,
    f: &mut JsFormatter<'_, 'a>,
) {
    f.context_mut().set_top_level_semicolons(top_level_semicolons(program, semicolons));
    match layout {
        ProgramLayout::Bare => write!(f, FormatProgramBody(program)),
        ProgramLayout::Indented => {
            let is_empty = is_empty_block(&program.body) && program.directives.is_empty();
            // Nothing to print (`onclick=" ; "`) leaves the value empty
            if is_empty && program.comments.is_empty() {
                return;
            }
            write!(
                f,
                soft_block_indent(&format_with(|f| {
                    if is_empty {
                        // Comments only (`onclick="/* c */"`) print as the program's own, without a space before them
                        write!(f, format_dangling_comments(program.span));
                    } else {
                        write!(f, FormatProgramBody(program));
                    }
                }))
            );
        }
    }
}

/// How `program`'s top-level statements end under `semicolons`.
fn top_level_semicolons(
    program: &Program<'_>,
    semicolons: ProgramSemicolons,
) -> TopLevelSemicolons {
    let lone_expression = match program.body.as_slice() {
        [Statement::ExpressionStatement(statement)] if program.directives.is_empty() => {
            Some(statement)
        }
        _ => None,
    };
    match semicolons {
        ProgramSemicolons::AsOptions => TopLevelSemicolons::AsOptions,
        ProgramSemicolons::LoneJsxOmitted => lone_expression
            .filter(|statement| statement.expression.is_jsx())
            .map_or(TopLevelSemicolons::AsOptions, |statement| {
                TopLevelSemicolons::Omitted(statement.span)
            }),
        ProgramSemicolons::LoneExpressionOmitted => lone_expression
            .map_or(TopLevelSemicolons::AsOptions, |statement| {
                TopLevelSemicolons::Omitted(statement.span)
            }),
    }
}
