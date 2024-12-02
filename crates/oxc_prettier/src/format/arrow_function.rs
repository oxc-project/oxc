use oxc_ast::ast::*;

use crate::{
    group,
    ir::{Doc, DocBuilder},
    Format, Prettier,
};

pub(super) fn print_arrow_function<'a>(
    p: &mut Prettier<'a>,
    expr: &ArrowFunctionExpression<'a>,
) -> Doc<'a> {
    let mut parts = p.vec();

    if !p.options.semi && p.options.arrow_parens.is_always() {
        parts.push(p.text(";"));
    }

    if expr.r#async {
        parts.push(p.text("async "));
    }

    if let Some(type_params) = &expr.type_parameters {
        parts.push(type_params.format(p));
    }

    let parameters = expr.params.format(p);
    parts.push(group!(p, parameters));

    if let Some(return_type) = &expr.return_type {
        parts.push(p.text(": "));
        parts.push(return_type.type_annotation.format(p));
    }

    parts.push(p.text(" => "));

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

    p.array(parts)
}
