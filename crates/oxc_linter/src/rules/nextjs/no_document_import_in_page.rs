use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    rule::Rule,
    utils::is_document_page,
};

fn no_document_import_in_page_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("`<Document />` from `next/document` should not be imported outside of `pages/_document.js`. See: https://nextjs.org/docs/messages/no-document-import-in-page").with_help("Prevent importing `next/document` outside of `pages/_document.js`.").with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoDocumentImportInPage;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prevent importing `next/document` outside of `pages/_document.js`.
    ///
    /// ### Why is this bad?
    ///
    /// Importing `next/document` outside of `pages/_document.js` can cause
    /// unexpected issues in your Next.js application.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// // `components/MyDocument.jsx`
    /// import Document from 'next/document'
    ///
    /// class MyDocument extends Document {
    ///   //...
    /// }
    ///
    /// export default MyDocument
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// // `pages/_document.jsx`
    /// import Document from 'next/document'
    ///
    /// class MyDocument extends Document {
    ///   //...
    /// }
    ///
    /// export default MyDocument
    /// ```
    NoDocumentImportInPage,
    nextjs,
    correctness
);

impl Rule for NoDocumentImportInPage {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ImportDeclaration(import_decl) = node.kind() else {
            return;
        };

        if import_decl.source.value.as_str() != "next/document" {
            return;
        }

        ctx.diagnostic(no_document_import_in_page_diagnostic(import_decl.span));
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        let Some(path) = ctx.file_path().to_str() else {
            return false;
        };

        !is_document_page(path)
    }
}

#[test]
fn test() {
    use std::path::PathBuf;

    use crate::tester::Tester;

    let pass = vec![
        (
            r#"import Document from "next/document"

			export default class MyDocument extends Document {
			  render() {
				return (
				  <Html>
				  </Html>
				);
			  }
			}
			"#,
            None,
            None,
            Some(PathBuf::from("pages/_document.js")),
        ),
        (
            r#"import Document from "next/document"

				export default class MyDocument extends Document {
				render() {
					return (
					<Html>
					</Html>
					);
				}
				}
			"#,
            None,
            None,
            Some(PathBuf::from("pages/_document.js")),
        ),
        (
            r#"import NextDocument from "next/document"

        	    export default class MyDocument extends NextDocument {
        	      render() {
        	        return (
        	          <Html>
        	          </Html>
        	        );
        	      }
        	    }
        	"#,
            None,
            None,
            Some(PathBuf::from("pages/_document.tsx")),
        ),
        (
            r#"import Document from "next/document"

            export default class MyDocument extends Document {
              render() {
                return (
                  <Html>
                  </Html>
                );
              }
            }
        	"#,
            None,
            None,
            Some(PathBuf::from("pages/_document.page.tsx")),
        ),
        (
            r#"import NDocument from "next/document"

            export default class Document extends NDocument {
              render() {
                return (
                  <Html>
                  </Html>
                );
              }
            }
        	"#,
            None,
            None,
            Some(PathBuf::from("pages/_document/index.js")),
        ),
        (
            r#"import NDocument from "next/document"

            export default class Document extends NDocument {
              render() {
                return (
                  <Html>
                  </Html>
                );
              }
            }
        	"#,
            None,
            None,
            Some(PathBuf::from("pages/_document/index.tsx")),
        ),
        (
            r#"import Document from "next/document"

        	    export default class MyDocument extends Document {
        	      render() {
        	        return (
        	          <Html>
        	          </Html>
        	        );
        	      }
        	    }
        	"#,
            None,
            None,
            Some(PathBuf::from("pagesapp/src/pages/_document.js")),
        ),
    ];

    let fail = vec![
        (
            r#"import Document from "next/document"

        	    export const Test = () => <p>Test</p>
			"#,
            None,
            None,
            Some(PathBuf::from("components/test.js")),
        ),
        (
            r#"import Document from "next/document"

        	    export const Test = () => <p>Test</p>
        	"#,
            None,
            None,
            Some(PathBuf::from("pages/test.js")),
        ),
        (
            r#"import Document from "next/document"

        	    export const Test = () => <p>Test</p>
			"#,
            None,
            None,
            Some(PathBuf::from("src/pages/user/test.tsx")),
        ),
        (
            r#"import Document from "next/document"

        	    export const Test = () => <p>Test</p>
			"#,
            None,
            None,
            Some(PathBuf::from("src/pages/user/_document.tsx")),
        ),
    ];

    Tester::new(NoDocumentImportInPage::NAME, NoDocumentImportInPage::PLUGIN, pass, fail)
        .test_and_snapshot();
}
