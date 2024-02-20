use oxc_ast::{
    ast::{Expression, JSXAttributeItem, JSXAttributeName, ObjectPropertyKind, PropertyKey},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use rustc_hash::FxHashSet;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-next(inline-script-id): `next/script` components with inline content must specify an `id` attribute.")]
#[diagnostic(severity(warning), help("See https://nextjs.org/docs/messages/inline-script-id"))]
struct InlineScriptIdDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct InlineScriptId;

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
    InlineScriptId,
    correctness
);

impl Rule for InlineScriptId {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ImportDefaultSpecifier(specifier) = node.kind() else {
            return;
        };
        let Some(AstKind::ImportDeclaration(import_decl)) = ctx.nodes().parent_kind(node.id())
        else {
            return;
        };

        if import_decl.source.value.as_str() != "next/script" {
            return;
        }

        'references_loop: for reference in
            ctx.semantic().symbol_references(specifier.local.symbol_id.get().unwrap())
        {
            let node = ctx.nodes().get_node(reference.node_id());

            let AstKind::JSXElementName(_) = node.kind() else { continue };
            let parent_node = ctx.nodes().parent_node(node.id()).unwrap();
            let AstKind::JSXOpeningElement(jsx_opening_element) = parent_node.kind() else {
                continue;
            };

            let Some(AstKind::JSXElement(jsx_element)) = ctx.nodes().parent_kind(parent_node.id())
            else {
                continue;
            };

            let mut prop_names_hash_set = FxHashSet::default();

            for prop in &jsx_opening_element.attributes {
                match prop {
                    JSXAttributeItem::Attribute(attr) => {
                        if let JSXAttributeName::Identifier(ident) = &attr.name {
                            prop_names_hash_set.insert(ident.name.clone());
                        }
                    }
                    JSXAttributeItem::SpreadAttribute(spread_attr) => {
                        if let Expression::ObjectExpression(obj_expr) =
                            spread_attr.argument.without_parenthesized()
                        {
                            for prop in &obj_expr.properties {
                                if let ObjectPropertyKind::ObjectProperty(obj_prop) = prop {
                                    if let PropertyKey::Identifier(ident) = &obj_prop.key {
                                        prop_names_hash_set.insert(ident.name.clone());
                                    }
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

            if jsx_element.children.len() > 0
                || prop_names_hash_set.contains("dangerouslySetInnerHTML")
            {
                ctx.diagnostic(InlineScriptIdDiagnostic(jsx_opening_element.name.span()));
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

    Tester::new(InlineScriptId::NAME, pass, fail).with_nextjs_plugin(true).test_and_snapshot();
}
