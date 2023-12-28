use oxc_ast::{
    ast::{
        JSXAttributeItem, JSXAttributeValue, JSXElementName, JSXExpression, JSXExpressionContainer,
    },
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{get_element_type, get_prop_value, has_jsx_prop_lowercase},
    AstNode,
};

#[derive(Debug, Error, Diagnostic)]
enum HtmlHasLangDiagnostic {
    #[error("eslint-plugin-jsx-a11y(html-has-lang): Missing lang attribute.")]
    #[diagnostic(severity(warning), help("Add a lang attribute to the html element whose value represents the primary language of document."))]
    MissingLangProp(#[label] Span),

    #[error("eslint-plugin-jsx-a11y(html-has-lang): Missing value for lang attribute")]
    #[diagnostic(severity(warning), help("Must have meaningful value for `lang` prop."))]
    MissingLangValue(#[label] Span),
}

#[derive(Debug, Default, Clone)]
pub struct HtmlHasLang;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Ensures that every HTML document has a lang attribute
    ///
    /// ### Why is this bad?
    /// If the language of a webpage is not specified,
    /// the screen reader assumes the default language set by the user.
    /// Language settings become an issue for users who speak multiple languages
    /// and access website in more than one language.
    ///
    ///
    /// ### Example
    /// ```javascript
    /// // Bad
    /// <html />
    ///
    /// // Good
    /// <html lang="en" />
    /// ```
    HtmlHasLang,
    correctness
);

impl Rule for HtmlHasLang {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(jsx_el) = node.kind() else {
            return;
        };

        let Some(element_type) = get_element_type(ctx, jsx_el) else {
            return;
        };

        if element_type != "html" {
            return;
        }

        let JSXElementName::Identifier(identifier) = &jsx_el.name else {
            return;
        };

        has_jsx_prop_lowercase(jsx_el, "lang").map_or_else(
            || ctx.diagnostic(HtmlHasLangDiagnostic::MissingLangProp(identifier.span)),
            |lang_prop| {
                if !is_valid_lang_prop(lang_prop) {
                    ctx.diagnostic(HtmlHasLangDiagnostic::MissingLangValue(jsx_el.span));
                }
            },
        );
    }
}

fn is_valid_lang_prop(item: &JSXAttributeItem) -> bool {
    match get_prop_value(item) {
        Some(JSXAttributeValue::ExpressionContainer(JSXExpressionContainer {
            expression: JSXExpression::Expression(expr),
            ..
        })) => !expr.is_undefined(),
        Some(JSXAttributeValue::StringLiteral(str)) => !str.value.as_str().is_empty(),
        _ => true,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    fn settings() -> serde_json::Value {
        serde_json::json!({
            "jsx-a11y": {
                "components": {
                    "HTMLTop": "html",
                }
            }
        })
    }

    let pass = vec![
        (r"<div />;", None, None),
        (r#"<html lang="en" />"#, None, None),
        (r#"<html lang="en-US" />"#, None, None),
        (r"<html lang={foo} />;", None, None),
        (r"<html lang />;", None, None),
        (r"<HTML />;", None, None),
        ("<HTMLTop lang='en' />", None, Some(settings())),
    ];

    let fail = vec![
        (r"<html />;", None, None),
        (r"<html {...props} />;", None, None),
        (r"<html lang={undefined} />;", None, None),
        (r#"<html lang="" />;"#, None, None),
        ("<HTMLTop />", None, Some(settings())),
    ];

    Tester::new_with_settings(HtmlHasLang::NAME, pass, fail)
        .with_jsx_a11y_plugin(true)
        .test_and_snapshot();
}
