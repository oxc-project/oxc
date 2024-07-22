use oxc_ast::AstKind;
use oxc_diagnostics::{LabeledSpan, OxcDiagnostic};
use oxc_macros::declare_oxc_lint;
use oxc_semantic::Reference;

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Default, Clone)]
pub struct NoDuplicateHead;

declare_oxc_lint!(
    /// ### What it does
    /// Prevent duplicate usage of <Head> in pages/_document.js.
    ///
    /// ### Why is this bad?
    /// This can cause unexpected behavior in your application.
    ///
    /// ### Example
    /// ```javascript
    /// import Document, { Html, Head, Main, NextScript } from 'next/document'
    /// class MyDocument extends Document {
    ///   static async getInitialProps(ctx) {
    ///   }
    ///   render() {
    ///     return (
    ///       <Html>
    ///         <Head />
    ///         <body>
    ///           <Main />
    ///           <NextScript />
    ///         </body>
    ///       </Html>
    ///    )
    ///  }
    ///}
    ///export default MyDocument
    /// ```
    NoDuplicateHead,
    correctness
);

impl Rule for NoDuplicateHead {
    fn run_on_symbol(&self, symbol_id: oxc_semantic::SymbolId, ctx: &LintContext<'_>) {
        let symbols = ctx.symbols();
        let name = symbols.get_name(symbol_id);
        if name != "Head" {
            return;
        }

        let flag = symbols.get_flag(symbol_id);
        if !flag.is_import() {
            return;
        }

        let scope_id = symbols.get_scope_id(symbol_id);
        if scope_id != ctx.scopes().root_scope_id() {
            return;
        }

        let nodes = ctx.nodes();
        let labels = symbols
            .get_resolved_references(symbol_id)
            .filter(|r| r.is_read())
            .filter(|r| {
                let kind = nodes.ancestors(r.node_id()).nth(2).map(|node_id| nodes.kind(node_id));
                matches!(kind, Some(AstKind::JSXOpeningElement(_)))
            })
            .map(Reference::span)
            .map(LabeledSpan::underline)
            .collect::<Vec<_>>();

        if labels.len() <= 1 {
            return;
        }

        ctx.diagnostic(
            OxcDiagnostic::warn("Do not include multiple instances of `<Head/>`")
                .with_help("Only use a single `<Head />` component in your custom document in `pages/_document.js`. See: https://nextjs.org/docs/messages/no-duplicate-head")
                .with_labels(labels),
        );
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "import Document, { Html, Head, Main, NextScript } from 'next/document'

			      class MyDocument extends Document {
			        static async getInitialProps(ctx) {
			          //...
			        }

			        render() {
			          return (
			            <Html>
			              <Head/>
			            </Html>
			          )
			        }
			      }

			      export default MyDocument
			    ",
        r#"import Document, { Html, Head, Main, NextScript } from 'next/document'

			      class MyDocument extends Document {
			        render() {
			          return (
			            <Html>
			              <Head>
			                <meta charSet="utf-8" />
			                <link
			                  href="https://fonts.googleapis.com/css2?family=Sarabun:ital,wght@0,400;0,700;1,400;1,700&display=swap"
			                  rel="stylesheet"
			                />
			              </Head>
			            </Html>
			          )
			        }
			      }

			      export default MyDocument
			    "#,
    ];

    let fail = vec![
        "
			      import Document, { Html, Main, NextScript } from 'next/document'
			      import Head from 'next/head'

			      class MyDocument extends Document {
			        render() {
			          return (
			            <Html>
			              <Head />
			              <Head />
			              <Head />
			            </Html>
			          )
			        }
			      }

			      export default MyDocument
			      ",
        r#"
			      import Document, { Html, Main, NextScript } from 'next/document'
			      import Head from 'next/head'

			      class MyDocument extends Document {
			        render() {
			          return (
			            <Html>
			              <Head>
			                <meta charSet="utf-8" />
			                <link
			                  href="https://fonts.googleapis.com/css2?family=Sarabun:ital,wght@0,400;0,700;1,400;1,700&display=swap"
			                  rel="stylesheet"
			                />
			              </Head>
			              <body>
			                <Main />
			                <NextScript />
			              </body>
			              <Head>
			                <script
			                  dangerouslySetInnerHTML={{
			                    __html: '',
			                  }}
			                />
			              </Head>
			            </Html>
			          )
			        }
			      }

			      export default MyDocument
			      "#,
    ];

    Tester::new(NoDuplicateHead::NAME, pass, fail).test_and_snapshot();
}
