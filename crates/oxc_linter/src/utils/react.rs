use oxc_ast::{
    ast::{
        CallExpression, Expression, JSXAttributeItem, JSXAttributeName, JSXAttributeValue,
        JSXChild, JSXElement, JSXElementName, JSXExpression, JSXExpressionContainer,
        JSXOpeningElement,
    },
    AstKind,
};
use oxc_semantic::AstNode;

use crate::LintContext;

pub fn is_create_element_call(call_expr: &CallExpression) -> bool {
    if let Some(member_expr) = call_expr.callee.get_member_expr() {
        return member_expr.static_property_name() == Some("createElement");
    }

    false
}

pub fn has_jsx_prop<'a, 'b>(
    node: &'b JSXOpeningElement<'a>,
    target_prop: &'b str,
) -> Option<&'b JSXAttributeItem<'a>> {
    node.attributes.iter().find(|attr| match attr {
        JSXAttributeItem::SpreadAttribute(_) => false,
        JSXAttributeItem::Attribute(attr) => {
            let JSXAttributeName::Identifier(name) = &attr.name else { return false };

            name.name.as_str() == target_prop
        }
    })
}

pub fn has_jsx_prop_lowercase<'a, 'b>(
    node: &'b JSXOpeningElement<'a>,
    target_prop: &'b str,
) -> Option<&'b JSXAttributeItem<'a>> {
    node.attributes.iter().find(|attr| match attr {
        JSXAttributeItem::SpreadAttribute(_) => false,
        JSXAttributeItem::Attribute(attr) => {
            let JSXAttributeName::Identifier(name) = &attr.name else { return false };

            name.name.as_str().to_lowercase() == target_prop.to_lowercase()
        }
    })
}

pub fn get_prop_value<'a, 'b>(item: &'b JSXAttributeItem<'a>) -> Option<&'b JSXAttributeValue<'a>> {
    if let JSXAttributeItem::Attribute(attr) = item {
        attr.0.value.as_ref()
    } else {
        None
    }
}

pub fn get_literal_prop_value<'a>(item: &'a JSXAttributeItem<'_>) -> Option<&'a str> {
    get_prop_value(item).and_then(|v| {
        if let JSXAttributeValue::StringLiteral(s) = v {
            Some(s.value.as_str())
        } else {
            None
        }
    })
}

// ref: https://github.com/jsx-eslint/eslint-plugin-jsx-a11y/blob/main/src/util/isHiddenFromScreenReader.js
pub fn is_hidden_from_screen_reader(node: &JSXOpeningElement) -> bool {
    if let JSXElementName::Identifier(iden) = &node.name {
        if iden.name.as_str().to_uppercase() == "INPUT" {
            if let Some(item) = has_jsx_prop_lowercase(node, "type") {
                let hidden = get_literal_prop_value(item);

                if hidden.is_some_and(|val| val.to_uppercase() == "HIDDEN") {
                    return true;
                }
            }
        }
    }

    has_jsx_prop_lowercase(node, "aria-hidden").map_or(false, |v| match get_prop_value(v) {
        None => true,
        Some(JSXAttributeValue::StringLiteral(s)) if s.value == "true" => true,
        _ => false,
    })
}

// ref: https://github.com/jsx-eslint/eslint-plugin-jsx-a11y/blob/main/src/util/hasAccessibleChild.js
pub fn object_has_accessible_child(node: &JSXElement<'_>) -> bool {
    node.children.iter().any(|child| match child {
        JSXChild::Text(text) => !text.value.is_empty(),
        JSXChild::Element(el) => !is_hidden_from_screen_reader(&el.opening_element),
        JSXChild::ExpressionContainer(JSXExpressionContainer {
            expression: JSXExpression::Expression(expr),
            ..
        }) => !expr.is_undefined(),
        _ => false,
    }) || has_jsx_prop_lowercase(&node.opening_element, "dangerouslySetInnerHTML").is_some()
        || has_jsx_prop_lowercase(&node.opening_element, "children").is_some()
}

const PRAGMA: &str = "React";
const CREATE_CLASS: &str = "createReactClass";

pub fn is_es5_component(node: &AstNode) -> bool {
    let AstKind::CallExpression(call_expr) = node.kind() else { return false };

    if let Expression::MemberExpression(member_expr) = &call_expr.callee {
        if let Expression::Identifier(ident) = member_expr.object() {
            return ident.name == PRAGMA
                && member_expr.static_property_name() == Some(CREATE_CLASS);
        }
    }

    if let Some(ident_reference) = call_expr.callee.get_identifier_reference() {
        return ident_reference.name == CREATE_CLASS;
    }

    false
}

const COMPONENT: &str = "Component";
const PURE_COMPONENT: &str = "PureComponent";

pub fn is_es6_component(node: &AstNode) -> bool {
    let AstKind::Class(class_expr) = node.kind() else { return false };
    if let Some(super_class) = &class_expr.super_class {
        if let Expression::MemberExpression(member_expr) = super_class {
            if let Expression::Identifier(ident) = member_expr.object() {
                return ident.name == PRAGMA
                    && member_expr
                        .static_property_name()
                        .is_some_and(|name| name == COMPONENT || name == PURE_COMPONENT);
            }
        }

        if let Some(ident_reference) = super_class.get_identifier_reference() {
            return ident_reference.name == COMPONENT || ident_reference.name == PURE_COMPONENT;
        }
    }

    false
}

pub fn get_parent_es5_component<'a, 'b>(
    node: &'b AstNode<'a>,
    ctx: &'b LintContext<'a>,
) -> Option<&'b AstNode<'a>> {
    ctx.nodes().ancestors(node.id()).skip(1).find_map(|node_id| {
        is_es5_component(ctx.nodes().get_node(node_id)).then(|| ctx.nodes().get_node(node_id))
    })
}

pub fn get_parent_es6_component<'a, 'b>(
    node: &'b AstNode<'a>,
    ctx: &'b LintContext<'a>,
) -> Option<&'b AstNode<'a>> {
    ctx.nodes().ancestors(node.id()).skip(1).find_map(|node_id| {
        is_es6_component(ctx.nodes().get_node(node_id)).then(|| ctx.nodes().get_node(node_id))
    })
}
