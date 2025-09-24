use oxc_ast::{
    AstKind,
    ast::{ImportDeclarationSpecifier, JSXChild, JSXElementName},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_script_component_in_head_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prevent usage of `next/script` in `next/head` component.")
        .with_help("See https://nextjs.org/docs/messages/no-script-component-in-head")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoScriptComponentInHead;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prevent usage of `next/script` in `next/head` component.
    ///
    /// ### Why is this bad?
    ///
    /// The `next/script` component should not be used in a `next/head` component.
    /// Instead move the `<Script />` component outside of `<Head>` instead.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// import Script from 'next/script'
    /// import Head from 'next/head'
    ///
    /// export default function Index() {
    ///   return (
    ///     <Head>
    ///       <title>Next.js</title>
    ///       <Script src="/my-script.js" />
    ///     </Head>
    ///   )
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// import Script from 'next/script'
    /// import Head from 'next/head'
    ///
    /// export default function Index() {
    ///   return (
    ///     <>
    ///       <Head>
    ///         <title>Next.js</title>
    ///       </Head>
    ///       <Script src="/my-script.js" />
    ///     </>
    ///   )
    /// }
    /// ```
    NoScriptComponentInHead,
    nextjs,
    correctness
);

impl Rule for NoScriptComponentInHead {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ImportDeclaration(import_decl) = node.kind() else {
            return;
        };

        if import_decl.source.value.as_str() != "next/head" {
            return;
        }

        let Some(import_specifiers) = &import_decl.specifiers else {
            return;
        };

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
                    && let JSXElementName::IdentifierReference(child_element_name) =
                        &child_element.opening_element.name
                    && child_element_name.name.as_str() == "Script"
                {
                    ctx.diagnostic(no_script_component_in_head_diagnostic(
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
        r#"
            import Script from "next/script";
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
		    }
        "#,
    ];

    Tester::new(NoScriptComponentInHead::NAME, NoScriptComponentInHead::PLUGIN, pass, fail)
        .test_and_snapshot();
}
