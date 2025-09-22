use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    rule::Rule,
};

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
    /// Prevents the usage of `next/head` inside a Next.js document.
    ///
    /// ### Why is this bad?
    ///
    /// Importing `next/head` inside `pages/_document.js` can cause
    /// unexpected issues in your Next.js application.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// import Document, { Html, Main, NextScript } from 'next/document'
    /// import Head from 'next/head';
    ///
    /// class MyDocument extends Document {
    ///   static async getInitialProps(ctx) {
    ///     //...
    ///   }
    ///
    ///   render() {
    ///     return (
    ///       <Html>
    ///         <Head></Head>
    ///       </Html>
    ///     )
    ///   }
    /// }
    ///
    /// export default MyDocument
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// import Document, { Html, Head, Main, NextScript } from 'next/document'
    ///
    /// class MyDocument extends Document {
    ///   static async getInitialProps(ctx) {
    ///     //...
    ///   }
    ///
    ///   render() {
    ///     return (
    ///       <Html>
    ///         <Head></Head>
    ///       </Html>
    ///     )
    ///   }
    /// }
    ///
    /// export default MyDocument
    /// ```
    NoHeadImportInDocument,
    nextjs,
    correctness
);

impl Rule for NoHeadImportInDocument {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ImportDeclaration(import_decl) = node.kind() else {
            return;
        };

        if import_decl.source.value.as_str() != "next/head" {
            return;
        }

        ctx.diagnostic(no_head_import_in_document_diagnostic(import_decl.span));
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        let full_file_path = ctx.file_path();

        if let Some(file_name) =
            full_file_path.file_name().as_ref().and_then(|file_name| file_name.to_str())
        {
            // check `_document.*` case
            if file_name.starts_with("_document.") {
                return true;
            // check `_document/index.*` case
            } else if file_name.starts_with("index")
                && let Some(file_name) = full_file_path
                    .parent()
                    .as_ref()
                    .and_then(|p| p.file_name())
                    .and_then(|file_name| file_name.to_str())
                && file_name.starts_with("_document")
            {
                return true;
            }
        }

        false
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
