use std::borrow::Cow;

use oxc_ast::{
    ast::{
        CallExpression, Expression, JSXAttributeItem, JSXAttributeName, JSXAttributeValue,
        JSXChild, JSXElement, JSXExpression, JSXOpeningElement, MemberExpression,
    },
    match_member_expression, AstKind,
};
use oxc_ecmascript::ToBoolean;
use oxc_semantic::AstNode;

use crate::{LintContext, OxlintSettings};

pub fn is_create_element_call(call_expr: &CallExpression) -> bool {
    match &call_expr.callee {
        Expression::StaticMemberExpression(member_expr) => {
            member_expr.property.name == "createElement"
        }
        Expression::ComputedMemberExpression(member_expr) => {
            member_expr.static_property_name().is_some_and(|name| name == "createElement")
        }
        Expression::Identifier(ident) => ident.name == "createElement",
        _ => false,
    }
}

pub fn has_jsx_prop<'a, 'b>(
    node: &'b JSXOpeningElement<'a>,
    target_prop: &'b str,
) -> Option<&'b JSXAttributeItem<'a>> {
    node.attributes
        .iter()
        .find(|attr| attr.as_attribute().is_some_and(|attr| attr.is_identifier(target_prop)))
}

pub fn has_jsx_prop_ignore_case<'a, 'b>(
    node: &'b JSXOpeningElement<'a>,
    target_prop: &'b str,
) -> Option<&'b JSXAttributeItem<'a>> {
    node.attributes.iter().find(|attr| {
        attr.as_attribute().is_some_and(|attr| attr.is_identifier_ignore_case(target_prop))
    })
}

pub fn get_prop_value<'a, 'b>(item: &'b JSXAttributeItem<'a>) -> Option<&'b JSXAttributeValue<'a>> {
    item.as_attribute().and_then(|item| item.value.as_ref())
}

pub fn get_jsx_attribute_name<'a>(attr: &JSXAttributeName<'a>) -> Cow<'a, str> {
    match attr {
        JSXAttributeName::NamespacedName(name) => {
            Cow::Owned(format!("{}:{}", name.namespace.name, name.property.name))
        }
        JSXAttributeName::Identifier(ident) => Cow::Borrowed(ident.name.as_str()),
    }
}

pub fn get_string_literal_prop_value<'a>(item: &'a JSXAttributeItem<'_>) -> Option<&'a str> {
    get_prop_value(item).and_then(JSXAttributeValue::as_string_literal).map(|s| s.value.as_str())
}

// ref: https://github.com/jsx-eslint/eslint-plugin-jsx-a11y/blob/main/src/util/isHiddenFromScreenReader.js
pub fn is_hidden_from_screen_reader<'a>(
    ctx: &LintContext<'a>,
    node: &JSXOpeningElement<'a>,
) -> bool {
    if let Some(name) = get_element_type(ctx, node) {
        if name.eq_ignore_ascii_case("input") {
            if let Some(item) = has_jsx_prop_ignore_case(node, "type") {
                let hidden = get_string_literal_prop_value(item);

                if hidden.is_some_and(|val| val.eq_ignore_ascii_case("hidden")) {
                    return true;
                }
            }
        }
    }

    has_jsx_prop_ignore_case(node, "aria-hidden").map_or(false, |v| match get_prop_value(v) {
        None => true,
        Some(JSXAttributeValue::StringLiteral(s)) if s.value == "true" => true,
        Some(JSXAttributeValue::ExpressionContainer(container)) => {
            if let Some(expr) = container.expression.as_expression() {
                expr.to_boolean().unwrap_or(false)
            } else {
                false
            }
        }
        _ => false,
    })
}

// ref: https://github.com/jsx-eslint/eslint-plugin-jsx-a11y/blob/main/src/util/hasAccessibleChild.js
pub fn object_has_accessible_child<'a>(ctx: &LintContext<'a>, node: &JSXElement<'a>) -> bool {
    node.children.iter().any(|child| match child {
        JSXChild::Text(text) => !text.value.is_empty(),
        JSXChild::Element(el) => !is_hidden_from_screen_reader(ctx, &el.opening_element),
        JSXChild::ExpressionContainer(container) => {
            !matches!(&container.expression, JSXExpression::NullLiteral(_))
                && !container.expression.is_undefined()
        }
        _ => false,
    }) || has_jsx_prop_ignore_case(&node.opening_element, "dangerouslySetInnerHTML").is_some()
        || has_jsx_prop_ignore_case(&node.opening_element, "children").is_some()
}

pub fn is_presentation_role(jsx_opening_el: &JSXOpeningElement) -> bool {
    let Some(role) = has_jsx_prop(jsx_opening_el, "role") else {
        return false;
    };

    matches!(get_string_literal_prop_value(role), Some("presentation" | "none"))
}

// TODO: Should re-implement
// https://github.com/jsx-eslint/eslint-plugin-jsx-a11y/blob/4c7e7815c12a797587bb8e3cdced7f3003848964/src/util/isInteractiveElement.js
// with `oxc-project/aria-query` which is currently W.I.P.
//
// Until then, use simplified version by https://html.spec.whatwg.org/multipage/dom.html#interactive-content
pub fn is_interactive_element(element_type: &str, jsx_opening_el: &JSXOpeningElement) -> bool {
    // Interactive contents are...
    // - button, details, embed, iframe, label, select, textarea
    // - input (if the `type` attribute is not in the Hidden state)
    // - a (if the `href` attribute is present)
    // - audio, video (if the `controls` attribute is present)
    // - img (if the `usemap` attribute is present)
    match element_type {
        "button" | "details" | "embed" | "iframe" | "label" | "select" | "textarea" => true,
        "input" => {
            if let Some(input_type) = has_jsx_prop(jsx_opening_el, "type") {
                if get_string_literal_prop_value(input_type)
                    .is_some_and(|val| val.eq_ignore_ascii_case("hidden"))
                {
                    return false;
                }
            }
            true
        }
        "a" => has_jsx_prop(jsx_opening_el, "href").is_some(),
        "audio" | "video" => has_jsx_prop(jsx_opening_el, "controls").is_some(),
        "img" => has_jsx_prop(jsx_opening_el, "usemap").is_some(),
        _ => false,
    }
}

const PRAGMA: &str = "React";
const CREATE_CLASS: &str = "createReactClass";

pub fn is_es5_component(node: &AstNode) -> bool {
    let AstKind::CallExpression(call_expr) = node.kind() else {
        return false;
    };

    if let Some(member_expr) = call_expr.callee.as_member_expression() {
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
    let AstKind::Class(class_expr) = node.kind() else {
        return false;
    };
    if let Some(super_class) = &class_expr.super_class {
        if let Some(member_expr) = super_class.as_member_expression() {
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

pub fn get_parent_component<'a, 'b>(
    node: &'b AstNode<'a>,
    ctx: &'b LintContext<'a>,
) -> Option<&'b AstNode<'a>> {
    for node_id in ctx.nodes().ancestors(node.id()) {
        let node = ctx.nodes().get_node(node_id);
        if is_es5_component(node) || is_es6_component(node) {
            return Some(node);
        }
    }
    None
}

/// Resolve element type(name) using jsx-a11y settings
/// ref:
/// <https://github.com/jsx-eslint/eslint-plugin-jsx-a11y/blob/main/src/util/getElementType.js>
pub fn get_element_type<'c, 'a>(
    context: &'c LintContext<'a>,
    element: &JSXOpeningElement<'a>,
) -> Option<Cow<'c, str>> {
    let name = element.name.get_identifier_name()?;

    let OxlintSettings { jsx_a11y, .. } = context.settings();

    let polymorphic_prop = jsx_a11y
        .polymorphic_prop_name
        .as_ref()
        .and_then(|polymorphic_prop_name_value| {
            has_jsx_prop_ignore_case(element, polymorphic_prop_name_value)
        })
        .and_then(get_prop_value)
        .and_then(JSXAttributeValue::as_string_literal)
        .map(|s| s.value.as_str());

    let raw_type = polymorphic_prop.unwrap_or_else(|| name.as_str());
    match jsx_a11y.components.get(raw_type) {
        Some(component) => Some(Cow::Borrowed(component)),
        None => Some(Cow::Borrowed(raw_type)),
    }
}

pub fn parse_jsx_value(value: &JSXAttributeValue) -> Result<f64, ()> {
    match value {
        JSXAttributeValue::StringLiteral(str) => str.value.parse().or(Err(())),
        JSXAttributeValue::ExpressionContainer(container) => match &container.expression {
            JSXExpression::StringLiteral(str) => str.value.parse().or(Err(())),
            JSXExpression::TemplateLiteral(tmpl) => {
                tmpl.quasis.first().unwrap().value.raw.parse().or(Err(()))
            }
            JSXExpression::NumericLiteral(num) => Ok(num.value),
            _ => Err(()),
        },
        _ => Err(()),
    }
}

/// Checks whether the `name` follows the official conventions of React Hooks.
///
/// Identifies `use(...)` as a valid hook.
///
/// Hook names must start with use followed by a capital letter,
/// like useState (built-in) or useOnlineStatus (custom).
pub fn is_react_hook_name(name: &str) -> bool {
    name.starts_with("use") && name.chars().nth(3).map_or(true, char::is_uppercase)
    // uncomment this check if react decided to drop the idea of `use` hook.
    // <https://react.dev/reference/react/use> It is currently in `Canary` builds.
    // name.starts_with("use") && name.chars().nth(3).is_some_and(char::is_uppercase)
}

/// Checks whether the `name` follows the official conventions of React Hooks.
///
/// Identifies `use(...)` as a valid hook.
///
/// Hook names must start with use followed by a capital letter,
/// like useState (built-in) or useOnlineStatus (custom).
pub fn is_react_hook(expr: &Expression) -> bool {
    match expr {
        match_member_expression!(Expression) => {
            // SAFETY: We already have checked that `expr` is a member expression using the
            // `match_member_expression` macro.

            let expr = unsafe { expr.as_member_expression().unwrap_unchecked() };
            let MemberExpression::StaticMemberExpression(static_expr) = expr else { return false };

            let is_valid_property = is_react_hook_name(&static_expr.property.name);
            let is_valid_namespace = match &static_expr.object {
                Expression::Identifier(ident) => {
                    // TODO: test PascalCase
                    ident.name.chars().next().is_some_and(char::is_uppercase)
                }
                _ => false,
            };
            is_valid_namespace && is_valid_property
        }
        Expression::Identifier(ident) => is_react_hook_name(ident.name.as_str()),
        _ => false,
    }
}

/// Checks if the node is a React component name. React component names must
/// always start with an uppercase letter.
pub fn is_react_component_name(name: &str) -> bool {
    name.chars().next().is_some_and(|c| c.is_ascii_uppercase())
}

/// Checks if the node is a React component name or React hook,
/// `is_react_component_name`, `is_react_hook_name`
pub fn is_react_component_or_hook_name(name: &str) -> bool {
    is_react_component_name(name) || is_react_hook_name(name)
}

pub fn is_react_function_call(call: &CallExpression, expected_call: &str) -> bool {
    let Some(subject) = call.callee_name() else { return false };

    if subject != expected_call {
        return false;
    }

    if let Some(member) = call.callee.as_member_expression() {
        matches! {
            member.object().get_identifier_reference(),
            Some(ident) if ident.name.as_str() == PRAGMA
        }
    } else {
        true
    }
}
