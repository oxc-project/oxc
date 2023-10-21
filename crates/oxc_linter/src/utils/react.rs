use oxc_ast::ast::{CallExpression, JSXAttributeItem, JSXAttributeName, JSXOpeningElement};

pub fn is_create_element_call(call_expr: &CallExpression) -> bool {
    if let Some(member_expr) = call_expr.callee.get_member_expr() {
        return member_expr.static_property_name() == Some("createElement");
    }

    false
}

pub fn has_jsx_prop<'a>(
    node: &'a JSXOpeningElement<'a>,
    target_prop: &str,
) -> Option<&'a JSXAttributeItem<'a>> {
    node.attributes.iter().find(|attr| match attr {
        JSXAttributeItem::SpreadAttribute(_) => false,
        JSXAttributeItem::Attribute(attr) => {
            let JSXAttributeName::Identifier(name) = &attr.name else { return false };

            name.name.as_str() == target_prop
        }
    })
}
