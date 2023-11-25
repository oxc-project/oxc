use oxc_ast::ast::{AssignmentExpression, VariableDeclarator};

use crate::{
    doc::{Doc, DocBuilder, Group},
    format, group, indent_if_break, line, ss, string, Format, Prettier,
};

pub(super) fn print_assignment_expression<'a>(
    p: &mut Prettier<'a>,
    assignment_expr: &AssignmentExpression<'a>,
) -> Doc<'a> {
    group![
        p,
        format!(p, assignment_expr.left),
        ss!(" "),
        string!(p, assignment_expr.operator.as_str()),
        indent_if_break!(p, line!(), format!(p, assignment_expr.right))
    ]
}

pub(super) fn print_variable_declarator<'a>(
    p: &mut Prettier<'a>,
    variable_declarator: &VariableDeclarator<'a>,
) -> Doc<'a> {
    let mut parts = p.vec();
    parts.push(variable_declarator.id.format(p));
    if let Some(init) = &variable_declarator.init {
        parts.push(ss!(" = "));
        parts.push(init.format(p));
    }
    Doc::Group(Group { contents: parts, should_break: false })
}
