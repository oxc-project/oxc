mod boolean;
pub use self::boolean::*;
use oxc_ast::{
    ast::{
        BindingPatternKind, Expression, FormalParameters, FunctionBody, LogicalExpression,
        MemberExpression, Statement,
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
    let first_func_param = arg.items.get(0)?;
    let BindingPatternKind::BindingIdentifier(first_func_param) = &first_func_param.pattern.kind
    else {
        return None;
    };
    Some(first_func_param.name.as_str())
}

pub fn get_return_identifier_name<'a>(body: &'a FunctionBody<'_>) -> Option<&'a str> {
    match body.statements.get(0)? {
        Statement::BlockStatement(block_stmt) => {
            let Statement::ReturnStatement(return_stmt) = block_stmt.body.get(0)? else {
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
