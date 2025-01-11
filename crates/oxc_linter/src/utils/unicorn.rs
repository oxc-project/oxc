mod boolean;
use oxc_ast::{
    ast::{
        BindingPatternKind, Expression, FormalParameters, FunctionBody, LogicalExpression,
        MemberExpression, Statement,
    },
    AstKind,
};
use oxc_semantic::AstNode;
use oxc_span::cmp::ContentEq;
use oxc_syntax::operator::LogicalOperator;

pub use self::boolean::*;
use crate::LintContext;

pub fn is_node_value_not_dom_node(expr: &Expression) -> bool {
    matches!(
        expr,
        Expression::ArrayExpression(_)
            | Expression::ArrowFunctionExpression(_)
            | Expression::ClassExpression(_)
            | Expression::FunctionExpression(_)
            | Expression::ObjectExpression(_)
            | Expression::TemplateLiteral(_)
            | Expression::StringLiteral(_)
    )
}

pub fn is_empty_stmt(stmt: &Statement) -> bool {
    match stmt {
        Statement::BlockStatement(block_stmt) => {
            if block_stmt.body.is_empty() || block_stmt.body.iter().all(|node| is_empty_stmt(node))
            {
                return true;
            }
            false
        }
        Statement::EmptyStatement(_) => true,
        _ => false,
    }
}

// ref: https://github.com/sindresorhus/eslint-plugin-unicorn/blob/v56.0.0/rules/utils/array-or-object-prototype-property.js
pub fn is_prototype_property(
    member_expr: &MemberExpression,
    property: &str,
    object: Option<&str>,
) -> bool {
    if !member_expr.static_property_name().is_some_and(|name| name == property)
        || member_expr.optional()
    {
        return false;
    }

    // `Object.prototype.method` or `Array.prototype.method`
    if let Some(member_expr_obj) = member_expr.object().as_member_expression() {
        if let Expression::Identifier(iden) = member_expr_obj.object() {
            if member_expr_obj.static_property_name().is_some_and(|name| name == "prototype")
                && object.is_some_and(|val| val == iden.name)
                && !member_expr.optional()
                && !member_expr_obj.optional()
            {
                return true;
            }
        }
    };

    match object {
        // `[].method`
        Some("Array") => is_empty_array_expression(member_expr.object()),

        // `{}.method`
        Some("Object") => {
            if let Expression::ObjectExpression(obj_expr) = member_expr.object() {
                obj_expr.properties.len() == 0
            } else {
                false
            }
        }
        _ => false,
    }
}

pub fn is_empty_array_expression(expr: &Expression) -> bool {
    if let Expression::ArrayExpression(array_expr) = expr {
        array_expr.elements.len() == 0
    } else {
        false
    }
}

pub fn is_empty_object_expression(expr: &Expression) -> bool {
    if let Expression::ObjectExpression(object_expr) = expr {
        object_expr.properties.len() == 0
    } else {
        false
    }
}

pub fn is_logical_expression(node: &AstNode) -> bool {
    matches!(
        node.kind(),
        AstKind::LogicalExpression(LogicalExpression {
            operator: LogicalOperator::And | LogicalOperator::Or,
            ..
        })
    )
}

// gets the name of the first parameter of a function
pub fn get_first_parameter_name<'a>(arg: &'a FormalParameters) -> Option<&'a str> {
    let first_func_param = arg.items.first()?;
    let BindingPatternKind::BindingIdentifier(first_func_param) = &first_func_param.pattern.kind
    else {
        return None;
    };
    Some(first_func_param.name.as_str())
}

pub fn get_return_identifier_name<'a>(body: &'a FunctionBody<'_>) -> Option<&'a str> {
    match body.statements.first()? {
        Statement::BlockStatement(block_stmt) => {
            let Statement::ReturnStatement(return_stmt) = block_stmt.body.first()? else {
                return None;
            };

            let Some(Expression::Identifier(ident)) = return_stmt.argument.as_ref() else {
                return None;
            };

            Some(ident.name.as_str())
        }
        Statement::ReturnStatement(return_stmt) => {
            let return_expr = return_stmt.argument.as_ref()?;
            match return_expr {
                Expression::Identifier(ident) => Some(ident.name.as_str()),
                _ => None,
            }
        }
        Statement::ExpressionStatement(expr_stmt) => {
            let Expression::Identifier(ident) = &expr_stmt.expression else {
                return None;
            };

            Some(ident.name.as_str())
        }
        _ => None,
    }
}

/// Compares two expressions to see if they are the same.
pub fn is_same_expression(left: &Expression, right: &Expression, ctx: &LintContext) -> bool {
    if let Expression::ChainExpression(left_chain_expr) = left {
        if let Some(right_member_expr) = right.as_member_expression() {
            if let Some(v) = left_chain_expr.expression.as_member_expression() {
                return is_same_member_expression(v, right_member_expr, ctx);
            }
        }
    }

    if let Some(left_chain_expr) = left.as_member_expression() {
        if let Expression::ChainExpression(right_member_expr) = right {
            if let Some(v) = right_member_expr.expression.as_member_expression() {
                return is_same_member_expression(left_chain_expr, v, ctx);
            }
        }
    }

    match (left, right) {
        // super // this
        (Expression::Super(_), Expression::Super(_))
        | (Expression::ThisExpression(_), Expression::ThisExpression(_))
        | (Expression::NullLiteral(_), Expression::NullLiteral(_)) => return true,

        (Expression::Identifier(left_ident), Expression::Identifier(right_ident)) => {
            return left_ident.name == right_ident.name;
        }

        (Expression::StringLiteral(left_str), Expression::StringLiteral(right_str)) => {
            return left_str.value == right_str.value;
        }
        (Expression::StringLiteral(string_lit), Expression::TemplateLiteral(template_lit))
        | (Expression::TemplateLiteral(template_lit), Expression::StringLiteral(string_lit)) => {
            return template_lit.is_no_substitution_template()
                && string_lit.value == template_lit.quasi().unwrap();
        }
        (Expression::TemplateLiteral(left_str), Expression::TemplateLiteral(right_str)) => {
            return left_str.quasis.content_eq(&right_str.quasis)
                && left_str.expressions.len() == right_str.expressions.len()
                && left_str
                    .expressions
                    .iter()
                    .zip(right_str.expressions.iter())
                    .all(|(left, right)| is_same_expression(left, right, ctx));
        }
        (Expression::NumericLiteral(left_num), Expression::NumericLiteral(right_num)) => {
            return left_num.raw == right_num.raw;
        }
        (Expression::RegExpLiteral(left_regexp), Expression::RegExpLiteral(right_regexp)) => {
            return left_regexp.regex.pattern.source_text(ctx.source_text())
                == right_regexp.regex.pattern.source_text(ctx.source_text())
                && left_regexp.regex.flags == right_regexp.regex.flags;
        }
        (Expression::BooleanLiteral(left_bool), Expression::BooleanLiteral(right_bool)) => {
            return left_bool.value == right_bool.value;
        }

        (
            Expression::BinaryExpression(left_bin_expr),
            Expression::BinaryExpression(right_bin_expr),
        ) => {
            return left_bin_expr.operator == right_bin_expr.operator
                && is_same_expression(
                    left_bin_expr.left.get_inner_expression(),
                    right_bin_expr.left.get_inner_expression(),
                    ctx,
                )
                && is_same_expression(
                    left_bin_expr.right.get_inner_expression(),
                    right_bin_expr.right.get_inner_expression(),
                    ctx,
                );
        }

        (
            Expression::UnaryExpression(left_unary_expr),
            Expression::UnaryExpression(right_unary_expr),
        ) => {
            return left_unary_expr.operator == right_unary_expr.operator
                && is_same_expression(
                    left_unary_expr.argument.get_inner_expression(),
                    right_unary_expr.argument.get_inner_expression(),
                    ctx,
                );
        }

        (
            Expression::ChainExpression(left_chain_expr),
            Expression::ChainExpression(right_chain_expr),
        ) => {
            if let Some(left_member_expr) = left_chain_expr.expression.as_member_expression() {
                if let Some(right_member_expr) = right_chain_expr.expression.as_member_expression()
                {
                    return is_same_member_expression(left_member_expr, right_member_expr, ctx);
                }
            }
        }
        _ => {}
    }

    if let (Some(left_member_expr), Some(right_member_expr)) =
        (left.as_member_expression(), right.as_member_expression())
    {
        return is_same_member_expression(left_member_expr, right_member_expr, ctx);
    }

    false
}

pub fn is_same_member_expression(
    left: &MemberExpression,
    right: &MemberExpression,
    ctx: &LintContext,
) -> bool {
    let left_static_property_name = left.static_property_name();
    let right_static_property_name = right.static_property_name();

    match (left_static_property_name, right_static_property_name) {
        (Some(left_static_property_name), Some(right_static_property_name)) => {
            if left_static_property_name != right_static_property_name {
                return false;
            }
        }
        (Some(_), None) | (None, Some(_)) => {
            return false;
        }
        (None, None) => {
            if let (
                MemberExpression::PrivateFieldExpression(left),
                MemberExpression::PrivateFieldExpression(right),
            ) = (left, right)
            {
                return left.field.name == right.field.name
                    && is_same_expression(&left.object, &right.object, ctx);
            }
        }
    }

    if let (
        MemberExpression::ComputedMemberExpression(left),
        MemberExpression::ComputedMemberExpression(right),
    ) = (left, right)
    {
        // TODO(camc314): refactor this to go through `is_same_reference` and introduce some sort of `context` to indicate how the two values should be compared.
        match (&left.expression, &right.expression) {
            // x['/regex/'] === x[/regex/]
            // x[/regex/] === x['/regex/']
            (Expression::StringLiteral(string_lit), Expression::RegExpLiteral(regex_lit))
            | (Expression::RegExpLiteral(regex_lit), Expression::StringLiteral(string_lit)) => {
                if string_lit.value != regex_lit.raw.as_ref().unwrap() {
                    return false;
                }
            }
            // ex) x[`/regex/`] === x[/regex/]
            // ex) x[/regex/] === x[`/regex/`]
            (Expression::TemplateLiteral(template_lit), Expression::RegExpLiteral(regex_lit))
            | (Expression::RegExpLiteral(regex_lit), Expression::TemplateLiteral(template_lit)) => {
                if !(template_lit.is_no_substitution_template()
                    && template_lit.quasi().unwrap() == regex_lit.raw.as_ref().unwrap())
                {
                    return false;
                }
            }
            _ => {
                if !is_same_expression(
                    left.expression.get_inner_expression(),
                    right.expression.get_inner_expression(),
                    ctx,
                ) {
                    return false;
                }
            }
        }
    }

    is_same_expression(
        left.object().get_inner_expression(),
        right.object().get_inner_expression(),
        ctx,
    )
}
