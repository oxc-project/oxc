use std::borrow::Cow;

use oxc_ast::{
    AstKind,
    ast::{
        ArrowFunctionExpression, CallExpression, Expression, Function, FunctionBody,
        JSXAttributeItem, JSXAttributeName, JSXAttributeValue, JSXChild, JSXElement,
        JSXElementName, JSXExpression, JSXFragment, JSXMemberExpression, JSXMemberExpressionObject,
        JSXOpeningElement, Statement, StaticMemberExpression,
    },
};
use oxc_ast_visit::{Visit, walk};
use oxc_ecmascript::{ToBoolean, WithoutGlobalReferenceInformation};
use oxc_semantic::AstNode;
use oxc_syntax::scope::ScopeFlags;

use crate::globals::HTML_TAG;
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

// TODO: Move the a11y methods to their own util for jsx-a11y?

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

// ref: https://github.com/jsx-eslint/eslint-plugin-jsx-a11y/blob/8f75961d965e47afb88854d324bd32fafde7acfe/src/util/isPresentationRole.js
pub fn is_presentation_role(jsx_opening_el: &JSXOpeningElement) -> bool {
    let Some(role) = has_jsx_prop(jsx_opening_el, "role") else {
        return false;
    };

    matches!(get_string_literal_prop_value(role), Some("presentation" | "none"))
}

// ref: https://github.com/jsx-eslint/eslint-plugin-jsx-a11y/blob/8f75961d965e47afb88854d324bd32fafde7acfe/src/util/isAbstractRole.js
pub fn is_abstract_role<'a>(ctx: &LintContext<'a>, jsx_opening_el: &JSXOpeningElement<'a>) -> bool {
    // Do not test custom JSX components, we do not know what
    // low-level DOM element this maps to.
    let element_type = get_element_type(ctx, jsx_opening_el);
    if !HTML_TAG.contains(element_type.as_ref()) {
        return false;
    }

    let Some(role) = has_jsx_prop(jsx_opening_el, "role") else {
        return false;
    };

    matches!(
        get_string_literal_prop_value(role),
        Some(
            "command"
                | "composite"
                | "input"
                | "landmark"
                | "range"
                | "roletype"
                | "section"
                | "sectionhead"
                | "select"
                | "structure"
                | "widget"
                | "window"
        )
    )
}

// ref: https://github.com/jsx-eslint/eslint-plugin-jsx-a11y/blob/4c7e7815c12a797587bb8e3cdced7f3003848964/src/util/isInteractiveElement.js
//
// See also https://html.spec.whatwg.org/multipage/dom.html#interactive-content
pub fn is_interactive_element(element_type: &str, jsx_opening_el: &JSXOpeningElement) -> bool {
    // Interactive contents are...
    // - a, area (when they have `href`)
    // - audio, video
    // - button, canvas, datalist, details, embed, iframe, label, menuitem,
    //   option, select, summary, textarea, td, th, tr
    // - input (unless `type` is hidden)
    // - img (when `usemap` is present)
    match element_type {
        "audio" | "button" | "canvas" | "datalist" | "details" | "embed" | "iframe" | "label"
        | "menuitem" | "option" | "select" | "summary" | "td" | "th" | "tr" | "textarea"
        | "video" => true,
        "input" => {
            if let Some(input_type) = has_jsx_prop(jsx_opening_el, "type")
                && get_string_literal_prop_value(input_type)
                    .is_some_and(|val| val.eq_ignore_ascii_case("hidden"))
            {
                return false;
            }
            true
        }
        "a" | "area" => has_jsx_prop(jsx_opening_el, "href").is_some(),
        "img" => has_jsx_prop(jsx_opening_el, "usemap").is_some(),
        _ => false,
    }
}

// ref: https://github.com/jsx-eslint/eslint-plugin-jsx-a11y/blob/8f75961d965e47afb88854d324bd32fafde7acfe/src/util/isNonInteractiveElement.js
pub fn is_non_interactive_element(element_type: &str, jsx_opening_el: &JSXOpeningElement) -> bool {
    // Do not test custom JSX components, we do not know what
    // low-level DOM element this maps to.
    if !HTML_TAG.contains(element_type.as_ref()) {
        return false;
    }

    match element_type {
        // <header> elements do not technically have semantics, unless the
        // element is a direct descendant of <body>, and this plugin cannot
        // reliably test that.
        // @see https://www.w3.org/TR/wai-aria-practices/examples/landmarks/banner.html
        "header" => false,
        // Only treat <section> as non-interactive when it has an accessible name.
        "section" => {
            has_jsx_prop_ignore_case(jsx_opening_el, "aria-label").is_some()
                || has_jsx_prop_ignore_case(jsx_opening_el, "aria-labelledby").is_some()
        }
        _ => NON_INTERACTIVE_ELEMENT_TYPES.contains(&element_type),
    }
}

// Based on https://github.com/jsx-eslint/eslint-plugin-jsx-a11y/blob/8f75961d965e47afb88854d324bd32fafde7acfe/src/util/isNonInteractiveElement.js
const NON_INTERACTIVE_ELEMENT_TYPES: [&str; 59] = [
    "abbr",
    "address",
    "article",
    "aside",
    "blockquote",
    "br",
    "caption",
    "code",
    "dd",
    "del",
    "details",
    "dfn",
    "dialog",
    "dir",
    "dl",
    "dt",
    "em",
    "fieldset",
    "figcaption",
    "figure",
    "footer",
    "form",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "hr",
    "html",
    "iframe",
    "img",
    "ins",
    "label",
    "legend",
    "li",
    "main",
    "mark",
    "marquee",
    "menu",
    "meter",
    "nav",
    "ol",
    "optgroup",
    "output",
    "p",
    "pre",
    "progress",
    "ruby",
    "section",
    "strong",
    "sub",
    "sup",
    "table",
    "tbody",
    "tfoot",
    "thead",
    "time",
    "ul",
];

const INTERACTIVE_ROLES: [&str; 27] = [
    "button",
    "checkbox",
    "columnheader",
    "combobox",
    "gridcell",
    "link",
    "listbox",
    "menu",
    "menubar",
    "menuitem",
    "menuitemcheckbox",
    "menuitemradio",
    "option",
    "radio",
    "radiogroup",
    "row",
    "rowheader",
    "scrollbar",
    "searchbox",
    "separator",
    "slider",
    "spinbutton",
    "switch",
    "tab",
    "textbox",
    // Per the original rule:
    // > 'toolbar' does not descend from widget, but it does support
    // > aria-activedescendant, thus in practice we treat it as a widget.
    "toolbar",
    "treeitem",
];

const NON_INTERACTIVE_ROLES: [&str; 47] = [
    "alert",
    "alertdialog",
    "application",
    "article",
    "banner",
    "blockquote",
    "caption",
    "cell",
    "complementary",
    "contentinfo",
    "definition",
    "deletion",
    "dialog",
    "directory",
    "document",
    "feed",
    "figure",
    "form",
    "grid",
    "group",
    "heading",
    "img",
    "insertion",
    "list",
    "listitem",
    "log",
    "main",
    "marquee",
    "math",
    "navigation",
    "note",
    "paragraph",
    // per the original impl:
    // >  The `progressbar` is descended from `widget`, but in practice, its
    // > value is always `readonly`, so we treat it as a non-interactive role.
    "progressbar",
    "region",
    "row",
    "rowgroup",
    "search",
    "status",
    "table",
    "tablist",
    "tabpanel",
    "term",
    "time",
    "timer",
    "tooltip",
    "tree",
    "treegrid",
];

// ref: https://github.com/jsx-eslint/eslint-plugin-jsx-a11y/blob/8f75961d965e47afb88854d324bd32fafde7acfe/src/util/isInteractiveRole.js
pub fn is_interactive_role(role: &str) -> bool {
    INTERACTIVE_ROLES.contains(&role)
}

// ref: https://github.com/jsx-eslint/eslint-plugin-jsx-a11y/blob/8f75961d965e47afb88854d324bd32fafde7acfe/src/util/isInteractiveRole.js
pub fn is_non_interactive_role(role: &str) -> bool {
    NON_INTERACTIVE_ROLES.contains(&role)
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

/// Checks if a function call is a Higher-Order Component (HOC)
pub fn is_hoc_call(callee_name: &str, ctx: &LintContext) -> bool {
    // Check built-in HOCs with exact matching (matches ESLint behavior)
    // Matches: memo, forwardRef, React.memo, React.forwardRef
    if matches!(callee_name, "memo" | "forwardRef" | "React.memo" | "React.forwardRef") {
        return true;
    }

    // Check component wrapper functions from settings
    ctx.settings().react.is_component_wrapper_function(callee_name)
}

/// Finds the innermost function with JSX in a chain of HOC calls
#[derive(Debug)]
pub enum InnermostFunction<'a> {
    Function(&'a Function<'a>),
    /// Arrow functions never have an id, so we don't need to store the reference
    ArrowFunction,
}

pub fn find_innermost_function_with_jsx<'a>(
    expr: &'a Expression<'a>,
    ctx: &LintContext<'_>,
) -> Option<InnermostFunction<'a>> {
    match expr {
        Expression::CallExpression(call) => {
            // Check if this is a HOC call
            if let Some(callee_name) = call.callee_name()
                && is_hoc_call(callee_name, ctx)
            {
                // This is a HOC, recursively check the first argument
                if let Some(first_arg) = call.arguments.first()
                    && let Some(inner_expr) = first_arg.as_expression()
                {
                    return find_innermost_function_with_jsx(inner_expr, ctx);
                }
            }
            None
        }
        Expression::FunctionExpression(func) => {
            // Check if this function contains JSX
            if function_contains_jsx(func) { Some(InnermostFunction::Function(func)) } else { None }
        }
        Expression::ArrowFunctionExpression(arrow_func) => {
            // Check if this arrow function contains JSX
            if expression_contains_jsx(expr) {
                Some(InnermostFunction::ArrowFunction)
            } else {
                // Check if this arrow function returns another function that contains JSX
                if arrow_func.expression {
                    // Expression-bodied arrow function: () => () => <div />
                    if arrow_func.body.statements.len() == 1
                        && let Statement::ExpressionStatement(expr_stmt) =
                            &arrow_func.body.statements[0]
                    {
                        return find_innermost_function_with_jsx(&expr_stmt.expression, ctx);
                    }
                } else {
                    // Block-bodied arrow function: () => { return () => <div /> }
                    for stmt in &arrow_func.body.statements {
                        if let Statement::ReturnStatement(ret_stmt) = stmt
                            && let Some(expr) = &ret_stmt.argument
                        {
                            return find_innermost_function_with_jsx(expr, ctx);
                        }
                    }
                }
                None
            }
        }
        _ => None,
    }
}

/// Visitor that searches for JSX elements within a function body.
/// Stops at nested function boundaries to avoid detecting JSX from child components.
struct JsxFinder {
    found: bool,
}

impl JsxFinder {
    fn new() -> Self {
        Self { found: false }
    }
}

impl<'a> Visit<'a> for JsxFinder {
    fn visit_jsx_element(&mut self, _elem: &JSXElement<'a>) {
        self.found = true;
        // Don't walk children - we found what we need
    }

    fn visit_jsx_fragment(&mut self, _frag: &JSXFragment<'a>) {
        self.found = true;
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if crate::utils::is_create_element_call(call) {
            self.found = true;
        }
        if !self.found {
            walk::walk_call_expression(self, call);
        }
    }

    // Don't recurse into nested functions - they're separate components
    fn visit_function(&mut self, _func: &Function<'a>, _flags: ScopeFlags) {}
    fn visit_arrow_function_expression(&mut self, _arrow: &ArrowFunctionExpression<'a>) {}
}

/// Checks if a function contains JSX anywhere in its body.
/// This uses a visitor pattern to handle JSX in if/else blocks, try/catch,
/// loops, and other control flow constructs.
pub fn function_contains_jsx(func: &Function) -> bool {
    if let Some(body) = &func.body {
        return function_body_contains_jsx(body);
    }
    false
}

/// Checks if a function body contains JSX anywhere.
pub fn function_body_contains_jsx(body: &FunctionBody) -> bool {
    let mut finder = JsxFinder::new();
    finder.visit_function_body(body);
    finder.found
}

/// Checks if a function-like expression (function or arrow function) contains JSX
pub fn expression_contains_jsx(expr: &Expression) -> bool {
    match expr {
        Expression::FunctionExpression(func) => function_contains_jsx(func),
        Expression::ArrowFunctionExpression(arrow_func) => {
            function_body_contains_jsx(&arrow_func.body)
        }
        _ => false,
    }
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
