use oxc_ast::{AstKind, ast::{JSXElementName, JSXAttributeItem, JSXAttributeValue, JSXIdentifier}};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{Error},
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode, utils::has_jsx_prop_lowercase};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-jsx-a11y(anchor-has-content):")]
#[diagnostic(severity(warning), help(""))]
struct AnchorHasContentDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct AnchorHasContent;


declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce that anchors have content and that the content is accessible to screen readers. 
    /// Accessible means that it is not hidden using the `aria-hidden` prop. 
    /// 
    /// Alternatively, you may use the `title` prop or the `aria-label` prop.
    ///
    /// ### Why is this bad?
    ///
    ///
    /// ### Example
    /// 
    /// #### good
    /// 
    /// ```
    /// <a>Anchor Content!</a>
    ///  <a><TextWrapper /></a>
    ///  <a dangerouslySetInnerHTML={{ __html: 'foo' }} />
    ///  <a title='foo' />
    ///  <a aria-label='foo' />
    /// ```
    /// 
    /// #### bad
    /// 
    /// ```
    /// <a />
    /// <a><TextWrapper aria-hidden /></a>
    /// ```
    /// 
    AnchorHasContent,
    correctness
);

fn get_prop_value<'a, 'b>(item: &'b JSXAttributeItem<'a>) -> Option<&'b JSXAttributeValue<'a>> {
    if let JSXAttributeItem::Attribute(attr) = item {
        attr.0.value.as_ref()
    } else {
        None
    }
}

fn match_valid_prop(attr_name: &JSXAttributeItem<'_>) -> bool {
    match get_prop_value(attr_name) {
        // Some(JSXAttributeValue::JSXIdentifier(exp)) => {
        //     dbg!(exp);
        //     return false
        // },
        _ => false,

    }
}

impl Rule for AnchorHasContent {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(jsx_el) = node.kind() else { return };
        let JSXElementName::Identifier(iden) = &jsx_el.name else { return };
        let name = iden.name.as_str();
        dbg!(name);


        if name == "a" {
            let mut has_content = false;
            let mut has_title = false;
            let mut has_aria_label = false;
            let mut has_aria_hidden = false;

            // check attr
            for attr in &jsx_el.attributes {
                let JSXAttributeItem::Attribute(jsx_attr) = attr else { continue };  
                dbg!(jsx_attr);
                // check is title or aria-label
                // let name = jsx_attr.name;
                if let Some(attr_name) = has_jsx_prop_lowercase(jsx_el, "title") {
                    if match_valid_prop(attr_name) {
                        // pass
                        // return
                    }
                }
                // let JSXAttributeName::Identifier(iden) = &jsx_attr.name else { continue };
                // let name = iden.name.as_str();
                // dbg!(name);
            }

            // check content
        }

        // custom component
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (r#"<div />;"#, None),
        // (r#"<a>Foo</a>"#, None),
        // (r#"<a><Bar /></a>"#, None),
        // (r#"<a>{foo}</a>"#, None),
        // (r#"<a>{foo.bar}</a>"#, None),
        // (r#"<a dangerouslySetInnerHTML={{ __html: "foo" }} />"#, None),
        // (r#"<a children={children} />"#, None),
        // // (r#"<Link />"#, None),
        (r#"<a title={title} />"#, None),
        // (r#"<a aria-label={ariaLabel} />"#, None),
        // (r#"<a title={title} aria-label={ariaLabel} />"#, None),
    ];

    let fail = vec![
        // (r#"<a />"#, None),
        // (r#"<a><Bar aria-hidden /></a>"#, None),
        // (r#"<a>{undefined}</a>"#, None),
        // (r#"<Link />"#, None),
    ];

    Tester::new(AnchorHasContent::NAME, pass, fail).test_and_snapshot();
}
