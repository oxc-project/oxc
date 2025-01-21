use itertools::Itertools;
use oxc_ast::{ast::JSXAttributeItem, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{Atom, GetSpan, Span};
use rustc_hash::FxHashMap;

use crate::{
    context::{ContextHost, LintContext},
    fixer::{Fix, RuleFix},
    rule::Rule,
    utils::is_same_member_expression,
    AstNode,
};

fn jsx_props_no_spread_multiple_identifiers_diagnostic(
    spans: Vec<Span>,
    prop_name: &str,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Prop '{prop_name}' is spread multiple times."))
        .with_help("Remove all but one spread.")
        .with_labels(spans)
}

fn jsx_props_no_spread_multiple_member_expressions_diagnostic(
    spans: Vec<Span>,
    member_name: &str,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("'{member_name}' is spread multiple times."))
        .with_help("Remove all but one spread.")
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
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <App {...props} myAttr="1" {...props} />
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <App myAttr="1" {...props} />
    /// <App {...props} myAttr="1" />
    /// ```
    JsxPropsNoSpreadMulti,
    react,
    correctness,
    fix
);

impl Rule for JsxPropsNoSpreadMulti {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::JSXOpeningElement(jsx_opening_el) = node.kind() {
            let spread_attrs =
                jsx_opening_el.attributes.iter().filter_map(JSXAttributeItem::as_spread);

            let mut identifier_names: FxHashMap<Atom, Span> = FxHashMap::default();
            let mut member_expressions = Vec::new();
            let mut duplicate_spreads: FxHashMap<Atom, Vec<Span>> = FxHashMap::default();

            for spread_attr in spread_attrs {
                let argument_without_parenthesized = spread_attr.argument.without_parentheses();

                if let Some(identifier_name) =
                    argument_without_parenthesized.get_identifier_reference().map(|arg| arg.name)
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
                ctx.diagnostic_with_fix(
                    jsx_props_no_spread_multiple_identifiers_diagnostic(
                        spans.clone(),
                        &identifier_name,
                    ),
                    |_fixer| {
                        spans
                            .iter()
                            .rev()
                            .skip(1)
                            .map(|span| Fix::delete(*span))
                            .collect::<RuleFix<'a>>()
                    },
                );
            }

            member_expressions.iter().tuple_combinations().for_each(
                |((left, left_span), (right, right_span))| {
                    if is_same_member_expression(left, right, ctx) {
                        // 'foo.bar'
                        let member_prop_name = ctx.source_range(left.span());
                        ctx.diagnostic_with_fix(
                            jsx_props_no_spread_multiple_member_expressions_diagnostic(
                                vec![*left_span, *right_span],
                                member_prop_name,
                            ),
                            |fixer| fixer.delete_range(*left_span),
                        );
                    }
                },
            );
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_jsx()
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
    let fix = vec![
        ("<App {...props} {...props} />", "<App  {...props} />"),
        ("<App {...props.foo} {...props.foo} />", "<App  {...props.foo} />"),
        ("<App {...(props.foo.baz)} {...(props.foo.baz)} />", "<App  {...(props.foo.baz)} />"),
        (r#"<div {...props} a="a" {...props} />"#, r#"<div  a="a" {...props} />"#),
        ("<div {...props} {...props} {...props} />", "<div   {...props} />"),
    ];

    Tester::new(JsxPropsNoSpreadMulti::NAME, JsxPropsNoSpreadMulti::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
