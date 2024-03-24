use oxc_ast::{
    ast::{ImportDeclarationSpecifier, JSXChild, JSXElementName, ModuleDeclaration},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-next(no-script-component-in-head): Prevent usage of `next/script` in `next/head` component.")]
#[diagnostic(
    severity(warning),
    help("See https://nextjs.org/docs/messages/no-script-component-in-head")
)]
struct NoScriptComponentInHeadDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoScriptComponentInHead;

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
    NoScriptComponentInHead,
    correctness
);

impl Rule for NoScriptComponentInHead {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ModuleDeclaration(ModuleDeclaration::ImportDeclaration(import_decl)) =
            node.kind()
        else {
            return;
        };

        if import_decl.source.value.as_str() != "next/head" {
            return;
        }

        let Some(import_specifiers) = &import_decl.specifiers else { return };

        let Some(default_import) = import_specifiers.iter().find_map(|import_specifier| {
            let ImportDeclarationSpecifier::ImportDefaultSpecifier(import_default_specifier) =
                import_specifier
            else {
                return None;
            };

            Some(import_default_specifier)
        }) else {
            return;
        };

        for reference in
            ctx.semantic().symbol_references(default_import.local.symbol_id.get().unwrap())
        {
            let Some(node) = ctx.nodes().parent_node(reference.node_id()) else { return };

            let AstKind::JSXElementName(_) = node.kind() else { continue };
            let parent_node = ctx.nodes().parent_node(node.id()).unwrap();
            let AstKind::JSXOpeningElement(jsx_opening_element) = parent_node.kind() else {
                continue;
            };
            let Some(AstKind::JSXElement(jsx_element)) = ctx.nodes().parent_kind(parent_node.id())
            else {
                continue;
            };

            for child in &jsx_element.children {
                if let JSXChild::Element(child_element) = child {
                    if let JSXElementName::Identifier(child_element_name) =
                        &child_element.opening_element.name
                    {
                        if child_element_name.name.as_str() == "Script" {
                            ctx.diagnostic(NoScriptComponentInHeadDiagnostic(
                                jsx_opening_element.name.span(),
                            ));
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"import Script from "next/script";
			     const Head = ({children}) => children
			
			    export default function Index() {
			      return (
			        <Head>
			          <Script></Script>
			        </Head>
			      );
			    }
			    "#,
    ];

    let fail = vec![
        r#"
			      import Head from "next/head";
			      import Script from "next/script";
			
			      export default function Index() {
			        return (
			            <Head>
			              <Script></Script>
			            </Head>
			        );
			      }"#,
    ];

    Tester::new(NoScriptComponentInHead::NAME, pass, fail).test_and_snapshot();
}
