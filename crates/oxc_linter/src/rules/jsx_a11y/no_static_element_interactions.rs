use oxc_ast::{AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, Span};

use crate::{
    AstNode,
    context::LintContext,
    globals::HTML_TAG,
    rule::Rule,
    utils::{has_jsx_prop, get_prop_value, get_element_type,
        is_hidden_from_screen_reader, is_presentation_role,
        is_interactive_element, is_interactive_role, is_noninteractive_element,
        is_noninteractive_role, is_abstract_role, is_nonliteral_property},
};

fn no_static_element_interactions_diagnostic(span: Span) -> OxcDiagnostic {
    // See <https://oxc.rs/docs/contribute/linter/adding-rules.html#diagnostics> for details
    OxcDiagnostic::warn("Do not attach interactive event handlers to static elements without an interactive role.")
        .with_help("Use a semantic interactive element (like <button> or <a>) or add an appropriate role if this element must handle interaction.")
        .with_label(span)
}

#[derive(Debug, Clone)]
pub struct NoStaticElementInteractions(Box<NoStaticElementInteractionsConfig>);

#[derive(Debug, Clone)]
struct NoStaticElementInteractionsConfig {
    handlers: Vec<CompactStr>,
    allow_expression_values: bool,
}

impl Default for NoStaticElementInteractions {
    fn default() -> Self {
        Self(Box::new(NoStaticElementInteractionsConfig {
            handlers: vec![
                CompactStr::new("onClick"),
                CompactStr::new("onMouseDown"),
                CompactStr::new("onMouseUp"),
                CompactStr::new("onKeyPress"),
                CompactStr::new("onKeyDown"),
                CompactStr::new("onKeyUp"),
            ],
            allow_expression_values: true
        }))
    }
}

// See <https://github.com/oxc-project/oxc/issues/6050> for documentation details.
declare_oxc_lint!(
    /// ### What it does
    ///
    /// Flags static, non-semantic elements (like `<div>`/`<span>` and other tags without an
    /// implicit ARIA role) that attach mouse or keyboard handlers without also declaring an
    /// appropriate `role`.
    ///
    /// If you make something clickable/focusable, it needs a semantic role so assistive tech can
    /// announce what it is.
    ///
    /// The WAI-ARIA `role` attribute maps an element to a semantic control. If you add interactive
    /// behavior to a static element, you should give it a role (and, typically, pair this with
    /// focus and keyboard support handled by other rules).
    ///
    /// Common interactive roles include:
    /// * button
    /// * link
    /// * checkbox
    /// * menuitem
    /// * menuitemcheckbox
    /// * menuitemradio
    /// * option
    /// * radio
    /// * searchbox
    /// * switch
    /// * textbox
    ///
    /// Note: Adding a role to your element does not add behavior. When a semantic HTML element
    /// like <button> is used, then it will also respond to Enter key presses when it has focus.
    /// The developer is responsible for providing the expected behavior of an element that the
    /// role suggests it would have: focusability and key press support.
    ///
    /// ### Why is this bad?
    ///
    /// Assigning interactivity to a static element without a role is invisible to many users.
    /// Users of assistive technology will not be made aware of the interactivity, and will not be
    /// able to use it.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    ///
    /// ```jsx
    /// <div
    ///   onClick={onClickHandler}
    ///   onKeyPress={onKeyPressHandler}
    ///   tabindex="0">
    ///   Save
    /// </div>
    /// ```
    ///
    /// ```jsx
    /// <div onClick={onClickHandler}>Save</div>
    /// ```
    ///
    /// ```jsx
    /// <span onKeyDown={onKeyDownHandler} tabIndex={0}>Open</span>
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    ///
    /// ```jsx
    /// <div
    ///   onClick={onClickHandler}
    ///   onKeyPress={onKeyPressHandler}
    ///   role="button"
    ///   tabindex="0">
    ///   Save
    /// </div>
    /// ```
    ///
    /// Provide an appropriate role:
    ///
    /// ```jsx
    /// <div role="button" onClick={onClickHandler}>Save</div>
    /// ```
    ///
    /// Prefer native controls when possible (best):
    /// ```jsx
    /// <button onClick={onClickHandler}>Save</button>
    /// ```
    ///
    /// ### Options
    ///
    /// #### `allowExpressionValues`
    ///
    /// `{ type: boolean, default: true }`
    ///
    /// Dynamic role values are allowed when `allowExpressionValues` is true:
    /// ```jsx
    /// <div role={isLink ? "link" : "button"} onClick={onClickHandler}>Go</div>
    /// ```
    ///
    /// #### `handlers`
    ///
    /// ```ts
    /// {
    ///     type: string[],
    ///     default: [
    ///         "onClick",
    ///         "onMouseDown",
    ///         "onMouseUp",
    ///         "onKeyPress",
    ///         "onKeyDown",
    ///         "onKeyUp",
    ///     ]
    /// }
    /// ```
    ///
    NoStaticElementInteractions,
    jsx_a11y,
    suspicious,
);

impl Rule for NoStaticElementInteractions {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(jsx_opening_el) = node.kind() else {
            return;
        };

        let element_type = get_element_type(ctx, jsx_opening_el);
        if !HTML_TAG.contains(element_type.as_ref()) {
            return;
        }

        let has_interactive_props = self.0.handlers.iter().any(|prop| {
            has_jsx_prop(jsx_opening_el, prop)
                .is_some_and(|attr| get_prop_value(attr)
                .is_some())
        });

        if !has_interactive_props
            || is_hidden_from_screen_reader(ctx, jsx_opening_el)
            || is_presentation_role(jsx_opening_el)
        {
            return;
        }

        if is_interactive_element(element_type.as_ref(), jsx_opening_el)
            || is_interactive_role(jsx_opening_el)
            || is_noninteractive_element(element_type.as_ref(), jsx_opening_el)
            || is_noninteractive_role(element_type.as_ref(), jsx_opening_el)
            || is_abstract_role(element_type.as_ref(), jsx_opening_el)
        {
            // This rule has no opinion about abstract roles.
            return;
        }

        if self.0.allow_expression_values && is_nonliteral_property(jsx_opening_el) {
            return;
        }

        ctx.diagnostic(no_static_element_interactions_diagnostic(jsx_opening_el.span));
    }

    fn from_configuration(value: serde_json::Value) -> Self {
        let default = Self::default();

        let Some(config) = value.get(0) else {
            return default;
        };

        Self(Box::new(NoStaticElementInteractionsConfig {
            handlers: config
                .get("handlers")
                .and_then(serde_json::Value::as_array)
                .map_or(default.0.handlers, |v| {
                    v.iter().map(|v| CompactStr::new(v.as_str().unwrap())).collect()
                }),
            allow_expression_values: config
                .get("allowExpressionValues")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(default.0.allow_expression_values),
        }))
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("<div onCopy={() => {}} />;", None),
        ("<div role={ROLE_BUTTON} onClick={() => {}} />;", None),
        (
            "<div role={BUTTON} onClick={() => {}} />;",
            Some(serde_json::json!([{ "allowExpressionValues": true }])),
        ),
        (
            r#"<div role={isButton ? "button" : "link"} onClick={() => {}} />;"#,
            Some(serde_json::json!([{ "allowExpressionValues": true }])),
        ),
        (r#"<div role="window" onClick={() => {}} />;"#, None),
        (r#"<div role="button" onClick={() => {}} />;"#, None),
        (r#"<div role="dialog" onClick={() => {}} />;"#, None),
        (r#"<BadTag onClick={() => {}} />;"#, None),
        (r#"<input type="hidden" onClick={() => {}} />;"#, None),
        (r#"<div aria-hidden onClick={() => {}} />;"#, None),
        (r#"<aside aria-label onClick={() => {}} />;"#, None),
    ];

    let fail = vec![
        ("<div onMouseDown={() => {}} />;", None),
        ("<div onClick={() => {}} />;", None),
        ("<div onKeyUp={() => {}} />;", None),
        (r#"<div role="badrole" onClick={() => {}} />;"#, None),
        (
            "<div role={BUTTON} onClick={() => {}} />;",
            Some(serde_json::json!([{ "allowExpressionValues": false }])),
        ),
        (
            r#"<div role={isButton ? "button" : "link"} onClick={() => {}} />;"#,
            Some(serde_json::json!([{ "allowExpressionValues": false }])),
        ),
        (
            "<div onCopy={() => {}} />;",
            Some(serde_json::json!([{ "handlers": ["onCopy"] }]))
        ),
        (r#"<div type="hidden" onClick={() => {}} />;"#, None),
    ];

    Tester::new(NoStaticElementInteractions::NAME, NoStaticElementInteractions::PLUGIN, pass, fail)
        .test_and_snapshot();
}
