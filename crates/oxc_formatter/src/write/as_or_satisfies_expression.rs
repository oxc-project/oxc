use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{
    ast_nodes::{AstNode, AstNodes},
    format_args,
    formatter::{FormatResult, Formatter, prelude::*},
    write,
    write::FormatWrite,
};

impl<'a> FormatWrite<'a> for AstNode<'a, TSAsExpression<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let is_callee_or_object = is_callee_or_object_context(self.span(), self.parent);
        format_as_or_satisfies_expression(
            self.expression(),
            self.type_annotation(),
            is_callee_or_object,
            "as",
            f,
        )
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSSatisfiesExpression<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let is_callee_or_object = is_callee_or_object_context(self.span(), self.parent);
        format_as_or_satisfies_expression(
            self.expression(),
            self.type_annotation(),
            is_callee_or_object,
            "satisfies",
            f,
        )
    }
}

fn format_as_or_satisfies_expression<'a>(
    expression: &AstNode<'a, Expression>,
    type_annotation: &AstNode<'a, TSType>,
    is_callee_or_object: bool,
    operation: &'static str,
    f: &mut Formatter<'_, 'a>,
) -> FormatResult<()> {
    let format_inner = format_with(|f| {
        write!(f, [expression, space(), text(operation)])?;
        write!(f, [space(), type_annotation])
    });

    if is_callee_or_object {
        write!(f, [group(&soft_block_indent(&format_inner))])
    } else {
        write!(f, [format_inner])
    }
}

fn is_callee_or_object_context(span: Span, parent: &AstNodes<'_>) -> bool {
    match parent {
        // Callee
        AstNodes::CallExpression(_) | AstNodes::NewExpression(_)
        // Static member
        | AstNodes::StaticMemberExpression(_) => true,
        AstNodes::ComputedMemberExpression(member) => {
            member.object.span() == span
        }
        _ => false,
    }
}
