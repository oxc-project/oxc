use oxc_ast::{ast::JSXAttributeItem, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    globals::HTML_TAG,
    rule::Rule,
    utils::{get_element_type, has_jsx_prop_ignore_case},
    AstNode,
};

fn scope_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("The scope prop can only be used on <th> elements")
        .with_help("Must use scope prop only on <th> elements")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct Scope;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// The scope prop should be used only on `<th>` elements.
    ///
    /// ### Why is this bad?
    /// The scope attribute makes table navigation much easier for screen reader users, provided that it is used correctly.
    /// Incorrectly used, scope can make table navigation much harder and less efficient.
    /// A screen reader operates under the assumption that a table has a header and that this header specifies a scope. Because of the way screen readers function, having an accurate header makes viewing a table far more accessible and more efficient for people who use the device.
    ///
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <div scope />
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <th scope="col" />
    /// <th scope={scope} />
    /// ```
    Scope,
    jsx_a11y,
    correctness,
    fix
);

impl Rule for Scope {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(jsx_el) = node.kind() else {
            return;
        };

        let scope_attribute = match has_jsx_prop_ignore_case(jsx_el, "scope") {
            Some(v) => match v {
                JSXAttributeItem::Attribute(attr) => attr,
                JSXAttributeItem::SpreadAttribute(_) => {
                    return;
                }
            },
            None => {
                return;
            }
        };

        let element_type = get_element_type(ctx, jsx_el);

        if element_type == "th" {
            return;
        }

        if !HTML_TAG.contains(&element_type) {
            return;
        }

        ctx.diagnostic_with_fix(scope_diagnostic(scope_attribute.span), |fixer| {
            fixer.delete_range(scope_attribute.span)
        });
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    fn settings() -> serde_json::Value {
        serde_json::json!({
            "settings": { "jsx-a11y": {
                "components": {
                    "Foo": "div",
                    "TableHeader": "th"
                }
            } }
        })
    }

    let pass = vec![
        (r"<div />;", None, None),
        (r"<div foo />;", None, None),
        (r"<th scope />", None, None),
        (r"<th scope='row' />", None, None),
        (r"<th scope={foo} />", None, None),
        (r"<th scope={'col'} {...props} />", None, None),
        (r"<Foo scope='bar' {...props} />", None, None),
        (r"<TableHeader scope='row' />", None, Some(settings())),
    ];

    let fail =
        vec![(r"<div scope />", None, None), (r"<Foo scope='bar' />;", None, Some(settings()))];

    let fix = vec![
        (r"<div scope />", r"<div  />", None),
        (r"<h1 scope='bar' />;", r"<h1  />;", Some(settings())),
    ];

    Tester::new(Scope::NAME, Scope::PLUGIN, pass, fail)
        .expect_fix(fix)
        .with_jsx_a11y_plugin(true)
        .test_and_snapshot();
}
