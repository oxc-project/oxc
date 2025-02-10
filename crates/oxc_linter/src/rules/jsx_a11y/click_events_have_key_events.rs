use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    globals::HTML_TAG,
    rule::Rule,
    utils::{
        get_element_type, has_jsx_prop, is_hidden_from_screen_reader, is_interactive_element,
        is_presentation_role,
    },
    AstNode,
};

fn click_events_have_key_events_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Enforce a clickable non-interactive element has at least one keyboard event listener.")
        .with_help("Visible, non-interactive elements with click handlers must have one of keyup, keydown, or keypress listener.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct ClickEventsHaveKeyEvents;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce onClick is accompanied by at least one of the following: onKeyUp, onKeyDown, onKeyPress.
    ///
    /// ### Why is this bad?
    ///
    /// Coding for the keyboard is important for users with physical disabilities who cannot use a mouse, AT compatibility, and screenreader users.
    /// This does not apply for interactive or hidden elements.
    ///
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <div onClick={() => void 0} />
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <div onClick={() => void 0} onKeyDown={() => void 0} />
    /// ```
    ClickEventsHaveKeyEvents,
    jsx_a11y,
    correctness
);

impl Rule for ClickEventsHaveKeyEvents {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(jsx_opening_el) = node.kind() else {
            return;
        };

        if has_jsx_prop(jsx_opening_el, "onClick").is_none() {
            return;
        };

        // Check only native DOM elements or custom component via settings
        let element_type = get_element_type(ctx, jsx_opening_el);

        if !HTML_TAG.contains(&element_type) {
            return;
        };

        if is_hidden_from_screen_reader(ctx, jsx_opening_el) || is_presentation_role(jsx_opening_el)
        {
            return;
        }

        if is_interactive_element(&element_type, jsx_opening_el) {
            return;
        }

        if ["onKeyUp", "onKeyDown", "onKeyPress"]
            .iter()
            .find_map(|prop| has_jsx_prop(jsx_opening_el, prop))
            .is_some()
        {
            return;
        }

        ctx.diagnostic(click_events_have_key_events_diagnostic(jsx_opening_el.span));
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (r"<div onClick={() => void 0} onKeyDown={foo}/>;", None, None, None),
        (r"<div onClick={() => void 0} onKeyUp={foo} />;", None, None, None),
        (r"<div onClick={() => void 0} onKeyPress={foo}/>;", None, None, None),
        (r"<div onClick={() => void 0} onKeyDown={foo} onKeyUp={bar} />;", None, None, None),
        (r"<div onClick={() => void 0} onKeyDown={foo} {...props} />;", None, None, None),
        (r#"<div className="foo" />;"#, None, None, None),
        (r"<div onClick={() => void 0} aria-hidden />;", None, None, None),
        (r"<div onClick={() => void 0} aria-hidden={true} />;", None, None, None),
        (r"<div onClick={() => void 0} aria-hidden={false} onKeyDown={foo} />;", None, None, None),
        (
            r"<div onClick={() => void 0} onKeyDown={foo} aria-hidden={undefined} />;",
            None,
            None,
            None,
        ),
        (r#"<input type="text" onClick={() => void 0} />"#, None, None, None),
        (r"<input onClick={() => void 0} />", None, None, None),
        (r#"<button onClick={() => void 0} className="foo" />"#, None, None, None),
        (r#"<select onClick={() => void 0} className="foo" />"#, None, None, None),
        (r#"<textarea onClick={() => void 0} className="foo" />"#, None, None, None),
        (r#"<a onClick={() => void 0} href="http://x.y.z" />"#, None, None, None),
        (r#"<a onClick={() => void 0} href="http://x.y.z" tabIndex="0" />"#, None, None, None),
        (r#"<input onClick={() => void 0} type="hidden" />;"#, None, None, None),
        (r#"<div onClick={() => void 0} role="presentation" />;"#, None, None, None),
        (r#"<div onClick={() => void 0} role="none" />;"#, None, None, None),
        (r"<TestComponent onClick={doFoo} />", None, None, None),
        (r"<Button onClick={doFoo} />", None, None, None),
        (r"<Footer onClick={doFoo} />", None, None, None),
    ];

    let fail = vec![
        (r"<div onClick={() => void 0} />;", None, None, None),
        (r"<div onClick={() => void 0} role={undefined} />;", None, None, None),
        (r"<div onClick={() => void 0} {...props} />;", None, None, None),
        (r"<section onClick={() => void 0} />;", None, None, None),
        (r"<main onClick={() => void 0} />;", None, None, None),
        (r"<article onClick={() => void 0} />;", None, None, None),
        (r"<header onClick={() => void 0} />;", None, None, None),
        (r"<footer onClick={() => void 0} />;", None, None, None),
        (r"<div onClick={() => void 0} aria-hidden={false} />;", None, None, None),
        (r"<a onClick={() => void 0} />", None, None, None),
        (r#"<a tabIndex="0" onClick={() => void 0} />"#, None, None, None),
        (
            r"<Footer onClick={doFoo} />",
            None,
            Some(serde_json::json!({
                "settings": { "jsx-a11y": {
                    "components": {
                        "Footer": "footer",
                    }
                } }
            })),
            None,
        ),
    ];

    Tester::new(ClickEventsHaveKeyEvents::NAME, ClickEventsHaveKeyEvents::PLUGIN, pass, fail)
        .test_and_snapshot();
}
