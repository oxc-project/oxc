//! Angular utility functions for linting rules.
//!
//! This module provides shared utilities for Angular-specific lint rules.

use oxc_ast::{
    AstKind,
    ast::{
        Argument, CallExpression, Decorator, Expression, ObjectExpression, ObjectPropertyKind,
        PropertyKey,
    },
};
use oxc_span::Span;

use crate::LintContext;

/// Names of legacy Angular decorators that should be replaced with signal-based APIs.
pub const LEGACY_INPUT_DECORATORS: [&str; 6] =
    ["Input", "Output", "ViewChild", "ViewChildren", "ContentChild", "ContentChildren"];

/// Signal-based replacements for legacy decorators.
pub const SIGNAL_REPLACEMENTS: [(&str, &str); 6] = [
    ("Input", "input"),
    ("Output", "output"),
    ("ViewChild", "viewChild"),
    ("ViewChildren", "viewChildren"),
    ("ContentChild", "contentChild"),
    ("ContentChildren", "contentChildren"),
];

/// Angular lifecycle methods in execution order.
pub const ANGULAR_LIFECYCLE_METHODS: [&str; 9] = [
    "ngOnChanges",
    "ngOnInit",
    "ngDoCheck",
    "ngAfterContentInit",
    "ngAfterContentChecked",
    "ngAfterViewInit",
    "ngAfterViewChecked",
    "ngOnDestroy",
    "ngDoBootstrap",
];

/// Lifecycle methods in execution order (for sorting).
pub const LIFECYCLE_METHODS_ORDERED: [&str; 8] = [
    "ngOnChanges",
    "ngOnInit",
    "ngDoCheck",
    "ngAfterContentInit",
    "ngAfterContentChecked",
    "ngAfterViewInit",
    "ngAfterViewChecked",
    "ngOnDestroy",
];

/// Mapping from lifecycle method names to their corresponding interface names.
pub const LIFECYCLE_METHOD_INTERFACES: [(&str, &str); 9] = [
    ("ngOnChanges", "OnChanges"),
    ("ngOnInit", "OnInit"),
    ("ngDoCheck", "DoCheck"),
    ("ngAfterContentInit", "AfterContentInit"),
    ("ngAfterContentChecked", "AfterContentChecked"),
    ("ngAfterViewInit", "AfterViewInit"),
    ("ngAfterViewChecked", "AfterViewChecked"),
    ("ngOnDestroy", "OnDestroy"),
    ("ngDoBootstrap", "DoBootstrap"),
];

/// Native DOM event names that should not be used as output names.
pub const NATIVE_EVENT_NAMES: [&str; 67] = [
    "abort",
    "animationcancel",
    "animationend",
    "animationiteration",
    "animationstart",
    "auxclick",
    "beforeinput",
    "blur",
    "cancel",
    "canplay",
    "canplaythrough",
    "change",
    "click",
    "close",
    "contextmenu",
    "copy",
    "cuechange",
    "cut",
    "dblclick",
    "drag",
    "dragend",
    "dragenter",
    "dragleave",
    "dragover",
    "dragstart",
    "drop",
    "durationchange",
    "emptied",
    "ended",
    "error",
    "focus",
    "focusin",
    "focusout",
    "formdata",
    "gotpointercapture",
    "input",
    "invalid",
    "keydown",
    "keypress",
    "keyup",
    "load",
    "loadeddata",
    "loadedmetadata",
    "loadstart",
    "lostpointercapture",
    "mousedown",
    "mouseenter",
    "mouseleave",
    "mousemove",
    "mouseout",
    "mouseover",
    "mouseup",
    "paste",
    "pause",
    "play",
    "playing",
    "pointercancel",
    "pointerdown",
    "pointerenter",
    "pointerleave",
    "pointermove",
    "pointerout",
    "pointerover",
    "pointerup",
    "progress",
    "ratechange",
    "reset",
    // Additional events follow in extension
];

/// Extended native event names (continuation).
pub const NATIVE_EVENT_NAMES_EXTENDED: [&str; 25] = [
    "resize",
    "scroll",
    "securitypolicyviolation",
    "seeked",
    "seeking",
    "select",
    "selectionchange",
    "selectstart",
    "slotchange",
    "stalled",
    "submit",
    "suspend",
    "timeupdate",
    "toggle",
    "touchcancel",
    "touchend",
    "touchmove",
    "touchstart",
    "transitioncancel",
    "transitionend",
    "transitionrun",
    "transitionstart",
    "volumechange",
    "waiting",
    "wheel",
];

/// Angular decorator types that support specific lifecycle methods.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AngularDecoratorType {
    Component,
    Directive,
    Injectable,
    Pipe,
    NgModule,
}

/// Selector style types for validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectorStyle {
    KebabCase,
    CamelCase,
}

/// Selector type (element or attribute).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectorType {
    Element,
    Attribute,
}

/// Check if an identifier reference is imported from `@angular/core`.
///
/// This function verifies that an identifier with the given name is actually
/// imported from the Angular core package, not from another library.
pub fn is_angular_core_import(
    ident: &oxc_ast::ast::IdentifierReference,
    ctx: &LintContext<'_>,
) -> bool {
    let reference = ctx.scoping().get_reference(ident.reference_id());
    let Some(symbol_id) = reference.symbol_id() else {
        return false;
    };

    if !ctx.scoping().symbol_flags(symbol_id).is_import() {
        return false;
    }

    let declaration_id = ctx.scoping().symbol_declaration(symbol_id);
    let AstKind::ImportDeclaration(import_decl) = ctx.nodes().parent_kind(declaration_id) else {
        return false;
    };

    import_decl.source.value.as_str() == "@angular/core"
}

/// Get the name of a decorator if it's a simple identifier or call expression.
pub fn get_decorator_name<'a>(decorator: &'a Decorator<'a>) -> Option<&'a str> {
    match &decorator.expression {
        Expression::Identifier(ident) => Some(ident.name.as_str()),
        Expression::CallExpression(call) => {
            if let Expression::Identifier(ident) = &call.callee {
                Some(ident.name.as_str())
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Get the identifier reference from a decorator expression.
pub fn get_decorator_identifier<'a>(
    decorator: &'a Decorator<'a>,
) -> Option<&'a oxc_ast::ast::IdentifierReference<'a>> {
    match &decorator.expression {
        Expression::Identifier(ident) => Some(ident.as_ref()),
        Expression::CallExpression(call) => {
            if let Expression::Identifier(ident) = &call.callee {
                Some(ident.as_ref())
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Get the call expression from a decorator if it's a call.
pub fn get_decorator_call<'a>(decorator: &'a Decorator<'a>) -> Option<&'a CallExpression<'a>> {
    if let Expression::CallExpression(call) = &decorator.expression {
        Some(call.as_ref())
    } else {
        None
    }
}

/// Extract the metadata object from a `@Component` or `@Directive` decorator.
pub fn get_component_metadata<'a>(
    decorator: &'a Decorator<'a>,
) -> Option<&'a ObjectExpression<'a>> {
    let call = get_decorator_call(decorator)?;

    // The metadata object should be the first argument
    let first_arg = call.arguments.first()?;

    if let Argument::ObjectExpression(obj) = first_arg { Some(obj.as_ref()) } else { None }
}

/// Get a property value from a metadata object by key name.
/// Supports identifier keys, string literal keys, and computed keys with string/template literals.
pub fn get_metadata_property<'a>(
    obj: &'a ObjectExpression<'a>,
    key: &str,
) -> Option<&'a Expression<'a>> {
    for property in &obj.properties {
        if let ObjectPropertyKind::ObjectProperty(prop) = property {
            let key_matches = match &prop.key {
                // Standard identifier key: `changeDetection: value`
                PropertyKey::StaticIdentifier(ident) => ident.name.as_str() == key,
                // String literal key: `'changeDetection': value`
                PropertyKey::StringLiteral(lit) => lit.value.as_str() == key,
                _ => {
                    // For computed keys, check the expression
                    if prop.computed {
                        match prop.key.as_expression() {
                            // Computed string literal: `['changeDetection']: value`
                            Some(Expression::StringLiteral(lit)) => lit.value.as_str() == key,
                            // Computed template literal with no expressions: `[\`changeDetection\`]: value`
                            Some(Expression::TemplateLiteral(tpl))
                                if tpl.expressions.is_empty() =>
                            {
                                tpl.quasis.first().is_some_and(|q| q.value.raw.as_str() == key)
                            }
                            _ => false,
                        }
                    } else {
                        false
                    }
                }
            };

            if key_matches {
                return Some(&prop.value);
            }
        }
    }
    None
}

/// Get the signal replacement function name for a legacy decorator.
pub fn get_signal_replacement(decorator_name: &str) -> Option<&'static str> {
    SIGNAL_REPLACEMENTS
        .iter()
        .find(|(legacy, _)| *legacy == decorator_name)
        .map(|(_, signal)| *signal)
}

/// Check if a decorator is a legacy Angular decorator that should be replaced.
pub fn is_legacy_angular_decorator(name: &str) -> bool {
    LEGACY_INPUT_DECORATORS.contains(&name)
}

/// Get the corresponding interface name for a lifecycle method.
pub fn get_lifecycle_interface_for_method(method: &str) -> Option<&'static str> {
    LIFECYCLE_METHOD_INTERFACES.iter().find(|(m, _)| *m == method).map(|(_, interface)| *interface)
}

/// Check if a method name is an Angular lifecycle method.
pub fn is_lifecycle_method(name: &str) -> bool {
    ANGULAR_LIFECYCLE_METHODS.contains(&name)
}

/// Check if a name is a native DOM event name.
pub fn is_native_event_name(name: &str) -> bool {
    NATIVE_EVENT_NAMES.contains(&name) || NATIVE_EVENT_NAMES_EXTENDED.contains(&name)
}

/// Get the order index of a lifecycle method for sorting purposes.
pub fn get_lifecycle_method_order(name: &str) -> Option<usize> {
    LIFECYCLE_METHODS_ORDERED.iter().position(|&m| m == name)
}

/// Check if a lifecycle method is valid for a given decorator type.
pub fn is_lifecycle_valid_for_decorator(method: &str, decorator: AngularDecoratorType) -> bool {
    match decorator {
        AngularDecoratorType::Component | AngularDecoratorType::Directive => {
            // Components and Directives support all lifecycle methods except ngDoBootstrap
            method != "ngDoBootstrap" && is_lifecycle_method(method)
        }
        AngularDecoratorType::Injectable | AngularDecoratorType::Pipe => {
            // Injectables and Pipes only support ngOnDestroy
            method == "ngOnDestroy"
        }
        AngularDecoratorType::NgModule => {
            // NgModules only support ngDoBootstrap
            method == "ngDoBootstrap"
        }
    }
}

/// Check if a string matches kebab-case format.
pub fn is_kebab_case(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    s.chars().all(|c| c.is_ascii_lowercase() || c == '-' || c.is_ascii_digit())
        && !s.starts_with('-')
        && !s.ends_with('-')
        && !s.contains("--")
}

/// Check if a string matches camelCase format.
pub fn is_camel_case(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let first = s.chars().next().unwrap();
    if !first.is_ascii_lowercase() {
        return false;
    }
    // No hyphens or underscores in camelCase
    !s.contains('-') && !s.contains('_')
}

/// Check if a selector matches the specified style.
pub fn check_selector_style(selector: &str, style: SelectorStyle) -> bool {
    match style {
        SelectorStyle::KebabCase => is_kebab_case(selector),
        SelectorStyle::CamelCase => is_camel_case(selector),
    }
}

/// Check if a selector starts with one of the specified prefixes.
pub fn check_selector_prefix(selector: &str, prefixes: &[&str]) -> bool {
    if prefixes.is_empty() {
        return true;
    }
    prefixes.iter().any(|prefix| {
        if let Some(rest) = selector.strip_prefix(prefix) {
            // Check that after the prefix, either:
            // - The selector ends (exact match)
            // - The next char is uppercase (for camelCase: appButton)
            // - The next char is '-' (for kebab-case: app-button)
            rest.is_empty()
                || rest.starts_with('-')
                || rest.chars().next().is_some_and(|c| c.is_ascii_uppercase())
        } else {
            false
        }
    })
}

/// Parse a selector to determine its type (element vs attribute).
pub fn parse_selector_type(selector: &str) -> Option<SelectorType> {
    let trimmed = selector.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Attribute selector: [attrName] or [attr][other]
    if trimmed.starts_with('[') {
        return Some(SelectorType::Attribute);
    }

    // Element selector: element-name or element-name[attr]
    if trimmed.chars().next().is_some_and(|c| c.is_ascii_alphabetic()) {
        return Some(SelectorType::Element);
    }

    None
}

/// Extract the main selector name from a potentially complex selector.
/// For "[appDir]" returns "appDir", for "app-comp" returns "app-comp".
pub fn extract_selector_name(selector: &str) -> Option<&str> {
    let trimmed = selector.trim();

    // Handle attribute selectors
    if trimmed.starts_with('[') {
        let end = trimmed.find(']')?;
        let name = &trimmed[1..end];
        // Handle [attr=value] format
        if let Some(eq_pos) = name.find('=') {
            return Some(&name[..eq_pos]);
        }
        return Some(name);
    }

    // Handle element selectors
    // Find where the element name ends (at '[', '.', '#', ':' or end)
    let end_chars = ['[', '.', '#', ':'];
    let end_pos = trimmed.find(|c| end_chars.contains(&c)).unwrap_or(trimmed.len());
    let name = &trimmed[..end_pos];

    if name.is_empty() { None } else { Some(name) }
}

/// Check if an output name starts with "on" prefix.
pub fn has_on_prefix(name: &str) -> bool {
    if !name.starts_with("on") {
        return false;
    }
    // "on" alone is not allowed
    if name.len() == 2 {
        return true;
    }
    // "onX" where X is not lowercase is not allowed (e.g., "onClick", "onBlur")
    name.chars().nth(2).is_some_and(|c| !c.is_ascii_lowercase())
}

/// Find the span of a property key in an object expression.
pub fn find_property_key_span(obj: &ObjectExpression<'_>, key: &str) -> Option<Span> {
    use oxc_span::GetSpan;

    for property in &obj.properties {
        if let ObjectPropertyKind::ObjectProperty(prop) = property
            && let PropertyKey::StaticIdentifier(ident) = &prop.key
                && ident.name.as_str() == key {
                    return Some(ident.span());
                }
    }
    None
}

/// Get the string value from a metadata property if it's a string literal or static template literal.
pub fn get_metadata_string_value<'a>(obj: &'a ObjectExpression<'a>, key: &str) -> Option<&'a str> {
    let value = get_metadata_property(obj, key)?;
    match value {
        Expression::StringLiteral(lit) => Some(lit.value.as_str()),
        Expression::TemplateLiteral(lit) => {
            // Only handle static templates (no expressions)
            if lit.expressions.is_empty() && lit.quasis.len() == 1 {
                Some(lit.quasis[0].value.raw.as_str())
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Check if a class implements a specific interface by checking the implements clause.
pub fn class_implements_interface(class: &oxc_ast::ast::Class<'_>, interface_name: &str) -> bool {
    if class.implements.is_empty() {
        return false;
    }

    class.implements.iter().any(|ts_impl| {
        if let oxc_ast::ast::TSTypeName::IdentifierReference(ident) = &ts_impl.expression {
            return ident.name.as_str() == interface_name;
        }
        false
    })
}

/// Get the decorator type from a decorator name.
pub fn get_decorator_type(name: &str) -> Option<AngularDecoratorType> {
    match name {
        "Component" => Some(AngularDecoratorType::Component),
        "Directive" => Some(AngularDecoratorType::Directive),
        "Injectable" => Some(AngularDecoratorType::Injectable),
        "Pipe" => Some(AngularDecoratorType::Pipe),
        "NgModule" => Some(AngularDecoratorType::NgModule),
        _ => None,
    }
}

/// Find Angular decorators on a class.
pub fn get_class_angular_decorator<'a, 'b>(
    class: &'b oxc_ast::ast::Class<'a>,
    ctx: &'b LintContext<'a>,
) -> Option<(AngularDecoratorType, &'b Decorator<'a>)> {
    for decorator in &class.decorators {
        let Some(name) = get_decorator_name(decorator) else {
            continue;
        };
        let Some(decorator_type) = get_decorator_type(name) else {
            continue;
        };
        // Verify it's from @angular/core
        let Some(ident) = get_decorator_identifier(decorator) else {
            continue;
        };
        if is_angular_core_import(ident, ctx) {
            return Some((decorator_type, decorator));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_legacy_angular_decorator() {
        assert!(is_legacy_angular_decorator("Input"));
        assert!(is_legacy_angular_decorator("Output"));
        assert!(is_legacy_angular_decorator("ViewChild"));
        assert!(!is_legacy_angular_decorator("Component"));
        assert!(!is_legacy_angular_decorator("Injectable"));
    }

    #[test]
    fn test_get_signal_replacement() {
        assert_eq!(get_signal_replacement("Input"), Some("input"));
        assert_eq!(get_signal_replacement("Output"), Some("output"));
        assert_eq!(get_signal_replacement("ViewChild"), Some("viewChild"));
        assert_eq!(get_signal_replacement("Unknown"), None);
    }

    #[test]
    fn test_is_lifecycle_method() {
        assert!(is_lifecycle_method("ngOnInit"));
        assert!(is_lifecycle_method("ngOnDestroy"));
        assert!(is_lifecycle_method("ngAfterViewInit"));
        assert!(!is_lifecycle_method("onClick"));
        assert!(!is_lifecycle_method("constructor"));
    }

    #[test]
    fn test_get_lifecycle_interface() {
        assert_eq!(get_lifecycle_interface_for_method("ngOnInit"), Some("OnInit"));
        assert_eq!(get_lifecycle_interface_for_method("ngOnDestroy"), Some("OnDestroy"));
        assert_eq!(get_lifecycle_interface_for_method("ngOnChanges"), Some("OnChanges"));
        assert_eq!(get_lifecycle_interface_for_method("unknown"), None);
    }

    #[test]
    fn test_is_native_event_name() {
        assert!(is_native_event_name("click"));
        assert!(is_native_event_name("blur"));
        assert!(is_native_event_name("change"));
        assert!(is_native_event_name("wheel"));
        assert!(!is_native_event_name("customEvent"));
        assert!(!is_native_event_name("valueChange"));
    }

    #[test]
    fn test_is_kebab_case() {
        assert!(is_kebab_case("my-component"));
        assert!(is_kebab_case("app-header"));
        assert!(is_kebab_case("app-header-nav"));
        assert!(!is_kebab_case("myComponent"));
        assert!(!is_kebab_case("MyComponent"));
        assert!(!is_kebab_case("-invalid"));
        assert!(!is_kebab_case("invalid-"));
        assert!(!is_kebab_case("my--component"));
    }

    #[test]
    fn test_is_camel_case() {
        assert!(is_camel_case("myComponent"));
        assert!(is_camel_case("appHeader"));
        assert!(is_camel_case("component"));
        assert!(!is_camel_case("MyComponent"));
        assert!(!is_camel_case("my-component"));
        assert!(!is_camel_case("my_component"));
    }

    #[test]
    fn test_check_selector_prefix() {
        assert!(check_selector_prefix("app-header", &["app"]));
        assert!(check_selector_prefix("appHeader", &["app"]));
        assert!(check_selector_prefix("my-component", &["app", "my"]));
        assert!(!check_selector_prefix("other-header", &["app"]));
        // With empty prefixes, everything passes
        assert!(check_selector_prefix("any-thing", &[]));
    }

    #[test]
    fn test_parse_selector_type() {
        assert_eq!(parse_selector_type("app-component"), Some(SelectorType::Element));
        assert_eq!(parse_selector_type("[appDir]"), Some(SelectorType::Attribute));
        assert_eq!(parse_selector_type("  app-component  "), Some(SelectorType::Element));
        assert_eq!(parse_selector_type(""), None);
    }

    #[test]
    fn test_extract_selector_name() {
        assert_eq!(extract_selector_name("app-component"), Some("app-component"));
        assert_eq!(extract_selector_name("[appDir]"), Some("appDir"));
        assert_eq!(extract_selector_name("[attr=value]"), Some("attr"));
        assert_eq!(extract_selector_name("app[attr]"), Some("app"));
    }

    #[test]
    fn test_has_on_prefix() {
        assert!(has_on_prefix("onClick"));
        assert!(has_on_prefix("onBlur"));
        assert!(has_on_prefix("on"));
        assert!(!has_on_prefix("online"));
        assert!(!has_on_prefix("opened"));
        assert!(!has_on_prefix("valueChange"));
    }

    #[test]
    fn test_get_decorator_type() {
        assert_eq!(get_decorator_type("Component"), Some(AngularDecoratorType::Component));
        assert_eq!(get_decorator_type("Directive"), Some(AngularDecoratorType::Directive));
        assert_eq!(get_decorator_type("Injectable"), Some(AngularDecoratorType::Injectable));
        assert_eq!(get_decorator_type("Unknown"), None);
    }

    #[test]
    fn test_lifecycle_valid_for_decorator() {
        // Component supports all except ngDoBootstrap
        assert!(is_lifecycle_valid_for_decorator("ngOnInit", AngularDecoratorType::Component));
        assert!(is_lifecycle_valid_for_decorator("ngOnDestroy", AngularDecoratorType::Component));
        assert!(!is_lifecycle_valid_for_decorator(
            "ngDoBootstrap",
            AngularDecoratorType::Component
        ));

        // Injectable only supports ngOnDestroy
        assert!(!is_lifecycle_valid_for_decorator("ngOnInit", AngularDecoratorType::Injectable));
        assert!(is_lifecycle_valid_for_decorator("ngOnDestroy", AngularDecoratorType::Injectable));

        // NgModule only supports ngDoBootstrap
        assert!(is_lifecycle_valid_for_decorator("ngDoBootstrap", AngularDecoratorType::NgModule));
        assert!(!is_lifecycle_valid_for_decorator("ngOnInit", AngularDecoratorType::NgModule));
    }
}
