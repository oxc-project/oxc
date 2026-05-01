use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_str::CompactStr;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    context::LintContext,
    fixer::RuleFixer,
    rule::{DefaultRuleConfig, Rule},
    utils::{
        KEYBOARD_EVENT_HANDLERS, MOUSE_EVENT_HANDLERS, get_element_type,
        get_string_literal_prop_value, has_jsx_prop, has_jsx_prop_ignore_case, is_disabled_element,
        is_hidden_from_screen_reader, is_interactive_element, is_interactive_role,
        is_non_interactive_element, is_non_interactive_role, is_presentation_role,
    },
};

fn must_be_tabbable_diagnostic(role: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Elements with the '{role}' interactive role must be tabbable."))
        .with_help(
            "Add `tabIndex={0}` to make the element reachable via sequential keyboard navigation.",
        )
        .with_label(span)
}

fn must_be_focusable_diagnostic(role: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Elements with the '{role}' interactive role must be focusable."))
        .with_help("Add `tabIndex={0}` or `tabIndex={-1}` to make the element focusable.")
        .with_label(span)
}

const EVENT_HANDLERS: &[&[&str]] = &[MOUSE_EVENT_HANDLERS, KEYBOARD_EVENT_HANDLERS];

const DEFAULT_TABBABLE: &[&str] =
    &["button", "checkbox", "link", "searchbox", "spinbutton", "switch", "textbox"];

#[derive(Debug, Default, Clone, Deserialize)]
pub struct InteractiveSupportsFocus(Box<InteractiveSupportsFocusConfig>);

#[derive(Debug, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct InteractiveSupportsFocusConfig {
    /// An array of interactive ARIA roles that should be considered tabbable (require `tabIndex={0}`).
    /// Interactive roles not in this list are only required to be focusable (`tabIndex={-1}` is sufficient).
    /// Defaults to `["button", "checkbox", "link", "searchbox", "spinbutton", "switch", "textbox"]`.
    tabbable: Vec<CompactStr>,
}

impl Default for InteractiveSupportsFocusConfig {
    fn default() -> Self {
        Self { tabbable: DEFAULT_TABBABLE.iter().map(|s| CompactStr::new(s)).collect() }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce that elements with interactive roles and interaction handlers
    /// (mouse or key press) must be focusable.
    ///
    /// ### Why is this bad?
    ///
    /// Elements that handle user interaction (e.g., `onClick`) but are not
    /// natively focusable (like `<div>` or `<span>`) must be made focusable so
    /// that keyboard-only users and assistive technology users can reach and
    /// activate them.
    ///
    /// Without a `tabIndex`, these elements are unreachable via keyboard navigation,
    /// creating a barrier for users who cannot use a mouse.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <span onClick={submitForm} role="button">Submit</span>
    /// <a onClick={showNextPage} role="button">Next page</a>
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <div aria-hidden onClick={() => void 0} />
    /// <span onClick={doSomething} tabIndex={0} role="button">Click me!</span>
    /// <span onClick={doSomething} tabIndex={-1} role="menuitem">Click me too!</span>
    /// <a href="javascript:void(0);" onClick={doSomething}>Click ALL the things!</a>
    /// <button onClick={doSomething}>Click the button :)</button>
    /// ```
    InteractiveSupportsFocus,
    jsx_a11y,
    correctness,
    suggestion,
    config = InteractiveSupportsFocusConfig,
    version = "next",
);

impl Rule for InteractiveSupportsFocus {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(jsx_el) = node.kind() else {
            return;
        };

        let role_str =
            has_jsx_prop_ignore_case(jsx_el, "role").and_then(get_string_literal_prop_value);
        let element_type = get_element_type(ctx, jsx_el);
        let has_interactive_handler = EVENT_HANDLERS
            .iter()
            .flat_map(|handlers| handlers.iter())
            .any(|handler| has_jsx_prop(jsx_el, handler).is_some());
        let has_tab_index = has_jsx_prop_ignore_case(jsx_el, "tabIndex").is_some();

        if !has_interactive_handler
            || is_disabled_element(jsx_el)
            || is_hidden_from_screen_reader(ctx, jsx_el)
            || is_presentation_role(jsx_el)
        {
            return;
        }

        let Some(role) = role_str else { return };
        if !is_interactive_role(role)
            || is_interactive_element(&element_type, jsx_el)
            || is_non_interactive_role(role)
            || is_non_interactive_element(&element_type, jsx_el)
            || has_tab_index
        {
            return;
        }

        if self.0.tabbable.iter().any(|t| t.as_str() == role) {
            ctx.diagnostic_with_suggestion(
                must_be_tabbable_diagnostic(role, jsx_el.span),
                |fixer| fixer.insert_text_after(&jsx_el.name, " tabIndex={0}"),
            );
        } else {
            let fixer = RuleFixer::new(FixKind::Suggestion, ctx);
            ctx.diagnostic_with_suggestions(
                must_be_focusable_diagnostic(role, jsx_el.span),
                [
                    fixer.insert_text_after(&jsx_el.name, " tabIndex={0}"),
                    fixer.insert_text_after(&jsx_el.name, " tabIndex={-1}"),
                ],
            );
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    fn settings() -> serde_json::Value {
        serde_json::json!({
            "settings": { "jsx-a11y": {
                "components": {
                    "Div": "div",
                }
            }}
        })
    }

    let pass = vec![
        (r"<div />", None, None),
        (r"<div aria-hidden onClick={() => void 0} />", None, None),
        (r"<div aria-hidden={true == true} onClick={() => void 0} />", None, None),
        (r"<div aria-hidden={true === true} onClick={() => void 0} />", None, None),
        (r"<div aria-hidden={hidden !== false} onClick={() => void 0} />", None, None),
        (r"<div aria-hidden={hidden != false} onClick={() => void 0} />", None, None),
        (r"<div aria-hidden={1 < 2} onClick={() => void 0} />", None, None),
        (r"<div aria-hidden={1 <= 2} onClick={() => void 0} />", None, None),
        (r"<div aria-hidden={2 > 1} onClick={() => void 0} />", None, None),
        (r"<div aria-hidden={2 >= 1} onClick={() => void 0} />", None, None),
        (r"<div onClick={() => void 0} />;", None, None),
        (r"<div onClick={() => void 0} tabIndex={undefined} />;", None, None),
        (r#"<div onClick={() => void 0} tabIndex="bad" />;"#, None, None),
        (r"<div onClick={() => void 0} role={undefined} />;", None, None),
        (r#"<div role="section" onClick={() => void 0} />"#, None, None),
        (r"<div onClick={() => void 0} aria-hidden={false} />;", None, None),
        (r"<div onClick={() => void 0} {...props} />;", None, None),
        (r#"<input type="text" onClick={() => void 0} />"#, None, None),
        (r#"<input type="hidden" onClick={() => void 0} tabIndex="-1" />"#, None, None),
        (r"<input type={`hidden`} onClick={() => void 0} tabIndex={-1} />", None, None),
        (r"<input onClick={() => void 0} />", None, None),
        (r#"<input onClick={() => void 0} role="combobox" />"#, None, None),
        (r#"<button onClick={() => void 0} className="foo" />"#, None, None),
        (r#"<option onClick={() => void 0} className="foo" />"#, None, None),
        (r#"<select onClick={() => void 0} className="foo" />"#, None, None),
        (r"<area href='#' onClick={() => void 0} className='foo' />", None, None),
        (r#"<area onClick={() => void 0} className="foo" />"#, None, None),
        (r"<summary onClick={() => void 0} />", None, None),
        (r#"<textarea onClick={() => void 0} className="foo" />"#, None, None),
        (r#"<a onClick="showNextPage();">Next page</a>"#, None, None),
        (r#"<a onClick="showNextPage();" tabIndex={undefined}>Next page</a>"#, None, None),
        (r#"<a onClick="showNextPage();" tabIndex="bad">Next page</a>"#, None, None),
        (r"<a onClick={() => void 0} />", None, None),
        (r#"<a tabIndex="0" onClick={() => void 0} />"#, None, None),
        (r"<a tabIndex={dynamicTabIndex} onClick={() => void 0} />", None, None),
        (r"<a tabIndex={0} onClick={() => void 0} />", None, None),
        (r"<a role='button' href='#' onClick={() => void 0} />", None, None),
        (r#"<a onClick={() => void 0} href="http://x.y.z" />"#, None, None),
        (r#"<a onClick={() => void 0} href="http://x.y.z" tabIndex="0" />"#, None, None),
        (r#"<a onClick={() => void 0} href="http://x.y.z" tabIndex={0} />"#, None, None),
        (r#"<a onClick={() => void 0} href="http://x.y.z" role="button" />"#, None, None),
        (r"<TestComponent onClick={doFoo} />", None, None),
        (r#"<input onClick={() => void 0} type="hidden" />;"#, None, None),
        (r"<span onClick='submitForm();'>Submit</span>", None, None),
        (r"<span onClick='submitForm();' tabIndex={undefined}>Submit</span>", None, None),
        (r"<span onClick='submitForm();' tabIndex='bad'>Submit</span>", None, None),
        (r"<span onClick='doSomething();' tabIndex='0'>Click me!</span>", None, None),
        (r"<span onClick='doSomething();' tabIndex={0}>Click me!</span>", None, None),
        (r"<span onClick='doSomething();' tabIndex='-1'>Click me too!</span>", None, None),
        (
            r#"<a href="javascript:void(0);" onClick='doSomething();'>Click ALL the things!</a>"#,
            None,
            None,
        ),
        (r"<section onClick={() => void 0} />;", None, None),
        (r"<main onClick={() => void 0} />;", None, None),
        (r"<article onClick={() => void 0} />;", None, None),
        (r"<header onClick={() => void 0} />;", None, None),
        (r"<footer onClick={() => void 0} />;", None, None),
        (r#"<div role="button" tabIndex="0" onClick={() => void 0} />"#, None, None),
        (r#"<div role="checkbox" tabIndex="0" onClick={() => void 0} />"#, None, None),
        (r#"<div role="link" tabIndex="0" onClick={() => void 0} />"#, None, None),
        (r#"<div role="menuitem" tabIndex="0" onClick={() => void 0} />"#, None, None),
        (r#"<div role="menuitemcheckbox" tabIndex="0" onClick={() => void 0} />"#, None, None),
        (r#"<div role="menuitemradio" tabIndex="0" onClick={() => void 0} />"#, None, None),
        (r#"<div role="option" tabIndex="0" onClick={() => void 0} />"#, None, None),
        (r#"<div role="radio" tabIndex="0" onClick={() => void 0} />"#, None, None),
        (r#"<div role="spinbutton" tabIndex="0" onClick={() => void 0} />"#, None, None),
        (r#"<div role="switch" tabIndex="0" onClick={() => void 0} />"#, None, None),
        (r#"<div role="tablist" tabIndex="0" onClick={() => void 0} />"#, None, None),
        (r#"<div role="tab" tabIndex="0" onClick={() => void 0} />"#, None, None),
        (r#"<div role="textbox" tabIndex="0" onClick={() => void 0} />"#, None, None),
        (r#"<div role="textbox" aria-disabled="true" onClick={() => void 0} />"#, None, None),
        (r"<Foo.Bar onClick={() => void 0} aria-hidden={false} />;", None, None),
        (r"<Input onClick={() => void 0} type='hidden' />;", None, None),
        (r"<Div onClick={() => void 0} role='button' tabIndex='0' />", None, Some(settings())),
        // interactive roles with non-triggering handlers (onFocus)
        (r#"<div role="button" onFocus={() => void 0} />"#, None, None),
        (r#"<div role="checkbox" onFocus={() => void 0} />"#, None, None),
        (r#"<div role="link" onFocus={() => void 0} />"#, None, None),
        (r#"<div role="gridcell" onFocus={() => void 0} />"#, None, None),
        (r#"<div role="menuitem" onFocus={() => void 0} />"#, None, None),
        (r#"<div role="menuitemcheckbox" onFocus={() => void 0} />"#, None, None),
        (r#"<div role="menuitemradio" onFocus={() => void 0} />"#, None, None),
        (r#"<div role="option" onFocus={() => void 0} />"#, None, None),
        (r#"<div role="radio" onFocus={() => void 0} />"#, None, None),
        (r#"<div role="searchbox" onFocus={() => void 0} />"#, None, None),
        (r#"<div role="slider" onFocus={() => void 0} />"#, None, None),
        (r#"<div role="spinbutton" onFocus={() => void 0} />"#, None, None),
        (r#"<div role="switch" onFocus={() => void 0} />"#, None, None),
        (r#"<div role="tab" onFocus={() => void 0} />"#, None, None),
        (r#"<div role="textbox" onFocus={() => void 0} />"#, None, None),
        (r#"<div role="treeitem" onFocus={() => void 0} />"#, None, None),
        // interactive roles with triggering handler + tabIndex="0"
        (r#"<div role="gridcell" onClick={() => void 0} tabIndex="0" />"#, None, None),
        (r#"<div role="menuitem" onClick={() => void 0} tabIndex="0" />"#, None, None),
        (r#"<div role="menuitemcheckbox" onClick={() => void 0} tabIndex="0" />"#, None, None),
        (r#"<div role="menuitemradio" onClick={() => void 0} tabIndex="0" />"#, None, None),
        (r#"<div role="option" onClick={() => void 0} tabIndex="0" />"#, None, None),
        (r#"<div role="radio" onClick={() => void 0} tabIndex="0" />"#, None, None),
        (r#"<div role="slider" onClick={() => void 0} tabIndex="0" />"#, None, None),
        (r#"<div role="tab" onClick={() => void 0} tabIndex="0" />"#, None, None),
        (r#"<div role="treeitem" onClick={() => void 0} tabIndex="0" />"#, None, None),
    ];

    let fail = vec![
        (r"<Div onClick={() => void 0} role='button' />", None, Some(settings())),
        (r#"<div role="button" onClick={() => void 0} />"#, None, None),
        (r#"<div role="checkbox" onClick={() => void 0} />"#, None, None),
        (r#"<div role="link" onClick={() => void 0} />"#, None, None),
        (r#"<div role="searchbox" onClick={() => void 0} />"#, None, None),
        (r#"<div role="spinbutton" onClick={() => void 0} />"#, None, None),
        (r#"<div role="switch" onClick={() => void 0} />"#, None, None),
        (r#"<div role="textbox" onClick={() => void 0} />"#, None, None),
        (r#"<div role="gridcell" onClick={() => void 0} />"#, None, None),
        (r#"<div role="menuitem" onClick={() => void 0} />"#, None, None),
        (r#"<div role="menuitemcheckbox" onClick={() => void 0} />"#, None, None),
        (r#"<div role="menuitemradio" onClick={() => void 0} />"#, None, None),
        (r#"<div role="option" onClick={() => void 0} />"#, None, None),
        (r#"<div role="radio" onClick={() => void 0} />"#, None, None),
        (r#"<div role="slider" onClick={() => void 0} />"#, None, None),
        (r#"<div role="tab" onClick={() => void 0} />"#, None, None),
        (r#"<div role="treeitem" onClick={() => void 0} />"#, None, None),
    ];

    let fix_tabbable = vec![(
        r#"<div role="button" onClick={() => void 0} />"#,
        r#"<div tabIndex={0} role="button" onClick={() => void 0} />"#,
    )];
    let fix_focusable = vec![(
        r#"<div role="menuitem" onClick={() => void 0} />"#,
        (
            r#"<div tabIndex={0} role="menuitem" onClick={() => void 0} />"#,
            r#"<div tabIndex={-1} role="menuitem" onClick={() => void 0} />"#,
        ),
    )];

    Tester::new(InteractiveSupportsFocus::NAME, InteractiveSupportsFocus::PLUGIN, pass, fail)
        .expect_fix(fix_tabbable)
        .expect_fix(fix_focusable)
        .test_and_snapshot();
}
