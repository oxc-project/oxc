use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{
    ast_nodes::{AstNode, AstNodes},
    format_args,
    formatter::{FormatResult, Formatter, prelude::*},
    generated::ast_nodes::{AstNode, AstNodes},
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
        // When as/satisfies is the object of a member expression or callee of a call,
        // wrap with indent and soft breaks to allow proper line breaking.
        // This matches Prettier's formatting: group([indent([softline, ...parts]), softline])
        write!(
            f,
            [group(&format_args!(
                indent(&format_args!(soft_line_break(), format_inner)),
                soft_line_break()
            ))]
        )
    } else {
        write!(f, [format_inner])
    }
}

fn is_callee_or_object_context(span: Span, parent: &AstNodes<'_>) -> bool {
    match parent {
        // Only the callee of a call expression needs special formatting
        AstNodes::CallExpression(call) => call.callee.span() == span,
        // Only the callee of a new expression needs special formatting
        AstNodes::NewExpression(new_expr) => new_expr.callee.span() == span,
        // All static member expressions need special formatting (as expression is always the object)
        AstNodes::StaticMemberExpression(_) => true,
        // Only when the as expression is the object of a computed member expression
        AstNodes::ComputedMemberExpression(member) => member.object.span() == span,
        _ => false,
    }
}
