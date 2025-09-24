use oxc_ast::{
    AstKind,
    ast::{Expression, JSXAttributeItem, JSXAttributeName, ObjectPropertyKind, PropertyKey},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use rustc_hash::FxHashSet;

use crate::{AstNode, context::LintContext, rule::Rule};

fn inline_script_id_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "`next/script` components with inline content must specify an `id` attribute.",
    )
    .with_help("See https://nextjs.org/docs/messages/inline-script-id")
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct InlineScriptId;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces that all `next/script` components with inline content or `dangerouslySetInnerHTML` must have an `id` prop.
    ///
    /// ### Why is this bad?
    ///
    /// Next.js requires a unique `id` prop for inline scripts to properly deduplicate them during page renders.
    /// Without an `id`, the same inline script might be executed multiple times, leading to unexpected behavior
    /// or performance issues. This is particularly important for scripts that modify global state or perform
    /// one-time initializations.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// import Script from 'next/script';
    ///
    /// export default function Page() {
    ///   return (
    ///     <Script>
    ///       {`console.log('Hello world');`}
    ///     </Script>
    ///   );
    /// }
    ///
    /// // Also incorrect with dangerouslySetInnerHTML
    /// export default function Page() {
    ///   return (
    ///     <Script
    ///       dangerouslySetInnerHTML={{
    ///         __html: `console.log('Hello world');`
    ///       }}
    ///     />
    ///   );
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// import Script from 'next/script';
    ///
    /// export default function Page() {
    ///   return (
    ///     <Script id="my-script">
    ///       {`console.log('Hello world');`}
    ///     </Script>
    ///   );
    /// }
    ///
    /// // Correct with dangerouslySetInnerHTML
    /// export default function Page() {
    ///   return (
    ///     <Script
    ///       id="my-script"
    ///       dangerouslySetInnerHTML={{
    ///         __html: `console.log('Hello world');`
    ///       }}
    ///     />
    ///   );
    /// }
    ///
    /// // No id required for external scripts
    /// export default function Page() {
    ///   return (
    ///     <Script src="https://example.com/script.js" />
    ///   );
    /// }
    /// ```
    InlineScriptId,
    nextjs,
    correctness
);

impl Rule for InlineScriptId {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ImportDefaultSpecifier(specifier) = node.kind() else {
            return;
        };
        let AstKind::ImportDeclaration(import_decl) = ctx.nodes().parent_kind(node.id()) else {
            return;
        };

        if import_decl.source.value.as_str() != "next/script" {
            return;
        }

        'references_loop: for reference in
            ctx.semantic().symbol_references(specifier.local.symbol_id())
        {
            let parent_node = ctx.nodes().parent_node(reference.node_id());
            let AstKind::JSXOpeningElement(jsx_opening_element) = parent_node.kind() else {
                continue;
            };

            let AstKind::JSXElement(jsx_element) = ctx.nodes().parent_kind(parent_node.id()) else {
                continue;
            };

            let mut prop_names_hash_set = FxHashSet::default();

            for prop in &jsx_opening_element.attributes {
                match prop {
                    JSXAttributeItem::Attribute(attr) => {
                        if let JSXAttributeName::Identifier(ident) = &attr.name {
                            prop_names_hash_set.insert(ident.name);
                        }
                    }
                    JSXAttributeItem::SpreadAttribute(spread_attr) => {
                        if let Expression::ObjectExpression(obj_expr) =
                            spread_attr.argument.without_parentheses()
                        {
                            for prop in &obj_expr.properties {
                                if let ObjectPropertyKind::ObjectProperty(obj_prop) = prop
                                    && let PropertyKey::StaticIdentifier(ident) = &obj_prop.key
                                {
                                    prop_names_hash_set.insert(ident.name);
                                }
                            }
                        } else {
                            continue 'references_loop;
                        }
                    }
                }
            }

            if prop_names_hash_set.contains("id") {
                continue;
            }

            if !jsx_element.children.is_empty()
                || prop_names_hash_set.contains("dangerouslySetInnerHTML")
            {
                ctx.diagnostic(inline_script_id_diagnostic(jsx_opening_element.name.span()));
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"import Script from 'next/script';
			
			      export default function TestPage() {
			        return (
			          <Script id="test-script">
			            {`console.log('Hello world');`}
			          </Script>
			        )
			      }"#,
        r#"import Script from 'next/script';
			
			      export default function TestPage() {
			        return (
			          <Script
			            id="test-script"
			            dangerouslySetInnerHTML={{
			              __html: `console.log('Hello world');`
			            }}
			          />
			        )
			      }"#,
        r#"import Script from 'next/script';
			
			      export default function TestPage() {
			        return (
			          <Script src="https://example.com" />
			        )
			      }"#,
        r#"import MyScript from 'next/script';
			
			      export default function TestPage() {
			        return (
			          <MyScript id="test-script">
			            {`console.log('Hello world');`}
			          </MyScript>
			        )
			      }"#,
        r#"import MyScript from 'next/script';
			
			      export default function TestPage() {
			        return (
			          <MyScript
			            id="test-script"
			            dangerouslySetInnerHTML={{
			              __html: `console.log('Hello world');`
			            }}
			          />
			        )
			      }"#,
        r#"import Script from 'next/script';
			
			      export default function TestPage() {
			        return (
			          <Script {...{ strategy: "lazyOnload" }} id={"test-script"}>
			            {`console.log('Hello world');`}
			          </Script>
			        )
			      }"#,
        r#"import Script from 'next/script';
			
			      export default function TestPage() {
			        return (
			          <Script {...{ strategy: "lazyOnload", id: "test-script" }}>
			            {`console.log('Hello world');`}
			          </Script>
			        )
			      }"#,
        r#"import Script from 'next/script';
			      const spread = { strategy: "lazyOnload" }
			      export default function TestPage() {
			        return (
			          <Script {...spread} id={"test-script"}>
			            {`console.log('Hello world');`}
			          </Script>
			        )
			      }"#,
    ];

    let fail = vec![
        r"import Script from 'next/script';
			
			        export default function TestPage() {
			          return (
			            <Script>
			              {`console.log('Hello world');`}
			            </Script>
			          )
			        }",
        r"import Script from 'next/script';
			
			        export default function TestPage() {
			          return (
			            <Script
			              dangerouslySetInnerHTML={{
			                __html: `console.log('Hello world');`
			              }}
			            />
			          )
			        }",
        r"import MyScript from 'next/script';
			
			        export default function TestPage() {
			          return (
			            <MyScript>
			              {`console.log('Hello world');`}
			            </MyScript>
			          )
			        }",
        r"import MyScript from 'next/script';
			
			        export default function TestPage() {
			          return (
			            <MyScript
			              dangerouslySetInnerHTML={{
			                __html: `console.log('Hello world');`
			              }}
			            />
			          )
			        }",
    ];

    Tester::new(InlineScriptId::NAME, InlineScriptId::PLUGIN, pass, fail)
        .with_nextjs_plugin(true)
        .test_and_snapshot();
}
