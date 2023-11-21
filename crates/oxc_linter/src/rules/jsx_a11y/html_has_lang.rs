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

use crate::{context::LintContext, rule::Rule, utils::has_jsx_prop_lowercase, AstNode};

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

#[derive(Debug, Error, Diagnostic)]
enum HtmlHasLangDiagnostic {
    #[error("eslint-plugin-jsx-a11y(html-has-lang): Missing lang attribute.")]
    #[diagnostic(severity(warning), help("Add a lang attribute to the html element whose value represents the primary language of document."))]
    MissingLangProp(#[label] Span),

    #[error("eslint-plugin-jsx-a11y(html-has-lang): Missing value for lang attribute")]
    #[diagnostic(severity(warning), help("Must have meaningful value for `lang` prop."))]
    MissingLangValue(#[label] Span),
}

fn get_prop_value<'a, 'b>(item: &'b JSXAttributeItem<'a>) -> Option<&'b JSXAttributeValue<'a>> {
    if let JSXAttributeItem::Attribute(attr) = item {
        attr.0.value.as_ref()
    } else {
        None
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

impl Rule for HtmlHasLang {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(jsx_el) = node.kind() else {
            return;
        };
        let JSXElementName::Identifier(identifier) = &jsx_el.name else {
            return;
        };

        let name = identifier.name.as_str();
        if name != "html" {
            return;
        }

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

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (r"<div />;", None),
        (r#"<html lang="en" />"#, None),
        (r#"<html lang="en-US" />"#, None),
        (r"<html lang={foo} />;", None),
        (r"<html lang />;", None),
        (r"<HTML />;", None),
        // TODO: When polymorphic components are supported
        // (r#"<HTMLTop lang="en" />"#, None),
    ];

    let fail = vec![
        (r"<html />;", None),
        (r"<html {...props} />;", None),
        (r"<html lang={undefined} />;", None),
        (r#"<html lang="" />;"#, None),
        // TODO: When polymorphic components are supported
        // (r"<HTMLTop />;", None),
    ];

    Tester::new(HtmlHasLang::NAME, pass, fail).with_jsx_a11y_plugin(true).test_and_snapshot();
}
