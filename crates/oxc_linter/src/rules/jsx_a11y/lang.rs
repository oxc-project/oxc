use language_tags::LanguageTag;
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
#[error("eslint-plugin-jsx-a11y(lang): Lang attribute must have a valid value.")]
#[diagnostic(severity(warning), help("Set a valid value for lang attribute."))]
struct LangDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct Lang;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// The lang prop on the <html> element must be a valid IETF's BCP 47 language tag.
    ///
    /// ### Why is this bad?
    ///
    /// If the language of a webpage is not specified as valid,
    /// the screen reader assumes the default language set by the user.
    /// Language settings become an issue for users who speak multiple languages
    /// and access website in more than one language.
    ///
    ///
    /// ### Example
    ///
    /// // good
    /// ```javascript
    /// <html lang="en">
    /// <html lang="en-US">
    /// ```
    ///
    /// // bad
    /// ```javascript
    /// <html>
    /// <html lang="foo">
    /// ````
    ///
    /// ### Resources
    /// - [eslint-plugin-jsx-a11y/lang](https://github.com/jsx-eslint/eslint-plugin-jsx-a11y/blob/main/docs/rules/lang.md)
    /// - [IANA Language Subtag Registry](https://www.iana.org/assignments/language-subtag-registry/language-subtag-registry)
    Lang,
    correctness
);

impl Rule for Lang {
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
            || ctx.diagnostic(LangDiagnostic(identifier.span)),
            |lang_prop| {
                if !is_valid_lang_prop(lang_prop) {
                    if let JSXAttributeItem::Attribute(attr) = lang_prop {
                        ctx.diagnostic(LangDiagnostic(attr.span));
                    }
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
        Some(JSXAttributeValue::StringLiteral(str)) => {
            let language_tag = LanguageTag::parse(str.value.as_str()).unwrap();
            language_tag.is_valid()
        }
        _ => true,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    fn settings() -> serde_json::Value {
        serde_json::json!({
            "jsx-a11y": {
                "polymorphicPropName": "as",
                "components": {
                    "Foo": "html",
                }
            }
        })
    }

    let pass = vec![
        ("<div />;", None, None),
        ("<div foo='bar' />;", None, None),
        ("<div lang='foo' />;", None, None),
        ("<html lang='en' />", None, None),
        ("<html lang='en-US' />", None, None),
        ("<html lang='zh-Hans' />", None, None),
        ("<html lang='zh-Hant-HK' />", None, None),
        ("<html lang='zh-yue-Hant' />", None, None),
        ("<html lang='ja-Latn' />", None, None),
        ("<html lang={foo} />", None, None),
        ("<HTML lang='foo' />", None, None),
        ("<Foo lang={undefined} />", None, None),
        ("<Foo lang='en' />", None, Some(settings())),
        ("<Box as='html' lang='en'  />", None, Some(settings())),
    ];

    let fail = vec![
        ("<html lang='foo' />", None, None),
        ("<html lang='zz-LL' />", None, None),
        ("<html lang={undefined} />", None, None),
        ("<Foo lang={undefined} />", None, Some(settings())),
        ("<Box as='html' lang='foo' />", None, Some(settings())),
    ];

    Tester::new_with_settings(Lang::NAME, pass, fail).test_and_snapshot();
}
