use oxc_ast::{AstKind, ast::JSXAttributeItem};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    context::LintContext,
    globals::HTML_TAG,
    rule::{DefaultRuleConfig, Rule},
    utils::{get_element_type, has_jsx_prop},
};

fn no_autofocus_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("The `autoFocus` attribute is found here, which can cause usability issues for sighted and non-sighted users.")
        .with_help("Remove the `autoFocus` attribute.")
        .with_label(span)
}

#[derive(Debug, Default, Clone, JsonSchema, Deserialize)]
#[serde(default)]
pub struct NoAutofocus {
    /// Determines if developer-created components are checked.
    #[serde(rename = "ignoreNonDOM")]
    ignore_non_dom: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce that `autoFocus` prop is not used on elements.
    ///
    /// ### Why is this bad?
    ///
    /// Autofocusing elements can cause usability issues for sighted and
    /// non-sighted users alike. It can be disorienting when focus is shifted
    /// without user input and can interfere with assistive technologies.
    /// Users should control when and where focus moves on a page.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <div autoFocus />
    /// <div autoFocus="true" />
    /// <div autoFocus="false" />
    /// <div autoFocus={undefined} />
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <div />
    /// ```
    NoAutofocus,
    jsx_a11y,
    correctness,
    fix,
    config = NoAutofocus,
);

impl Rule for NoAutofocus {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        Ok(serde_json::from_value::<DefaultRuleConfig<Self>>(value)
            .unwrap_or_default()
            .into_inner())
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXElement(jsx_el) = node.kind() else {
            return;
        };
        let Some(autofocus) = has_jsx_prop(&jsx_el.opening_element, "autoFocus") else {
            return;
        };

        let element_type = get_element_type(ctx, &jsx_el.opening_element);

        if self.ignore_non_dom {
            if HTML_TAG.contains(element_type.as_ref())
                && let JSXAttributeItem::Attribute(attr) = autofocus
            {
                ctx.diagnostic_with_fix(no_autofocus_diagnostic(attr.span), |fixer| {
                    fixer.delete(&attr.span)
                });
            }
            return;
        }

        if let JSXAttributeItem::Attribute(attr) = autofocus {
            ctx.diagnostic_with_fix(no_autofocus_diagnostic(attr.span), |fixer| {
                fixer.delete(&attr.span)
            });
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
                    "Button": "button",
                }
            } }
        })
    }

    let pass = vec![
        ("<div />;", None, None),
        ("<div autofocus />;", None, None),
        ("<input autofocus='true' />;", None, None),
        ("<Foo bar />", None, None),
        ("<Button />", None, None),
        ("<Foo />", Some(serde_json::json!([{ "ignoreNonDOM": true }])), None),
        ("<Foo />", Some(serde_json::json!([{ "ignoreNonDOM": false }])), None),
        ("<Foo autoFocus />", Some(serde_json::json!([{ "ignoreNonDOM": true }])), None),
        ("<Foo autoFocus='true' />", Some(serde_json::json!([{ "ignoreNonDOM": true }])), None),
        ("<div><div autofocus /></div>", Some(serde_json::json!([{ "ignoreNonDOM": true }])), None),
        ("<Button />", None, Some(settings())),
        ("<Button />", Some(serde_json::json!([{ "ignoreNonDOM": true }])), Some(settings())),
    ];

    let fail = vec![
        ("<div autoFocus />", None, None),
        ("<div autoFocus={true} />", None, None),
        // the value of ignoreNonDOM should not impact these failing, as div is a dom element.
        ("<div autoFocus={true} />", Some(serde_json::json!([{ "ignoreNonDOM": true }])), None),
        ("<div autoFocus={true} />", Some(serde_json::json!([{ "ignoreNonDOM": false }])), None),
        ("<div autoFocus={false} />", None, None),
        ("<div autoFocus={undefined} />", None, None),
        ("<div autoFocus='true' />", None, None),
        ("<div autoFocus='false' />", None, None),
        ("<input autoFocus />", None, None),
        ("<Foo autoFocus />", None, None),
        ("<Button autoFocus />", None, None),
        (
            "<Button autoFocus />",
            Some(serde_json::json!([{ "ignoreNonDOM": true }])),
            Some(settings()),
        ),
    ];

    let fix = vec![
        ("<div autoFocus />", "<div  />"),
        ("<div autoFocus={true} />", "<div  />"),
        ("<div autoFocus='true' />", "<div  />"),
        ("<Button autoFocus='true' />", "<Button  />"),
        ("<input autoFocus />", "<input  />"),
        ("<div autoFocus>foo</div>", "<div >foo</div>"),
        ("<div autoFocus id='lol'>foo</div>", "<div  id='lol'>foo</div>"),
    ];

    Tester::new(NoAutofocus::NAME, NoAutofocus::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
