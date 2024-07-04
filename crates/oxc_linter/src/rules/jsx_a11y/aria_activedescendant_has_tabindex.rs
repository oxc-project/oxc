use oxc_ast::{
    ast::{JSXAttribute, JSXAttributeItem, JSXElementName},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    globals::HTML_TAG,
    rule::Rule,
    utils::{get_element_type, has_jsx_prop_lowercase, is_interactive_element, parse_jsx_value},
    AstNode,
};

fn aria_activedescendant_has_tabindex_diagnostic(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("eslint-plugin-jsx-a11y(aria-activedescendant-has-tabindex): Enforce elements with aria-activedescendant are tabbable.")
        .with_help("An element that manages focus with `aria-activedescendant` must have a tabindex.")
        .with_label(span0)
}

#[derive(Debug, Default, Clone)]
pub struct AriaActivedescendantHasTabindex;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce elements with aria-activedescendant are tabbable.
    ///
    /// ### Example
    /// ```jsx
    /// // Good
    /// <CustomComponent />
    /// <CustomComponent aria-activedescendant={someID} />
    /// <CustomComponent aria-activedescendant={someID} tabIndex={0} />
    /// <CustomComponent aria-activedescendant={someID} tabIndex={-1} />
    /// <div />
    /// <input />
    /// <div tabIndex={0} />
    /// <div aria-activedescendant={someID} tabIndex={0} />
    /// <div aria-activedescendant={someID} tabIndex="0" />
    /// <div aria-activedescendant={someID} tabIndex={1} />
    /// <div aria-activedescendant={someID} tabIndex={-1} />
    /// <div aria-activedescendant={someID} tabIndex="-1" />
    /// <input aria-activedescendant={someID} />
    /// <input aria-activedescendant={someID} tabIndex={0} />
    /// <input aria-activedescendant={someID} tabIndex={-1} />
    ///
    /// // Bad
    /// <div aria-activedescendant={someID} />
    /// ```
    AriaActivedescendantHasTabindex,
    correctness
);

impl Rule for AriaActivedescendantHasTabindex {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(jsx_opening_el) = node.kind() else {
            return;
        };

        if has_jsx_prop_lowercase(jsx_opening_el, "aria-activedescendant").is_none() {
            return;
        };

        let Some(element_type) = get_element_type(ctx, jsx_opening_el) else {
            return;
        };

        if !HTML_TAG.contains(&element_type) {
            return;
        };

        if let Some(JSXAttributeItem::Attribute(tab_index_attr)) =
            has_jsx_prop_lowercase(jsx_opening_el, "tabIndex")
        {
            if !is_valid_tab_index_attr(tab_index_attr) {
                return;
            }
        } else if is_interactive_element(&element_type, jsx_opening_el) {
            return;
        }

        let JSXElementName::Identifier(identifier) = &jsx_opening_el.name else {
            return;
        };

        ctx.diagnostic(aria_activedescendant_has_tabindex_diagnostic(identifier.span));
    }
}

fn is_valid_tab_index_attr(attr: &JSXAttribute) -> bool {
    attr.value
        .as_ref()
        .and_then(|value| parse_jsx_value(value).ok())
        .map_or(false, |parsed_value| parsed_value < -1.0)
}

#[test]
fn test() {
    use crate::tester::Tester;

    fn settings() -> serde_json::Value {
        serde_json::json!({
            "settings": { "jsx-a11y": {
                "components": {
                    "CustomComponent": "div",
                }
            } }
        })
    }

    let pass = vec![
        (r"<CustomComponent />;", None, None, None),
        (r"<CustomComponent aria-activedescendant={someID} />;", None, None, None),
        (r"<CustomComponent aria-activedescendant={someID} tabIndex={0} />;", None, None, None),
        (r"<CustomComponent aria-activedescendant={someID} tabIndex={-1} />;", None, None, None),
        (
            r"<CustomComponent aria-activedescendant={someID} tabIndex={0} />;",
            None,
            Some(settings()),
            None,
        ),
        (r"<div />;", None, None, None),
        (r"<input />;", None, None, None),
        (r"<div tabIndex={0} />;", None, None, None),
        (r"<div aria-activedescendant={someID} tabIndex={0} />;", None, None, None),
        (r"<div aria-activedescendant={someID} tabIndex='0' />;", None, None, None),
        (r"<div aria-activedescendant={someID} tabIndex={1} />;", None, None, None),
        (r"<input aria-activedescendant={someID} />;", None, None, None),
        (r"<input aria-activedescendant={someID} tabIndex={1} />;", None, None, None),
        (r"<input aria-activedescendant={someID} tabIndex={0} />;", None, None, None),
        (r"<input aria-activedescendant={someID} tabIndex={-1} />;", None, None, None),
        (r"<div aria-activedescendant={someID} tabIndex={-1} />;", None, None, None),
        (r"<div aria-activedescendant={someID} tabIndex='-1' />;", None, None, None),
        (r"<input aria-activedescendant={someID} tabIndex={-1} />;", None, None, None),
    ];

    let fail = vec![
        (r"<div aria-activedescendant={someID} />;", None, None, None),
        (r"<CustomComponent aria-activedescendant={someID} />;", None, Some(settings()), None),
    ];

    Tester::new(AriaActivedescendantHasTabindex::NAME, pass, fail).test_and_snapshot();
}
