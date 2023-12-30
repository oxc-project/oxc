mod boolean;
use crate::LintContext;

pub use self::boolean::*;
use oxc_ast::{
    ast::{
        BindingPatternKind, ChainElement, Expression, FormalParameters, FunctionBody,
        LogicalExpression, MemberExpression, Statement,
    },
    AstKind,
};
use oxc_semantic::AstNode;
use oxc_syntax::operator::LogicalOperator;

pub fn is_node_value_not_dom_node(expr: &Expression) -> bool {
    matches!(
        expr,
        Expression::ArrayExpression(_)
            | Expression::ArrowExpression(_)
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

// ref: https://github.com/sindresorhus/eslint-plugin-unicorn/blob/main/rules/utils/array-or-object-prototype-property.js
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
    if let Expression::MemberExpression(member_expr_obj) = member_expr.object() {
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

pub fn is_same_reference(left: &Expression, right: &Expression, ctx: &LintContext) -> bool {
    match (left, right) {
        (
            Expression::ChainExpression(left_chain_expr),
            Expression::MemberExpression(right_member_expr),
        ) => {
            if let ChainElement::MemberExpression(v) = &left_chain_expr.expression {
                return is_same_member_expression(v, right_member_expr, ctx);
            }
        }
        (
            Expression::MemberExpression(left_chain_expr),
            Expression::ChainExpression(right_member_expr),
        ) => {
            if let ChainElement::MemberExpression(v) = &right_member_expr.expression {
                return is_same_member_expression(left_chain_expr, v, ctx);
            }
        }

        // super // this
        (Expression::Super(_), Expression::Super(_))
        | (Expression::ThisExpression(_), Expression::ThisExpression(_))
        | (Expression::NullLiteral(_), Expression::NullLiteral(_)) => return true,

        (Expression::Identifier(left_ident), Expression::Identifier(right_ident)) => {
            return left_ident.name == right_ident.name
        }

        (Expression::StringLiteral(left_str), Expression::StringLiteral(right_str)) => {
            return left_str.value == right_str.value
        }
        (Expression::NumberLiteral(left_num), Expression::NumberLiteral(right_num)) => {
            return left_num.raw == right_num.raw
        }
        (Expression::RegExpLiteral(left_regexp), Expression::RegExpLiteral(right_regexp)) => {
            return left_regexp.regex.pattern == right_regexp.regex.pattern
                && left_regexp.regex.flags == right_regexp.regex.flags
        }
        (Expression::BooleanLiteral(left_bool), Expression::BooleanLiteral(right_bool)) => {
            return left_bool.value == right_bool.value
        }

        (
            Expression::ChainExpression(left_chain_expr),
            Expression::ChainExpression(right_chain_expr),
        ) => {
            if let ChainElement::MemberExpression(left_member_expr) = &left_chain_expr.expression {
                if let ChainElement::MemberExpression(right_member_expr) =
                    &right_chain_expr.expression
                {
                    return is_same_member_expression(left_member_expr, right_member_expr, ctx);
                }
            }
        }
        (
            Expression::MemberExpression(left_member_expr),
            Expression::MemberExpression(right_member_expr),
        ) => return is_same_member_expression(left_member_expr, right_member_expr, ctx),
        _ => {}
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
        _ => {}
    }

    if let (
        MemberExpression::ComputedMemberExpression(left),
        MemberExpression::ComputedMemberExpression(right),
    ) = (left, right)
    {
        if !is_same_reference(&left.expression, &right.expression, ctx) {
            return false;
        }
    }

    return is_same_reference(left.object(), right.object(), ctx);
}
