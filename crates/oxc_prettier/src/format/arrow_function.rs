#[allow(clippy::wildcard_imports)]
use oxc_ast::ast::*;

use crate::{doc::Doc, ss, Format, Prettier};

pub(super) fn print_arrow_function<'a>(
    p: &mut Prettier<'a>,
    expr: &ArrowExpression<'a>,
) -> Doc<'a> {
    let mut parts = p.vec();

    parts.push(ss!("() => "));
    if expr.expression {
        let stmt = &expr.body.statements[0];
        match stmt {
            // ExpressionStatement will add a semicolon and Hardline, But we don't need it
            // So we only need to format the expression of the ExpressionStatement
            Statement::ExpressionStatement(expr_stmt) => parts.push(expr_stmt.expression.format(p)),
            _ => parts.push(stmt.format(p)),
        }
    } else {
        parts.push(expr.body.format(p));
    }

    Doc::Array(parts)
}
