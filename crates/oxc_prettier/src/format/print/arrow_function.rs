use oxc_allocator::Vec;
use oxc_ast::ast::*;

use crate::{array, group, ir::Doc, text, ArrowParens, Format, Prettier};

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
    p: &mut Prettier<'a>,
    expr: &ArrowFunctionExpression<'a>,
) -> bool {
    fn can_print_params_without_parens<'a>(
        p: &mut Prettier<'a>,
        expr: &ArrowFunctionExpression<'a>,
    ) -> bool {
        // TODO
        // const parameters = getFunctionParameters(node);
        // return (
        //   parameters.length === 1 &&
        //   !node.typeParameters &&
        //   !hasComment(node, CommentCheckFlags.Dangling) &&
        //   parameters[0].type === "Identifier" &&
        //   !parameters[0].typeAnnotation &&
        //   !hasComment(parameters[0]) &&
        //   !parameters[0].optional &&
        //   !node.predicate &&
        //   !node.returnType
        // );

        if expr.params.parameters_count() != 1 {
            return false;
        }

        true
    }

    match p.options.arrow_parens {
        ArrowParens::Always => false,
        ArrowParens::Avoid => can_print_params_without_parens(p, expr),
    }
}
