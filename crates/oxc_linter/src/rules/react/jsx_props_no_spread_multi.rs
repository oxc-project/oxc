use rustc_hash::FxHashMap;

use oxc_ast::{ast::JSXAttributeItem, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{Atom, Span};

use itertools::Itertools;

use crate::{context::LintContext, rule::Rule, utils::is_same_member_expression, AstNode};

fn jsx_props_no_spread_multiple_identifiers_diagnostic(
    spans: Vec<Span>,
    prop_name: &str,
) -> OxcDiagnostic {
    OxcDiagnostic::warn("Disallow JSX prop spreading the same identifier multiple times.")
        .with_help(format!("Prop '{prop_name}' is spread multiple times."))
        .with_labels(spans)
}

fn jsx_props_no_spread_multiple_member_expressions_diagnostic(spans: Vec<Span>) -> OxcDiagnostic {
    OxcDiagnostic::warn("Disallow JSX prop spreading the same member expression multiple times.")
        .with_labels(spans)
}

#[derive(Debug, Default, Clone)]
pub struct JsxPropsNoSpreadMulti;

declare_oxc_lint!(
    /// ### What it does
    /// Enforces that any unique expression is only spread once.
    ///
    /// ### Why is this bad?
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
    pending // TODO: add auto-fix to remove the first spread. Removing the second one would change program behavior.
);

impl Rule for JsxPropsNoSpreadMulti {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::JSXOpeningElement(jsx_opening_el) = node.kind() {
            let spread_attrs =
                jsx_opening_el.attributes.iter().filter_map(JSXAttributeItem::as_spread);

            let mut identifier_names: FxHashMap<&Atom, Span> = FxHashMap::default();
            let mut member_expressions = Vec::new();
            let mut duplicate_spreads: FxHashMap<&Atom, Vec<Span>> = FxHashMap::default();

            for spread_attr in spread_attrs {
                let argument_without_parenthesized = spread_attr.argument.without_parenthesized();

                if let Some(identifier_name) =
                    argument_without_parenthesized.get_identifier_reference().map(|arg| &arg.name)
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
                if let Some(member_expression) =
                    argument_without_parenthesized.as_member_expression()
                {
                    member_expressions.push((member_expression, spread_attr.span));
                }
            }

            for (identifier_name, spans) in duplicate_spreads {
                ctx.diagnostic(jsx_props_no_spread_multiple_identifiers_diagnostic(
                    spans,
                    identifier_name,
                ));
            }

            member_expressions.iter().tuple_combinations().for_each(
                |((left, left_span), (right, right_span))| {
                    if is_same_member_expression(left, right, ctx) {
                        ctx.diagnostic(jsx_props_no_spread_multiple_member_expressions_diagnostic(
                            vec![*left_span, *right_span],
                        ));
                    }
                },
            );
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
        "
        const props = {};
        <App {...props.x} {...props.foo} />
      ",
        "
        const props = {};
        <App {...(props.foo).baz} {...(props.y.baz)} />
      ",
    ];

    let fail = vec![
        "
          const props = {};
          <App {...props} {...props} />
        ",
        "
          const props = {};
          <App {...props.foo} {...props.foo} />
        ",
        "
          const props = {};
          <App {...(props.foo).baz} {...(props.foo.baz)} />
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
