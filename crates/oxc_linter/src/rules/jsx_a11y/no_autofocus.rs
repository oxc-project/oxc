use oxc_ast::{AstKind, ast::JSXAttributeItem};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use schemars::JsonSchema;

use crate::{
    AstNode,
    context::LintContext,
    globals::HTML_TAG,
    rule::Rule,
    utils::{get_element_type, has_jsx_prop},
};

fn no_autofocus_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("The `autofocus` attribute is found here, which can cause usability issues for sighted and non-sighted users")
        .with_help("Remove `autofocus` attribute")
        .with_label(span)
}

#[derive(Debug, Default, Clone, JsonSchema)]
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

impl NoAutofocus {
    pub fn set_option(&mut self, value: bool) {
        self.ignore_non_dom = value;
    }
}

impl Rule for NoAutofocus {
    fn from_configuration(value: serde_json::Value) -> Self {
        let mut no_focus = Self::default();

        if let Some(arr) = value.as_array()
            && arr.iter().any(|v| {
                if let serde_json::Value::Object(obj) = v
                    && let Some(serde_json::Value::Bool(val)) = obj.get("ignoreNonDOM")
                {
                    return *val;
                }
                false
            })
        {
            no_focus.set_option(true);
        }

        no_focus
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
    fn config() -> serde_json::Value {
        serde_json::json!([2,{
            "ignoreNonDOM": true
        }])
    }

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
        ("<Foo autoFocus />", Some(config()), None),
        ("<div><div autofocus /></div>", Some(config()), None),
        ("<Button />", None, Some(settings())),
        ("<Button />", Some(config()), Some(settings())),
    ];

    let fail = vec![
        ("<div autoFocus />", None, None),
        ("<div autoFocus={true} />", None, None),
        ("<div autoFocus={false} />", None, None),
        ("<div autoFocus={undefined} />", None, None),
        ("<div autoFocus='true' />", None, None),
        ("<div autoFocus='false' />", None, None),
        ("<input autoFocus />", None, None),
        ("<Foo autoFocus />", None, None),
        ("<Button autoFocus />", None, None),
        ("<Button autoFocus />", Some(config()), Some(settings())),
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
