use std::collections::HashMap;

use oxc_ast::{ast::JSXAttributeItem, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{Atom, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

fn jsx_props_no_spread_multi_diagnostic(spans: Vec<Span>) -> OxcDiagnostic {
    OxcDiagnostic::warn("Disallow JSX prop spreading the same identifier multiple times.")
        .with_help("Remove duplicate spread attributes.")
        .with_labels(spans)
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

            let mut identifier_names: HashMap<&Atom, Span> = HashMap::new();
            let mut duplicate_spreads: HashMap<&Atom, Vec<Span>> = HashMap::new();

            for spread_attr in spread_attrs {
                if let Some(identifier_name) =
                    spread_attr.argument.get_identifier_reference().map(|arg| &arg.name)
                {
                    identifier_names
                        .entry(identifier_name)
                        .and_modify(|first_span| {
                            duplicate_spreads
                                .entry(identifier_name)
                                .or_insert_with(|| vec![*first_span])
                                .push(spread_attr.span);
                        })
                        .or_insert(spread_attr.span);
                }
            }

            for spans in duplicate_spreads.values() {
                ctx.diagnostic(jsx_props_no_spread_multi_diagnostic(spans.clone()));
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
