use oxc_ast::{
    ast::{JSXAttributeItem, JSXAttributeValue, JSXExpression},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{get_element_type, get_prop_value, has_jsx_prop_ignore_case},
    AstNode,
};

fn missing_lang_prop(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Missing lang attribute.")
        .with_help("Add a lang attribute to the html element whose value represents the primary language of document.")
        .with_label(span)
}

fn missing_lang_value(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Missing value for lang attribute")
        .with_help("Must have meaningful value for `lang` prop.")
        .with_label(span)
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
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <html />
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <html lang="en" />
    /// ```
    HtmlHasLang,
    jsx_a11y,
    correctness
);

impl Rule for HtmlHasLang {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(jsx_el) = node.kind() else {
            return;
        };

        let element_type = get_element_type(ctx, jsx_el);

        if element_type != "html" {
            return;
        }

        has_jsx_prop_ignore_case(jsx_el, "lang").map_or_else(
            || ctx.diagnostic(missing_lang_prop(jsx_el.name.span())),
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
        Some(JSXAttributeValue::ExpressionContainer(container)) => match &container.expression {
            JSXExpression::EmptyExpression(_)
            | JSXExpression::NullLiteral(_)
            | JSXExpression::BooleanLiteral(_)
            | JSXExpression::NumericLiteral(_) => false,
            JSXExpression::Identifier(id) => id.name != "undefined",
            JSXExpression::StringLiteral(str) => !str.value.as_str().is_empty(),
            JSXExpression::TemplateLiteral(t) => {
                !t.expressions.is_empty()
                    || t.quasis.iter().filter(|q| !q.value.raw.is_empty()).count() > 0
            }
            _ => true,
        },
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
        (r#"<html lang={"en-US"} />"#, None, None, None),
        (r"<html lang={`en-US`} />", None, None, None),
        (r"<html lang={`${foo}`} />", None, None, None),
        (r"<html lang={foo} />;", None, None, None),
        (r"<html lang />;", None, None, None),
        (r"<HTML />;", None, None, None),
        ("<HTMLTop lang='en' />", None, Some(settings()), None),
    ];

    let fail = vec![
        (r"<html />;", None, None, None),
        (r"<html {...props} />;", None, None, None),
        (r"<html lang={undefined} />;", None, None, None),
        (r"<html lang={null} />;", None, None, None),
        (r"<html lang={false} />;", None, None, None),
        (r"<html lang={1} />;", None, None, None),
        (r"<html lang={''} />;", None, None, None),
        (r"<html lang={``} />;", None, None, None),
        (r#"<html lang="" />;"#, None, None, None),
        ("<HTMLTop />", None, Some(settings()), None),
    ];

    Tester::new(HtmlHasLang::NAME, HtmlHasLang::PLUGIN, pass, fail)
        .with_jsx_a11y_plugin(true)
        .test_and_snapshot();
}
