use oxc_ast::{
    ast::{JSXAttributeItem, JSXAttributeName, JSXElementName},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, utils::get_string_literal_prop_value, AstNode};

fn no_css_tags_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not include stylesheets manually.")
        .with_help("See https://nextjs.org/docs/messages/no-css-tags")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoCssTags;

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
    NoCssTags,
    nextjs,
    correctness
);

impl Rule for NoCssTags {
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

        // check for `rel="stylesheet"` and `href` (href must not be a url

        let mut rel_attr = None;
        let mut href_attr = None;

        for attr_item in &jsx_opening_element.attributes {
            if let JSXAttributeItem::Attribute(attr) = attr_item {
                let JSXAttributeName::Identifier(name) = &attr.name else {
                    continue;
                };

                match name.name.as_str() {
                    "rel" => {
                        rel_attr = Some(attr_item);
                    }
                    "href" => {
                        href_attr = Some(attr_item);
                    }
                    _ => {}
                }
            }
        }

        let (Some(rel_attr), Some(href_attr)) = (rel_attr, href_attr) else {
            return;
        };

        let Some(rel_prop_value) = get_string_literal_prop_value(rel_attr) else {
            return;
        };
        let Some(href_prop_value) = get_string_literal_prop_value(href_attr) else {
            return;
        };

        if rel_prop_value == "stylesheet"
            && !(href_prop_value.starts_with("https://") || href_prop_value.starts_with("http://"))
        {
            ctx.diagnostic(no_css_tags_diagnostic(jsx_opening_element_name.span));
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
			            </div>
			          );
			        }
			    }",
        r#"import {Head} from 'next/document';
			      export class Blah extends Head {
			        render() {
			          return (
			            <div>
			              <h1>Hello title</h1>
			              <link href="https://fonts.googleapis.com/css?family=Open+Sans&display=swap" rel="stylesheet" />
			            </div>
			          );
			        }
			    }"#,
        r"import {Head} from 'next/document';
			      export class Blah extends Head {
			        render(props) {
			          return (
			            <div>
			              <h1>Hello title</h1>
			              <link {...props} />
			            </div>
			          );
			        }
			    }",
        r#"import {Head} from 'next/document';
			      export class Blah extends Head {
			        render(props) {
			          return (
			            <div>
			              <h1>Hello title</h1>
			              <link rel="stylesheet" {...props} />
			            </div>
			          );
			        }
			    }"#,
    ];

    let fail = vec![
        r#"
			      import {Head} from 'next/document';

			        export class Blah extends Head {
			          render() {
			            return (
			              <div>
			                <h1>Hello title</h1>
			                <link href="/_next/static/css/styles.css" rel="stylesheet" />
			              </div>
			            );
			          }
			      }"#,
        r#"
			      <div>
			        <link href="/_next/static/css/styles.css" rel="stylesheet" />
			      </div>"#,
    ];

    Tester::new(NoCssTags::NAME, NoCssTags::PLUGIN, pass, fail)
        .with_nextjs_plugin(true)
        .test_and_snapshot();
}
