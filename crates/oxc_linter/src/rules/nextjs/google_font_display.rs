use oxc_ast::{ast::JSXElementName, AstKind};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{self, Error},
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{get_string_literal_prop_value, has_jsx_prop_lowercase},
    AstNode,
};

#[derive(Debug, Error, Diagnostic)]
enum GoogleFontDisplayDiagnostic {
    #[error("eslint-plugin-next(google-font-display): A font-display parameter is missing (adding `&display=optional` is recommended).")]
    #[diagnostic(
        severity(warning),
        help("See https://nextjs.org/docs/messages/google-font-display")
    )]
    FontDisplayParameterMissing(#[label] Span),

    #[error(
        "eslint-plugin-next(google-font-display): `{1}` is not a recommended font-display value."
    )]
    #[diagnostic(
        severity(warning),
        help("See https://nextjs.org/docs/messages/google-font-display")
    )]
    NotRecommendedFontDisplayValue(#[label] Span, String),
}

#[derive(Debug, Default, Clone)]
pub struct GoogleFontDisplay;

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
    GoogleFontDisplay,
    correctness
);

impl Rule for GoogleFontDisplay {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(jsx_opening_element) = node.kind() else { return };

        let JSXElementName::Identifier(jsx_opening_element_name) = &jsx_opening_element.name else {
            return;
        };

        if jsx_opening_element_name.name.as_str() != "link" {
            return;
        }

        let Some(href_prop) = has_jsx_prop_lowercase(jsx_opening_element, "href") else {
            return;
        };

        let Some(href_prop_value) = get_string_literal_prop_value(href_prop) else { return };

        if href_prop_value.starts_with("https://fonts.googleapis.com/css") {
            let Ok(url) = url::Url::parse(href_prop_value) else { return };

            let Some((_, display_value)) = url.query_pairs().find(|(key, _)| key == "display")
            else {
                ctx.diagnostic(GoogleFontDisplayDiagnostic::FontDisplayParameterMissing(
                    jsx_opening_element_name.span,
                ));
                return;
            };

            if matches!(&*display_value, "auto" | "block" | "fallback") {
                ctx.diagnostic(GoogleFontDisplayDiagnostic::NotRecommendedFontDisplayValue(
                    href_prop.span(),
                    display_value.to_string(),
                ));
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

    Tester::new_without_config(GoogleFontDisplay::NAME, pass, fail).test_and_snapshot();
}
