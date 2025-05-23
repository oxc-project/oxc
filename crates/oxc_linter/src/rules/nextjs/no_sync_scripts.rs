use oxc_ast::{
    AstKind,
    ast::{JSXAttributeItem, JSXAttributeName, JSXElementName},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use rustc_hash::FxHashSet;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_sync_scripts_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prevent synchronous scripts.")
        .with_help("See https://nextjs.org/docs/messages/no-sync-scripts")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoSyncScripts;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule prevents the use of synchronous `<script>` tags in Next.js applications.
    /// It requires that any `<script>` tag with a `src` attribute must also have either
    /// the `async` or `defer` attribute.
    ///
    /// ### Why is this bad?
    ///
    /// Synchronous scripts can block the page rendering and negatively impact performance.
    /// In Next.js applications, it's recommended to use `async` or `defer` attributes
    /// to load scripts asynchronously, which improves page load times and user experience.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// // Synchronous script without async/defer
    /// <script src="https://example.com/script.js"></script>
    ///
    /// // Dynamic src without async/defer
    /// <script src={dynamicSrc}></script>
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// // Script with async attribute
    /// <script src="https://example.com/script.js" async></script>
    ///
    /// // Script with defer attribute
    /// <script src="https://example.com/script.js" defer></script>
    ///
    /// // Script with spread props (allowed as it might include async/defer)
    /// <script {...props}></script>
    /// ```
    NoSyncScripts,
    nextjs,
    correctness
);

impl Rule for NoSyncScripts {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(jsx_opening_element) = node.kind() else {
            return;
        };

        let JSXElementName::Identifier(jsx_opening_element_name) = &jsx_opening_element.name else {
            return;
        };

        if jsx_opening_element_name.name.as_str() != "script" {
            return;
        }

        let attributes_hs =
            jsx_opening_element
                .attributes
                .iter()
                .filter_map(|v| {
                    if let JSXAttributeItem::Attribute(v) = v { Some(&v.name) } else { None }
                })
                .filter_map(|v| {
                    if let JSXAttributeName::Identifier(v) = v { Some(v.name) } else { None }
                })
                .collect::<FxHashSet<_>>();

        if attributes_hs.contains("src")
            && !attributes_hs.contains("async")
            && !attributes_hs.contains("defer")
        {
            ctx.diagnostic(no_sync_scripts_diagnostic(jsx_opening_element_name.span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"import {Head} from 'next/document';
			
			      export class Blah extends Head {
			        render() {
			          return (
			            <div>
			              <h1>Hello title</h1>
			              <script src='https://blah.com' async></script>
			            </div>
			          );
			        }
			    }",
        r"import {Head} from 'next/document';
			
			      export class Blah extends Head {
			        render(props) {
			          return (
			            <div>
			              <h1>Hello title</h1>
			              <script {...props} ></script>
			            </div>
			          );
			        }
			    }",
    ];

    let fail = vec![
        r"
			      import {Head} from 'next/document';
			
			        export class Blah extends Head {
			          render() {
			            return (
			              <div>
			                <h1>Hello title</h1>
			                <script src='https://blah.com'></script>
			              </div>
			            );
			          }
			      }",
        r"
			      import {Head} from 'next/document';
			
			        export class Blah extends Head {
			          render(props) {
			            return (
			              <div>
			                <h1>Hello title</h1>
			                <script src={props.src}></script>
			              </div>
			            );
			          }
			      }",
    ];

    Tester::new(NoSyncScripts::NAME, NoSyncScripts::PLUGIN, pass, fail).test_and_snapshot();
}
