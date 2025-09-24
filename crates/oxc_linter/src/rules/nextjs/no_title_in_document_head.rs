use oxc_ast::{
    AstKind,
    ast::{ImportDeclarationSpecifier, JSXChild, JSXElementName},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_title_in_document_head_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prevent usage of `<title>` with `Head` component from `next/document`.")
        .with_help("See https://nextjs.org/docs/messages/no-title-in-document-head")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoTitleInDocumentHead;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prevent usage of `<title>` with `Head` component from `next/document`.
    ///
    /// ### Why is this bad?
    ///
    /// A `<title>` element should only be used for any `<head>` code that is common for all pages.
    /// Title tags should be defined at the page-level using `next/head` instead.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// import {Head} from 'next/document'
    ///
    /// export function Home() {
    ///   return (
    ///     <div>
    ///       <Head>
    ///         <title>My page title</title>
    ///       </Head>
    ///     </div>
    ///   )
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// import Head from 'next/head'
    ///
    /// export function Home() {
    ///   return (
    ///     <div>
    ///       <Head>
    ///         <title>My page title</title>
    ///       </Head>
    ///     </div>
    ///   )
    /// }
    /// ```
    NoTitleInDocumentHead,
    nextjs,
    correctness
);

impl Rule for NoTitleInDocumentHead {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ImportDeclaration(import_decl) = node.kind() else {
            return;
        };

        if import_decl.source.value.as_str() != "next/document" {
            return;
        }

        let Some(import_specifiers) = &import_decl.specifiers else {
            return;
        };

        let Some(default_import) = import_specifiers.iter().find_map(|import_specifier| {
            let ImportDeclarationSpecifier::ImportSpecifier(import_default_specifier) =
                import_specifier
            else {
                return None;
            };

            Some(import_default_specifier)
        }) else {
            return;
        };

        for reference in ctx.semantic().symbol_references(default_import.local.symbol_id()) {
            let parent_node = ctx.nodes().parent_node(reference.node_id());
            let AstKind::JSXOpeningElement(jsx_opening_element) = parent_node.kind() else {
                continue;
            };
            let AstKind::JSXElement(jsx_element) = ctx.nodes().parent_kind(parent_node.id()) else {
                continue;
            };

            for child in &jsx_element.children {
                if let JSXChild::Element(child_element) = child
                    && let JSXElementName::Identifier(child_element_name) =
                        &child_element.opening_element.name
                    && child_element_name.name.as_str() == "title"
                {
                    ctx.diagnostic(no_title_in_document_head_diagnostic(
                        jsx_opening_element.name.span(),
                    ));
                }
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"import Head from "next/head";

			     class Test {
			      render() {
			        return (
			          <Head>
			            <title>My page title</title>
			          </Head>
			        );
			      }
			     }"#,
        r#"import Document, { Html, Head } from "next/document";

			     class MyDocument extends Document {
			      render() {
			        return (
			          <Html>
			            <Head>
			            </Head>
			          </Html>
			        );
			      }
			     }

			     export default MyDocument;
			     "#,
    ];

    let fail = vec![
        r#"
			      import { Head } from "next/document";

			      class Test {
			        render() {
			          return (
			            <Head>
			              <title>My page title</title>
			            </Head>
			          );
			        }
			      }"#,
    ];

    Tester::new(NoTitleInDocumentHead::NAME, NoTitleInDocumentHead::PLUGIN, pass, fail)
        .test_and_snapshot();
}
