//! An embedded JS/TS part ([`crate::format_to_ir`]): the text each [`JsRoot`] is parsed in,
//! the checks that it is the one its root names, and how it prints.
//! A kind's own rules live in its module; this one dispatches on the root.
//! JS as the child; JS as the parent (css-in-js and the like) is `print/template/embed/`.

mod expression;
mod params;
mod program;

pub use expression::object_prints_double_brace;

use std::ops::Range;

use oxc_ast::{Comment, ast::*};
use oxc_diagnostics::OxcDiagnostic;
use oxc_formatter_core::Format;
use oxc_syntax::{identifier::is_white_space, line_terminator::is_line_terminator};

use crate::{
    JsRoot, ProgramSemicolons,
    ast_nodes::{AstNode, AstNodeKind},
    formatter::{JsFormatter, prelude::*},
};

use super::block_statement::is_empty_block;

/// The text the code is wrapped in to parse as `root` says.
/// A line break before the closing one lets a trailing line comment parse; [`check_root`] checks whether the code closes it.
pub fn root_wrapper(root: JsRoot) -> (&'static str, &'static str) {
    match root {
        JsRoot::Program { .. } => ("", ""),
        JsRoot::Expression(_) => ("(", "\n)"),
        JsRoot::FunctionParams { .. } => ("function _(", "\n) {}"),
        JsRoot::TypeParameters => ("type T<", "\n> = any"),
    }
}

/// Checks that `program`, the code at `source` of its wrapped text ([`root_wrapper`]),
/// is the one `root` names and prints without changing what the host reads of it
/// (`inlines_code`: [`crate::JsEmbeddedIn::inlines_code`]).
/// Returns [`FormatRoot::ends_in_line_comment`].
pub fn check_root(
    program: &Program<'_>,
    source: Range<usize>,
    root: JsRoot,
    inlines_code: bool,
) -> Result<bool, OxcDiagnostic> {
    // A host that inlines the code needs a line comment it ends in closed, or it comments out what follows there.
    // Unclosed, the comment runs up to the code's end
    if inlines_code
        && program
            .comments
            .last()
            .is_some_and(|comment| comment.is_line() && comment.span.end as usize == source.end)
    {
        return Err(OxcDiagnostic::error(
            "A line comment ends the code without a line break; the code is preserved as-is",
        ));
    }
    // A line comment among the comments after the code's last token (`a // c⏎/* d */` too) makes it print on its own lines,
    // so a line break closes it again before what the host puts after it
    let mut end = source.end;
    let ends_in_line_comment = program
        .comments
        .iter()
        .rev()
        .take_while(|comment| {
            let after = &program.source_text[comment.span.end as usize..end];
            end = comment.span.start as usize;
            after.chars().all(|c| is_white_space(c) || is_line_terminator(c))
        })
        .any(Comment::is_line);

    if matches!(root, JsRoot::Program { semicolons: ProgramSemicolons::MarksStatements, .. })
        && !program.body.is_empty()
    {
        // The host reads code without a `;` as an expression, which statements are not (`if (a) b()` does not compile there):
        // with one added, it would
        if !program.source_text[source.clone()].contains(';') {
            return Err(OxcDiagnostic::error(
                "The code is statements without a `;`; the code is preserved as-is",
            ));
        }
        // Without a `;`, the host reads comments as an expression, which they are not (`@click="; /* c */"` is statements)
        if is_empty_block(&program.body)
            && program.directives.is_empty()
            && !program.comments.is_empty()
        {
            return Err(OxcDiagnostic::error(
                "Only comments and `;` are the code; the code is preserved as-is",
            ));
        }
    }

    match (root, program.body.as_slice()) {
        (JsRoot::Program { .. }, _)
        // `function _(` … `) {}`
        | (JsRoot::FunctionParams { .. }, [Statement::FunctionDeclaration(_)])
        // `type T<` … `> = any`, never empty: `type T<> = any` does not parse
        | (JsRoot::TypeParameters, [Statement::TSTypeAliasDeclaration(_)]) => {}
        (JsRoot::Expression(expression), _) => {
            expression::check(program, source, expression, inlines_code)?;
        }
        _ => return Err(shape_error()),
    }
    Ok(ends_in_line_comment)
}

fn shape_error() -> OxcDiagnostic {
    OxcDiagnostic::error("The code is not the one its root names")
}

/// An embedded part's program (see [`crate::format_to_ir`]), printed as the code its root names.
pub struct FormatRoot<'a, 'b> {
    pub program: &'b AstNode<'b, 'a, Program<'a>>,
    pub root: JsRoot,
    /// The comments the code ends in hold a line comment, which a line break closes:
    /// it goes on its own lines, so one closes it again.
    pub ends_in_line_comment: bool,
}

impl<'a> Format<'a, JsFormatContext<'a>> for FormatRoot<'a, '_> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        let program = self.program;
        if let JsRoot::Program { semicolons, layout } = self.root {
            return program::write_program(program, semicolons, layout, f);
        }
        let body = program.body();
        let statement = body.iter().next().expect("`check_root` checked the code's shape");
        match (self.root, statement.as_kind()) {
            (JsRoot::Expression(root), AstNodeKind::ExpressionStatement(statement)) => {
                expression::write_expression(
                    program,
                    &statement.inner().expression,
                    root,
                    self.ends_in_line_comment,
                    f,
                );
            }
            (JsRoot::FunctionParams { parens_when_many }, AstNodeKind::Function(function)) => {
                params::write_function_params(
                    &function.params(),
                    parens_when_many,
                    self.ends_in_line_comment,
                    f,
                );
            }
            (JsRoot::TypeParameters, AstNodeKind::TSTypeAliasDeclaration(declaration)) => {
                let type_parameters = declaration.type_parameters().expect(
                    "`check_root` checked the code's shape: `type T<> = any` does not parse",
                );
                params::write_type_parameters(&type_parameters, f);
            }
            _ => unreachable!("`check_root` checked the code's shape"),
        }
    }
}
