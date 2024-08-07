use oxc_ast::{
    ast::{JSXAttributeItem, JSXAttributeValue, JSXElementName},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{get_element_type, get_prop_value, has_jsx_prop_ignore_case},
    AstNode,
};

fn missing_lang_prop(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Missing lang attribute.")
        .with_help("Add a lang attribute to the html element whose value represents the primary language of document.")
        .with_label(span0)
}

fn missing_lang_value(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Missing value for lang attribute")
        .with_help("Must have meaningful value for `lang` prop.")
        .with_label(span0)
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

        has_jsx_prop_ignore_case(jsx_el, "lang").map_or_else(
            || ctx.diagnostic(missing_lang_prop(identifier.span)),
            |lang_prop| {
                if !is_valid_lang_prop(lang_prop) {
                    ctx.diagnostic(missing_lang_value(jsx_el.span));
                }
            },
        );
    }
}

fn is_valid_lang_prop(item: &JSXAttributeItem) -> bool {
    match get_prop_value(item) {
        Some(JSXAttributeValue::ExpressionContainer(container)) => {
            !container.expression.is_expression() || !container.expression.is_undefined()
        }
        Some(JSXAttributeValue::StringLiteral(str)) => !str.value.as_str().is_empty(),
        _ => true,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    fn settings() -> serde_json::Value {
        serde_json::json!({
            "settings": { "jsx-a11y": {
                "components": {
                    "HTMLTop": "html",
                }
            } }
        })
    }

    let pass = vec![
        (r"<div />;", None, None, None),
        (r#"<html lang="en" />"#, None, None, None),
        (r#"<html lang="en-US" />"#, None, None, None),
        (r"<html lang={foo} />;", None, None, None),
        (r"<html lang />;", None, None, None),
        (r"<HTML />;", None, None, None),
        ("<HTMLTop lang='en' />", None, Some(settings()), None),
    ];

    let fail = vec![
        (r"<html />;", None, None, None),
        (r"<html {...props} />;", None, None, None),
        (r"<html lang={undefined} />;", None, None, None),
        (r#"<html lang="" />;"#, None, None, None),
        ("<HTMLTop />", None, Some(settings()), None),
    ];

    Tester::new(HtmlHasLang::NAME, pass, fail).with_jsx_a11y_plugin(true).test_and_snapshot();
}
