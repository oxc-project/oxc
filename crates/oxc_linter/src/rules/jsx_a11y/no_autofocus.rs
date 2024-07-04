use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    globals::HTML_TAG,
    rule::Rule,
    utils::{get_element_type, has_jsx_prop},
    AstNode,
};

fn no_autofocus_diagnostic(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("eslint-plugin-jsx-a11y(no-autofocus): The `autofocus` attribute is found here, which can cause usability issues for sighted and non-sighted users")
        .with_help("Remove `autofocus` attribute")
        .with_label(span0)
}

#[derive(Debug, Default, Clone)]
pub struct NoAutofocus {
    ignore_non_dom: bool,
}

declare_oxc_lint!(
    /// ### What it does
    /// Enforce that autoFocus prop is not used on elements. Autofocusing elements can cause usability issues for sighted and non-sighted users, alike.
    ///
    /// ### Rule Option
    /// This rule takes one optional object argument of type object:
    ///
    /// ```
    /// {
    ///     "rules": {
    ///         "jsx-a11y/no-autofocus": [ 2, {
    ///             "ignoreNonDOM": true
    ///         }],
    ///     }
    /// }
    /// ```
    ///
    /// For the `ignoreNonDOM` option, this determines if developer created components are checked.
    ///
    /// ### Example
    /// // good
    ///
    /// ```javascript
    /// <div />
    /// ```
    ///
    /// // bad
    ///
    /// ```
    /// <div autoFocus />
    /// <div autoFocus="true" />
    /// <div autoFocus="false" />
    /// <div autoFocus={undefined} />
    /// ```
    ///
    NoAutofocus,
    correctness
);

impl NoAutofocus {
    pub fn set_option(&mut self, value: bool) {
        self.ignore_non_dom = value;
    }
}

impl Rule for NoAutofocus {
    fn from_configuration(value: serde_json::Value) -> Self {
        let mut no_focus = Self::default();

        if let Some(arr) = value.as_array() {
            if arr.iter().any(|v| {
                if let serde_json::Value::Object(obj) = v {
                    if let Some(serde_json::Value::Bool(val)) = obj.get("ignoreNonDOM") {
                        return *val;
                    }
                }
                false
            }) {
                no_focus.set_option(true);
            }
        }

        no_focus
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::JSXElement(jsx_el) = node.kind() {
            if let Option::Some(autofocus) = has_jsx_prop(&jsx_el.opening_element, "autoFocus") {
                let Some(element_type) = get_element_type(ctx, &jsx_el.opening_element) else {
                    return;
                };
                if self.ignore_non_dom {
                    if HTML_TAG.contains(&element_type) {
                        if let oxc_ast::ast::JSXAttributeItem::Attribute(attr) = autofocus {
                            ctx.diagnostic(no_autofocus_diagnostic(attr.span));
                        }
                    }
                    return;
                }

                if let oxc_ast::ast::JSXAttributeItem::Attribute(attr) = autofocus {
                    ctx.diagnostic(no_autofocus_diagnostic(attr.span));
                }
            }
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
        ("<div />;", None, None, None),
        ("<div autofocus />;", None, None, None),
        ("<input autofocus='true' />;", None, None, None),
        ("<Foo bar />", None, None, None),
        ("<Button />", None, None, None),
        ("<Foo autoFocus />", Some(config()), None, None),
        ("<div><div autofocus /></div>", Some(config()), None, None),
        ("<Button />", None, Some(settings()), None),
        ("<Button />", Some(config()), Some(settings()), None),
    ];

    let fail = vec![
        ("<div autoFocus />", None, None, None),
        ("<div autoFocus={true} />", None, None, None),
        ("<div autoFocus={false} />", None, None, None),
        ("<div autoFocus={undefined} />", None, None, None),
        ("<div autoFocus='true' />", None, None, None),
        ("<div autoFocus='false' />", None, None, None),
        ("<input autoFocus />", None, None, None),
        ("<Foo autoFocus />", None, None, None),
        ("<Button autoFocus />", None, None, None),
        ("<Button autoFocus />", Some(config()), Some(settings()), None),
    ];

    Tester::new(NoAutofocus::NAME, pass, fail).test_and_snapshot();
}
