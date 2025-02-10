use language_tags::LanguageTag;
use oxc_ast::{
    ast::{JSXAttributeItem, JSXAttributeValue},
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

fn lang_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Lang attribute must have a valid value.")
        .with_help("Set a valid value for lang attribute.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct Lang;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// The lang prop on the `<html>` element must be a valid IETF's BCP 47 language tag.
    ///
    /// ### Why is this bad?
    ///
    /// If the language of a webpage is not specified as valid,
    /// the screen reader assumes the default language set by the user.
    /// Language settings become an issue for users who speak multiple languages
    /// and access website in more than one language.
    ///
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <html>
    /// <html lang="foo">
    /// ````
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <html lang="en">
    /// <html lang="en-US">
    /// ```
    ///
    /// ### Resources
    /// - [eslint-plugin-jsx-a11y/lang](https://github.com/jsx-eslint/eslint-plugin-jsx-a11y/blob/v6.9.0/docs/rules/lang.md)
    /// - [IANA Language Subtag Registry](https://www.iana.org/assignments/language-subtag-registry/language-subtag-registry)
    Lang,
    jsx_a11y,
    correctness
);

impl Rule for Lang {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(jsx_el) = node.kind() else {
            return;
        };

        let element_type = get_element_type(ctx, jsx_el);

        if element_type != "html" {
            return;
        }

        has_jsx_prop_ignore_case(jsx_el, "lang").map_or_else(
            || ctx.diagnostic(lang_diagnostic(jsx_el.name.span())),
            |lang_prop| {
                if !is_valid_lang_prop(lang_prop) {
                    if let JSXAttributeItem::Attribute(attr) = lang_prop {
                        ctx.diagnostic(lang_diagnostic(attr.span));
                    }
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
        Some(JSXAttributeValue::StringLiteral(str)) => {
            LanguageTag::parse(str.value.as_str()).as_ref().is_ok_and(LanguageTag::is_valid)
        }
        _ => true,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    fn settings() -> serde_json::Value {
        serde_json::json!({
            "settings": { "jsx-a11y": {
                "polymorphicPropName": "as",
                "components": {
                    "Foo": "html",
                }
            } }
        })
    }

    let pass = vec![
        ("<div />;", None, None, None),
        ("<div foo='bar' />;", None, None, None),
        ("<div lang='foo' />;", None, None, None),
        ("<html lang='en' />", None, None, None),
        ("<html lang='en-US' />", None, None, None),
        ("<html lang='zh-Hans' />", None, None, None),
        ("<html lang='zh-Hant-HK' />", None, None, None),
        ("<html lang='zh-yue-Hant' />", None, None, None),
        ("<html lang='ja-Latn' />", None, None, None),
        ("<html lang={foo} />", None, None, None),
        ("<HTML lang='foo' />", None, None, None),
        ("<Foo lang={undefined} />", None, None, None),
        ("<Foo lang='en' />", None, Some(settings()), None),
        ("<Box as='html' lang='en'  />", None, Some(settings()), None),
    ];

    let fail = vec![
        ("<html lang='foo' />", None, None, None),
        ("<html lang='n'></html>", None, None, None),
        ("<html lang='zz-LL' />", None, None, None),
        ("<html lang={undefined} />", None, None, None),
        ("<Foo lang={undefined} />", None, Some(settings()), None),
        ("<Box as='html' lang='foo' />", None, Some(settings()), None),
    ];

    Tester::new(Lang::NAME, Lang::PLUGIN, pass, fail).test_and_snapshot();
}
