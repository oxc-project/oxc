use oxc_ast::{ast::JSXAttributeItem, AstKind};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext, globals::VALID_ARIA_PROPS, rule::Rule, utils::get_attribute_name, AstNode,
};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-jsx-a11y(aria-props): Invalid ARIA prop.")]
#[diagnostic(severity(warning), help("`{1}` is an invalid ARIA attribute."))]
struct AriaPropsDiagnostic(#[label] pub Span, String);

#[derive(Debug, Default, Clone)]
pub struct AriaProps;

declare_oxc_lint!(
    /// ### What it does
    /// Enforces that elements do not use invalid ARIA attributes.
    ///
    /// ### Why is this bad?
    /// Using invalid ARIA attributes can mislead screen readers and other assistive technologies.
    /// It may cause the accessibility features of the website to fail, making it difficult
    /// for users with disabilities to use the site effectively.
    ///
    /// ### Example
    /// ```javascript
    /// // Bad
    /// <input aria-labeledby="address_label" />
    ///
    /// // Good
    /// <input aria-labelledby="address_label" />
    /// ```
    AriaProps,
    correctness
);
impl Rule for AriaProps {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::JSXAttributeItem(JSXAttributeItem::Attribute(attr)) = node.kind() {
            let name = get_attribute_name(&attr.name).to_lowercase();
            if name.starts_with("aria-") && !VALID_ARIA_PROPS.contains(&name) {
                ctx.diagnostic(AriaPropsDiagnostic(attr.span, name));
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"<div />",
        r"<div></div>",
        r#"<div aria="wee"></div>"#,
        r#"<div abcARIAdef="true"></div>"#,
        r#"<div fooaria-foobar="true"></div>"#,
        r#"<div fooaria-hidden="true"></div>"#,
        r"<Bar baz />",
        r#"<input type="text" aria-errormessage="foobar" />"#,
    ];

    let fail = vec![
        r#"<div aria-="foobar" />"#,
        r#"<div aria-labeledby="foobar" />"#,
        r#"<div aria-skldjfaria-klajsd="foobar" />"#,
    ];

    Tester::new_without_config(AriaProps::NAME, pass, fail).test_and_snapshot();
}
