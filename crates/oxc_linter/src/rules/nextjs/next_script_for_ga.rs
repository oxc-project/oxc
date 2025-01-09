use oxc_ast::{
    ast::{
        Expression, JSXAttributeItem, JSXAttributeValue, JSXElementName, JSXExpression,
        JSXOpeningElement, ObjectProperty, ObjectPropertyKind, PropertyKey,
    },
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{get_string_literal_prop_value, has_jsx_prop_ignore_case},
    AstNode,
};

fn next_script_for_ga_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "Prefer `next/script` component when using the inline script for Google Analytics.",
    )
    .with_help("See https://nextjs.org/docs/messages/next-script-for-ga")
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NextScriptForGa;

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
    NextScriptForGa,
    nextjs,
    correctness
);

impl Rule for NextScriptForGa {
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

        // Check if the Alternative async tag is being used to add GA.
        // https://developers.google.com/analytics/devguides/collection/analyticsjs#alternative_async_tag
        // https://developers.google.com/analytics/devguides/collection/gtagjs
        if let Some(src_prop) = has_jsx_prop_ignore_case(jsx_opening_element, "src") {
            if let Some(src_prop_value) = get_string_literal_prop_value(src_prop) {
                if SUPPORTED_SRCS.iter().any(|s| src_prop_value.contains(s)) {
                    ctx.diagnostic(next_script_for_ga_diagnostic(jsx_opening_element_name.span));
                    return;
                }
            }
        }

        // Check if inline script is being used to add GA.
        // https://developers.google.com/analytics/devguides/collection/analyticsjs#the_google_analytics_tag
        // https://developers.google.com/tag-manager/quickstart
        if let Some(danger_value) = get_dangerously_set_inner_html_prop_value(jsx_opening_element) {
            let Expression::TemplateLiteral(template_literal) = &danger_value.value else {
                return;
            };
            let template_literal = template_literal.quasis[0].value.raw.as_str();
            if SUPPORTED_HTML_CONTENT_URLS.iter().any(|s| template_literal.contains(s)) {
                ctx.diagnostic(next_script_for_ga_diagnostic(jsx_opening_element_name.span));
            }
        }
    }
}

const SUPPORTED_SRCS: [&str; 2] =
    ["www.google-analytics.com/analytics.js", "www.googletagmanager.com/gtag/js"];

const SUPPORTED_HTML_CONTENT_URLS: [&str; 2] =
    ["www.google-analytics.com/analytics.js", "www.googletagmanager.com/gtm.js"];

fn get_dangerously_set_inner_html_prop_value<'a>(
    jsx_opening_element: &'a JSXOpeningElement<'a>,
) -> Option<&'a ObjectProperty<'a>> {
    let Some(JSXAttributeItem::Attribute(dangerously_set_inner_html_prop)) =
        has_jsx_prop_ignore_case(jsx_opening_element, "dangerouslysetinnerhtml")
    else {
        return None;
    };
    let Some(JSXAttributeValue::ExpressionContainer(object_expr)) =
        &dangerously_set_inner_html_prop.value
    else {
        return None;
    };
    let JSXExpression::ObjectExpression(object_expr) = &object_expr.expression else {
        return None;
    };

    if let Some(html_prop) = object_expr.properties.iter().find_map(|prop| {
        if let ObjectPropertyKind::ObjectProperty(html_prop) = prop {
            if let PropertyKey::StaticIdentifier(html_prop_ident) = &html_prop.key {
                if html_prop_ident.name == "__html" {
                    Some(html_prop)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }) {
        return Some(html_prop);
    }

    None
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"import Script from 'next/script'
			
			      export class Blah extends Head {
			        render() {
			          return (
			            <div>
			              <h1>Hello title</h1>
			              <Script
			                src="https://www.googletagmanager.com/gtag/js?id=GA_MEASUREMENT_ID"
			                strategy="lazyOnload"
			              />
			              <Script id="google-analytics">
			                {`
			                  window.dataLayer = window.dataLayer || [];
			                  function gtag(){window.dataLayer.push(arguments);}
			                  gtag('js', new Date());
			
			                  gtag('config', 'GA_MEASUREMENT_ID');
			                `}
			              </Script>
			            </div>
			          );
			        }
			    }"#,
        r#"import Script from 'next/script'
			
			      export class Blah extends Head {
			        render() {
			          return (
			            <div>
			              <h1>Hello title</h1>
			              <Script id="google-analytics">
			                {`(function(i,s,o,g,r,a,m){i['GoogleAnalyticsObject']=r;i[r]=i[r]||function(){
			                    (i[r].q=i[r].q||[]).push(arguments)},i[r].l=1*new Date();a=s.createElement(o),
			                    m=s.getElementsByTagName(o)[0];a.async=1;a.src=g;m.parentNode.insertBefore(a,m)
			                    })(window,document,'script','https://www.google-analytics.com/analytics.js','ga');
			
			                    ga('create', 'UA-XXXXX-Y', 'auto');
			                    ga('send', 'pageview');
			                })`}
			              </Script>
			            </div>
			          );
			        }
			    }"#,
        r#"import Script from 'next/script'
			
			        export class Blah extends Head {
			        render() {
			            return (
			            <div>
			                <h1>Hello title</h1>
			                <Script id="google-analytics">
			                    {`window.ga=window.ga||function(){(ga.q=ga.q||[]).push(arguments)};ga.l=+new Date;
			                    ga('create', 'UA-XXXXX-Y', 'auto');
			                    ga('send', 'pageview');
			                    })`}
			                </Script>
			            </div>
			            );
			        }
			    }"#,
        r"export class Blah extends Head {
			          render() {
			            return (
			              <div>
			                <h1>Hello title</h1>
			                <script dangerouslySetInnerHTML={{}} />
			              </div>
			            );
			          }
			      }",
    ];

    let fail = vec![
        r"
			        export class Blah extends Head {
			          render() {
			            return (
			              <div>
			                <h1>Hello title</h1>
			                <script async src='https://www.googletagmanager.com/gtag/js?id=${GA_TRACKING_ID}' />
			                <script
			                  dangerouslySetInnerHTML={{
			                    __html: `
			                      window.dataLayer = window.dataLayer || [];
			                      function gtag(){dataLayer.push(arguments);}
			                      gtag('js', new Date());
			                      gtag('config', '${GA_TRACKING_ID}', {
			                        page_path: window.location.pathname,
			                      });
			                  `,
			                }}/>
			              </div>
			            );
			          }
			      }",
        r"
			        export class Blah extends Head {
			          render() {
			            return (
			              <div>
			                <h1>Hello title</h1> qqq
			                {/* Google Tag Manager - Global base code */}
			                <script
			                dangerouslySetInnerHTML={{
			                  __html: `
			                    (function(w,d,s,l,i){w[l]=w[l]||[];w[l].push({'gtm.start':
			                    new Date().getTime(),event:'gtm.js'});var f=d.getElementsByTagName(s)[0],
			                    j=d.createElement(s),dl=l!='dataLayer'?'&l='+l:'';j.async=true;j.src=
			                    'https://www.googletagmanager.com/gtm.js?id='+i+dl;f.parentNode.insertBefore(j,f);
			                    })(window,document,'script','dataLayer', '${GTM_ID}');
			                  `,
			                }}/>
			              </div>
			            );
			          }
			      }",
        r"
			        export class Blah extends Head {
			          render() {
			            return (
			              <div>
			                <h1>Hello title</h1>
			                <script dangerouslySetInnerHTML={{
			                    __html: `
			                      (function(i,s,o,g,r,a,m){i['GoogleAnalyticsObject']=r;i[r]=i[r]||function(){
			                        (i[r].q=i[r].q||[]).push(arguments)},i[r].l=1*new Date();a=s.createElement(o),
			                        m=s.getElementsByTagName(o)[0];a.async=1;a.src=g;m.parentNode.insertBefore(a,m)
			                        })(window,document,'script','https://www.google-analytics.com/analytics.js','ga');
			
			                        ga('create', 'UA-XXXXX-Y', 'auto');
			                        ga('send', 'pageview');
			                    `,
			                  }}/>
			              </div>
			            );
			          }
			      }",
        r"
			        export class Blah extends Head {
			          render() {
			            return (
			              <div>
			                <h1>Hello title</h1>
			                <script dangerouslySetInnerHTML={{
			                    __html: `
			                        window.ga=window.ga||function(){(ga.q=ga.q||[]).push(arguments)};ga.l=+new Date;
			                        ga('create', 'UA-XXXXX-Y', 'auto');
			                        ga('send', 'pageview');
			                    `,
			                  }}/>
			                <script async src='https://www.google-analytics.com/analytics.js'></script>
			              </div>
			            );
			          }
			      }",
        r"
			        export class Blah extends Head {
			          createGoogleAnalyticsMarkup() {
			            return {
			              __html: `
			                window.dataLayer = window.dataLayer || [];
			                function gtag(){dataLayer.push(arguments);}
			                gtag('js', new Date());
			                gtag('config', 'UA-148481588-2');`,
			            };
			          }
			
			          render() {
			            return (
			              <div>
			                <h1>Hello title</h1>
			                <script dangerouslySetInnerHTML={this.createGoogleAnalyticsMarkup()} />
			                <script async src='https://www.google-analytics.com/analytics.js'></script>
			              </div>
			            );
			          }
			      }",
    ];

    Tester::new(NextScriptForGa::NAME, NextScriptForGa::PLUGIN, pass, fail)
        .with_nextjs_plugin(true)
        .test_and_snapshot();
}
