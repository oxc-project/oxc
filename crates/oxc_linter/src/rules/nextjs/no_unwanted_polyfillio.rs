use cow_utils::CowUtils;
use oxc_ast::{
    ast::{JSXAttributeItem, JSXAttributeName, JSXAttributeValue},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;
use oxc_span::Span;
use phf::{phf_set, Set};

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{find_url_query_value, get_next_script_import_local_name},
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
    /// Prevent duplicate polyfills from Polyfill.io.
    ///
    /// ### Why is this bad?
    /// You are using polyfills from Polyfill.io and including polyfills already shipped with Next.js. This unnecessarily increases page weight which can affect loading performance.
    ///
    /// ### Example
    /// ```javascript
    /// <script src='https://polyfill.io/v3/polyfill.min.js?features=Array.prototype.copyWithin'></script>
    ///
    /// <script src='https://polyfill.io/v3/polyfill.min.js?features=WeakSet%2CPromise%2CPromise.prototype.finally%2Ces2015%2Ces5%2Ces6'></script>
    /// ```
    NoUnwantedPolyfillio,
    nextjs,
    correctness
);

// Keep in sync with next.js polyfills file : https://github.com/vercel/next.js/blob/v15.0.2/packages/next-polyfill-nomodule/src/index.js
const NEXT_POLYFILLED_FEATURES: Set<&'static str> = phf_set! {
    "Array.prototype.@@iterator",
    "Array.prototype.at",
    "Array.prototype.copyWithin",
    "Array.prototype.fill",
    "Array.prototype.find",
    "Array.prototype.findIndex",
    "Array.prototype.flatMap",
    "Array.prototype.flat",
    "Array.from",
    "Array.prototype.includes",
    "Array.of",
    "Function.prototype.name",
    "fetch",
    "Map",
    "Number.EPSILON",
    "Number.Epsilon",
    "Number.isFinite",
    "Number.isNaN",
    "Number.isInteger",
    "Number.isSafeInteger",
    "Number.MAX_SAFE_INTEGER",
    "Number.MIN_SAFE_INTEGER",
    "Number.parseFloat",
    "Number.parseInt",
    "Object.assign",
    "Object.entries",
    "Object.fromEntries",
    "Object.getOwnPropertyDescriptor",
    "Object.getOwnPropertyDescriptors",
    "Object.is",
    "Object.keys",
    "Object.values",
    "Reflect",
    "Set",
    "Symbol",
    "Symbol.asyncIterator",
    "String.prototype.codePointAt",
    "String.prototype.endsWith",
    "String.fromCodePoint",
    "String.prototype.includes",
    "String.prototype.@@iterator",
    "String.prototype.padEnd",
    "String.prototype.padStart",
    "String.prototype.repeat",
    "String.raw",
    "String.prototype.startsWith",
    "String.prototype.trimEnd",
    "String.prototype.trimStart",
    "URL",
    "URL.prototype.toJSON",
    "URLSearchParams",
    "WeakMap",
    "WeakSet",
    "Promise",
    "Promise.prototype.finally",
    "es2015", // Should be covered by babel-preset-env instead.
    "es2016", // contains polyfilled 'Array.prototype.includes', 'String.prototype.padEnd' and 'String.prototype.padStart'
    "es2017", // contains polyfilled 'Object.entries', 'Object.getOwnPropertyDescriptors', 'Object.values', 'String.prototype.padEnd' and 'String.prototype.padStart'
    "es2018", // contains polyfilled 'Promise.prototype.finally' and ''Symbol.asyncIterator'
    "es2019", // Contains polyfilled 'Object.fromEntries' and polyfilled 'Array.prototype.flat', 'Array.prototype.flatMap', 'String.prototype.trimEnd' and 'String.prototype.trimStart'
    "es5", // Should be covered by babel-preset-env instead.
    "es6", // Should be covered by babel-preset-env instead.
    "es7", // contains polyfilled 'Array.prototype.includes', 'String.prototype.padEnd' and 'String.prototype.padStart'
};

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

        if jsx_el.attributes.len() == 0 {
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
