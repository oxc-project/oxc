use oxc_ast::{
    ast::{JSXAttributeItem, JSXAttributeValue},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, utils::has_jsx_prop_ignore_case, AstNode};

fn no_access_key_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("No access key attribute allowed.")
        .with_help("Remove the accessKey attribute. Inconsistencies between keyboard shortcuts and keyboard commands used by screenreaders and keyboard-only users create a11y complications.")
        .with_label(span)
}

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
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <div accessKey="h" />
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <div />
    /// ```
    NoAccessKey,
    jsx_a11y,
    correctness,
    suggestion,
);

impl Rule for NoAccessKey {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(jsx_el) = node.kind() else {
            return;
        };
        if let Some(JSXAttributeItem::Attribute(attr)) =
            has_jsx_prop_ignore_case(jsx_el, "accessKey")
        {
            match attr.value.as_ref() {
                Some(JSXAttributeValue::StringLiteral(_)) => {
                    ctx.diagnostic_with_suggestion(no_access_key_diagnostic(attr.span), |fixer| {
                        fixer.delete(&attr.span)
                    });
                }
                Some(JSXAttributeValue::ExpressionContainer(container)) => {
                    if container.expression.is_expression() && !container.expression.is_undefined()
                    {
                        ctx.diagnostic_with_suggestion(
                            no_access_key_diagnostic(attr.span),
                            |fixer| fixer.delete(&attr.span),
                        );
                    }
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

    let fix = vec![
        (r#"<div accesskey="h" />"#, r"<div  />"),
        (r#"<div accessKey="h" />"#, r"<div  />"),
        (r#"<div accessKey="h" {...props} />"#, r"<div  {...props} />"),
        (r#"<div acCesSKeY="y" />"#, r"<div  />"),
        (r#"<div accessKey={"y"} />"#, r"<div  />"),
        (r"<div accessKey={`${y}`} />", r"<div  />"),
        (r"<div accessKey={`${undefined}y${undefined}`} />", r"<div  />"),
        (r"<div accessKey={`This is ${bad}`} />", r"<div  />"),
        (r"<div accessKey={accessKey} />", r"<div  />"),
        (r"<div accessKey={`${undefined}`} />", r"<div  />"),
        (r"<div accessKey={`${undefined}${undefined}`} />", r"<div  />"),
    ];

    Tester::new(NoAccessKey::NAME, NoAccessKey::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
