use oxc_ast::AstKind;
use oxc_ast::ast::{JSXAttributeItem, JSXAttributeValue};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use rustc_hash::FxHashSet;

use crate::{context::LintContext, rule::Rule, utils::has_jsx_prop_ignore_case};

fn use_unique_element_ids_diagnostic(span: Span, id: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Duplicate element id `{id}`."))
        .with_help("Element `id` attributes must be unique within a document. Use a unique id for each element.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct UseUniqueElementIds;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces that element `id` attributes are unique within a file.
    ///
    /// ### Why is this bad?
    ///
    /// Duplicate `id` attributes break `getElementById`, cause issues with
    /// label associations, ARIA references, and fragment navigation.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <div id="foo" />
    /// <div id="foo" />
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <div id="foo" />
    /// <div id="bar" />
    /// ```
    UseUniqueElementIds,
    jsx_a11y,
    correctness,
    pending
);

impl Rule for UseUniqueElementIds {
    fn run_once(&self, ctx: &LintContext<'_>) {
        let mut seen_ids: FxHashSet<String> = FxHashSet::default();

        for node in ctx.nodes().iter() {
            let AstKind::JSXOpeningElement(jsx_el) = node.kind() else {
                continue;
            };

            let Some(JSXAttributeItem::Attribute(attr)) = has_jsx_prop_ignore_case(jsx_el, "id")
            else {
                continue;
            };

            let Some(JSXAttributeValue::StringLiteral(id_value)) = &attr.value else {
                continue;
            };

            let id = id_value.value.as_str();
            if id.is_empty() {
                continue;
            }

            if !seen_ids.insert(id.to_string()) {
                ctx.diagnostic(use_unique_element_ids_diagnostic(attr.span, id));
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass =
        vec![r#"<><div id="foo" /><div id="bar" /></>"#, "<div />", r#"<div id="unique" />"#];

    let fail = vec![r#"<><div id="foo" /><div id="foo" /></>"#];

    Tester::new(UseUniqueElementIds::NAME, UseUniqueElementIds::PLUGIN, pass, fail)
        .test_and_snapshot();
}
