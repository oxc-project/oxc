use oxc_ast::{ast::ModuleDeclaration, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_head_import_in_document_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prevent usage of `next/head` in `pages/_document.js`.")
        .with_help("See https://nextjs.org/docs/messages/no-head-import-in-document")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoHeadImportInDocument;

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
    NoHeadImportInDocument,
    nextjs,
    correctness
);

impl Rule for NoHeadImportInDocument {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ModuleDeclaration(ModuleDeclaration::ImportDeclaration(import_decl)) =
            node.kind()
        else {
            return;
        };

        if import_decl.source.value.as_str() != "next/head" {
            return;
        }

        let full_file_path = ctx.file_path();

        if let Some(file_name) = full_file_path.file_name() {
            if let Some(file_name) = file_name.to_str() {
                // check `_document.*` case

                if file_name.starts_with("_document.") {
                    ctx.diagnostic(no_head_import_in_document_diagnostic(import_decl.span));
                // check `_document/index.*` case
                } else if file_name.starts_with("index") {
                    if let Some(p) = full_file_path.parent() {
                        if let Some(file_name) = p.file_name() {
                            if let Some(file_name) = file_name.to_str() {
                                if file_name.starts_with("_document") {
                                    ctx.diagnostic(no_head_import_in_document_diagnostic(
                                        import_decl.span,
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn test() {
    use std::path::PathBuf;

    use crate::tester::Tester;

    let pass = vec![
        (
            r"import Document, { Html, Head, Main, NextScript } from 'next/document'

			      class MyDocument extends Document {
			        static async getInitialProps(ctx) {
			          //...
			        }

			        render() {
			          return (
			            <Html>
			              <Head>
			              </Head>
			            </Html>
			          )
			        }
			      }

			      export default MyDocument
			    ",
            None,
            None,
            Some(PathBuf::from("pages/_document.tsx")),
        ),
        (
            r#"import Head from "next/head";

			      export default function IndexPage() {
			        return (
			          <Head>
			            <title>My page title</title>
			            <meta name="viewport" content="initial-scale=1.0, width=device-width" />
			          </Head>
			        );
			      }
			    "#,
            None,
            None,
            Some(PathBuf::from("pages/index.tsx")),
        ),
    ];

    let fail = vec![
        (
            r"
			      import Document, { Html, Main, NextScript } from 'next/document'
			      import Head from 'next/head'

			      class MyDocument extends Document {
			        render() {
			          return (
			            <Html>
			              <Head />
			              <body>
			                <Main />
			                <NextScript />
			              </body>
			            </Html>
			          )
			        }
			      }

			      export default MyDocument
			      ",
            None,
            None,
            Some(PathBuf::from("pages/_document.js")),
        ),
        (
            r"
			      import Document, { Html, Main, NextScript } from 'next/document'
			      import Head from 'next/head'

			      class MyDocument extends Document {
			        render() {
			          return (
			            <Html>
			              <Head />
			              <body>
			                <Main />
			                <NextScript />
			              </body>
			            </Html>
			          )
			        }
			      }

			      export default MyDocument
			      ",
            None,
            None,
            Some(PathBuf::from("pages/_document.tsx")),
        ),
        (
            r"
			      import Document, { Html, Main, NextScript } from 'next/document'
			      import Head from 'next/head'

			      class MyDocument extends Document {
			        render() {
			          return (
			            <Html>
			              <Head />
			              <body>
			                <Main />
			                <NextScript />
			              </body>
			            </Html>
			          )
			        }
			      }

			      export default MyDocument
			      ",
            None,
            None,
            Some(PathBuf::from("pages/_document.page.tsx")),
        ),
        (
            r"
			      import Document, { Html, Main, NextScript } from 'next/document'
			      import Head from 'next/head'

			      class MyDocument extends Document {
			        render() {
			          return (
			            <Html>
			              <Head />
			              <body>
			                <Main />
			                <NextScript />
			              </body>
			            </Html>
			          )
			        }
			      }

			      export default MyDocument
			      ",
            None,
            None,
            Some(PathBuf::from("pages/_document/index.tsx")),
        ),
        (
            r"
			      import Document, { Html, Main, NextScript } from 'next/document'
			      import Head from 'next/head'

			      class MyDocument extends Document {
			        render() {
			          return (
			            <Html>
			              <Head />
			              <body>
			                <Main />
			                <NextScript />
			              </body>
			            </Html>
			          )
			        }
			      }

			      export default MyDocument
			      ",
            None,
            None,
            Some(PathBuf::from("pages/_document/index.tsx")),
        ),
    ];

    Tester::new(NoHeadImportInDocument::NAME, NoHeadImportInDocument::PLUGIN, pass, fail)
        .test_and_snapshot();
}
