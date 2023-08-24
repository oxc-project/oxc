use std::hash::{Hash, Hasher};

use oxc_ast::ast::{
    Expression, JSXAttribute, JSXAttributeName, JSXAttributeValue, JSXElementName, JSXIdentifier,
    JSXMemberExpression, JSXNamespacedName, TSAccessibility, TemplateLiteral,
};
use rustc_hash::FxHasher;

pub fn jsx_attribute_to_constant_string<'a>(attr: &'a JSXAttribute<'a>) -> Option<String> {
    attr.value.as_ref().and_then(|attr_value| match attr_value {
        JSXAttributeValue::StringLiteral(slit) => slit.value.to_string().into(),
        JSXAttributeValue::ExpressionContainer(expr) => match &expr.expression {
            oxc_ast::ast::JSXExpression::Expression(expr) => expr_to_maybe_const_string(expr),
            oxc_ast::ast::JSXExpression::EmptyExpression(_) => None,
        },
        JSXAttributeValue::Element(_) | JSXAttributeValue::Fragment(_) => None,
    })
}

pub fn expr_to_maybe_const_string<'a>(expr: &'a Expression<'a>) -> Option<String> {
    match expr {
        Expression::StringLiteral(slit) => Some(slit.value.to_string()),
        Expression::TemplateLiteral(tlit) => {
            try_get_constant_string_field_value_from_template_lit(tlit.0)
        }
        _ => None,
    }
}

pub fn try_get_constant_string_field_value_from_template_lit(
    tlit: &TemplateLiteral,
) -> Option<String> {
    if tlit.expressions.len() == 0 && tlit.quasis.len() == 1 {
        let quasi = &tlit.quasis[0].value;
        Some(quasi.cooked.as_ref().unwrap_or(&quasi.raw).to_string())
    } else {
        None
    }
}

pub fn accessibility_to_string(accessibility: TSAccessibility) -> String {
    match accessibility {
        TSAccessibility::Private => "private",
        TSAccessibility::Protected => "protected",
        TSAccessibility::Public => "public",
    }
    .to_owned()
}

pub fn jsx_identifier_to_string(jsx_identifier: &JSXIdentifier) -> String {
    jsx_identifier.name.to_string()
}

pub fn jsx_namespaced_name_to_string(jsx_namespaced_name: &JSXNamespacedName) -> String {
    format!("{}:{}", &jsx_namespaced_name.namespace.name, &jsx_namespaced_name.property.name)
}

pub fn jsx_membexpr_to_string(jsx_member_expr: &JSXMemberExpression) -> String {
    let mut parts = vec![jsx_member_expr.property.name.to_string()];
    let mut obj = &jsx_member_expr.object;
    loop {
        match obj {
            oxc_ast::ast::JSXMemberExpressionObject::Identifier(ident) => {
                parts.push(ident.name.to_string());
                break;
            }
            oxc_ast::ast::JSXMemberExpressionObject::MemberExpression(inner_memexpr) => {
                parts.push(inner_memexpr.property.name.to_string());
                obj = &inner_memexpr.object;
            }
        }
    }
    parts.reverse();
    parts.join(".")
}

pub fn jsx_element_name_to_string(jsx_element_name: &JSXElementName) -> String {
    match jsx_element_name {
        JSXElementName::Identifier(ident) => jsx_identifier_to_string(ident),
        JSXElementName::NamespacedName(nsn) => jsx_namespaced_name_to_string(nsn),
        JSXElementName::MemberExpression(memexpr) => jsx_membexpr_to_string(memexpr),
    }
}

pub fn jsx_attribute_name_to_string(jsx_attribute_name: &JSXAttributeName) -> String {
    match jsx_attribute_name {
        JSXAttributeName::Identifier(ident) => jsx_identifier_to_string(ident),
        JSXAttributeName::NamespacedName(nsn) => jsx_namespaced_name_to_string(nsn),
    }
}

pub fn strip_parens_from_expr<'a, 'b: 'a>(
    to_strip: &'b Expression<'a>,
    strip_all: bool,
) -> &'b Expression<'a> {
    match to_strip {
        Expression::ParenthesizedExpression(paren_expr) if strip_all => {
            strip_parens_from_expr(&paren_expr.expression, true)
        }
        Expression::ParenthesizedExpression(paren_expr) => &paren_expr.expression,
        _ => to_strip,
    }
}

pub fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut hasher = FxHasher::default();
    t.hash(&mut hasher);
    hasher.finish()
}
