use oxc_ast::ast::{Expression, MemberExpression, Statement};

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
        Some("Array") => {
            if let Expression::ArrayExpression(array_expr) = member_expr.object() {
                array_expr.elements.len() == 0
            } else {
                false
            }
        }

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
