use oxc_ast::ast::*;

use crate::{array, doc::Doc, format, group, ss, string, Prettier};

use super::Format;

pub(super) fn print_assignment<'a>(
    p: &mut Prettier<'a>,
    assignment: &AssignmentExpression<'a>,
) -> Doc<'a> {
    let parts = array![
        p,
        format!(p, assignment.left),
        ss!(" "),
        string!(p, assignment.operator.as_str()),
        ss!(" "),
        format!(p, assignment.right)
    ];

    group!(p, parts)
}
