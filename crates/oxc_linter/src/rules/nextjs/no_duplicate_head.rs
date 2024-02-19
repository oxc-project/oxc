use oxc_ast::{
    ast::{Expression, ImportDeclarationSpecifier::ImportDefaultSpecifier},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-next(no-duplicate-head):")]
#[diagnostic(severity(warning), help("Do not include multiple instances of `<Head/>`. See: https://nextjs.org/docs/messages/no-duplicate-head"))]
struct NoDuplicateHeadDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoDuplicateHead;

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
    NoDuplicateHead,
    correctness
);

impl Rule for NoDuplicateHead {
    fn run_once(&self, ctx: &LintContext) {
        let mut document_import_name = None;
        let nodes = ctx.semantic().nodes();
        for node in nodes.iter() {
            match node.kind() {
                oxc_ast::AstKind::ImportDeclaration(decl) => {
                    if decl.source.value == "next/document" {
                        let document_import = decl.specifiers.as_ref().and_then(|specifiers| {
                            specifiers.iter().find(|spec| matches!(spec, ImportDefaultSpecifier(_)))
                        });
                        if let Some(ImportDefaultSpecifier(document_import_specifier)) =
                            document_import
                        {
                            document_import_name =
                                Some(document_import_specifier.local.name.clone());
                        }
                    }
                }
                oxc_ast::AstKind::ReturnStatement(stmt) => {
                    let document_class_id = nodes.ancestors(node.id()).find(|node_id| {
                        matches!(nodes.get_node(*node_id).kind(),
                        AstKind::Class(class)
                            if class
                                .super_class
                                .as_ref()
                                .and_then(|sc| sc.as_identifier())
                                .map(|id| &id.name)
                                == document_import_name.as_ref())
                    });
                    if document_class_id.map(|id| nodes.get_node(id)).is_none() {
                        continue;
                    }

                    let Some(Expression::JSXElement(ref element)) =
                        stmt.argument.as_ref().map(oxc_ast::ast::Expression::get_inner_expression)
                    else {
                        continue;
                    };
                    let head_components = element
                        .children
                        .iter()
                        .filter(|child| {
                            child
                                .as_element()
                                .and_then(|e| e.opening_element.name.as_identifier())
                                .map(|id| id.name == "Head")
                                .unwrap_or_default()
                        })
                        .collect::<Vec<_>>();
                    if head_components.len() > 1 {
                        for component in head_components {
                            ctx.diagnostic(NoDuplicateHeadDiagnostic(component.span()));
                        }
                    }
                }
                _ => continue,
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"import Document, { Html, Head, Main, NextScript } from 'next/document'
			
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
        r"
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
        r"
			      import Document, { Html, Main, NextScript } from 'next/document'
			      import Head from 'next/head'
			
			      class MyDocument extends Document {
			        render() {
			          return (
			            <Html>
			              <Head>
			                <meta charSet='utf-8' />
			                <link
			                  href='https://fonts.googleapis.com/css2?family=Sarabun:ital,wght@0,400;0,700;1,400;1,700&display=swap'
			                  rel='stylesheet'
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
			      ",
    ];

    Tester::new(NoDuplicateHead::NAME, pass, fail).test_and_snapshot();
}
