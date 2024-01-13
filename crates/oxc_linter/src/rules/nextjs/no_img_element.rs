use oxc_ast::{ast::JSXElementName, AstKind};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-next(no-img-element): Prevent usage of `<img>` element due to slower LCP and higher bandwidth.")]
#[diagnostic(severity(warning), help("See https://nextjs.org/docs/messages/no-img-element"))]
struct NoImgElementDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoImgElement;

declare_oxc_lint!(
    /// ### What it does
    ///
    ///
    /// ### Why is this bad?
    ///
    ///
    /// ### Example
    /// ```javascript
    /// ```
    NoImgElement,
    correctness
);

impl Rule for NoImgElement {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(jsx_opening_element) = node.kind() else { return };

        let JSXElementName::Identifier(jsx_opening_element_name) = &jsx_opening_element.name else {
            return;
        };

        if jsx_opening_element_name.name.as_str() != "img" {
            return;
        }

        let Some(parent) = ctx.nodes().parent_node(node.id()) else { return };
        let Some(parent) = ctx.nodes().parent_node(parent.id()) else { return };

        if let AstKind::JSXElement(maybe_picture_jsx_elem) = parent.kind() {
            if let JSXElementName::Identifier(jsx_opening_element_name) =
                &maybe_picture_jsx_elem.opening_element.name
            {
                if jsx_opening_element_name.name.as_str() == "picture" {
                    return;
                }
            }
        }

        ctx.diagnostic(NoImgElementDiagnostic(jsx_opening_element_name.span));
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"import { Image } from 'next/image';
			
			      export class MyComponent {
			        render() {
			          return (
			            <div>
			              <Image
			                src="/test.png"
			                alt="Test picture"
			                width={500}
			                height={500}
			              />
			            </div>
			          );
			        }
			      }"#,
        r#"export class MyComponent {
			        render() {
			          return (
			            <picture>
			              <img
			                src="/test.png"
			                alt="Test picture"
			                width={500}
			                height={500}
			              />
			            </picture>
			          );
			        }
			      }"#,
        r#"export class MyComponent {
			        render() {
			          return (
			            <div>
			              <picture>
			                <source media="(min-width:650px)" srcset="/test.jpg"/>
			                <img
			                  src="/test.png"
			                  alt="Test picture"
			                  style="width:auto;"
			                />
			              </picture>
			            </div>
			          );
			        }
			      }"#,
    ];

    let fail = vec![
        r#"
			      export class MyComponent {
			        render() {
			          return (
			            <div>
			              <img
			                src="/test.png"
			                alt="Test picture"
			                width={500}
			                height={500}
			              />
			            </div>
			          );
			        }
			      }"#,
        r#"
			      export class MyComponent {
			        render() {
			          return (
			            <img
			              src="/test.png"
			              alt="Test picture"
			              width={500}
			              height={500}
			            />
			          );
			        }
			      }"#,
    ];

    Tester::new(NoImgElement::NAME, pass, fail).test_and_snapshot();
}
