use cow_utils::CowUtils;
use oxc_ast::{
    AstKind,
    ast::{JSXAttributeItem, JSXAttributeName, JSXAttributeValue},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;
use oxc_span::Span;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{NEXT_POLYFILLED_FEATURES, find_url_query_value, get_next_script_import_local_name},
};

fn no_unwanted_polyfillio_diagnostic(polyfill_name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "No duplicate polyfills from Polyfill.io are allowed. {polyfill_name} already shipped with Next.js."
    ))
    .with_help("See https://nextjs.org/docs/messages/no-unwanted-polyfillio")
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoUnwantedPolyfillio;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prevent duplicate polyfills from Polyfill.io.
    ///
    /// ### Why is this bad?
    ///
    /// You are using polyfills from Polyfill.io and including polyfills already shipped with Next.js. This unnecessarily increases page weight which can affect loading performance.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// <script src='https://polyfill.io/v3/polyfill.min.js?features=Array.prototype.copyWithin'></script>
    ///
    /// <script src='https://polyfill.io/v3/polyfill.min.js?features=WeakSet%2CPromise%2CPromise.prototype.finally%2Ces2015%2Ces5%2Ces6'></script>
    /// ```
    NoUnwantedPolyfillio,
    nextjs,
    correctness
);

impl Rule for NoUnwantedPolyfillio {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(jsx_el) = node.kind() else {
            return;
        };

        let Some(tag_name) = jsx_el.name.get_identifier_name() else {
            return;
        };

        if tag_name.as_str() != "script" {
            let next_script_import_local_name = get_next_script_import_local_name(ctx);
            if !matches!(next_script_import_local_name, Some(import) if tag_name == import) {
                return;
            }
        }

        if jsx_el.attributes.is_empty() {
            return;
        }

        let Some(JSXAttributeItem::Attribute(src)) = jsx_el.attributes.iter().find(|attr| {
            matches!(
                attr,
                JSXAttributeItem::Attribute(jsx_attr)
                    if matches!(
                        &jsx_attr.name,
                        JSXAttributeName::Identifier(id) if id.name.as_str() == "src"
                    )
            )
        }) else {
            return;
        };

        let Some(JSXAttributeValue::StringLiteral(src_value)) = &src.value else {
            return;
        };

        if src_value.value.as_str().starts_with("https://cdn.polyfill.io/v2/")
            || src_value.value.as_str().starts_with("https://polyfill.io/v3/")
        {
            let Some(features_value) = find_url_query_value(src_value.value.as_str(), "features")
            else {
                return;
            };

            // Replace URL encoded values
            let features_value = features_value.cow_replace("%2C", ",");

            let unwanted_features: Vec<&str> = features_value
                .split(',')
                .filter(|feature| NEXT_POLYFILLED_FEATURES.contains(feature))
                .collect();
            if !unwanted_features.is_empty() {
                ctx.diagnostic(no_unwanted_polyfillio_diagnostic(
                    &format!(
                        "{} {}",
                        unwanted_features.join(", "),
                        if unwanted_features.len() > 1 { "are" } else { "is" }
                    ),
                    src.span,
                ));
            }
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
                  <script src='https://polyfill.io/v3/polyfill.min.js?features=AbortController'></script>
                </div>
              );
            }
        }",
        r"import {Head} from 'next/document';

          export class Blah extends Head {
            render() {
              return (
                <div>
                  <h1>Hello title</h1>
                  <script src='https://polyfill.io/v3/polyfill.min.js?features=IntersectionObserver'></script>
                </div>
              );
            }
        }",
        r"
          import Script from 'next/script';

          export function MyApp({ Component, pageProps }) {
              return (
                <div>
                  <Component {...pageProps} />
                  <Script src='https://polyfill.io/v3/polyfill.min.js?features=IntersectionObserver' />
                </div>
              );
        }",
    ];

    let fail = vec![
        r"import {Head} from 'next/document';

                  export class Blah extends Head {
                    render() {
                      return (
                        <div>
                          <h1>Hello title</h1>
                          <script src='https://polyfill.io/v3/polyfill.min.js?features=WeakSet%2CPromise%2CPromise.prototype.finally%2Ces2015%2Ces5%2Ces6'></script>
                        </div>
                      );
                    }
                }",
        r"
                  export class Blah {
                    render() {
                      return (
                        <div>
                          <h1>Hello title</h1>
                          <script src='https://polyfill.io/v3/polyfill.min.js?features=Array.prototype.copyWithin'></script>
                        </div>
                      );
                    }
                }",
        r"import NextScript from 'next/script';

                  export function MyApp({ Component, pageProps }) {
                      return (
                        <div>
                          <Component {...pageProps} />
                          <NextScript src='https://polyfill.io/v3/polyfill.min.js?features=Array.prototype.copyWithin' />
                        </div>
                      );
                }",
        r"import {Head} from 'next/document';

                    export class ES2019Features extends Head {
                      render() {
                        return (
                          <div>
                            <h1>Hello title</h1>
                            <script src='https://polyfill.io/v3/polyfill.min.js?features=Object.fromEntries'></script>
                          </div>
                        );
                      }
                  }",
    ];

    Tester::new(NoUnwantedPolyfillio::NAME, NoUnwantedPolyfillio::PLUGIN, pass, fail)
        .test_and_snapshot();
}
