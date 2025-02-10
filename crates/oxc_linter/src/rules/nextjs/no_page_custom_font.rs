use oxc_ast::{
    ast::{Class, Function, JSXAttributeItem, JSXAttributeValue, JSXElementName},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

fn not_added_in_document(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Custom fonts not added in `pages/_document.js` will only load for a single page. This is discouraged.")
        .with_help("See: https://nextjs.org/docs/messages/no-page-custom-font")
        .with_label(span)
}

fn link_outside_of_head(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Using `<link />` outside of `<Head>` will disable automatic font optimization. This is discouraged.")
        .with_help("See: 'https://nextjs.org/docs/messages/no-page-custom-font")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoPageCustomFont;

declare_oxc_lint!(
    /// ### What it does
    /// Prevent page-only custom fonts.
    ///
    /// ### Why is this bad?
    /// * The custom font you're adding was added to a page - this only adds the font to the specific page and not the entire application.
    /// * The custom font you're adding was added to a separate component within pages/_document.js - this disables automatic font optimization.
    ///
    /// ### Example
    /// ```javascript
    /// ```
    NoPageCustomFont,
    nextjs,
    correctness,
);

impl Rule for NoPageCustomFont {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(element) = node.kind() else {
            return;
        };
        let JSXElementName::Identifier(ident) = &element.name else { return };

        if ident.name != "link" {
            return;
        }

        let is_custom_font = element.attributes.iter().any(|attr| {
            matches!(&attr,
              JSXAttributeItem::Attribute(attr) if attr.is_identifier("href") && attr.value.as_ref().is_some_and(|value|
              matches!(value, JSXAttributeValue::StringLiteral(literal) if literal.value.starts_with("https://fonts.googleapis.com/css"))
            ))
        });

        if !is_custom_font {
            return;
        }

        let in_document = ctx.file_path().file_name().is_some_and(|file_name| {
            file_name.to_str().is_some_and(|file_name| file_name.starts_with("_document."))
        });
        let span = ctx.nodes().parent_kind(node.id()).unwrap().span();
        let diagnostic = if in_document {
            if is_inside_export_default(node, ctx) {
                return;
            }
            link_outside_of_head(span)
        } else {
            not_added_in_document(span)
        };
        ctx.diagnostic(diagnostic);
    }
}

fn is_inside_export_default(node: &AstNode<'_>, ctx: &LintContext<'_>) -> bool {
    let mut is_inside_export_default = false;
    for parent_node in ctx.nodes().ancestors(node.id()) {
        // export default function/class
        let kind = parent_node.kind();
        if matches!(kind, AstKind::ExportDefaultDeclaration(_)) {
            is_inside_export_default = true;
            break;
        }

        // function variable() {}; export default variable;
        let id = match kind {
            AstKind::ArrowFunctionExpression(_) => None,
            AstKind::Function(Function { id, .. }) | AstKind::Class(Class { id, .. }) => id.clone(),
            _ => continue,
        };

        let name = id.map_or_else(
            || {
                let parent_parent_kind = ctx.nodes().parent_kind(parent_node.id())?;

                let AstKind::VariableDeclarator(declarator) = parent_parent_kind else {
                    return None;
                };
                declarator.id.get_identifier_name().map(|id| id.to_string())
            },
            |id| Some(id.name.to_string()),
        );
        let Some(name) = name else {
            continue;
        };
        if ctx
            .module_record()
            .local_export_entries
            .iter()
            .any(|e| e.local_name.is_default() && e.local_name.name().is_some_and(|n| n == name))
        {
            is_inside_export_default = true;
        }
    }
    is_inside_export_default
}

#[test]
fn test() {
    use std::path::PathBuf;

    use crate::tester::Tester;

    let filename = Some(PathBuf::from("pages/_document.jsx"));
    let pass = vec![
        (
            r#"import Document, { Html, Head } from "next/document";
			class MyDocument extends Document {
				render() {
					return (
						<Html>
							<Head>
								<link
									href="https://fonts.googleapis.com/css2?family=Krona+One&display=swap"
									rel="stylesheet"
								/>
							</Head>
						</Html>
					);
				}
			}
			export default MyDocument;"#,
            None,
            None,
            filename.clone(),
        ),
        (
            r#"import NextDocument, { Html, Head } from "next/document";
			    class Document extends NextDocument {
			      render() {
			        return (
			          <Html>
			            <Head>
			              <link
			                href="https://fonts.googleapis.com/css2?family=Krona+One&display=swap"
			                rel="stylesheet"
			              />
			            </Head>
			          </Html>
			        );
			      }
			    }
			    export default Document;
			    "#,
            None,
            None,
            filename.clone(),
        ),
        (
            r#"export default function CustomDocument() {
			      return (
			        <Html>
			          <Head>
			            <link
			              href="https://fonts.googleapis.com/css2?family=Krona+One&display=swap"
			              rel="stylesheet"
			            />
			          </Head>
			        </Html>
			      )
			    }"#,
            None,
            None,
            filename.clone(),
        ),
        (
            r#"function CustomDocument() {
			      return (
			        <Html>
			          <Head>
			            <link
			              href="https://fonts.googleapis.com/css2?family=Krona+One&display=swap"
			              rel="stylesheet"
			            />
			          </Head>
			        </Html>
			      )
			    }

			    export default CustomDocument;
			    "#,
            None,
            None,
            filename.clone(),
        ),
        (
            r#"
			      import Document, { Html, Head } from "next/document";
			      class MyDocument {
			        render() {
			          return (
			            <Html>
			              <Head>
			                <link
			                  href="https://fonts.googleapis.com/css2?family=Krona+One&display=swap"
			                  rel="stylesheet"
			                />
			              </Head>
			            </Html>
			          );
			        }
			      }

			      export default MyDocument;"#,
            None,
            None,
            filename.clone(),
        ),
        (
            r#"export default function() {
			      return (
			        <Html>
			          <Head>
			            <link
			              href="https://fonts.googleapis.com/css2?family=Krona+One&display=swap"
			              rel="stylesheet"
			            />
			          </Head>
			        </Html>
			      )
			    }"#,
            None,
            None,
            filename.clone(),
        ),
        (
            r#"function a() {
			      return (
			        <Html>
			          <Head>
                  <link
                    rel="apple-touch-startup-image"
                    href="/assets/public/pwa/splash/apple-splash-2048-2732.jpg"
                  />
			          </Head>
			        </Html>
			      )
			    }"#,
            None,
            None,
            filename.clone(),
        ),
    ];

    let fail = vec![
        (
            r#"
			      import Head from 'next/head'
			      export default function IndexPage() {
			        return (
			          <div>
			            <Head>
			              <link
			                href="https://fonts.googleapis.com/css2?family=Inter"
			                rel="stylesheet"
			              />
			            </Head>
			            <p>Hello world!</p>
			          </div>
			        )
			      }
			      "#,
            None,
            None,
            Some(PathBuf::from("pages/index.tsx")),
        ),
        (
            r#"
			      import Head from 'next/head'


			      function Links() {
			        return (
			          <>
			            <link
			              href="https://fonts.googleapis.com/css2?family=Inter"
			              rel="stylesheet"
			            />
			            <link
			              href="https://fonts.googleapis.com/css2?family=Open+Sans"
			              rel="stylesheet"
			              />
			          </>
			        )
			      }

			      export default function IndexPage() {
			        return (
			          <div>
			            <Head>
			              <Links />
			            </Head>
			            <p>Hello world!</p>
			          </div>
			        )
			      }
			      "#,
            None,
            None,
            filename,
        ),
    ];

    Tester::new(NoPageCustomFont::NAME, NoPageCustomFont::PLUGIN, pass, fail).test_and_snapshot();
}
