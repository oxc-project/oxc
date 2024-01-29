use oxc_ast::{ast::ModuleDeclaration, AstKind};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, utils::is_document_page, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-next(no-document-import-in-page): `<Document />` from `next/document` should not be imported outside of `pages/_document.js`. See: https://nextjs.org/docs/messages/no-document-import-in-page")]
#[diagnostic(
    severity(warning),
    help("Prevent importing `next/document` outside of `pages/_document.js`.")
)]
struct NoDocumentImportInPageDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoDocumentImportInPage;

declare_oxc_lint!(
    /// ### What it does
    /// Prevent importing `next/document` outside of `pages/_document.js`.
    ///
    /// ### Why is this bad?
    ///
    ///
    /// ### Example
    /// ```javascript
    /// ```
    NoDocumentImportInPage,
    correctness
);

impl Rule for NoDocumentImportInPage {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ModuleDeclaration(ModuleDeclaration::ImportDeclaration(import_decl)) =
            node.kind()
        else {
            return;
        };

        if import_decl.source.value.as_str() != "next/document" {
            return;
        }

        let Some(path) = ctx.file_path().to_str() else { return };

        if is_document_page(path) {
            return;
        }

        ctx.diagnostic(NoDocumentImportInPageDiagnostic(import_decl.span));
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    use std::path::PathBuf;

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

    Tester::new(NoDocumentImportInPage::NAME, pass, fail).test_and_snapshot();
}
