use oxc_ast::{ast::JSXElementName, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{find_url_query_value, get_string_literal_prop_value, has_jsx_prop_ignore_case},
    AstNode,
};

fn font_display_parameter_missing(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "A font-display parameter is missing (adding `&display=optional` is recommended).",
    )
    .with_help("See https://nextjs.org/docs/messages/google-font-display")
    .with_label(span)
}

fn not_recommended_font_display_value(span: Span, font_display_value: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("`{font_display_value}` is not a recommended font-display value."))
        .with_help("See https://nextjs.org/docs/messages/google-font-display")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct GoogleFontDisplay;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce font-display behavior with Google Fonts.
    ///
    /// ### Why is this bad?
    ///
    /// Specifying display=optional minimizes the risk of invisible text or
    /// layout shift. If swapping to the custom font after it has loaded is
    /// important to you, then use `display=swap`` instead.
    ///
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule:
    ///
    /// ```jsx
    /// import Head from "next/head";
    ///
    /// export default Test = () => {
    ///     return (
    ///         <Head>
    ///             <link
    ///                 href="https://fonts.googleapis.com/css2?family=Krona+One"
    ///                 rel="stylesheet"
    ///             />
    ///         </Head>
    ///     );
    /// };
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    ///
    /// ```jsx
    /// import Head from "next/head";
    ///
    /// export default Test = () => {
    ///     return (
    ///         <Head>
    ///             <link
    ///                 href="https://fonts.googleapis.com/css2?family=Krona+One&display=optional"
    ///                 rel="stylesheet"
    ///             />
    ///         </Head>
    ///     );
    /// };
    /// ```
    ///
    GoogleFontDisplay,
    nextjs,
    correctness
);

impl Rule for GoogleFontDisplay {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(jsx_opening_element) = node.kind() else {
            return;
        };

        let JSXElementName::Identifier(jsx_opening_element_name) = &jsx_opening_element.name else {
            return;
        };

        if jsx_opening_element_name.name.as_str() != "link" {
            return;
        }

        let Some(href_prop) = has_jsx_prop_ignore_case(jsx_opening_element, "href") else {
            return;
        };

        let Some(href_prop_value) = get_string_literal_prop_value(href_prop) else {
            return;
        };

        if href_prop_value.starts_with("https://fonts.googleapis.com/css") {
            let Some(display_value) = find_url_query_value(href_prop_value, "display") else {
                ctx.diagnostic(font_display_parameter_missing(jsx_opening_element_name.span));
                return;
            };

            if matches!(display_value, "auto" | "block" | "fallback") {
                ctx.diagnostic(not_recommended_font_display_value(href_prop.span(), display_value));
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"import Head from "next/head";

			     export default Test = () => {
			      return (
			        <Head>
			          <link href={test} rel="test" />
			          <link
			            href={process.env.NEXT_PUBLIC_CANONICAL_URL}
			            rel="canonical"
			          />
			          <link
			            href={new URL("../public/favicon.ico", import.meta.url).toString()}
			            rel="icon"
			          />
			          <link
			            href="https://fonts.googleapis.com/css2?family=Krona+One&display=optional"
			            rel="stylesheet"
			          />
			        </Head>
			      );
			     };
			    "#,
        r#"import Document, { Html, Head } from "next/document";

			     class MyDocument extends Document {
			      render() {
			        return (
			          <Html>
			            <Head>
			              <link
			                href="https://fonts.googleapis.com/css?family=Krona+One&display=swap"
			                rel="stylesheet"
			              />
			            </Head>
			          </Html>
			        );
			      }
			     }

			     export default MyDocument;
			    "#,
        r#"import Document, { Html, Head } from "next/document";

			     class MyDocument extends Document {
			      render() {
			        return (
			          <Html>
			            <Head>
			              <link
			                href="https://fonts.googleapis.com/css?family=Krona+One&display=swap"
			                rel="stylesheet"
			                crossOrigin=""
			              />
			            </Head>
			          </Html>
			        );
			      }
			     }

			     export default MyDocument;
			    "#,
    ];

    let fail = vec![
        r#"import Head from "next/head";

				export default Test = () => {
				return (
					<Head>
					<link
						href="https://fonts.googleapis.com/css2?family=Krona+One"
						rel="stylesheet"
					/>
					</Head>
				);
				};
			     "#,
        r#"import Head from "next/head";

			      export default Test = () => {
			       return (
			         <Head>
			           <link
			             href="https://fonts.googleapis.com/css2?family=Krona+One&display=block"
			             rel="stylesheet"
			           />
			         </Head>
			       );
			      };
			     "#,
        r#"import Head from "next/head";

			      export default Test = () => {
			       return (
			         <Head>
			           <link
			             href="https://fonts.googleapis.com/css2?family=Krona+One&display=auto"
			             rel="stylesheet"
			           />
			         </Head>
			       );
			      };
			     "#,
        r#"import Head from "next/head";

			      export default Test = () => {
			       return (
			         <Head>
			           <link
			             href="https://fonts.googleapis.com/css2?display=fallback&family=Krona+One"
			             rel="stylesheet"
			           />
			         </Head>
			       );
			      };
			     "#,
    ];

    Tester::new(GoogleFontDisplay::NAME, GoogleFontDisplay::PLUGIN, pass, fail)
        .with_nextjs_plugin(true)
        .test_and_snapshot();
}
