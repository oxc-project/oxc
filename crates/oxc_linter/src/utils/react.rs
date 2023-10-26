use oxc_ast::{
    ast::{CallExpression, Expression, JSXAttributeItem, JSXAttributeName, JSXOpeningElement},
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
