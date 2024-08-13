use std::collections::HashSet;

use oxc_ast::{ast::JSXAttributeItem, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn jsx_props_no_spread_multi_diagnostic(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Disallow JSX prop spreading the same identifier multiple times.")
        .with_help("Remove duplicate spread attributes.")
        .with_label(span0)
}

#[derive(Debug, Default, Clone)]
pub struct JsxPropsNoSpreadMulti;

declare_oxc_lint!(
    /// ### What it does
    /// Enforces that any unique expression is only spread once.
    /// Generally spreading the same expression twice is an indicator of a mistake since any attribute between the spreads may be overridden when the intent was not to.
    /// Even when that is not the case this will lead to unnecessary computations being performed.
    ///
    /// ### Example
    /// ```jsx
    /// // Bad
    /// <App {...props} myAttr="1" {...props} />
    ///
    /// // Good
    /// <App myAttr="1" {...props} />
    /// <App {...props} myAttr="1" />
    /// ```
    JsxPropsNoSpreadMulti,
    correctness,
);

impl Rule for JsxPropsNoSpreadMulti {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::JSXOpeningElement(jsx_opening_el) = node.kind() {
            let spread_attrs = jsx_opening_el.attributes.iter().filter_map(|attr| {
                if let JSXAttributeItem::SpreadAttribute(spread_attr) = attr {
                    if spread_attr.argument.is_identifier_reference() {
                        return Some(spread_attr);
                    }
                }
                None
            });

            let mut identifier_names = HashSet::new();

            for spread_attr in spread_attrs {
                let identifier_name =
                    &spread_attr.argument.get_identifier_reference().unwrap().name;
                if !identifier_names.insert(identifier_name) {
                    ctx.diagnostic(jsx_props_no_spread_multi_diagnostic(spread_attr.span));
                }
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "
          const a = {};
          <App {...a} />
        ",
        "
          const a = {};
          const b = {};
          <App {...a} {...b} />
        ",
    ];

    let fail = vec![
        "
          const props = {};
          <App {...props} {...props} />
        ",
        r#"
          const props = {};
          <div {...props} a="a" {...props} />
        "#,
        "
          const props = {};
          <div {...props} {...props} {...props} />
        ",
    ];

    Tester::new(JsxPropsNoSpreadMulti::NAME, pass, fail).test_and_snapshot();
}
