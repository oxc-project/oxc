use oxc_ast::AstKind;
use oxc_diagnostics::{LabeledSpan, OxcDiagnostic};
use oxc_macros::declare_oxc_lint;
use oxc_span::GetSpan;

use crate::{context::LintContext, rule::Rule};

fn no_duplicate_head(labels: Vec<LabeledSpan>) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not include multiple instances of `<Head/>`")
        .with_help("Only use a single `<Head />` component in your custom document in `pages/_document.js`. See: https://nextjs.org/docs/messages/no-duplicate-head")
        .with_labels(labels)
}
#[derive(Debug, Default, Clone)]
pub struct NoDuplicateHead;

declare_oxc_lint!(
    /// ### What it does
    /// Prevent duplicate usage of `<Head>` in `pages/_document.js``.
    ///
    /// ### Why is this bad?
    /// This can cause unexpected behavior in your application.
    ///
    /// ### Example
    /// ```jsx
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
    nextjs,
    correctness
);

impl Rule for NoDuplicateHead {
    fn run_on_symbol(&self, symbol_id: oxc_semantic::SymbolId, ctx: &LintContext<'_>) {
        let symbols = ctx.symbols();
        let name = symbols.get_name(symbol_id);
        if name != "Head" {
            return;
        }

        let flags = symbols.get_flags(symbol_id);
        if !flags.is_import() {
            return;
        }

        let scope_id = symbols.get_scope_id(symbol_id);
        if scope_id != ctx.scopes().root_scope_id() {
            return;
        }

        // 1 x `<Head>` is fine, more than 1 is not.
        // Avoid allocating a `Vec`, or looking up span in common case
        // where only a single `<Head>` is found.
        let mut first_node_id = None;
        let mut labels = vec![];
        let nodes = ctx.nodes();
        let get_label = |node_id| {
            let span = nodes.kind(node_id).span();
            LabeledSpan::underline(span)
        };

        for reference in symbols.get_resolved_references(symbol_id) {
            if !reference.is_read() {
                continue;
            }

            if !matches!(
                nodes.ancestor_ids(reference.node_id()).nth(2).map(|node_id| nodes.kind(node_id)),
                Some(AstKind::JSXOpeningElement(_))
            ) {
                continue;
            }

            let node_id = reference.node_id();
            #[allow(clippy::unnecessary_unwrap)]
            if first_node_id.is_none() {
                // First `<Head>` found
                first_node_id = Some(node_id);
            } else if labels.is_empty() {
                // 2nd `<Head>` found - populate `labels` with both
                let first_node_id = first_node_id.unwrap();
                labels.extend([get_label(first_node_id), get_label(node_id)]);
            } else {
                // Further `<Head>` found - add to `node_ids`
                labels.push(get_label(node_id));
            }
        }

        // `labels` is empty if 0 or 1 `<Head>` found
        if labels.is_empty() {
            return;
        }

        ctx.diagnostic(no_duplicate_head(labels));
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

    Tester::new(NoDuplicateHead::NAME, NoDuplicateHead::PLUGIN, pass, fail).test_and_snapshot();
}
