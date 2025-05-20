use oxc_ast::{AstKind, ast::JSXElementName};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    context::LintContext,
    rule::Rule,
    utils::{get_string_literal_prop_value, has_jsx_prop_ignore_case},
};

fn google_font_preconnect_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(r#"`rel="preconnect"` is missing from Google Font."#)
        .with_help("See: https://nextjs.org/docs/messages/google-font-preconnect")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct GoogleFontPreconnect;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces the presence of `rel="preconnect"` when using Google Fonts via `<link>` tags.
    ///
    /// ### Why is this bad?
    ///
    /// When using Google Fonts, it's recommended to include a preconnect resource hint to establish early connections to the required origin.
    /// Without preconnect, the browser needs to perform DNS lookups, TCP handshakes, and TLS negotiations before it can download the font files,
    /// which can delay font loading and impact performance.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// <link href="https://fonts.gstatic.com" />
    /// <link rel="preload" href="https://fonts.gstatic.com" />
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// <link rel="preconnect" href="https://fonts.gstatic.com" />
    /// ```
    GoogleFontPreconnect,
    nextjs,
    correctness
);

impl Rule for GoogleFontPreconnect {
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

        let preconnect_missing =
            has_jsx_prop_ignore_case(jsx_opening_element, "rel").is_none_or(|rel_prop| {
                let rel_prop_value = get_string_literal_prop_value(rel_prop);
                rel_prop_value != Some("preconnect")
            });

        if href_prop_value.starts_with("https://fonts.gstatic.com") && preconnect_missing {
            ctx.diagnostic(google_font_preconnect_diagnostic(jsx_opening_element_name.span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"export const Test = () => (
			        <div>
			          <link rel="preconnect" href="https://fonts.gstatic.com"/>
			          <link
			            href={process.env.NEXT_PUBLIC_CANONICAL_URL}
			            rel="canonical"
			          />
			          <link
			            href={new URL("../public/favicon.ico", import.meta.url).toString()}
			            rel="icon"
			          />
			        </div>
			      )
			    "#,
    ];

    let fail = vec![
        r#"
			      export const Test = () => (
			        <div>
			          <link href="https://fonts.gstatic.com"/>
			        </div>
			      )
			    "#,
        r#"
			      export const Test = () => (
			        <div>
			          <link rel="preload" href="https://fonts.gstatic.com"/>
			        </div>
			      )
			    "#,
    ];

    Tester::new(GoogleFontPreconnect::NAME, GoogleFontPreconnect::PLUGIN, pass, fail)
        .with_nextjs_plugin(true)
        .test_and_snapshot();
}
