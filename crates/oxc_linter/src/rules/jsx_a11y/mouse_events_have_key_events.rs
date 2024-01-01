use oxc_ast::{
    ast::{JSXAttributeValue, JSXExpression, JSXExpressionContainer},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{self, Error},
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    context::LintContext,
    globals::HTML_TAG,
    rule::Rule,
    utils::{get_element_type, get_prop_value, has_jsx_prop},
    AstNode,
};

#[derive(Debug, Error, Diagnostic)]
enum MouseEventsHaveKeyEventsDiagnostic {
    #[error("eslint-plugin-jsx-a11y(mouse-events-have-key-events): {1} must be accompanied by onFocus for accessibility.")]
    #[diagnostic(severity(warning), help("Try to add onFocus."))]
    MissOnFocus(#[label] Span, String),

    #[error("eslint-plugin-jsx-a11y(mouse-events-have-key-events): {1} must be accompanied by onBlur for accessibility.")]
    #[diagnostic(severity(warning), help("Try to add onBlur."))]
    MissOnBlur(#[label] Span, String),
}

#[derive(Debug, Default, Clone)]
pub struct MouseEventsHaveKeyEvents(Box<MouseEventsHaveKeyEventsConfig>);

#[derive(Debug, Clone)]
pub struct MouseEventsHaveKeyEventsConfig {
    hover_in_handlers: Vec<String>,
    hover_out_handlers: Vec<String>,
}

impl Default for MouseEventsHaveKeyEventsConfig {
    fn default() -> Self {
        Self {
            hover_in_handlers: vec!["onMouseOver".to_string()],
            hover_out_handlers: vec!["onMouseOut".to_string()],
        }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce onmouseover/onmouseout are accompanied by onfocus/onblur.
    ///
    /// ### Why is this bad?
    ///
    /// Coding for the keyboard is important for users with physical disabilities who cannot use a mouse,
    /// AT compatibility, and screenreader users.
    ///
    /// ### Example
    /// ```jsx
    /// // Good
    /// <div onMouseOver={() => void 0} onFocus={() => void 0} />
    ///
    /// // Bad
    /// <div onMouseOver={() => void 0} />
    /// ```
    MouseEventsHaveKeyEvents,
    correctness
);

impl Rule for MouseEventsHaveKeyEvents {
    fn from_configuration(value: serde_json::Value) -> Self {
        let mut config = MouseEventsHaveKeyEventsConfig::default();

        if let Some(hover_in_handlers_config) = value
            .get(0)
            .and_then(|v| v.get("hoverInHandlers"))
            .and_then(serde_json::Value::as_array)
        {
            config.hover_in_handlers = hover_in_handlers_config
                .iter()
                .filter_map(serde_json::Value::as_str)
                .map(ToString::to_string)
                .collect();
        }

        if let Some(hover_out_handlers_config) = value
            .get(0)
            .and_then(|v| v.get("hoverOutHandlers"))
            .and_then(serde_json::Value::as_array)
        {
            config.hover_out_handlers = hover_out_handlers_config
                .iter()
                .filter_map(serde_json::Value::as_str)
                .map(ToString::to_string)
                .collect();
        }

        Self(Box::new(config))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(jsx_opening_el) = node.kind() else {
            return;
        };

        let Some(el_type) = get_element_type(ctx, jsx_opening_el) else {
            return;
        };

        if !HTML_TAG.contains(&el_type) {
            return;
        }

        for handler in &self.0.hover_in_handlers {
            if let Some(jsx_attr) = has_jsx_prop(jsx_opening_el, handler) {
                if get_prop_value(jsx_attr).is_none() {
                    continue;
                }

                match has_jsx_prop(jsx_opening_el, "onFocus").and_then(get_prop_value) {
                    Some(JSXAttributeValue::ExpressionContainer(JSXExpressionContainer {
                        expression: JSXExpression::Expression(expr),
                        ..
                    })) => {
                        if expr.is_undefined() {
                            ctx.diagnostic(MouseEventsHaveKeyEventsDiagnostic::MissOnFocus(
                                jsx_attr.span(),
                                String::from(handler),
                            ));
                        }
                    }
                    None => {
                        ctx.diagnostic(MouseEventsHaveKeyEventsDiagnostic::MissOnFocus(
                            jsx_attr.span(),
                            String::from(handler),
                        ));
                    }
                    _ => {}
                }

                break;
            }
        }

        for handler in &self.0.hover_out_handlers {
            if let Some(jsx_attr) = has_jsx_prop(jsx_opening_el, handler) {
                if get_prop_value(jsx_attr).is_none() {
                    continue;
                }

                match has_jsx_prop(jsx_opening_el, "onBlur").and_then(get_prop_value) {
                    Some(JSXAttributeValue::ExpressionContainer(JSXExpressionContainer {
                        expression: JSXExpression::Expression(expr),
                        ..
                    })) => {
                        if expr.is_undefined() {
                            ctx.diagnostic(MouseEventsHaveKeyEventsDiagnostic::MissOnBlur(
                                jsx_attr.span(),
                                String::from(handler),
                            ));
                        }
                    }
                    None => {
                        ctx.diagnostic(MouseEventsHaveKeyEventsDiagnostic::MissOnBlur(
                            jsx_attr.span(),
                            String::from(handler),
                        ));
                    }
                    _ => {}
                }

                break;
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("<div onMouseOver={() => void 0} onFocus={() => void 0} />;", None),
        ("<div onMouseOver={() => void 0} onFocus={() => void 0} {...props} />;", None),
        ("<div onMouseOver={handleMouseOver} onFocus={handleFocus} />;", None),
        ("<div onMouseOver={handleMouseOver} onFocus={handleFocus} {...props} />;", None),
        ("<div />;", None),
        ("<div onBlur={() => {}} />", None),
        ("<div onFocus={() => {}} />", None),
        ("<div onMouseOut={() => void 0} onBlur={() => void 0} />", None),
        ("<div onMouseOut={() => void 0} onBlur={() => void 0} {...props} />", None),
        ("<div onMouseOut={handleMouseOut} onBlur={handleOnBlur} />", None),
        ("<div onMouseOut={handleMouseOut} onBlur={handleOnBlur} {...props} />", None),
        ("<MyElement />", None),
        ("<MyElement onMouseOver={() => {}} />", None),
        ("<MyElement onMouseOut={() => {}} />", None),
        ("<MyElement onBlur={() => {}} />", None),
        ("<MyElement onFocus={() => {}} />", None),
        ("<MyElement onMouseOver={() => {}} {...props} />", None),
        ("<MyElement onMouseOut={() => {}} {...props} />", None),
        ("<MyElement onBlur={() => {}} {...props} />", None),
        ("<MyElement onFocus={() => {}} {...props} />", None),
        (
            "<div onMouseOver={() => {}} onMouseOut={() => {}} />",
            Some(serde_json::json!([{ "hoverInHandlers": [], "hoverOutHandlers": [] }])),
        ),
        (
            "<div onMouseOver={() => {}} onFocus={() => {}} />",
            Some(serde_json::json!([{ "hoverInHandlers": ["onMouseOver"] }])),
        ),
        (
            "<div onMouseEnter={() => {}} onFocus={() => {}} />",
            Some(serde_json::json!([{ "hoverInHandlers": ["onMouseEnter"] }])),
        ),
        (
            "<div onMouseOut={() => {}} onBlur={() => {}} />",
            Some(serde_json::json!([{ "hoverOutHandlers": ["onMouseOut"] }])),
        ),
        (
            "<div onMouseLeave={() => {}} onBlur={() => {}} />",
            Some(serde_json::json!([{ "hoverOutHandlers": ["onMouseLeave"] }])),
        ),
        (
            "<div onMouseOver={() => {}} onMouseOut={() => {}} />",
            Some(serde_json::json!([
              { "hoverInHandlers": ["onPointerEnter"], "hoverOutHandlers": ["onPointerLeave"] },
            ])),
        ),
        (
            "<div onMouseLeave={() => {}} />",
            Some(serde_json::json!([{ "hoverOutHandlers": ["onPointerLeave"] }])),
        ),
    ];

    let fail = vec![
        ("<div onMouseOver={() => void 0} />;", None),
        ("<div onMouseOut={() => void 0} />", None),
        ("<div onMouseOver={() => void 0} onFocus={undefined} />;", None),
        ("<div onMouseOut={() => void 0} onBlur={undefined} />", None),
        ("<div onMouseOver={() => void 0} {...props} />", None),
        ("<div onMouseOut={() => void 0} {...props} />", None),
        (
            "<div onMouseOver={() => {}} onMouseOut={() => {}} />",
            Some(serde_json::json!([
              { "hoverInHandlers": ["onMouseOver"], "hoverOutHandlers": ["onMouseOut"] },
            ])),
        ),
        (
            "<div onPointerEnter={() => {}} onPointerLeave={() => {}} />",
            Some(serde_json::json!([
              { "hoverInHandlers": ["onPointerEnter"], "hoverOutHandlers": ["onPointerLeave"] },
            ])),
        ),
        (
            "<div onMouseOver={() => {}} />",
            Some(serde_json::json!([{ "hoverInHandlers": ["onMouseOver"] }])),
        ),
        (
            "<div onPointerEnter={() => {}} />",
            Some(serde_json::json!([{ "hoverInHandlers": ["onPointerEnter"] }])),
        ),
        (
            "<div onMouseOut={() => {}} />",
            Some(serde_json::json!([{ "hoverOutHandlers": ["onMouseOut"] }])),
        ),
        (
            "<div onPointerLeave={() => {}} />",
            Some(serde_json::json!([{ "hoverOutHandlers": ["onPointerLeave"] }])),
        ),
    ];

    Tester::new(MouseEventsHaveKeyEvents::NAME, pass, fail).test_and_snapshot();
}
