use oxc_ast::{AstKind, ast::JSXAttributeItem};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    context::LintContext,
    globals::HTML_TAG,
    rule::Rule,
    utils::{get_element_type, has_jsx_prop_ignore_case},
};

fn scope_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("The `scope` prop can only be used on `<th>` elements")
        .with_help("Remove the `scope` prop on elements other than `<th>`.")
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
    ///
    /// The scope attribute makes table navigation much easier for screen reader users, provided that it is used correctly.
    /// Incorrectly used, scope can make table navigation much harder and less efficient.
    /// A screen reader operates under the assumption that a table has a header and that this header specifies a scope. Because of the way screen readers function, having an accurate header makes viewing a table far more accessible and more efficient for people who use the device.
    ///
    /// ### Examples
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
    recommended,
    fix = conditional_fix_suggestion,
    version = "0.0.19",
    short_description = "The scope prop should be used only on `<th>` elements.",
);

impl Rule for Scope {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(jsx_el) = node.kind() else {
            return;
        };

        let Some(JSXAttributeItem::Attribute(scope_attribute)) =
            has_jsx_prop_ignore_case(jsx_el, "scope")
        else {
            return;
        };

        let element_type = get_element_type(ctx, jsx_el);

        if element_type == "th" {
            return;
        }

        if !HTML_TAG.contains(element_type.as_ref()) {
            return;
        }

        let is_safe_to_delete = scope_attribute.value.as_ref().is_none_or(is_safe_to_remove_value);
        let diagnostic = scope_diagnostic(scope_attribute.span);

        if is_safe_to_delete {
            ctx.diagnostic_with_fix(diagnostic, |fixer| fixer.delete_range(scope_attribute.span));
        } else {
            ctx.diagnostic_with_suggestion(diagnostic, |fixer| {
                fixer.delete_range(scope_attribute.span)
            });
        }
    }
}

/// Returns `true` when removing the attribute will not discard any
/// potentially side-effecting expression.
fn is_safe_to_remove_value(value: &oxc_ast::ast::JSXAttributeValue) -> bool {
    match value {
        oxc_ast::ast::JSXAttributeValue::StringLiteral(_) => true,
        oxc_ast::ast::JSXAttributeValue::ExpressionContainer(container) => {
            container.expression.as_expression().is_none_or(oxc_ast::ast::Expression::is_literal)
        }
        oxc_ast::ast::JSXAttributeValue::Element(_)
        | oxc_ast::ast::JSXAttributeValue::Fragment(_) => false,
    }
}

#[test]
fn test() {
    use crate::{
        fixer::FixKind,
        tester::{ExpectFixTestCase, Tester},
    };

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

    let fail = vec![
        (r"<div scope />", None, None),
        (r"<Foo scope='bar' />;", None, Some(settings())),
        (r"<div scope={track()} />", None, None),
    ];

    let fix: Vec<ExpectFixTestCase> = vec![
        (r"<div scope />", r"<div  />", None).into(),
        (r"<Foo scope='bar' />;", r"<Foo  />;", None, Some(settings())).into(),
        (r"<h1 scope='bar' />;", r"<h1  />;", None).into(),
        (r"<div scope={'col'} />", r"<div  />", None).into(),
        (r"<div scope={track()} />", r"<div  />", None, FixKind::Suggestion).into(),
    ];

    Tester::new(Scope::NAME, Scope::PLUGIN, pass, fail).expect_fix(fix).test_and_snapshot();
}
