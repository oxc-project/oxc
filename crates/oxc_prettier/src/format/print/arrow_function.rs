use oxc_allocator::Vec;
use oxc_ast::ast::*;

use crate::{ArrowParens, Format, Prettier, array, group, ir::Doc, text};

pub fn print_arrow_function<'a>(
    p: &mut Prettier<'a>,
    expr: &ArrowFunctionExpression<'a>,
) -> Doc<'a> {
    let mut parts = Vec::new_in(p.allocator);

    if expr.r#async {
        parts.push(text!("async "));
    }

    if let Some(type_params) = &expr.type_parameters {
        parts.push(type_params.format(p));
    }

    let params_doc = expr.params.format(p);
    parts.push(group!(p, [params_doc]));

    if let Some(return_type) = &expr.return_type {
        parts.push(text!(": "));
        parts.push(return_type.type_annotation.format(p));
    }

    parts.push(text!(" => "));

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

    array!(p, parts)
}

pub fn should_print_params_without_parens<'a>(
    p: &Prettier<'a>,
    expr: &ArrowFunctionExpression<'a>,
) -> bool {
    match p.options.arrow_parens {
        ArrowParens::Always => false,
        ArrowParens::Avoid => {
            // TODO: hasComment(node, CommentCheckFlags.Dangling) &&
            let expr_has_dangling_comment = false;
            if (expr.type_parameters.is_some() || expr_has_dangling_comment) {
                return false;
            }

            if expr.params.rest.is_some() || expr.params.items.len() != 1 {
                return false;
            }
            let first_param_pat =
                &expr.params.items.first().expect("There should be at least one param").pattern;

            // TODO: hasComment(firstParam)
            let first_param_has_comment = false;
            first_param_pat.kind.is_binding_identifier()
                && first_param_pat.type_annotation.is_none()
                && !first_param_pat.optional
                && !first_param_has_comment
        }
    }
}
