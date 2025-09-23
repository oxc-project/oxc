use oxc_ast::{
    AstKind,
    ast::{JSXAttributeItem, JSXElementName},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_img_element_diagnostic(span: Span, src_span: Option<Span>) -> OxcDiagnostic {
    let mut diagnostic = OxcDiagnostic::warn("Using `<img>` could result in slower LCP and higher bandwidth.")
		.with_help("Consider using `<Image />` from `next/image` or a custom image loader to automatically optimize images.\nSee https://nextjs.org/docs/messages/no-img-element")
        .with_label(span.label("Use `<Image />` from `next/image` instead."));
    if let Some(src_span) = src_span {
        diagnostic = diagnostic.and_label(src_span.label("Use a static image import instead."));
    }
    diagnostic
}

#[derive(Debug, Default, Clone)]
pub struct NoImgElement;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prevent the usage of `<img>` element due to slower
    /// [LCP](https://nextjs.org/learn/seo/lcp) and higher bandwidth.
    ///
    /// ### Why is this bad?
    ///
    /// `<img>` elements are not optimized for performance and can result in
    /// slower LCP and higher bandwidth.  Using [`<Image />`](https://nextjs.org/docs/pages/api-reference/components/image)
    /// from `next/image` will automatically optimize images and serve them as
    /// static assets.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// export function MyComponent() {
    /// 	return (
    /// 		<div>
    /// 			<img src="/test.png" alt="Test picture" />
    /// 		</div>
    /// 	);
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// import Image from "next/image";
    /// import testImage from "./test.png"
    /// export function MyComponent() {
    /// 	return (
    /// 		<div>
    ///             <Image src={testImage} alt="Test picture" />
    ///         </div>
    ///     );
    /// }
    /// ```
    NoImgElement,
    nextjs,
    correctness,
    pending // TODO: add `import Image from "next/image"` (if missing), then change `<img />` to `<Image />`
);

impl Rule for NoImgElement {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(jsx_opening_element) = node.kind() else {
            return;
        };

        let JSXElementName::Identifier(jsx_opening_element_name) = &jsx_opening_element.name else {
            return;
        };

        if jsx_opening_element_name.name != "img" {
            return;
        }

        let Some(grandparent) = ctx.nodes().ancestor_kinds(node.id()).nth(1) else {
            return;
        };

        if let AstKind::JSXElement(maybe_picture_jsx_elem) = grandparent
            && let JSXElementName::Identifier(jsx_opening_element_name) =
                &maybe_picture_jsx_elem.opening_element.name
            && jsx_opening_element_name.name.as_str() == "picture"
        {
            return;
        }

        let src_span: Option<Span> = jsx_opening_element
            .attributes
            .iter()
            .filter_map(JSXAttributeItem::as_attribute)
            .find_map(|attr| {
                let ident = attr.name.as_identifier()?;
                let value = attr.value.as_ref()?;
                let lit = value.as_string_literal()?;
                (ident.name == "src").then(|| lit.span())
            });

        ctx.diagnostic(no_img_element_diagnostic(jsx_opening_element_name.span, src_span));
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
        // src is not a string literal, so diagnostic won't label it.
        "
		import somePicture from './foo.png';
		export const MyComponent = () => (
		   <img src={somePicture.src} alt='foo' />
		);
		",
    ];

    Tester::new(NoImgElement::NAME, NoImgElement::PLUGIN, pass, fail).test_and_snapshot();
}
