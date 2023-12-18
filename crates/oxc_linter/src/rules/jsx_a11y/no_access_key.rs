use oxc_ast::{
    ast::{JSXAttributeItem, JSXAttributeValue, JSXExpression, JSXExpressionContainer},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, utils::has_jsx_prop_lowercase, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-jsx-a11y(no-access-key): No access key attribute allowed.")]
#[diagnostic(severity(warning), help("Remove the accessKey attribute. Inconsistencies between keyboard shortcuts and keyboard commands used by screenreaders and keyboard-only users create a11y complications."))]
struct NoAccessKeyDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoAccessKey;

declare_oxc_lint!(
    /// ### What it does
    /// Enforces that the `accessKey` prop is not used on any element to avoid complications with keyboard commands used by a screenreader.
    ///
    /// ### Why is this bad?
    /// Access keys are HTML attributes that allow web developers to assign keyboard shortcuts to elements.
    /// Inconsistencies between keyboard shortcuts and keyboard commands used by screenreaders and keyboard-only users create accessibility complications so to avoid complications, access keys should not be used.
    ///
    /// ### Example
    /// ```javascript
    /// // Bad
    /// <div accessKey="h" />
    ///
    /// // Good
    /// <div />
    /// ```
    NoAccessKey,
    correctness
);

impl Rule for NoAccessKey {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(jsx_el) = node.kind() else { return };
        if let Some(JSXAttributeItem::Attribute(attr)) = has_jsx_prop_lowercase(jsx_el, "accessKey")
        {
            match attr.value.as_ref() {
                Some(JSXAttributeValue::StringLiteral(_)) => {
                    ctx.diagnostic(NoAccessKeyDiagnostic(attr.span));
                }
                Some(JSXAttributeValue::ExpressionContainer(JSXExpressionContainer {
                    expression: JSXExpression::Expression(expr),
                    ..
                })) => {
                    if expr.is_identifier_reference() & expr.is_undefined() {
                        return;
                    }
                    ctx.diagnostic(NoAccessKeyDiagnostic(attr.span));
                }
                _ => {}
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![r"<div />;", r"<div {...props} />", r"<div accessKey={undefined} />"];

    let fail = vec![
        r#"<div accesskey="h" />"#,
        r#"<div accessKey="h" />"#,
        r#"<div accessKey="h" {...props} />"#,
        r#"<div acCesSKeY="y" />"#,
        r#"<div accessKey={"y"} />"#,
        r"<div accessKey={`${y}`} />",
        r"<div accessKey={`${undefined}y${undefined}`} />",
        r"<div accessKey={`This is ${bad}`} />",
        r"<div accessKey={accessKey} />",
        r"<div accessKey={`${undefined}`} />",
        r"<div accessKey={`${undefined}${undefined}`} />",
    ];

    Tester::new_without_config(NoAccessKey::NAME, pass, fail).test_and_snapshot();
}
