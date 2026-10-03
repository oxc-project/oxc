use oxc_ast::{
    AstKind,
    ast::{Expression, JSXAttributeItem, JSXAttributeValue},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_str::CompactStr;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    context::LintContext,
    globals::HTML_TAG,
    rule::{DefaultRuleConfig, Rule},
    utils::{get_element_type, get_prop_value, has_jsx_prop},
};

fn miss_on_focus(span: Span, attr_name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "`{attr_name}` must be accompanied by `onFocus` for accessibility."
    ))
    .with_help("Try to add `onFocus`.")
    .with_label(span)
}

fn miss_on_blur(span: Span, attr_name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("`{attr_name}` must be accompanied by `onBlur` for accessibility."))
        .with_help("Try to add `onBlur`.")
        .with_label(span)
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct MouseEventsHaveKeyEvents(Box<MouseEventsHaveKeyEventsConfig>);

#[derive(Debug, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct MouseEventsHaveKeyEventsConfig {
    /// List of hover-in mouse event handlers that require corresponding keyboard event handlers.
    hover_in_handlers: Vec<CompactStr>,
    /// List of hover-out mouse event handlers that require corresponding keyboard event handlers.
    hover_out_handlers: Vec<CompactStr>,
}

impl Default for MouseEventsHaveKeyEventsConfig {
    fn default() -> Self {
        Self {
            hover_in_handlers: vec!["onMouseOver".into()],
            hover_out_handlers: vec!["onMouseOut".into()],
        }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce `onMouseOver`/`onMouseOut` are accompanied by `onFocus`/`onBlur`.
    ///
    /// ### Why is this bad?
    ///
    /// Coding for the keyboard is important for users with physical disabilities who cannot use a mouse,
    /// AT compatibility, and screen reader users.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <div onMouseOver={() => void 0} />
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <div onMouseOver={() => void 0} onFocus={() => void 0} />
    /// ```
    MouseEventsHaveKeyEvents,
    jsx_a11y,
    correctness,
    config = MouseEventsHaveKeyEventsConfig,
    version = "0.1.1",
    short_description = "Enforce `onMouseOver`/`onMouseOut` are accompanied by `onFocus`/`onBlur`.",
);

impl Rule for MouseEventsHaveKeyEvents {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        DefaultRuleConfig::<Self>::from_value(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(jsx_opening_el) = node.kind() else {
            return;
        };

        let el_type = get_element_type(ctx, jsx_opening_el);

        if !HTML_TAG.contains(el_type.as_ref()) {
            return;
        }

        for handler in &self.0.hover_in_handlers {
            if let Some(jsx_attr) = has_jsx_prop(jsx_opening_el, handler) {
                if !has_handler_value(jsx_attr) {
                    continue;
                }

                if !has_jsx_prop(jsx_opening_el, "onFocus").is_some_and(has_handler_value) {
                    ctx.diagnostic(miss_on_focus(jsx_attr.span(), handler));
                }

                break;
            }
        }

        for handler in &self.0.hover_out_handlers {
            if let Some(jsx_attr) = has_jsx_prop(jsx_opening_el, handler) {
                if !has_handler_value(jsx_attr) {
                    continue;
                }

                if !has_jsx_prop(jsx_opening_el, "onBlur").is_some_and(has_handler_value) {
                    ctx.diagnostic(miss_on_blur(jsx_attr.span(), handler));
                }

                break;
            }
        }
    }
}

fn has_handler_value(attribute: &JSXAttributeItem<'_>) -> bool {
    match get_prop_value(attribute) {
        Some(JSXAttributeValue::ExpressionContainer(container)) => {
            let Some(expression) = container.expression.as_expression() else {
                return true;
            };
            let expression = expression.get_inner_expression();
            !expression.is_undefined() && !matches!(expression, Expression::NullLiteral(_))
        }
        Some(_) => true,
        None => false,
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

    Tester::new(MouseEventsHaveKeyEvents::NAME, MouseEventsHaveKeyEvents::PLUGIN, pass, fail)
        .test_and_snapshot();
}

#[test]
fn test_nullish_handlers() {
    use crate::tester::Tester;

    let configured = Some(serde_json::json!([{
        "hoverInHandlers": ["onMouseOver", "onMouseEnter"],
        "hoverOutHandlers": ["onMouseOut", "onMouseLeave"]
    }]));
    let pass = vec![
        ("<div onMouseOver={undefined} />", None),
        ("<div onMouseOut={undefined} />", None),
        ("<div onMouseOver={null} />", None),
        ("<div onMouseOut={null} />", None),
        ("<div onMouseOver={undefined} onMouseOut={null} />", None),
        ("<div onMouseOver />", None),
        ("<div onMouseOut />", None),
        ("<div onMouseOver={handler} onFocus={handler} />", None),
        ("<div onMouseOut={handler} onBlur={handler} />", None),
        ("<div onMouseOver={null} onMouseEnter={handler} onFocus={handler} />", configured.clone()),
        (
            "<div onMouseOut={undefined} onMouseLeave={handler} onBlur={handler} />",
            configured.clone(),
        ),
        ("<Custom onMouseOver={handler} onFocus={null} />", None),
    ];
    let fail = vec![
        ("<div onMouseOver={handler} onFocus={null} />", None),
        ("<div onMouseOut={handler} onBlur={null} />", None),
        ("<div onMouseOver={handler} onFocus={undefined} />", None),
        ("<div onMouseOut={handler} onBlur={undefined} />", None),
        ("<div onMouseOver={handler} onFocus />", None),
        ("<div onMouseOut={handler} onBlur />", None),
        ("<div onMouseOver={null} onMouseEnter={handler} />", configured.clone()),
        ("<div onMouseOut={undefined} onMouseLeave={handler} />", configured.clone()),
        ("<div onMouseOver={null} onMouseEnter={handler} onFocus={null} />", configured.clone()),
        ("<div onMouseOut={undefined} onMouseLeave={handler} onBlur={null} />", configured),
    ];
    Tester::new(MouseEventsHaveKeyEvents::NAME, MouseEventsHaveKeyEvents::PLUGIN, pass, fail)
        .with_snapshot_suffix("nullish_handlers")
        .test_and_snapshot();
}
