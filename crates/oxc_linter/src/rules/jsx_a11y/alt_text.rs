use oxc_ast::{
    ast::{
        JSXAttributeItem, JSXAttributeValue, JSXChild, JSXElement, JSXElementName, JSXExpression,
        JSXExpressionContainer, JSXOpeningElement,
    },
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::utils::has_jsx_prop_lowercase;
use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
enum AltTextDiagnostic {
    // <img>
    #[error("eslint-plugin-jsx-a11y(alt-text): Missing `alt` attribute.")]
    #[diagnostic(severity(warning), help("Must have `alt` prop, either with meaningful text, or an empty string for decorative images."))]
    MissingAltProp(#[label] Span),

    #[error("eslint-plugin-jsx-a11y(alt-text): Invalid `alt` value.")]
    #[diagnostic(
        severity(warning),
        help("Must have meaningful value for `alt` prop. Use alt=\"\" for presentational images.")
    )]
    MissingAltValue(#[label] Span),

    #[error("eslint-plugin-jsx-a11y(alt-text): Missing value for aria-label attribute.")]
    #[diagnostic(severity(warning), help("The aria-label attribute must have a value. The alt attribute is preferred over aria-label for images."))]
    AriaLabelValue(#[label] Span),

    #[error("eslint-plugin-jsx-a11y(alt-text): Missing value for aria-labelledby attribute.")]
    #[diagnostic(
        severity(warning),
        help("The alt attribute is preferred over aria-labelledby for images.")
    )]
    AriaLabelledByValue(#[label] Span),

    #[error("eslint-plugin-jsx-a11y(alt-text): ARIA used where native HTML could suffice.")]
    #[diagnostic(severity(warning), help("Prefer alt=\"\" over presentational role. Native HTML attributes should be preferred for accessibility before resorting to ARIA attributes."))]
    PreferAlt(#[label] Span),

    // <object>
    #[error("eslint-plugin-jsx-a11y(alt-text): Missing alternative text.")]
    #[diagnostic(severity(warning), help("Embedded <object> elements must have a text alternative through the `alt`, `aria-label`, or `aria-labelledby` prop."))]
    Object(#[label] Span),

    // <area>
    #[error("eslint-plugin-jsx-a11y(alt-text): Missing alternative text.")]
    #[diagnostic(severity(warning), help("Each area of an image map must have a text alternative through the `alt`, `aria-label`, or `aria-labelledby` prop."))]
    Area(#[label] Span),

    // <input type="image">
    #[error("eslint-plugin-jsx-a11y(alt-text): Missing alternative text.")]
    #[diagnostic(severity(warning), help("<input> elements with type=\"image\" must have a text alternative through the `alt`, `aria-label`, or `aria-labelledby` prop."))]
    InputTypeImage(#[label] Span),
}

#[derive(Debug, Clone)]
pub struct AltText {
    img: Option<Vec<String>>,
    object: Option<Vec<String>>,
    area: Option<Vec<String>>,
    input_type_image: Option<Vec<String>>,
}

impl std::default::Default for AltText {
    fn default() -> Self {
        Self {
            img: Some(vec![]),
            object: Some(vec![]),
            area: Some(vec![]),
            input_type_image: Some(vec![]),
        }
    }
}

fn get_prop_value<'a, 'b>(item: &'b JSXAttributeItem<'a>) -> Option<&'b JSXAttributeValue<'a>> {
    if let JSXAttributeItem::Attribute(attr) = item {
        attr.0.value.as_ref()
    } else {
        None
    }
}

fn get_literal_prop_value<'a>(item: &'a JSXAttributeItem<'_>) -> Option<&'a str> {
    get_prop_value(item).and_then(|v| {
        if let JSXAttributeValue::StringLiteral(s) = v {
            Some(s.value.as_str())
        } else {
            None
        }
    })
}

fn is_valid_alt_prop(item: &JSXAttributeItem<'_>) -> bool {
    match get_prop_value(item) {
        None => false,
        Some(JSXAttributeValue::ExpressionContainer(JSXExpressionContainer {
            expression: JSXExpression::Expression(expr),
            ..
        })) => !expr.is_null_or_undefined(),
        _ => true,
    }
}

fn is_presentation_role<'a>(item: &'a JSXAttributeItem<'a>) -> bool {
    get_literal_prop_value(item).map_or(false, |value| value == "presentation" || value == "none")
}

fn aria_label_has_value<'a>(item: &'a JSXAttributeItem<'a>) -> bool {
    match get_prop_value(item) {
        None => false,
        Some(JSXAttributeValue::StringLiteral(s)) if s.value.is_empty() => false,
        Some(JSXAttributeValue::ExpressionContainer(JSXExpressionContainer {
            expression: JSXExpression::Expression(expr),
            ..
        })) => !expr.is_undefined(),
        _ => true,
    }
}

// ref: https://github.com/jsx-eslint/eslint-plugin-jsx-a11y/blob/main/src/util/hasAccessibleChild.js
fn object_has_accessible_child(node: &JSXElement<'_>) -> bool {
    node.children.iter().any(|child| match child {
        JSXChild::Text(text) => !text.value.is_empty(),
        JSXChild::Fragment(_) => true,
        JSXChild::Element(el) => {
            let is_hidden_from_screen_reader =
                has_jsx_prop_lowercase(&el.opening_element, "aria-hidden").map_or(false, |v| {
                    match get_prop_value(v) {
                        None => true,
                        Some(JSXAttributeValue::StringLiteral(s)) if s.value == "true" => true,
                        _ => false,
                    }
                });
            !is_hidden_from_screen_reader
        }
        JSXChild::ExpressionContainer(JSXExpressionContainer {
            expression: JSXExpression::Expression(expr),
            ..
        }) => !expr.is_undefined(),
        _ => false,
    }) || has_jsx_prop_lowercase(&node.opening_element, "dangerouslySetInnerHTML").is_some()
        || has_jsx_prop_lowercase(&node.opening_element, "children").is_some()
}

fn img_rule<'a>(node: &'a JSXOpeningElement<'a>, ctx: &LintContext<'a>) {
    if let Some(alt_prop) = has_jsx_prop_lowercase(node, "alt") {
        if !is_valid_alt_prop(alt_prop) {
            ctx.diagnostic(AltTextDiagnostic::MissingAltValue(node.span));
        }
        return;
    }

    if has_jsx_prop_lowercase(node, "role").map_or(false, is_presentation_role) {
        ctx.diagnostic(AltTextDiagnostic::PreferAlt(node.span));
        return;
    }

    if let Some(aria_label_prop) = has_jsx_prop_lowercase(node, "aria-label") {
        if !aria_label_has_value(aria_label_prop) {
            ctx.diagnostic(AltTextDiagnostic::AriaLabelValue(node.span));
        }
        return;
    }

    if let Some(aria_labelledby_prop) = has_jsx_prop_lowercase(node, "aria-labelledby") {
        if !aria_label_has_value(aria_labelledby_prop) {
            ctx.diagnostic(AltTextDiagnostic::AriaLabelledByValue(node.span));
        }
        return;
    }

    ctx.diagnostic(AltTextDiagnostic::MissingAltProp(node.span));
}

fn object_rule<'a>(
    node: &'a JSXOpeningElement<'a>,
    parent: &'a JSXElement<'a>,
    ctx: &LintContext<'a>,
) {
    let has_aria_label =
        has_jsx_prop_lowercase(node, "aria-label").map_or(false, aria_label_has_value);
    let has_aria_labeledby =
        has_jsx_prop_lowercase(node, "aria-labelledby").map_or(false, aria_label_has_value);
    let has_label = has_aria_label || has_aria_labeledby;
    let has_title_attr = has_jsx_prop_lowercase(node, "title")
        .and_then(get_literal_prop_value)
        .map_or(false, |v| !v.is_empty());

    if has_label || has_title_attr || object_has_accessible_child(parent) {
        return;
    }
    ctx.diagnostic(AltTextDiagnostic::Object(node.span));
}

fn area_rule<'a>(node: &'a JSXOpeningElement<'a>, ctx: &LintContext<'a>) {
    let has_aria_label =
        has_jsx_prop_lowercase(node, "aria-label").map_or(false, aria_label_has_value);
    let has_aria_labeledby =
        has_jsx_prop_lowercase(node, "aria-labelledby").map_or(false, aria_label_has_value);
    let has_label = has_aria_label || has_aria_labeledby;
    if has_label {
        return;
    }
    has_jsx_prop_lowercase(node, "alt").map_or_else(
        || {
            ctx.diagnostic(AltTextDiagnostic::Area(node.span));
        },
        |alt_prop| {
            if !is_valid_alt_prop(alt_prop) {
                ctx.diagnostic(AltTextDiagnostic::Area(node.span));
            }
        },
    );
}

fn input_type_image_rule<'a>(node: &'a JSXOpeningElement<'a>, ctx: &LintContext<'a>) {
    let has_aria_label =
        has_jsx_prop_lowercase(node, "aria-label").map_or(false, aria_label_has_value);
    let has_aria_labeledby =
        has_jsx_prop_lowercase(node, "aria-labelledby").map_or(false, aria_label_has_value);
    let has_label = has_aria_label || has_aria_labeledby;
    if has_label {
        return;
    }
    has_jsx_prop_lowercase(node, "alt").map_or_else(
        || {
            ctx.diagnostic(AltTextDiagnostic::InputTypeImage(node.span));
        },
        |alt_prop| {
            if !is_valid_alt_prop(alt_prop) {
                ctx.diagnostic(AltTextDiagnostic::InputTypeImage(node.span));
            }
        },
    );
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce that all elements that require alternative text have meaningful
    /// information to relay back to the end user.
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
    /// `<img>`, `<area>`, `<input type="image">`, and `<object>`.
    ///
    /// ### How to fix it
    ///
    /// Ensure that the `alt` attribute is present and contains meaningful
    /// text that describes the element's content or purpose.
    ///
    /// ### Example
    /// ```javascript
    /// // Bad
    /// <img src="flower.jpg">
    ///
    /// // Good
    /// <img src="flower.jpg" alt="A close-up of a white daisy">
    /// ```
    AltText,
    correctness
);

impl Rule for AltText {
    fn from_configuration(value: serde_json::Value) -> Self {
        let mut alt_text = Self::default();
        if let Some(config) = value.get(0) {
            if let Some(elements) = config.get("elements").and_then(|v| v.as_array()) {
                alt_text = Self { img: None, object: None, area: None, input_type_image: None };
                for el in elements {
                    match el.as_str() {
                        Some("img") => alt_text.img = Some(vec![]),
                        Some("object") => alt_text.object = Some(vec![]),
                        Some("area") => alt_text.area = Some(vec![]),
                        Some("input[type=\"image\"]") => alt_text.input_type_image = Some(vec![]),
                        _ => {}
                    }
                }
            }

            for (tags, field) in [
                (&mut alt_text.img, "img"),
                (&mut alt_text.object, "object"),
                (&mut alt_text.area, "area"),
                (&mut alt_text.input_type_image, "input[type=\"image\"]"),
            ] {
                if let (Some(tags), Some(elements)) =
                    (tags, config.get(field).and_then(|v| v.as_array()))
                {
                    tags.extend(
                        elements.iter().filter_map(|v| v.as_str().map(ToString::to_string)),
                    );
                }
            }
        }

        alt_text
    }
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(jsx_el) = node.kind() else { return };
        let JSXElementName::Identifier(iden) = &jsx_el.name else { return };
        let name = iden.name.as_str();

        // <img>
        if let Some(custom_tags) = &self.img {
            if name == "img" || custom_tags.iter().any(|i| i == name) {
                img_rule(jsx_el, ctx);
                return;
            }
        }

        // <object>
        if let Some(custom_tags) = &self.object {
            if name == "object" || custom_tags.iter().any(|i| i == name) {
                let maybe_parent =
                    ctx.nodes().parent_node(node.id()).map(oxc_semantic::AstNode::kind);
                if let Some(AstKind::JSXElement(parent)) = maybe_parent {
                    object_rule(jsx_el, parent, ctx);
                    return;
                }
            }
        }

        // <area>
        if let Some(custom_tags) = &self.area {
            if name == "area" || custom_tags.iter().any(|i| i == name) {
                area_rule(jsx_el, ctx);
                return;
            }
        }

        // <input type="image">
        if let Some(custom_tags) = &self.input_type_image {
            let has_input_with_type_image = name.to_lowercase() == "input"
                && has_jsx_prop_lowercase(jsx_el, "type")
                    .map_or(false, |v| get_literal_prop_value(v).map_or(false, |v| v == "image"));
            if has_input_with_type_image || custom_tags.iter().any(|i| i == name) {
                input_type_image_rule(jsx_el, ctx);
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    fn array() -> serde_json::Value {
        serde_json::json!([{
            "img": ["Thumbnail", "Image"],
            "object": ["Object"],
            "area": ["Area"],
            "input[type=\"image\"]": ["InputImage"],
        }])
    }

    let pass = vec![
        (r#"<img alt="foo" />;"#, None),
        (r#"<img alt={"foo"} />;"#, None),
        (r#"<img alt={alt} />;"#, None),
        (r#"<img ALT="foo" />;"#, None),
        (r#"<img ALT={`This is the ${alt} text`} />;"#, None),
        (r#"<img ALt="foo" />;"#, None),
        (r#"<img alt="foo" salt={undefined} />;"#, None),
        (r#"<img {...this.props} alt="foo" />"#, None),
        (r#"<a />"#, None),
        (r#"<div />"#, None),
        (r#"<img alt={function(e) {} } />"#, None),
        (r#"<div alt={function(e) {} } />"#, None),
        (r#"<img alt={() => void 0} />"#, None),
        (r#"<IMG />"#, None),
        (r#"<UX.Layout>test</UX.Layout>"#, None),
        (r#"<img alt={alt || "Alt text" } />"#, None),
        (r#"<img alt={photo.caption} />;"#, None),
        (r#"<img alt={bar()} />;"#, None),
        (r#"<img alt={foo.bar || ""} />"#, None),
        (r#"<img alt={bar() || ""} />"#, None),
        (r#"<img alt={foo.bar() || ""} />"#, None),
        (r#"<img alt="" />"#, None),
        (r#"<img alt={`${undefined}`} />"#, None),
        (r#"<img alt=" " />"#, None),
        (r#"<img alt="" role="presentation" />"#, None),
        (r#"<img alt="" role="none" />"#, None),
        (r#"<img alt="" role={`presentation`} />"#, None),
        (r#"<img alt="" role={"presentation"} />"#, None),
        (r#"<img alt="this is lit..." role="presentation" />"#, None),
        (r#"<img alt={error ? "not working": "working"} />"#, None),
        (r#"<img alt={undefined ? "working": "not working"} />"#, None),
        (r#"<img alt={plugin.name + " Logo"} />"#, None),
        (r#"<img aria-label="foo" />"#, None),
        (r#"<img aria-labelledby="id1" />"#, None),
        (r#"<object aria-label="foo" />"#, None),
        (r#"<object aria-labelledby="id1" />"#, None),
        (r#"<object>Foo</object>"#, None),
        (r#"<object><p>This is descriptive!</p></object>"#, None),
        (r#"<Object />"#, None),
        (r#"<object title="An object" />"#, None),
        (r#"<area aria-label="foo" />"#, None),
        (r#"<area aria-labelledby="id1" />"#, None),
        (r#"<area alt="" />"#, None),
        (r#"<area alt="This is descriptive!" />"#, None),
        (r#"<area alt={altText} />"#, None),
        (r#"<Area />"#, None),
        (r#"<input />"#, None),
        (r#"<input type="foo" />"#, None),
        (r#"<input type="image" aria-label="foo" />"#, None),
        (r#"<input type="image" aria-labelledby="id1" />"#, None),
        (r#"<input type="image" alt="" />"#, None),
        (r#"<input type="image" alt="This is descriptive!" />"#, None),
        (r#"<input type="image" alt={altText} />"#, None),
        (r#"<InputImage />"#, None),
        (r#"<Input type="image" alt="" />"#, None),
        // TODO: When polymorphic components are supported
        // (r#"<SomeComponent as="input" type="image" alt="" />"#, None),
        (r#"<Thumbnail alt="foo" />;"#, Some(array())),
        (r#"<Thumbnail alt={"foo"} />;"#, Some(array())),
        (r#"<Thumbnail alt={alt} />;"#, Some(array())),
        (r#"<Thumbnail ALT="foo" />;"#, Some(array())),
        (r#"<Thumbnail ALT={`This is the ${alt} text`} />;"#, Some(array())),
        (r#"<Thumbnail ALt="foo" />;"#, Some(array())),
        (r#"<Thumbnail alt="foo" salt={undefined} />;"#, Some(array())),
        (r#"<Thumbnail {...this.props} alt="foo" />"#, Some(array())),
        (r#"<thumbnail />"#, Some(array())),
        (r#"<Thumbnail alt={function(e) {} } />"#, Some(array())),
        (r#"<div alt={function(e) {} } />"#, Some(array())),
        (r#"<Thumbnail alt={() => void 0} />"#, Some(array())),
        (r#"<THUMBNAIL />"#, Some(array())),
        (r#"<Thumbnail alt={alt || "foo" } />"#, Some(array())),
        (r#"<Image alt="foo" />;"#, Some(array())),
        (r#"<Image alt={"foo"} />;"#, Some(array())),
        (r#"<Image alt={alt} />;"#, Some(array())),
        (r#"<Image ALT="foo" />;"#, Some(array())),
        (r#"<Image ALT={`This is the ${alt} text`} />;"#, Some(array())),
        (r#"<Image ALt="foo" />;"#, Some(array())),
        (r#"<Image alt="foo" salt={undefined} />;"#, Some(array())),
        (r#"<Image {...this.props} alt="foo" />"#, Some(array())),
        (r#"<image />"#, Some(array())),
        (r#"<Image alt={function(e) {} } />"#, Some(array())),
        (r#"<div alt={function(e) {} } />"#, Some(array())),
        (r#"<Image alt={() => void 0} />"#, Some(array())),
        (r#"<IMAGE />"#, Some(array())),
        (r#"<Image alt={alt || "foo" } />"#, Some(array())),
        (r#"<Object aria-label="foo" />"#, Some(array())),
        (r#"<Object aria-labelledby="id1" />"#, Some(array())),
        (r#"<Object>Foo</Object>"#, Some(array())),
        (r#"<Object><p>This is descriptive!</p></Object>"#, Some(array())),
        (r#"<Object title="An object" />"#, Some(array())),
        (r#"<Area aria-label="foo" />"#, Some(array())),
        (r#"<Area aria-labelledby="id1" />"#, Some(array())),
        (r#"<Area alt="" />"#, Some(array())),
        (r#"<Area alt="This is descriptive!" />"#, Some(array())),
        (r#"<Area alt={altText} />"#, Some(array())),
        (r#"<InputImage aria-label="foo" />"#, Some(array())),
        (r#"<InputImage aria-labelledby="id1" />"#, Some(array())),
        (r#"<InputImage alt="" />"#, Some(array())),
        (r#"<InputImage alt="This is descriptive!" />"#, Some(array())),
        (r#"<InputImage alt={altText} />"#, Some(array())),
    ];

    let fail = vec![
        (r#"<img />;"#, None),
        (r#"<img alt />;"#, None),
        (r#"<img alt={undefined} />;"#, None),
        (r#"<img src="xyz" />"#, None),
        (r#"<img role />"#, None),
        (r#"<img {...this.props} />"#, None),
        // TODO: Could support if get_prop_value could evaluate
        // some logical expressions
        // (r#"<img alt={false || false} />"#, None),
        (r#"<img alt={undefined} role="presentation" />;"#, None),
        (r#"<img alt role="presentation" />;"#, None),
        (r#"<img role="presentation" />;"#, None),
        (r#"<img role="none" />;"#, None),
        (r#"<img aria-label={undefined} />"#, None),
        (r#"<img aria-labelledby={undefined} />"#, None),
        (r#"<img aria-label="" />"#, None),
        (r#"<img aria-labelledby="" />"#, None),
        // TODO: When polymorphic components are supported
        // (r#"<SomeComponent as="img" aria-label="" />"#, None),
        (r#"<object />"#, None),
        (r#"<object><div aria-hidden /></object>"#, None),
        (r#"<object title={undefined} />"#, None),
        (r#"<object aria-label="" />"#, None),
        (r#"<object aria-labelledby="" />"#, None),
        (r#"<object aria-label={undefined} />"#, None),
        (r#"<object aria-labelledby={undefined} />"#, None),
        (r#"<area />"#, None),
        (r#"<area alt />"#, None),
        (r#"<area alt={undefined} />"#, None),
        (r#"<area src="xyz" />"#, None),
        (r#"<area {...this.props} />"#, None),
        (r#"<area aria-label="" />"#, None),
        (r#"<area aria-label={undefined} />"#, None),
        (r#"<area aria-labelledby="" />"#, None),
        (r#"<area aria-labelledby={undefined} />"#, None),
        (r#"<input type="image" />"#, None),
        (r#"<input type="image" alt />"#, None),
        (r#"<input type="image" alt={undefined} />"#, None),
        (r#"<input type="image">Foo</input>"#, None),
        (r#"<input type="image" {...this.props} />"#, None),
        (r#"<input type="image" aria-label="" />"#, None),
        (r#"<input type="image" aria-label={undefined} />"#, None),
        (r#"<input type="image" aria-labelledby="" />"#, None),
        (r#"<input type="image" aria-labelledby={undefined} />"#, None),
        (r#"<Thumbnail />;"#, Some(array())),
        (r#"<Thumbnail alt />;"#, Some(array())),
        (r#"<Thumbnail alt={undefined} />;"#, Some(array())),
        (r#"<Thumbnail src="xyz" />"#, Some(array())),
        (r#"<Thumbnail {...this.props} />"#, Some(array())),
        (r#"<Image />;"#, Some(array())),
        (r#"<Image alt />;"#, Some(array())),
        (r#"<Image alt={undefined} />;"#, Some(array())),
        (r#"<Image src="xyz" />"#, Some(array())),
        (r#"<Image {...this.props} />"#, Some(array())),
        (r#"<Object />"#, Some(array())),
        (r#"<Object><div aria-hidden /></Object>"#, Some(array())),
        (r#"<Object title={undefined} />"#, Some(array())),
        (r#"<Area />"#, Some(array())),
        (r#"<Area alt />"#, Some(array())),
        (r#"<Area alt={undefined} />"#, Some(array())),
        (r#"<Area src="xyz" />"#, Some(array())),
        (r#"<Area {...this.props} />"#, Some(array())),
        (r#"<InputImage />"#, Some(array())),
        (r#"<InputImage alt />"#, Some(array())),
        (r#"<InputImage alt={undefined} />"#, Some(array())),
        (r#"<InputImage>Foo</InputImage>"#, Some(array())),
        (r#"<InputImage {...this.props} />"#, Some(array())),
        (r#"<Input type="image" />"#, None),
    ];

    Tester::new(AltText::NAME, pass, fail).test_and_snapshot();
}
