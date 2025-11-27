use std::borrow::Cow;

use oxc_ast::{
    AstKind,
    ast::{
        CallExpression, Expression, JSXAttributeItem, JSXAttributeName, JSXAttributeValue,
        JSXChild, JSXElement, JSXElementName, JSXExpression, JSXMemberExpression,
        JSXMemberExpressionObject, JSXOpeningElement, StaticMemberExpression,
    },
};
use oxc_ecmascript::{ToBoolean, WithoutGlobalReferenceInformation};
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
            Cow::Owned(format!("{}:{}", name.namespace.name, name.name.name))
        }
        JSXAttributeName::Identifier(ident) => Cow::Borrowed(ident.name.as_str()),
    }
}

pub fn get_string_literal_prop_value<'a>(item: &'a JSXAttributeItem<'_>) -> Option<&'a str> {
    get_prop_value(item).and_then(JSXAttributeValue::as_string_literal).map(|s| s.value.as_str())
}

// ref: https://github.com/jsx-eslint/eslint-plugin-jsx-a11y/blob/v6.9.0/src/util/isHiddenFromScreenReader.js
pub fn is_hidden_from_screen_reader<'a>(
    ctx: &LintContext<'a>,
    node: &JSXOpeningElement<'a>,
) -> bool {
    let name = get_element_type(ctx, node);
    if name.eq_ignore_ascii_case("input")
        && let Some(item) = has_jsx_prop_ignore_case(node, "type")
    {
        let hidden = get_string_literal_prop_value(item);

        if hidden.is_some_and(|val| val.eq_ignore_ascii_case("hidden")) {
            return true;
        }
    }

    has_jsx_prop_ignore_case(node, "aria-hidden").is_some_and(|v| match get_prop_value(v) {
        None => true,
        Some(JSXAttributeValue::StringLiteral(s)) if s.value == "true" => true,
        Some(JSXAttributeValue::ExpressionContainer(container)) => {
            if let Some(expr) = container.expression.as_expression() {
                expr.to_boolean(&WithoutGlobalReferenceInformation {}).unwrap_or(false)
            } else {
                false
            }
        }
        _ => false,
    })
}

// ref: https://github.com/jsx-eslint/eslint-plugin-jsx-a11y/blob/v6.9.0/src/util/hasAccessibleChild.js
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
            if let Some(input_type) = has_jsx_prop(jsx_opening_el, "type")
                && get_string_literal_prop_value(input_type)
                    .is_some_and(|val| val.eq_ignore_ascii_case("hidden"))
            {
                return false;
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

    if let Some(member_expr) = call_expr.callee.as_member_expression()
        && let Expression::Identifier(ident) = member_expr.object()
    {
        return ident.name == PRAGMA && member_expr.static_property_name() == Some(CREATE_CLASS);
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
        if let Some(member_expr) = super_class.as_member_expression()
            && let Expression::Identifier(ident) = member_expr.object()
        {
            return ident.name == PRAGMA
                && member_expr
                    .static_property_name()
                    .is_some_and(|name| name == COMPONENT || name == PURE_COMPONENT);
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
    ctx.nodes().ancestors(node.id()).find(|node| is_es5_component(node) || is_es6_component(node))
}

fn get_jsx_mem_expr_name<'a>(jsx_mem_expr: &JSXMemberExpression) -> Cow<'a, str> {
    let prefix = match &jsx_mem_expr.object {
        JSXMemberExpressionObject::IdentifierReference(id) => Cow::Borrowed(id.name.as_str()),
        JSXMemberExpressionObject::MemberExpression(mem_expr) => {
            Cow::Owned(format!("{}.{}", get_jsx_mem_expr_name(mem_expr), mem_expr.property.name))
        }
        JSXMemberExpressionObject::ThisExpression(_) => Cow::Borrowed("this"),
    };

    Cow::Owned(format!("{}.{}", prefix, jsx_mem_expr.property.name))
}

/// Resolve element type(name) using jsx-a11y settings
/// ref:
/// <https://github.com/jsx-eslint/eslint-plugin-jsx-a11y/blob/v6.9.0/src/util/getElementType.js>
pub fn get_element_type<'c, 'a>(
    context: &'c LintContext<'a>,
    element: &JSXOpeningElement<'a>,
) -> Cow<'c, str> {
    let name = match &element.name {
        JSXElementName::Identifier(id) => Cow::Borrowed(id.as_ref().name.as_str()),
        JSXElementName::IdentifierReference(id) => Cow::Borrowed(id.as_ref().name.as_str()),
        JSXElementName::NamespacedName(namespaced) => {
            Cow::Owned(format!("{}:{}", namespaced.namespace.name, namespaced.name.name))
        }
        JSXElementName::MemberExpression(jsx_mem_expr) => get_jsx_mem_expr_name(jsx_mem_expr),
        JSXElementName::ThisExpression(_) => Cow::Borrowed("this"),
    };

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

    let raw_type = polymorphic_prop.map_or(name, Cow::Borrowed);
    match jsx_a11y.components.get(raw_type.as_ref()) {
        Some(component) => Cow::Borrowed(component),
        None => raw_type,
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
    name.starts_with("use") && name.chars().nth(3).is_none_or(char::is_uppercase)
}

/// Checks whether the `name` follows the official conventions of React Hooks.
///
/// Identifies `use(...)` as a valid hook.
///
/// Hook names must start with use followed by a capital letter,
/// like useState (built-in) or useOnlineStatus (custom).
pub fn is_react_hook(expr: &Expression) -> bool {
    match expr {
        Expression::StaticMemberExpression(static_expr) => {
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
        matches!(
            member.object().get_identifier_reference(),
            Some(ident) if ident.name.as_str() == PRAGMA
        )
    } else {
        true
    }
}

/// Checks if a JSX opening element is a React Fragment.
/// Recognizes both `<Fragment>` and `<React.Fragment>` forms.
pub fn is_jsx_fragment(elem: &JSXOpeningElement) -> bool {
    match &elem.name {
        JSXElementName::IdentifierReference(ident) => ident.name == "Fragment",
        JSXElementName::MemberExpression(mem_expr) => {
            if let JSXMemberExpressionObject::IdentifierReference(ident) = &mem_expr.object {
                ident.name == "React" && mem_expr.property.name == "Fragment"
            } else {
                false
            }
        }
        JSXElementName::NamespacedName(_)
        | JSXElementName::Identifier(_)
        | JSXElementName::ThisExpression(_) => false,
    }
}

// check current node is this.state.xx
pub fn is_state_member_expression(expression: &StaticMemberExpression<'_>) -> bool {
    if let Expression::ThisExpression(_) = &expression.object {
        return expression.property.name == "state";
    }

    false
}

#[cfg(test)]
mod test {
    use super::*;

    use oxc_allocator::Allocator;
    use oxc_ast::AstBuilder;
    use oxc_span::Span;

    #[test]
    fn test_is_react_component_name() {
        // Good names:
        assert!(is_react_component_name("MyComponent"));
        assert!(is_react_component_name("X"));
        assert!(is_react_component_name("Component_Name")); // Allowed but horrible
        // This should be allowed:
        // ```jsx
        // function Form() {}
        // Form.Input = function Input() { ... };
        // <Form.Input />
        // ```
        assert!(is_react_component_name("Component.Name"));
        // Bad names:
        assert!(!is_react_component_name("myComponent"));
        assert!(!is_react_component_name("useSomething"));
        assert!(!is_react_component_name("x"));
        assert!(!is_react_component_name("componentname"));
        assert!(!is_react_component_name("use"));
    }

    #[test]
    fn test_is_react_hook() {
        let alloc = Allocator::default();
        let ast = AstBuilder::new(&alloc);

        // Identifier: useState
        let use_state = ast.expression_identifier(Span::default(), "useState");
        assert!(is_react_hook(&use_state));

        // Identifier: use
        let just_use = ast.expression_identifier(Span::default(), "use");
        assert!(is_react_hook(&just_use));

        // Identifier: userError, should not be considered a hook despite starting with "use"
        let user_error = ast.expression_identifier(Span::default(), "userError");
        assert!(!is_react_hook(&user_error));

        // Identifier that's not a hook
        let not_hook = ast.expression_identifier(Span::default(), "notAHook");
        assert!(!is_react_hook(&not_hook));

        // Static member: React.useEffect -> valid
        let react_obj = ast.expression_identifier(Span::default(), "React");
        let prop = ast.identifier_name(Span::default(), "useEffect");
        let react_use_effect =
            ast.member_expression_static(Span::default(), react_obj, prop, false).into();
        assert!(is_react_hook(&react_use_effect));

        // Static member: react.useEffect -> invalid because namespace isn't PascalCase
        let react_lower = ast.expression_identifier(Span::default(), "react");
        let prop2 = ast.identifier_name(Span::default(), "useEffect");
        let react_lower_use_effect =
            ast.member_expression_static(Span::default(), react_lower, prop2, false).into();
        assert!(!is_react_hook(&react_lower_use_effect));
    }

    #[test]
    fn test_is_react_hook_name() {
        // Good names:
        assert!(is_react_hook_name("useState"));
        assert!(is_react_hook_name("useFooBar"));
        assert!(is_react_hook_name("useEffect"));
        assert!(is_react_hook_name("use"));
        // Bad names:
        assert!(!is_react_hook_name("userError"));
        assert!(!is_react_hook_name("notAHook"));
        assert!(!is_react_hook_name("UseState"));
        assert!(!is_react_hook_name("Use"));
        assert!(!is_react_hook_name("user"));
        assert!(!is_react_hook_name("use_state"));
    }
}
