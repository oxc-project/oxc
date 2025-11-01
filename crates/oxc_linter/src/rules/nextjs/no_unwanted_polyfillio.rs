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
    context::{ContextHost, LintContext},
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

fn polyfill_io_security_warning(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "Using polyfill.io is a security risk due to a supply chain attack in 2024."
    )
    .with_help("Replace with a safe alternative like https://cdnjs.cloudflare.com/polyfill/ or use modern browser features directly. See: https://blog.cloudflare.com/polyfill-io-now-available-on-cdnjs-reduce-your-supply-chain-risk")
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoUnwantedPolyfillio;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prevent use of unsafe polyfill.io domains and duplicate polyfills.
    ///
    /// ### Why is this bad?
    ///
    /// **Security Risk:**
    /// The domains `cdn.polyfill.io` and `polyfill.io` were compromised in a supply chain attack in 2024,
    /// where the domain was acquired by a malicious actor and began injecting harmful code into websites.
    /// Over 380,000+ websites were affected. These domains should not be used under any circumstances.
    ///
    /// **Performance Issue:**
    /// For safe alternatives like `cdnjs.cloudflare.com/polyfill/`, including polyfills already shipped
    /// with Next.js unnecessarily increases page weight which can affect loading performance.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// // Security risk - compromised domain
    /// <script src='https://cdn.polyfill.io/v2/polyfill.min.js'></script>
    /// <script src='https://polyfill.io/v3/polyfill.min.js'></script>
    ///
    /// // Duplicate polyfills
    /// <script src='https://cdnjs.cloudflare.com/polyfill/v3/polyfill.min.js?features=Array.prototype.copyWithin'></script>
    /// <script src='https://cdnjs.cloudflare.com/polyfill/v3/polyfill.min.js?features=WeakSet%2CPromise'></script>
    /// ```
    NoUnwantedPolyfillio,
    nextjs,
    correctness
);

impl Rule for NoUnwantedPolyfillio {
    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_jsx()
    }

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

        let src_str = src_value.value.as_str();

        // Check for unsafe polyfill.io domains first
        // These domains were compromised in a supply chain attack in 2024
        if src_str.starts_with("https://cdn.polyfill.io/v2/")
            || src_str.starts_with("https://polyfill.io/v3/")
        {
            ctx.diagnostic(polyfill_io_security_warning(src.span));
            return;
        }

        // Check other domains for duplicate polyfills
        // https://community.fastly.com/t/new-options-for-polyfill-io-users/2540
        if src_str.starts_with("https://polyfill-fastly.net/")
            || src_str.starts_with("https://polyfill-fastly.io/")
            // https://blog.cloudflare.com/polyfill-io-now-available-on-cdnjs-reduce-your-supply-chain-risk
            || src_str.starts_with("https://cdnjs.cloudflare.com/polyfill/")
        {
            let Some(features_value) = find_url_query_value(src_str, "features") else {
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
                        <script src='https://cdnjs.cloudflare.com/polyfill/v3/polyfill.min.js?features=AbortController'></script>
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
                        <script src='https://cdnjs.cloudflare.com/polyfill/v3/polyfill.min.js?features=IntersectionObserver'></script>
                    </div>
                );
            }
        }",
        r"import Script from 'next/script';
        export function MyApp({ Component, pageProps }) {
            return (
                <div>
                    <Component {...pageProps} />
                    <Script src='https://cdnjs.cloudflare.com/polyfill/v3/polyfill.min.js?features=IntersectionObserver' />
                </div>
            );
        }",
    ];

    let fail = vec![
        // Security warnings for unsafe domains
        r"export class Blah {
            render() {
                return (
                    <div>
                        <h1>Hello title</h1>
                        <script src='https://polyfill.io/v3/polyfill.min.js'></script>
                    </div>
                );
            }
        }",
        r"import Script from 'next/script';
        export function MyApp({ Component, pageProps }) {
            return (
                <div>
                    <Component {...pageProps} />
                    <Script src='https://cdn.polyfill.io/v2/polyfill.min.js?features=Promise' />
                </div>
            );
        }",
        // Duplicate polyfill warnings for safe alternatives
        r"import Script from 'next/script';
        export function MyApp({ Component, pageProps }) {
            return (
                <div>
                <Component {...pageProps} />
                <Script src='https://polyfill-fastly.io/v3/polyfill.min.js?features=Array.prototype.copyWithin' />
                </div>
            );
        }",
        r"export function MyApp({ Component, pageProps }) {
            return (
                <div>
                    <Component {...pageProps} />
                    <script src='https://cdnjs.cloudflare.com/polyfill/v3/polyfill.min.js?features=Promise%2CObject.fromEntries' />
                </div>
            );
        }",
    ];

    Tester::new(NoUnwantedPolyfillio::NAME, NoUnwantedPolyfillio::PLUGIN, pass, fail)
        .test_and_snapshot();
}
