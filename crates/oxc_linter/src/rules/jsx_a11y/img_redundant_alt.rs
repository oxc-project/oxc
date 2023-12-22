use regex::Regex;

use oxc_ast::{
    ast::{
        Expression, JSXAttributeItem, JSXAttributeName, JSXAttributeValue, JSXElementName,
        JSXExpression, JSXExpressionContainer,
    },
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::utils::{get_prop_value, has_jsx_prop_lowercase, is_hidden_from_screen_reader};
use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-jsx-a11y(img-redundant-alt): Redundant alt attribute.")]
#[diagnostic(severity(warning), help("Provide no redundant alt text for image. Screen-readers already announce `img` tags as an image. You don’t need to use the words `image`, `photo,` or `picture` (or any specified custom words) in the alt prop."))]
struct ImgRedundantAltDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct ImgRedundantAlt(Box<ImgRedundantAltConfig>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImgRedundantAltConfig {
    types_to_validate: Vec<String>,
    redundant_words: Vec<String>,
}

impl std::ops::Deref for ImgRedundantAlt {
    type Target = ImgRedundantAltConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for ImgRedundantAltConfig {
    fn default() -> Self {
        Self {
            types_to_validate: COMPONENTS_FIXED_TO_VALIDATE
                .iter()
                .map(|&s| s.to_string())
                .collect(),
            redundant_words: REDUNDANT_WORDS.iter().map(|&s| s.to_string()).collect(),
        }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce img alt attribute does not contain the word image, picture, or photo. Screenreaders already announce img elements as an image.
    /// There is no need to use words such as image, photo, and/or picture.
    ///
    /// ### Why is this necessary?
    ///
    /// Alternative text is a critical component of accessibility for screen
    /// reader users, enabling them to understand the content and function
    /// of an element.
    ///
    /// ### What it checks
    ///
    /// This rule checks for alternative text on the following elements:
    /// `<img>` and the components which you define in options.components with the exception of components which is hidden from screen reader.
    ///
    /// ### Example
    /// ```javascript
    /// // Bad
    /// <img src="foo" alt="Photo of foo being weird." />
    /// <img src="bar" alt="Image of me at a bar!" />
    /// <img src="baz" alt="Picture of baz fixing a bug." />
    ///
    /// // Good
    /// <img src="foo" alt="Foo eating a sandwich." />
    /// <img src="bar" aria-hidden alt="Picture of me taking a photo of an image" /> // Will pass because it is hidden.
    /// <img src="baz" alt={`Baz taking a ${photo}`} /> // This is valid since photo is a variable name.
    /// ```
    ImgRedundantAlt,
    correctness
);
const COMPONENTS_FIXED_TO_VALIDATE: [&str; 1] = ["img"];
const REDUNDANT_WORDS: [&str; 3] = ["image", "photo", "picture"];

impl Rule for ImgRedundantAlt {
    fn from_configuration(value: serde_json::Value) -> Self {
        let mut img_redundant_alt = ImgRedundantAltConfig::default();
        if let Some(config) = value.get(0) {
            if let Some(components) = config.get("components").and_then(|v| v.as_array()) {
                img_redundant_alt
                    .types_to_validate
                    .extend(components.iter().filter_map(|v| v.as_str().map(ToString::to_string)));
            }

            if let Some(words) = config.get("words").and_then(|v| v.as_array()) {
                img_redundant_alt
                    .redundant_words
                    .extend(words.iter().filter_map(|v| v.as_str().map(ToString::to_string)));
            }
        }

        Self(Box::new(img_redundant_alt))
    }
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(jsx_el) = node.kind() else { return };
        let JSXElementName::Identifier(iden) = &jsx_el.name else { return };
        let name = iden.name.as_str();

        if !self.types_to_validate.iter().any(|comp| comp == name) {
            return;
        }

        if is_hidden_from_screen_reader(jsx_el) {
            return;
        }

        let alt_prop = match has_jsx_prop_lowercase(jsx_el, "alt") {
            Some(v) => v,
            None => {
                return;
            }
        };

        let alt_attribute = match get_prop_value(alt_prop) {
            Some(v) => v,
            None => {
                return;
            }
        };

        let alt_attribute_name = match alt_prop {
            JSXAttributeItem::Attribute(attr) => &attr.name,
            JSXAttributeItem::SpreadAttribute(_) => {
                return;
            }
        };

        let alt_attribute_name_span = match alt_attribute_name {
            JSXAttributeName::Identifier(iden) => iden.span,
            JSXAttributeName::NamespacedName(namespaced_name) => namespaced_name.span,
        };

        match alt_attribute {
            JSXAttributeValue::StringLiteral(lit) => {
                let alt_text = lit.value.as_str();

                if is_redundant_alt_text(alt_text, &self.redundant_words) {
                    ctx.diagnostic(ImgRedundantAltDiagnostic(alt_attribute_name_span));
                }
            }
            JSXAttributeValue::ExpressionContainer(JSXExpressionContainer {
                expression: JSXExpression::Expression(expression),
                ..
            }) => match expression {
                Expression::StringLiteral(lit) => {
                    let alt_text = lit.value.as_str();

                    if is_redundant_alt_text(alt_text, &self.redundant_words) {
                        ctx.diagnostic(ImgRedundantAltDiagnostic(alt_attribute_name_span));
                    }
                }
                Expression::TemplateLiteral(lit) => {
                    for quasi in &lit.quasis {
                        let alt_text = quasi.value.raw.as_str();

                        if is_redundant_alt_text(alt_text, &self.redundant_words) {
                            ctx.diagnostic(ImgRedundantAltDiagnostic(alt_attribute_name_span));
                        }
                    }
                }
                _ => {}
            },
            _ => {}
        };
    }
}

fn is_redundant_alt_text(alt_text: &str, redundant_words: &[String]) -> bool {
    let regexp = Regex::new(&format!(r"(?i)\b({})\b", redundant_words.join("|"),)).unwrap();

    regexp.is_match(alt_text)
}

#[test]
fn test() {
    use crate::tester::Tester;

    fn array() -> serde_json::Value {
        serde_json::json!([{
            "components": ["Image"],
            "words": ["Word1", "Word2"]
        }])
    }

    let pass = vec![
        (r"<img alt='foo' />;", None),
        (r"<img alt='picture of me taking a photo of an image' aria-hidden />", None),
        (r"<img aria-hidden alt='photo of image' />", None),
        (r"<img ALt='foo' />;", None),
        (r"<img {...this.props} alt='foo' />", None),
        (r"<img {...this.props} alt={'foo'} />", None),
        (r"<img {...this.props} alt={alt} />", None),
        (r"<a />", None),
        (r"<img />", None),
        (r"<IMG />", None),
        (r"<img alt={undefined} />", None),
        (r"<img alt={`this should pass for ${now}`} />", None),
        (r"<img alt={`this should pass for ${photo}`} />", None),
        (r"<img alt={`this should pass for ${image}`} />", None),
        (r"<img alt={`this should pass for ${picture}`} />", None),
        (r"<img alt={`${photo}`} />", None),
        (r"<img alt={`${image}`} />", None),
        (r"<img alt={`${picture}`} />", None),
        (r"<img alt={'undefined'} />", None),
        (r"<img alt={() => {}} />", None),
        (r"<img alt={function(e){}} />", None),
        (r"<img aria-hidden={false} alt='Doing cool things.' />", None),
        (r"<UX.Layout>test</UX.Layout>", None),
        (r"<img alt />", None),
        (r"<img alt={imageAlt} />", None),
        (r"<img alt={imageAlt.name} />", None),
        (r"<img alt={imageAlt?.name} />", None),
        (r"<img alt='Doing cool things' aria-hidden={foo?.bar}/>", None),
        (r"<img alt='Photography' />;", None),
        (r"<img alt='ImageMagick' />;", None),
        (r"<Image alt='Photo of a friend' />", None),
        // TODO we need components_settings to test this
        // (r"<Image alt='Foo' />", settings: Some(components_settings))
    ];

    let fail = vec![
        (r"<img alt='Photo of friend.' />;", None),
        (r"<img alt='Picture of friend.' />;", None),
        (r"<img alt='Image of friend.' />;", None),
        (r"<img alt='PhOtO of friend.' />;", None),
        (r"<img alt={'photo'} />;", None),
        (r"<img alt='piCTUre of friend.' />;", None),
        (r"<img alt='imAGE of friend.' />;", None),
        (r"<img alt='photo of cool person' aria-hidden={false} />", None),
        (r"<img alt='picture of cool person' aria-hidden={false} />", None),
        (r"<img alt='image of cool person' aria-hidden={false} />", None),
        (r"<img alt='photo' {...this.props} />", None),
        (r"<img alt='image' {...this.props} />", None),
        (r"<img alt='picture' {...this.props} />", None),
        (r"<img alt={`picture doing ${things}`} {...this.props} />", None),
        (r"<img alt={`photo doing ${things}`} {...this.props} />", None),
        (r"<img alt={`image doing ${things}`} {...this.props} />", None),
        (r"<img alt={`picture doing ${picture}`} {...this.props} />", None),
        (r"<img alt={`photo doing ${photo}`} {...this.props} />", None),
        (r"<img alt={`image doing ${image}`} {...this.props} />", None),
        // TODO we need components_settings to test this
        // (r"<Image alt='Photo of a friend' />", Some(components_settings),

        // TESTS FOR ARRAY OPTION TESTS
        (r"<img alt='Word1' />;", Some(array())),
        (r"<img alt='Word2' />;", Some(array())),
        (r"<Image alt='Word1' />;", Some(array())),
        (r"<Image alt='Word2' />;", Some(array())),
    ];

    Tester::new(ImgRedundantAlt::NAME, pass, fail).with_jsx_a11y_plugin(true).test_and_snapshot();
}
