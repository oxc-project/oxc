use oxc_ast::{ast::JSXAttributeItem, AstKind};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{has_jsx_prop_lowercase, parse_jsx_value},
    AstNode,
};

#[derive(Debug, Error, Diagnostic)]
#[error(
    "eslint-plugin-jsx-a11y(tab-index-no-positive): Avoid positive integer values for tabIndex."
)]
#[diagnostic(severity(warning), help("Change the tabIndex prop to a non-negative value"))]
struct TabIndexNoPositiveDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct TabIndexNoPositive;

declare_oxc_lint!(
    /// ### What it does
    /// Enforces that positive values for the tabIndex attribute are not used in JSX.
    ///
    /// ### Why is this bad?
    /// Using tabIndex values greater than 0 can make navigation and interaction difficult for keyboard and assistive technology users, disrupting the logical order of content.
    ///
    /// ### Example
    /// ```javascript
    /// // Bad
    /// <span tabIndex="1">foo</span>
    ///
    /// // Good
    /// <span tabIndex="0">foo</span>
    /// <span tabIndex="-1">bar</span>
    /// ```
    TabIndexNoPositive,
    correctness
);

impl Rule for TabIndexNoPositive {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(jsx_el) = node.kind() else { return };
        if let Some(tab_index_prop) = has_jsx_prop_lowercase(jsx_el, "tabIndex") {
            check_and_diagnose(tab_index_prop, ctx);
        }
    }
}

fn check_and_diagnose(attr: &JSXAttributeItem, ctx: &LintContext<'_>) {
    match attr {
        JSXAttributeItem::Attribute(attr) => attr.value.as_ref().map_or((), |value| {
            if let Ok(parsed_value) = parse_jsx_value(value) {
                if parsed_value > 0.0 {
                    ctx.diagnostic(TabIndexNoPositiveDiagnostic(attr.span));
                }
            }
        }),
        JSXAttributeItem::SpreadAttribute(_) => {}
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (r"<div />;", None),
        (r"<div {...props} />", None),
        (r#"<div id="main" />"#, None),
        (r"<div tabIndex={undefined} />", None),
        (r"<div tabIndex={`${undefined}`} />", None),
        (r"<div tabIndex={`${undefined}${undefined}`} />", None),
        (r"<div tabIndex={0} />", None),
        (r"<div tabIndex={-1} />", None),
        (r"<div tabIndex={null} />", None),
        (r"<div tabIndex={bar()} />", None),
        (r"<div tabIndex={bar} />", None),
        (r#"<div tabIndex={"foobar"} />"#, None),
        (r#"<div tabIndex="0" />"#, None),
        (r#"<div tabIndex="-1" />"#, None),
        (r#"<div tabIndex="-5" />"#, None),
        (r#"<div tabIndex="-5.5" />"#, None),
        (r"<div tabIndex={-5.5} />", None),
        (r"<div tabIndex={-5} />", None),
    ];

    let fail = vec![
        (r#"<div tabIndex="1" />"#, None),
        (r"<div tabIndex={1} />", None),
        (r#"<div tabIndex={"1"} />"#, None),
        (r"<div tabIndex={`1`} />", None),
        (r"<div tabIndex={1.589} />", None),
    ];

    Tester::new(TabIndexNoPositive::NAME, pass, fail).test_and_snapshot();
}
