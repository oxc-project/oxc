use oxc_ast::{ast::JSXElementName, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, utils::is_in_app_dir, AstNode};

fn no_head_element_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not use `<head>` element. Use `<Head />` from `next/head` instead.")
        .with_help("See https://nextjs.org/docs/messages/no-head-element")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoHeadElement;

declare_oxc_lint!(
    /// ### What it does
    /// Prevent usage of `<head>` element.
    ///
    /// ### Why is this bad?
    ///
    ///
    /// ### Example
    /// ```javascript
    /// ```
    NoHeadElement,
    nextjs,
    correctness
);

impl Rule for NoHeadElement {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::JSXOpeningElement(elem) = node.kind() {
            let JSXElementName::Identifier(id) = &elem.name else {
                return;
            };
            if id.name != "head" {
                return;
            }
            let Some(full_file_path) = ctx.file_path().to_str() else {
                return;
            };
            if is_in_app_dir(full_file_path) {
                return;
            }
            ctx.diagnostic(no_head_element_diagnostic(elem.span));
        }
    }
}

#[test]
fn test() {
    use std::path::PathBuf;

    use crate::tester::Tester;

    let pass = vec![
        (
            r"import Head from 'next/head';

			export class MyComponent {
			  render() {
				return (
				  <div>
					<Head>
					  <title>My page title</title>
					</Head>
				  </div>
				);
			  }
			}
		",
            None,
            None,
            Some(PathBuf::from("pages/index.js")),
        ),
        (
            r"import Head from 'next/head';

           	      export class MyComponent {
           	        render() {
           	          return (
           	            <div>
           	              <Head>
           	                <title>My page title</title>
           	              </Head>
           	            </div>
           	          );
           	        }
           	      }
        ",
            None,
            None,
            Some(PathBuf::from("pages/index.tsx")),
        ),
        (
            r"
        	      export default function Layout({ children }) {
        	        return (
        	          <html>
        	            <head>
        	              <title>layout</title>
        	            </head>
        	            <body>{children}</body>
        	          </html>
        	        );
        	      }
        	",
            None,
            None,
            Some(PathBuf::from("./app/layout.js")),
        ),
        (
            r"
        	      export default function Layout({ children }) {
        	        return (
        	          <html>
        	            <head>
        	              <title>layout</title>
        	            </head>
        	            <body>{children}</body>
        	          </html>
        	        );
        	      }
        	",
            None,
            None,
            Some(PathBuf::from("./app/layout.js")),
        ),
    ];

    let fail = vec![
        (
            r"
        	      export class MyComponent {
        	        render() {
        	          return (
        	            <div>
        	              <head>
        	                <title>My page title</title>
        	              </head>
        	            </div>
        	          );
        	        }
        	}",
            None,
            None,
            Some(PathBuf::from("./pages/index.js")),
        ),
        (
            r"import Head from 'next/head';

        	      export class MyComponent {
        	        render() {
        	          return (
        	            <div>
        	              <head>
        	                <title>My page title</title>
        	              </head>
        	              <Head>
        	                <title>My page title</title>
        	              </Head>
        	            </div>
        	          );
        	        }
        	}",
            None,
            None,
            Some(PathBuf::from("pages/index.tsx")),
        ),
    ];

    Tester::new(NoHeadElement::NAME, NoHeadElement::PLUGIN, pass, fail).test_and_snapshot();
}
