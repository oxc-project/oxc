use oxc_ast::{
    ast::{JSXAttributeItem, JSXAttributeName, JSXAttributeValue},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{get_next_script_import_local_name, is_document_page, is_in_app_dir},
    AstNode,
};

fn no_before_interactive_script_outside_document_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("next/script's `beforeInteractive` strategy should not be used outside of `pages/_document.js`")
        .with_help("See https://nextjs.org/docs/messages/no-before-interactive-script-outside-document")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoBeforeInteractiveScriptOutsideDocument;

declare_oxc_lint!(
    /// ### What it does
    /// Prevent usage of `next/script`'s `beforeInteractive` strategy outside of `pages/_document.js`.
    ///
    /// ### Why is this bad?
    ///
    ///
    /// ### Example
    /// ```javascript
    /// ```
    NoBeforeInteractiveScriptOutsideDocument,
    nextjs,
    correctness
);

impl Rule for NoBeforeInteractiveScriptOutsideDocument {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::JSXOpeningElement(jsx_el) = node.kind() {
            let Some(file_path) = ctx.file_path().to_str() else {
                return;
            };
            if is_in_app_dir(file_path) {
                return;
            }
            let Some(tag_name) = jsx_el.name.get_identifier_name() else {
                return;
            };
            if jsx_el.attributes.len() == 0 {
                return;
            }

            let Some(JSXAttributeItem::Attribute(strategy)) =
                jsx_el.attributes.iter().find(|attr| {
                    matches!(
                        attr,
                        JSXAttributeItem::Attribute(jsx_attr)
                            if matches!(
                                &jsx_attr.name,
                                JSXAttributeName::Identifier(id) if id.name.as_str() == "strategy"
                            )
                    )
                })
            else {
                return;
            };

            if let Some(JSXAttributeValue::StringLiteral(strategy_value)) = &strategy.value {
                if strategy_value.value.as_str() == "beforeInteractive" {
                    if is_document_page(file_path) {
                        return;
                    }
                    let next_script_import_local_name = get_next_script_import_local_name(ctx);
                    if !matches!(next_script_import_local_name, Some(import) if tag_name == import)
                    {
                        return;
                    }
                    ctx.diagnostic(no_before_interactive_script_outside_document_diagnostic(
                        strategy.span,
                    ));
                }
            }
        }
    }
}

#[test]
fn test() {
    use std::path::PathBuf;

    use crate::tester::Tester;

    let pass = vec![
        (
            r#"import Document, { Html, Main, NextScript } from 'next/document'
                  import Script from 'next/script'

                  class MyDocument extends Document {
                    render() {
                      return (
                        <Html>
                          <Head>
                            <meta charSet="utf-8" />
                          </Head>
                          <body>
                            <Main />
                            <NextScript />
                            <Script
                              id="scriptBeforeInteractive"
                              src="https://cdnjs.cloudflare.com/ajax/libs/lodash.js/4.17.20/lodash.min.js?a=scriptBeforeInteractive"
                              strategy="beforeInteractive"
                            ></Script>
                          </body>
                        </Html>
                      )
                    }
                  }

                  export default MyDocument
			"#,
            None,
            None,
            Some(PathBuf::from("pages/_document.js")),
        ),
        (
            r#"import Document, { Html, Main, NextScript } from 'next/document'
                  import ScriptComponent from 'next/script'

                  class MyDocument extends Document {
                    render() {
                      return (
                        <Html>
                          <Head>
                            <meta charSet="utf-8" />
                          </Head>
                          <body>
                            <Main />
                            <NextScript />
                            <ScriptComponent
                              id="scriptBeforeInteractive"
                              src="https://cdnjs.cloudflare.com/ajax/libs/lodash.js/4.17.20/lodash.min.js?a=scriptBeforeInteractive"
                              strategy="beforeInteractive"
                            ></ScriptComponent>
                          </body>
                        </Html>
                      )
                    }
                  }

                  export default MyDocument
			"#,
            None,
            None,
            Some(PathBuf::from("pages/_document.tsx")),
        ),
        (
            r#"import Document, { Html, Main, NextScript } from 'next/document'
                  import ScriptComponent from 'next/script'

                  class MyDocument extends Document {
                    render() {
                      return (
                        <Html>
                          <Head>
                            <meta charSet="utf-8" />
                          </Head>
                          <body>
                            <Main />
                            <NextScript />
                            <ScriptComponent
                              id="scriptBeforeInteractive"
                              src="https://cdnjs.cloudflare.com/ajax/libs/lodash.js/4.17.20/lodash.min.js?a=scriptBeforeInteractive"
                            ></ScriptComponent>
                          </body>
                        </Html>
                      )
                    }
                  }

                  export default MyDocument
			"#,
            None,
            None,
            Some(PathBuf::from("pages/_document.tsx")),
        ),
        (
            r#"import Script from "next/script";

                  export default function Index() {
                    return (
                      <html lang="en">
                        <body className={inter.className}>{children}</body>
                        <Script
                          src="https://cdnjs.cloudflare.com/ajax/libs/lodash.js/4.17.20/lodash.min.js?a=scriptBeforeInteractive"
                          strategy='beforeInteractive'
                        />
                      </html>
                    );
                  }
			"#,
            None,
            None,
            Some(PathBuf::from("/Users/user_name/projects/project-name/app/layout.tsx")),
        ),
        (
            r#"import Script from "next/script";

                  export default function test() {
                    return (
                      <html lang="en">
                        <body className={inter.className}>{children}</body>
                        <Script
                          src="https://cdnjs.cloudflare.com/ajax/libs/lodash.js/4.17.20/lodash.min.js?a=scriptBeforeInteractive"
                          strategy='beforeInteractive'
                        />
                      </html>
                    );
                  }
			"#,
            None,
            None,
            Some(PathBuf::from("C:\\Users\\username\\projects\\project-name\\app\\layout.tsx")),
        ),
        (
            r#"import Script from "next/script";

                  export default function Index() {
                    return (
                      <html lang="en">
                        <body className={inter.className}>{children}</body>
                        <Script
                          src="https://cdnjs.cloudflare.com/ajax/libs/lodash.js/4.17.20/lodash.min.js?a=scriptBeforeInteractive"
                          strategy='beforeInteractive'
                        />
                      </html>
                    );
                  }
			"#,
            None,
            None,
            Some(PathBuf::from("/Users/user_name/projects/project-name/src/app/layout.tsx")),
        ),
        (
            r#"import Script from "next/script";

                  export default function test() {
                    return (
                      <html lang="en">
                        <body className={inter.className}>{children}</body>
                        <Script
                          src="https://cdnjs.cloudflare.com/ajax/libs/lodash.js/4.17.20/lodash.min.js?a=scriptBeforeInteractive"
                          strategy='beforeInteractive'
                        />
                      </html>
                    );
                  }
			"#,
            None,
            None,
            Some(PathBuf::from(
                "C:\\Users\\username\\projects\\project-name\\src\\app\\layout.tsx",
            )),
        ),
    ];

    let fail = vec![
        (
            r#"import Head from "next/head";
                  import Script from "next/script";

                  export default function Index() {
                    return (
                      <Script
                        id="scriptBeforeInteractive"
                        src="https://cdnjs.cloudflare.com/ajax/libs/lodash.js/4.17.20/lodash.min.js?a=scriptBeforeInteractive"
                        strategy="beforeInteractive"
                      ></Script>
                    );
                  }
			"#,
            None,
            None,
            Some(PathBuf::from("pages/index.js")),
        ),
        (
            r#" import Head from "next/head";
             import Script from "next/script";

             export default function Index() {
               return (
                 <Script
                   id="scriptBeforeInteractive"
                   src="https://cdnjs.cloudflare.com/ajax/libs/lodash.js/4.17.20/lodash.min.js?a=scriptBeforeInteractive"
                   strategy="beforeInteractive"
                 ></Script>
               );
             }
			"#,
            None,
            None,
            Some(PathBuf::from("components/outside-known-dirs.js")),
        ),
        (
            r#" import Script from "next/script";

                  export default function Index() {
                    return (
                      <html lang="en">
                        <body className={inter.className}>{children}</body>
                        <Script
                          src="https://cdnjs.cloudflare.com/ajax/libs/lodash.js/4.17.20/lodash.min.js?a=scriptBeforeInteractive"
                          strategy='beforeInteractive'
                        />
                      </html>
                    );
                  }
			"#,
            None,
            None,
            Some(PathBuf::from("/Users/user_name/projects/project-name/pages/layout.tsx")),
        ),
        (
            r#" import Script from "next/script";

                  export default function Index() {
                    return (
                      <html lang="en">
                        <body className={inter.className}>{children}</body>
                        <Script
                          src="https://cdnjs.cloudflare.com/ajax/libs/lodash.js/4.17.20/lodash.min.js?a=scriptBeforeInteractive"
                          strategy='beforeInteractive'
                        />
                      </html>
                    );
                  }
			"#,
            None,
            None,
            Some(PathBuf::from("C:\\Users\\username\\projects\\project-name\\pages\\layout.tsx")),
        ),
        (
            r#" import Script from "next/script";

                  export default function Index() {
                    return (
                      <html lang="en">
                        <body className={inter.className}>{children}</body>
                        <Script
                          src="https://cdnjs.cloudflare.com/ajax/libs/lodash.js/4.17.20/lodash.min.js?a=scriptBeforeInteractive"
                          strategy='beforeInteractive'
                        />
                      </html>
                    );
                  }
			"#,
            None,
            None,
            Some(PathBuf::from("/Users/user_name/projects/project-name/src/pages/layout.tsx")),
        ),
        (
            r#" import Script from "next/script";

                  export default function test() {
                    return (
                      <html lang="en">
                        <body className={inter.className}>{children}</body>
                        <Script
                          src="https://cdnjs.cloudflare.com/ajax/libs/lodash.js/4.17.20/lodash.min.js?a=scriptBeforeInteractive"
                          strategy='beforeInteractive'
                        />
                      </html>
                    );
                  }
			"#,
            None,
            None,
            Some(PathBuf::from(
                "C:\\Users\\username\\projects\\project-name\\src\\pages\\layout.tsx",
            )),
        ),
    ];

    Tester::new(
        NoBeforeInteractiveScriptOutsideDocument::NAME,
        NoBeforeInteractiveScriptOutsideDocument::PLUGIN,
        pass,
        fail,
    )
    .test_and_snapshot();
}
