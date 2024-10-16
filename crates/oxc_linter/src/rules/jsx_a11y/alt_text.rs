use oxc_ast::{
    ast::{JSXAttributeItem, JSXAttributeValue, JSXElement, JSXOpeningElement},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, Span};

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{
        get_element_type, get_prop_value, get_string_literal_prop_value, has_jsx_prop_ignore_case,
        object_has_accessible_child,
    },
    AstNode,
};

fn missing_alt_prop(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Missing `alt` attribute.")
        .with_help("Must have `alt` prop, either with meaningful text, or an empty string for decorative images.")
        .with_label(span)
}

fn missing_alt_value(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Invalid `alt` value.")
        .with_help(
            "Must have meaningful value for `alt` prop. Use alt=\"\" for presentational images.",
        )
        .with_label(span)
}

fn aria_label_value(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Missing value for `aria-label` attribute.")
        .with_help("Give `aria-label` a meaningful value. Prever the `alt` attribute over `aria-label` for images.")
        .with_label(span)
}

fn aria_labelled_by_value(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Missing value for `aria-labelledby` attribute.")
        .with_help("Give `aria-labelledby` an ID to a label element. Prefer the `alt` attribute over `aria-labelledby` for images.")
        .with_label(span)
}

fn prefer_alt(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("ARIA used where native HTML could suffice.")
        .with_help("Prefer alt=\"\" over presentational role. Native HTML attributes should be preferred for accessibility before resorting to ARIA attributes.")
        .with_label(span)
}

fn object(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Missing alternative text.")
        .with_help("Embedded <object> elements must have a text alternative through the `alt`, `aria-label`, or `aria-labelledby` prop.")
        .with_label(span)
}

fn area(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Missing alternative text.")
        .with_help("Each area of an image map must have a text alternative through the `alt`, `aria-label`, or `aria-labelledby` prop.")
        .with_label(span)
}

fn input_type_image(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Missing alternative text.")
        .with_help("<input> elements with type=\"image\" must have a text alternative through the `alt`, `aria-label`, or `aria-labelledby` prop.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct AltText(Box<AltTextConfig>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AltTextConfig {
    img: Option<Vec<CompactStr>>,
    object: Option<Vec<CompactStr>>,
    area: Option<Vec<CompactStr>>,
    input_type_image: Option<Vec<CompactStr>>,
}

impl std::ops::Deref for AltText {
    type Target = AltTextConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::default::Default for AltTextConfig {
    fn default() -> Self {
        Self {
            img: Some(vec![]),
            object: Some(vec![]),
            area: Some(vec![]),
            input_type_image: Some(vec![]),
        }
    }
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
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <img src="flower.jpg" alt="A close-up of a white daisy" />
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <img src="flower.jpg" />
    /// ```
    AltText,
    correctness
);

impl Rule for AltText {
    fn from_configuration(value: serde_json::Value) -> Self {
        let mut alt_text = AltTextConfig::default();
        if let Some(config) = value.get(0) {
            if let Some(elements) = config.get("elements").and_then(|v| v.as_array()) {
                alt_text =
                    AltTextConfig { img: None, object: None, area: None, input_type_image: None };
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
                    tags.extend(elements.iter().filter_map(|v| v.as_str().map(CompactStr::from)));
                }
            }
        }

        Self(Box::new(alt_text))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(jsx_el) = node.kind() else {
            return;
        };
        let Some(name) = &get_element_type(ctx, jsx_el) else {
            return;
        };

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
            let has_input_with_type_image = name.eq_ignore_ascii_case("input")
                && has_jsx_prop_ignore_case(jsx_el, "type").map_or(false, |v| {
                    get_string_literal_prop_value(v).map_or(false, |v| v == "image")
                });
            if has_input_with_type_image || custom_tags.iter().any(|i| i == name) {
                input_type_image_rule(jsx_el, ctx);
            }
        }
    }
}

fn is_valid_alt_prop(item: &JSXAttributeItem<'_>) -> bool {
    match get_prop_value(item) {
        None => false,
        Some(JSXAttributeValue::ExpressionContainer(container)) => {
            if let Some(expr) = container.expression.as_expression() {
                !expr.is_null_or_undefined()
            } else {
                true
            }
        }
        _ => true,
    }
}

fn is_presentation_role<'a>(item: &'a JSXAttributeItem<'a>) -> bool {
    get_string_literal_prop_value(item)
        .map_or(false, |value| value == "presentation" || value == "none")
}

fn aria_label_has_value<'a>(item: &'a JSXAttributeItem<'a>) -> bool {
    match get_prop_value(item) {
        None => false,
        Some(JSXAttributeValue::StringLiteral(s)) if s.value.is_empty() => false,
        Some(JSXAttributeValue::ExpressionContainer(container)) => {
            !container.expression.is_expression() || !container.expression.is_undefined()
        }
        _ => true,
    }
}

fn img_rule<'a>(node: &'a JSXOpeningElement<'a>, ctx: &LintContext<'a>) {
    if let Some(alt_prop) = has_jsx_prop_ignore_case(node, "alt") {
        if !is_valid_alt_prop(alt_prop) {
            ctx.diagnostic(missing_alt_value(node.span));
        }
        return;
    }

    if has_jsx_prop_ignore_case(node, "role").map_or(false, is_presentation_role) {
        ctx.diagnostic(prefer_alt(node.span));
        return;
    }

    if let Some(aria_label_prop) = has_jsx_prop_ignore_case(node, "aria-label") {
        if !aria_label_has_value(aria_label_prop) {
            ctx.diagnostic(aria_label_value(node.span));
        }
        return;
    }

    if let Some(aria_labelledby_prop) = has_jsx_prop_ignore_case(node, "aria-labelledby") {
        if !aria_label_has_value(aria_labelledby_prop) {
            ctx.diagnostic(aria_labelled_by_value(node.span));
        }
        return;
    }

    ctx.diagnostic(missing_alt_prop(node.span));
}

fn object_rule<'a>(
    node: &'a JSXOpeningElement<'a>,
    parent: &'a JSXElement<'a>,
    ctx: &LintContext<'a>,
) {
    let has_aria_label =
        has_jsx_prop_ignore_case(node, "aria-label").map_or(false, aria_label_has_value);
    let has_aria_labelledby =
        has_jsx_prop_ignore_case(node, "aria-labelledby").map_or(false, aria_label_has_value);
    let has_label = has_aria_label || has_aria_labelledby;
    let has_title_attr = has_jsx_prop_ignore_case(node, "title")
        .and_then(get_string_literal_prop_value)
        .map_or(false, |v| !v.is_empty());

    if has_label || has_title_attr || object_has_accessible_child(ctx, parent) {
        return;
    }
    ctx.diagnostic(object(node.span));
}

fn area_rule<'a>(node: &'a JSXOpeningElement<'a>, ctx: &LintContext<'a>) {
    let has_aria_label =
        has_jsx_prop_ignore_case(node, "aria-label").map_or(false, aria_label_has_value);
    let has_aria_labelledby =
        has_jsx_prop_ignore_case(node, "aria-labelledby").map_or(false, aria_label_has_value);
    let has_label = has_aria_label || has_aria_labelledby;
    if has_label {
        return;
    }
    has_jsx_prop_ignore_case(node, "alt").map_or_else(
        || {
            ctx.diagnostic(area(node.span));
        },
        |alt_prop| {
            if !is_valid_alt_prop(alt_prop) {
                ctx.diagnostic(area(node.span));
            }
        },
    );
}

fn input_type_image_rule<'a>(node: &'a JSXOpeningElement<'a>, ctx: &LintContext<'a>) {
    let has_aria_label =
        has_jsx_prop_ignore_case(node, "aria-label").map_or(false, aria_label_has_value);
    let has_aria_labelledby =
        has_jsx_prop_ignore_case(node, "aria-labelledby").map_or(false, aria_label_has_value);
    let has_label = has_aria_label || has_aria_labelledby;
    if has_label {
        return;
    }
    has_jsx_prop_ignore_case(node, "alt").map_or_else(
        || {
            ctx.diagnostic(input_type_image(node.span));
        },
        |alt_prop| {
            if !is_valid_alt_prop(alt_prop) {
                ctx.diagnostic(input_type_image(node.span));
            }
        },
    );
}

#[test]
fn test() {
    use crate::tester::Tester;

    fn config() -> serde_json::Value {
        serde_json::json!([{
            "img": ["Thumbnail", "Image"],
            "object": ["Object"],
            "area": ["Area"],
            "input[type=\"image\"]": ["InputImage"],
        }])
    }

    let pass = vec![
        (r#"<img alt="foo" />;"#, None, None),
        (r#"<img alt={"foo"} />;"#, None, None),
        (r"<img alt={alt} />;", None, None),
        (r#"<img ALT="foo" />;"#, None, None),
        (r"<img ALT={`This is the ${alt} text`} />;", None, None),
        (r#"<img ALt="foo" />;"#, None, None),
        (r#"<img alt="foo" salt={undefined} />;"#, None, None),
        (r#"<img {...this.props} alt="foo" />"#, None, None),
        (r"<a />", None, None),
        (r"<div />", None, None),
        (r"<img alt={function(e) {} } />", None, None),
        (r"<div alt={function(e) {} } />", None, None),
        (r"<img alt={() => void 0} />", None, None),
        (r"<IMG />", None, None),
        (r"<UX.Layout>test</UX.Layout>", None, None),
        (r#"<img alt={alt || "Alt text" } />"#, None, None),
        (r"<img alt={photo.caption} />;", None, None),
        (r"<img alt={bar()} />;", None, None),
        (r#"<img alt={foo.bar || ""} />"#, None, None),
        (r#"<img alt={bar() || ""} />"#, None, None),
        (r#"<img alt={foo.bar() || ""} />"#, None, None),
        (r#"<img alt="" />"#, None, None),
        (r"<img alt={`${undefined}`} />", None, None),
        (r#"<img alt=" " />"#, None, None),
        (r#"<img alt="" role="presentation" />"#, None, None),
        (r#"<img alt="" role="none" />"#, None, None),
        (r#"<img alt="" role={`presentation`} />"#, None, None),
        (r#"<img alt="" role={"presentation"} />"#, None, None),
        (r#"<img alt="this is lit..." role="presentation" />"#, None, None),
        (r#"<img alt={error ? "not working": "working"} />"#, None, None),
        (r#"<img alt={undefined ? "working": "not working"} />"#, None, None),
        (r#"<img alt={plugin.name + " Logo"} />"#, None, None),
        (r#"<img aria-label="foo" />"#, None, None),
        (r#"<img aria-labelledby="id1" />"#, None, None),
        (r#"<object aria-label="foo" />"#, None, None),
        (r#"<object aria-labelledby="id1" />"#, None, None),
        (r"<object>Foo</object>", None, None),
        (r"<object><p>This is descriptive!</p></object>", None, None),
        (r"<Object />", None, None),
        (r#"<object title="An object" />"#, None, None),
        (r#"<area aria-label="foo" />"#, None, None),
        (r#"<area aria-labelledby="id1" />"#, None, None),
        (r#"<area alt="" />"#, None, None),
        (r#"<area alt="This is descriptive!" />"#, None, None),
        (r"<area alt={altText} />", None, None),
        (r"<Area />", None, None),
        (r"<input />", None, None),
        (r#"<input type="foo" />"#, None, None),
        (r#"<input type="image" aria-label="foo" />"#, None, None),
        (r#"<input type="image" aria-labelledby="id1" />"#, None, None),
        (r#"<input type="image" alt="" />"#, None, None),
        (r#"<input type="image" alt="This is descriptive!" />"#, None, None),
        (r#"<input type="image" alt={altText} />"#, None, None),
        (r"<InputImage />", None, None),
        (r#"<Input type="image" alt="" />"#, None, None),
        (r#"<SomeComponent as="input" type="image" alt="" />"#, None, None),
        (r#"<Thumbnail alt="foo" />;"#, Some(config()), None),
        (r#"<Thumbnail alt={"foo"} />;"#, Some(config()), None),
        (r"<Thumbnail alt={alt} />;", Some(config()), None),
        (r#"<Thumbnail ALT="foo" />;"#, Some(config()), None),
        (r"<Thumbnail ALT={`This is the ${alt} text`} />;", Some(config()), None),
        (r#"<Thumbnail ALt="foo" />;"#, Some(config()), None),
        (r#"<Thumbnail alt="foo" salt={undefined} />;"#, Some(config()), None),
        (r#"<Thumbnail {...this.props} alt="foo" />"#, Some(config()), None),
        (r"<thumbnail />", Some(config()), None),
        (r"<Thumbnail alt={function(e) {} } />", Some(config()), None),
        (r"<div alt={function(e) {} } />", Some(config()), None),
        (r"<Thumbnail alt={() => void 0} />", Some(config()), None),
        (r"<THUMBNAIL />", Some(config()), None),
        (r#"<Thumbnail alt={alt || "foo" } />"#, Some(config()), None),
        (r#"<Image alt="foo" />;"#, Some(config()), None),
        (r#"<Image alt={"foo"} />;"#, Some(config()), None),
        (r"<Image alt={alt} />;", Some(config()), None),
        (r#"<Image ALT="foo" />;"#, Some(config()), None),
        (r"<Image ALT={`This is the ${alt} text`} />;", Some(config()), None),
        (r#"<Image ALt="foo" />;"#, Some(config()), None),
        (r#"<Image alt="foo" salt={undefined} />;"#, Some(config()), None),
        (r#"<Image {...this.props} alt="foo" />"#, Some(config()), None),
        (r"<image />", Some(config()), None),
        (r"<Image alt={function(e) {} } />", Some(config()), None),
        (r"<div alt={function(e) {} } />", Some(config()), None),
        (r"<Image alt={() => void 0} />", Some(config()), None),
        (r"<IMAGE />", Some(config()), None),
        (r#"<Image alt={alt || "foo" } />"#, Some(config()), None),
        (r#"<Object aria-label="foo" />"#, Some(config()), None),
        (r#"<Object aria-labelledby="id1" />"#, Some(config()), None),
        (r"<Object>Foo</Object>", Some(config()), None),
        (r"<Object><p>This is descriptive!</p></Object>", Some(config()), None),
        (r#"<Object title="An object" />"#, Some(config()), None),
        (r#"<Area aria-label="foo" />"#, Some(config()), None),
        (r#"<Area aria-labelledby="id1" />"#, Some(config()), None),
        (r#"<Area alt="" />"#, Some(config()), None),
        (r#"<Area alt="This is descriptive!" />"#, Some(config()), None),
        (r"<Area alt={altText} />", Some(config()), None),
        (r#"<InputImage aria-label="foo" />"#, Some(config()), None),
        (r#"<InputImage aria-labelledby="id1" />"#, Some(config()), None),
        (r#"<InputImage alt="" />"#, Some(config()), None),
        (r#"<InputImage alt="This is descriptive!" />"#, Some(config()), None),
        (r"<InputImage alt={altText} />", Some(config()), None),
    ];

    let fail = vec![
        (r"<img />;", None, None),
        (r"<img alt />;", None, None),
        (r"<img alt={undefined} />;", None, None),
        (r#"<img src="xyz" />"#, None, None),
        (r"<img role />", None, None),
        (r"<img {...this.props} />", None, None),
        // TODO: Could support if get_prop_value could evaluate
        // some logical expressions
        // (r#"<img alt={false || false} />"#, None, None),
        (r#"<img alt={undefined} role="presentation" />;"#, None, None),
        (r#"<img alt role="presentation" />;"#, None, None),
        (r#"<img role="presentation" />;"#, None, None),
        (r#"<img role="none" />;"#, None, None),
        (r"<img aria-label={undefined} />", None, None),
        (r"<img aria-labelledby={undefined} />", None, None),
        (r#"<img aria-label="" />"#, None, None),
        (r#"<img aria-labelledby="" />"#, None, None),
        (
            r#"<SomeComponent as="img" aria-label="" />"#,
            None,
            Some(
                serde_json::json!({ "settings": { "jsx-a11y": { "polymorphicPropName": "as" } } }),
            ),
        ),
        (r"<object />", None, None),
        (r"<object><div aria-hidden /></object>", None, None),
        (r"<object title={undefined} />", None, None),
        (r#"<object aria-label="" />"#, None, None),
        (r#"<object aria-labelledby="" />"#, None, None),
        (r"<object aria-label={undefined} />", None, None),
        (r"<object aria-labelledby={undefined} />", None, None),
        (r"<area />", None, None),
        (r"<area alt />", None, None),
        (r"<area alt={undefined} />", None, None),
        (r#"<area src="xyz" />"#, None, None),
        (r"<area {...this.props} />", None, None),
        (r#"<area aria-label="" />"#, None, None),
        (r"<area aria-label={undefined} />", None, None),
        (r#"<area aria-labelledby="" />"#, None, None),
        (r"<area aria-labelledby={undefined} />", None, None),
        (r#"<input type="image" />"#, None, None),
        (r#"<input type="image" alt />"#, None, None),
        (r#"<input type="image" alt={undefined} />"#, None, None),
        (r#"<input type="image">Foo</input>"#, None, None),
        (r#"<input type="image" {...this.props} />"#, None, None),
        (r#"<input type="image" aria-label="" />"#, None, None),
        (r#"<input type="image" aria-label={undefined} />"#, None, None),
        (r#"<input type="image" aria-labelledby="" />"#, None, None),
        (r#"<input type="image" aria-labelledby={undefined} />"#, None, None),
        (r"<Thumbnail />;", Some(config()), None),
        (r"<Thumbnail alt />;", Some(config()), None),
        (r"<Thumbnail alt={undefined} />;", Some(config()), None),
        (r#"<Thumbnail src="xyz" />"#, Some(config()), None),
        (r"<Thumbnail {...this.props} />", Some(config()), None),
        (r"<Image />;", Some(config()), None),
        (r"<Image alt />;", Some(config()), None),
        (r"<Image alt={undefined} />;", Some(config()), None),
        (r#"<Image src="xyz" />"#, Some(config()), None),
        (r"<Image {...this.props} />", Some(config()), None),
        (r"<Object />", Some(config()), None),
        (r"<Object><div aria-hidden /></Object>", Some(config()), None),
        (r"<Object title={undefined} />", Some(config()), None),
        (r"<Area />", Some(config()), None),
        (r"<Area alt />", Some(config()), None),
        (r"<Area alt={undefined} />", Some(config()), None),
        (r#"<Area src="xyz" />"#, Some(config()), None),
        (r"<Area {...this.props} />", Some(config()), None),
        (r"<InputImage />", Some(config()), None),
        (r"<InputImage alt />", Some(config()), None),
        (r"<InputImage alt={undefined} />", Some(config()), None),
        (r"<InputImage>Foo</InputImage>", Some(config()), None),
        (r"<InputImage {...this.props} />", Some(config()), None),
        (r#"<Input type="image" />"#, None, None),
    ];

    Tester::new(AltText::NAME, pass, fail).with_jsx_a11y_plugin(true).test_and_snapshot();
}
