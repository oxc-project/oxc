use std::path::PathBuf;

use oxc_ast::{
    ast::{JSXAttributeItem, JSXAttributeName, JSXAttributeValue, JSXElementName},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_html_link_for_pages_config(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("")
        .with_help("See https://nextjs.org/docs/messages/no-html-link-for-pages")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoHtmlLinkForPages(Box<NoHtmlLinkForPagesConfig>);

#[derive(Debug, Default, Clone)]
pub struct NoHtmlLinkForPagesConfig {
    pages_dirs: Option<Vec<String>>,
}

impl std::ops::Deref for NoHtmlLinkForPages {
    type Target = NoHtmlLinkForPagesConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

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
    NoHtmlLinkForPages,
    nursery,
);

impl Rule for NoHtmlLinkForPages {
    fn from_configuration(value: serde_json::Value) -> Self {
        let pages_dirs = value.as_array().map(|dirs| {
            dirs.iter()
                .filter_map(|item| item.as_str().map(ToString::to_string))
                .collect::<Vec<_>>()
        });
        Self(Box::new(NoHtmlLinkForPagesConfig { pages_dirs }))
    }

    fn run_once(&self, ctx: &LintContext) {
        let pages_dirs = self.pages_dirs.as_ref().map_or_else(
            || {
                ctx.settings()
                    .next
                    .get_root_dirs()
                    .iter()
                    .flat_map(|item| {
                        vec![
                            PathBuf::from(item).join("pages"),
                            PathBuf::from(item).join("src/pages"),
                        ]
                    })
                    .collect::<Vec<_>>()
            },
            |dirs| dirs.iter().map(PathBuf::from).collect::<Vec<_>>(),
        );

        let found_pages_dirs = pages_dirs.iter().filter(|dir| dir.exists()).collect::<Vec<_>>();

        for node in ctx.nodes().iter() {
            let kind = node.kind();
            let AstKind::JSXOpeningElement(element) = kind else { continue };
            if matches!(&element.name, JSXElementName::Identifier(ident) if ident.name != "a") {
                continue;
            }

            let should_ignore = element.attributes.iter().any(|attr| {
                let JSXAttributeItem::Attribute(attr) = attr else {
                    return false;
                };

                let JSXAttributeName::Identifier(ident) = &attr.name else {
                    return false;
                };

                match ident.name.as_str() {
                    "target" => {
                        attr.value.as_ref().map_or(false, |value| {
                            matches!(&value, JSXAttributeValue::StringLiteral(value) if value.value == "_blank")
                        })
                    },
                    "download" => true,
                    "href" => {
                        attr.value.as_ref().map_or(false, |value| {
                            if let JSXAttributeValue::StringLiteral(literal) = value {
                                // Outgoing links are ignored
                                literal.value.starts_with("http://") || literal.value.starts_with("https://") || literal.value.starts_with("//")
                            } else {
                                true
                            }
                        })
                    },
                    _ => false
                }
            });

            if should_ignore {
                continue;
            }

            let Some(href) = element.attributes.iter().find_map(|item| match item {
                JSXAttributeItem::Attribute(attribute) => {
                    if attribute.is_identifier("href") {
                        attribute.value.as_ref().and_then(|value| {
                            if let JSXAttributeValue::StringLiteral(literal) = value {
                                Some(literal)
                            } else {
                                None
                            }
                        })
                    } else {
                        None
                    }
                }
                JSXAttributeItem::SpreadAttribute(_) => None,
            }) else {
                continue;
            };
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    use serde_json::json;
    use std::env;
    use std::path::PathBuf;

    let cwd = env::current_dir().unwrap().join("fixtures/next");
    let with_custom_pages_directory = cwd.join("with-custom-pages-dir");
    let custom_page_dir = with_custom_pages_directory.join("custom-pages");
    let filename = Some(PathBuf::from("foo.jsx"));

    let valid_code = r"
import Link from 'next/link';

export class Blah extends Head {
  render() {
    return (
      <div>
        <Link href='/'>
          <a>Homepage</a>
        </Link>
        <h1>Hello title</h1>
      </div>
    );
  }
}
";

    let invalid_static_code = r"
import Link from 'next/link';

export class Blah extends Head {
  render() {
    return (
      <div>
        <a href='/'>Homepage</a>
        <h1>Hello title</h1>
      </div>
    );
  }
}
";

    let pass = vec![(
        valid_code,
        Some(json!([custom_page_dir.to_string_lossy().to_string()])),
        None,
        filename.clone(),
    )];

    let fail = vec![(invalid_static_code, None, None, filename.clone())];

    Tester::new(NoHtmlLinkForPages::NAME, pass, fail)
        .with_nextjs_plugin(true)
        .change_rule_path("with-custom-pages-dir")
        .test_and_snapshot();
}
