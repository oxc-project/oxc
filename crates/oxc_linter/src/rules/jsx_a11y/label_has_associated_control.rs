use std::ops::Deref;

use oxc_ast::{
    AstKind,
    ast::{Expression, JSXAttributeItem, JSXAttributeValue, JSXChild, JSXElement},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_ecmascript::{StringToNumber, ToInt32};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_str::CompactStr;
use oxc_syntax::{
    identifier::is_white_space, line_terminator::is_line_terminator, operator::UnaryOperator,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    AstNode,
    context::LintContext,
    rule::Rule,
    utils::{get_element_type, get_jsx_attribute_name, has_jsx_prop, is_react_component_name},
};

fn label_has_associated_control_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("A form label must be associated with a control.")
        .with_help("Either give the label a `htmlFor` attribute with the id of the associated control, or wrap the label around the control.")
        .with_label(span)
}

fn label_has_associated_control_diagnostic_no_label(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("A form label must have accessible text.")
        .with_help("Ensure the label either has text inside it or is accessibly labelled using an attribute such as `aria-label`, or `aria-labelledby`. You can mark more attributes as accessible labels by configuring the `labelAttributes` option.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct LabelHasAssociatedControl(Box<LabelHasAssociatedControlConfig>);

const DEFAULT_CONTROL_COMPONENTS: [&str; 6] =
    ["input", "meter", "output", "progress", "select", "textarea"];

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct LabelHasAssociatedControlConfig {
    /// Maximum depth to search for a nested control.
    depth: u8,
    /// The type of association required between the label and the control.
    assert: Assert,
    /// Custom JSX components to be treated as labels.
    label_components: Vec<CompactStr>,
    /// Attributes to check for accessible label text.
    label_attributes: Vec<CompactStr>,
    /// Custom JSX components to be treated as form controls.
    control_components: Vec<CompactStr>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
enum Assert {
    /// Assert that the label uses `htmlFor` to associate a control.
    HtmlFor,
    /// Assert that the label has a nested control
    Nesting,
    /// Assert that the label uses both `htmlFor` and nesting for associating a control
    Both,
    /// Assert that the label uses either `htmlFor` or nesting for associating a control
    #[default]
    Either,
}

impl Deref for LabelHasAssociatedControl {
    type Target = LabelHasAssociatedControlConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for LabelHasAssociatedControlConfig {
    fn default() -> Self {
        Self {
            depth: 2,
            assert: Assert::Either,
            label_components: vec!["label".into()],
            label_attributes: vec!["alt".into(), "aria-label".into(), "aria-labelledby".into()],
            control_components: Vec::default(),
        }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce that a label tag has a text label and an associated control.
    ///
    /// ### Why is this bad?
    ///
    /// A form label that either isn't properly associated with a form control (such as an `<input>`), or doesn't contain accessible text, hinders accessibility for users using assistive technologies such as screen readers. The user may not have enough information to understand the purpose of the form control.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// function Foo(props) {
    ///     return <label {...props} />
    /// }
    ///
    /// <input type="text" />
    /// <label>Surname</label>
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// function Foo(props) {
    ///     const {
    ///         htmlFor,
    ///         ...otherProps
    ///     } = props;
    ///
    ///     return <label htmlFor={htmlFor} {...otherProps} />
    /// }
    ///
    /// <label>
    ///     <input type="text" />
    ///     Surname
    /// </label>
    /// ```
    LabelHasAssociatedControl,
    jsx_a11y,
    correctness,
    config = LabelHasAssociatedControlConfig,
    version = "0.9.1",
    short_description = "Enforce that a label tag has a text label and an associated control.",
);

impl Rule for LabelHasAssociatedControl {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        let mut config = LabelHasAssociatedControlConfig::default();

        let Some(options) = value.get(0) else {
            return Ok(Self(Box::new(config)));
        };

        if let Some(depth) = options.get("depth").and_then(serde_json::Value::as_u64) {
            config.depth = std::cmp::min(depth, 25).try_into().unwrap();
        }

        if let Some(assert) = options.get("assert").and_then(serde_json::Value::as_str) {
            config.assert = match assert {
                "htmlFor" => Assert::HtmlFor,
                "nesting" => Assert::Nesting,
                "both" => Assert::Both,
                _ => Assert::Either,
            };
        }

        if let Some(label_components) =
            options.get("labelComponents").and_then(serde_json::Value::as_array)
            && let Some(mut components) = label_components
                .iter()
                .map(serde_json::Value::as_str)
                .map(|component| component.map(CompactStr::from))
                .collect::<Option<Vec<CompactStr>>>()
        {
            config.label_components.append(&mut components);
        }

        if let Some(label_attributes) =
            options.get("labelAttributes").and_then(serde_json::Value::as_array)
            && let Some(mut attributes) = label_attributes
                .iter()
                .map(serde_json::Value::as_str)
                .map(|attribute| attribute.map(CompactStr::from))
                .collect::<Option<Vec<CompactStr>>>()
        {
            config.label_attributes.append(&mut attributes);
        }

        if let Some(control_components) =
            options.get("controlComponents").and_then(serde_json::Value::as_array)
        {
            config.control_components = control_components
                .iter()
                .map(serde_json::Value::as_str)
                .filter_map(|component| component.map(CompactStr::from))
                .collect::<Vec<CompactStr>>();
        }

        config.label_components.sort_unstable();
        config.label_components.dedup();

        config.label_attributes.sort_unstable();
        config.label_attributes.dedup();

        Ok(Self(Box::new(config)))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXElement(element) = node.kind() else {
            return;
        };

        let element_type = get_element_type(ctx, &element.opening_element);

        if self.label_components.binary_search(&element_type.into()).is_err() {
            return;
        }

        let has_html_for = if let Some(attributes) = ctx.settings().jsx_a11y.attributes.get("for") {
            attributes.iter().any(|attr| {
                has_jsx_prop(&element.opening_element, attr.as_str()).is_some_and(|attr| {
                    attr.as_attribute()
                        .is_some_and(|attr| has_attribute_value(attr.value.as_ref(), false))
                })
            })
        } else {
            has_jsx_prop(&element.opening_element, "htmlFor").is_some_and(|attr| {
                attr.as_attribute()
                    .is_some_and(|attr| has_attribute_value(attr.value.as_ref(), false))
            })
        };

        let has_control = self.has_nested_control(element, ctx);

        if !self.has_accessible_label(element, ctx) {
            ctx.diagnostic(label_has_associated_control_diagnostic_no_label(
                element.opening_element.span,
            ));
            return;
        }

        match self.assert {
            Assert::HtmlFor => {
                if has_html_for {
                    return;
                }
            }
            Assert::Nesting => {
                if has_control {
                    return;
                }
            }
            Assert::Both => {
                if has_html_for && has_control {
                    return;
                }
            }
            Assert::Either => {
                if has_html_for || has_control {
                    return;
                }
            }
        }

        ctx.diagnostic(label_has_associated_control_diagnostic(element.opening_element.span));
    }
}

impl LabelHasAssociatedControl {
    fn is_match_control_components(&self, name: &str) -> bool {
        DEFAULT_CONTROL_COMPONENTS.contains(&name)
            || self
                .control_components
                .iter()
                .any(|component| fast_glob::glob_match(component.as_str(), name))
    }

    fn has_accessible_label<'a>(&self, root: &JSXElement<'a>, ctx: &LintContext<'a>) -> bool {
        if self.has_labelling_prop(root) {
            return true;
        }

        for child in &root.children {
            if self.search_for_accessible_label(child, 1, ctx) {
                return true;
            }
        }

        false
    }

    fn has_labelling_prop(&self, element: &JSXElement<'_>) -> bool {
        element.opening_element.attributes.iter().any(|attribute| match attribute {
            JSXAttributeItem::Attribute(attr) => {
                let attr_name = get_jsx_attribute_name(&attr.name);
                self.label_attributes.binary_search(&attr_name.into()).is_ok()
                    && has_attribute_value(attr.value.as_ref(), true)
            }
            JSXAttributeItem::SpreadAttribute(_) => true,
        })
    }

    fn has_nested_control<'a>(&self, root: &JSXElement<'a>, ctx: &LintContext<'a>) -> bool {
        for child in &root.children {
            if self.search_for_nested_control(child, 1, ctx) {
                return true;
            }
        }

        false
    }

    fn search_for_nested_control<'a>(
        &self,
        node: &JSXChild<'a>,
        depth: u8,
        ctx: &LintContext<'a>,
    ) -> bool {
        if depth > self.depth {
            return false;
        }

        match node {
            JSXChild::ExpressionContainer(_) => true,
            JSXChild::Element(element) => {
                let element_type = get_element_type(ctx, &element.opening_element);
                if self.is_match_control_components(element_type.as_ref()) {
                    return true;
                }

                for child in &element.children {
                    if self.search_for_nested_control(child, depth + 1, ctx) {
                        return true;
                    }
                }

                false
            }
            JSXChild::Fragment(fragment) => {
                for child in &fragment.children {
                    if self.search_for_nested_control(child, depth + 1, ctx) {
                        return true;
                    }
                }

                false
            }
            JSXChild::Text(_) | JSXChild::Spread(_) => false,
        }
    }

    fn search_for_accessible_label<'a>(
        &self,
        node: &JSXChild<'a>,
        depth: u8,
        ctx: &LintContext<'a>,
    ) -> bool {
        if depth > self.depth {
            return false;
        }

        match node {
            JSXChild::ExpressionContainer(_) => true,
            JSXChild::Text(text) => !text.value.as_str().trim().is_empty(),
            JSXChild::Element(element) => {
                if self.has_labelling_prop(element) {
                    return true;
                }

                if element.children.is_empty() {
                    let name = get_element_type(ctx, &element.opening_element);
                    if is_react_component_name(&name)
                        && !self.is_match_control_components(name.as_ref())
                    {
                        return true;
                    }
                }

                for child in &element.children {
                    if self.search_for_accessible_label(child, depth + 1, ctx) {
                        return true;
                    }
                }

                false
            }
            JSXChild::Fragment(fragment) => {
                for child in &fragment.children {
                    if self.search_for_accessible_label(child, depth + 1, ctx) {
                        return true;
                    }
                }

                false
            }
            JSXChild::Spread(_) => false,
        }
    }
}

fn has_attribute_value(value: Option<&JSXAttributeValue<'_>>, trim_strings: bool) -> bool {
    match value {
        Some(JSXAttributeValue::StringLiteral(literal)) => {
            AttributeValue::from_literal(&literal.value).has_value(trim_strings)
        }
        Some(JSXAttributeValue::ExpressionContainer(container)) => container
            .expression
            .as_expression()
            .and_then(get_attribute_expression_value)
            .is_none_or(|value| value.has_value(trim_strings)),
        // Bare attributes and unknown values may provide a label, as in the upstream rule.
        _ => true,
    }
}

// Match jsx-ast-utils' primitive value extraction rather than runtime string truthiness.
#[derive(Clone, Copy)]
enum AttributeValue<'a> {
    String(&'a str),
    Number(f64),
    BigInt(bool),
}

impl<'a> AttributeValue<'a> {
    fn from_literal(text: &'a str) -> Self {
        if text.eq_ignore_ascii_case("false") {
            Self::Number(0.0)
        } else if text.eq_ignore_ascii_case("true") {
            Self::Number(1.0)
        } else {
            Self::String(text)
        }
    }

    fn has_value(self, trim_strings: bool) -> bool {
        match self {
            Self::String(text) => {
                let text = if trim_strings {
                    text.trim_matches(|c| is_white_space(c) || is_line_terminator(c))
                } else {
                    text
                };
                !text.is_empty()
            }
            Self::Number(value) => value != 0.0 && !value.is_nan(),
            Self::BigInt(nonzero) => nonzero,
        }
    }

    fn to_number(self) -> Option<f64> {
        match self {
            Self::String(text) => Some(text.string_to_number()),
            Self::Number(value) => Some(value),
            Self::BigInt(_) => None,
        }
    }
}

fn get_attribute_expression_value<'a>(
    expression: &'a Expression<'_>,
) -> Option<AttributeValue<'a>> {
    let value = match expression.get_inner_expression() {
        Expression::StringLiteral(literal) => AttributeValue::from_literal(&literal.value),
        Expression::TemplateLiteral(template) if template.expressions.is_empty() => {
            // jsx-ast-utils uses raw template text, including escape sequences.
            AttributeValue::String(template.quasis.first()?.value.raw.as_str())
        }
        Expression::NullLiteral(_) => AttributeValue::Number(0.0),
        Expression::BooleanLiteral(literal) => AttributeValue::Number(f64::from(literal.value)),
        Expression::NumericLiteral(literal) => AttributeValue::Number(literal.value),
        Expression::BigIntLiteral(literal) => AttributeValue::BigInt(!literal.is_zero()),
        Expression::Identifier(identifier) => match identifier.name.as_str() {
            "undefined" => AttributeValue::Number(f64::NAN),
            "Infinity" => AttributeValue::Number(f64::INFINITY),
            // Upstream represents unknown identifiers by their name.
            name => AttributeValue::String(name),
        },
        Expression::UnaryExpression(unary) => {
            let value = match unary.operator {
                // jsx-ast-utils returns undefined for both operators.
                UnaryOperator::Void | UnaryOperator::Typeof => f64::NAN,
                UnaryOperator::Delete => 1.0,
                operator => {
                    let argument = get_attribute_expression_value(&unary.argument)?;
                    if unary.operator == UnaryOperator::UnaryNegation
                        && matches!(argument, AttributeValue::BigInt(_))
                    {
                        return Some(argument);
                    }
                    match operator {
                        UnaryOperator::UnaryNegation => -argument.to_number()?,
                        UnaryOperator::UnaryPlus => argument.to_number()?,
                        UnaryOperator::LogicalNot => f64::from(!argument.has_value(false)),
                        UnaryOperator::BitwiseNot => f64::from(!argument.to_number()?.to_int_32()),
                        _ => unreachable!(),
                    }
                }
            };
            AttributeValue::Number(value)
        }
        _ => return None,
    };
    Some(value)
}

#[test]
fn test() {
    use crate::tester::Tester;

    fn component_settings() -> serde_json::Value {
        serde_json::json!({
            "settings": {
                "jsx-a11y": {
                    "components": {
                        "CustomInput": "input",
                        "CustomLabel": "label",
                    }
                }
            }
        })
    }

    fn attributes_settings() -> serde_json::Value {
        serde_json::json!({
            "settings": {
                "jsx-a11y": {
                    "attributes": {
                        "for": ["htmlFor", "for"]
                    }
                }
            }
        })
    }

    let pass = vec![
        (
            r#"<label htmlFor="js_id"><span><span><span>A label</span></span></span></label>"#,
            Some(serde_json::json!([{ "depth": 4, "assert": "htmlFor" }])),
            None,
        ),
        (
            r#"<label htmlFor="js_id" aria-label="A label" />"#,
            Some(serde_json::json!([{ "assert": "htmlFor" }])),
            None,
        ),
        (
            r#"<label htmlFor="js_id" aria-labelledby="A label" />"#,
            Some(serde_json::json!([{ "assert": "htmlFor" }])),
            None,
        ),
        (
            r#"<div><label htmlFor="js_id">A label</label><input id="js_id" /></div>"#,
            Some(serde_json::json!([{ "assert": "htmlFor" }])),
            None,
        ),
        (
            r#"<CustomLabel htmlFor="js_id" aria-label="A label" />"#,
            Some(serde_json::json!([{ "labelComponents": ["CustomLabel"], "assert": "htmlFor" }])),
            None,
        ),
        (
            r#"<CustomLabel htmlFor="js_id" label="A label" />"#,
            Some(serde_json::json!([{
                "labelComponents": ["CustomLabel"],
                "labelAttributes": ["label"],
                "assert": "htmlFor"
            }])),
            None,
        ),
        (
            r#"<CustomLabel htmlFor="js_id" aria-label="A label" />"#,
            Some(serde_json::json!([{ "assert": "htmlFor" }])),
            Some(component_settings()),
        ),
        (
            r#"<label htmlFor="js_id" label="A label" />"#,
            Some(serde_json::json!([{ "labelAttributes": ["label"], "assert": "htmlFor" }])),
            None,
        ),
        (
            r#"<CustomLabel htmlFor="js_id" aria-label="A label" />"#,
            Some(serde_json::json!([{ "controlComponents": ["Custom*"], "assert": "htmlFor" }])),
            None,
        ),
        (
            r#"<CustomLabel htmlFor="js_id" aria-label="A label" />"#,
            Some(serde_json::json!([{ "controlComponents": ["*Label"], "assert": "htmlFor" }])),
            None,
        ),
        (
            r#"<label htmlFor="js_id"><span><span><span>A label</span></span></span></label>"#,
            Some(serde_json::json!([{ "depth": 4, "assert": "either" }])),
            None,
        ),
        (
            r#"<label htmlFor="js_id" aria-label="A label" />"#,
            Some(serde_json::json!([{ "assert": "either" }])),
            None,
        ),
        (
            r#"<label htmlFor="js_id" aria-labelledby="A label" />"#,
            Some(serde_json::json!([{ "assert": "either" }])),
            None,
        ),
        (
            r#"<div><label htmlFor="js_id">A label</label><input id="js_id" /></div>"#,
            Some(serde_json::json!([{ "assert": "either" }])),
            None,
        ),
        (
            r#"<CustomLabel htmlFor="js_id" aria-label="A label" />"#,
            Some(serde_json::json!([{ "labelComponents": ["CustomLabel"], "assert": "either" }])),
            None,
        ),
        (
            r#"<CustomLabel htmlFor="js_id" label="A label" />"#,
            Some(serde_json::json!([{
                "labelComponents": ["CustomLabel"],
                "labelAttributes": ["label"],
                "assert": "either"
            }])),
            None,
        ),
        (
            r#"<CustomLabel htmlFor="js_id" aria-label="A label" />"#,
            Some(serde_json::json!([{ "assert": "either" }])),
            Some(component_settings()),
        ),
        (
            r#"<label htmlFor="js_id" label="A label" />"#,
            Some(serde_json::json!([{ "labelAttributes": ["label"], "assert": "either" }])),
            None,
        ),
        (
            r#"<CustomLabel htmlFor="js_id" aria-label="A label" />"#,
            Some(serde_json::json!([{ "controlComponents": ["Custom*"], "assert": "either" }])),
            None,
        ),
        (
            r#"<CustomLabel htmlFor="js_id" aria-label="A label" />"#,
            Some(serde_json::json!([{ "controlComponents": ["*Label"], "assert": "either" }])),
            None,
        ),
        (
            "<label>A label<input /></label>",
            Some(serde_json::json!([{ "assert": "nesting" }])),
            None,
        ),
        (
            "<label>A label<textarea /></label>",
            Some(serde_json::json!([{ "assert": "nesting" }])),
            None,
        ),
        (
            r#"<label><img alt="A label" /><input /></label>"#,
            Some(serde_json::json!([{ "assert": "nesting" }])),
            None,
        ),
        (
            r#"<label><img aria-label="A label" /><input /></label>"#,
            Some(serde_json::json!([{ "assert": "nesting" }])),
            None,
        ),
        (
            "<label><span>A label<input /></span></label>",
            Some(serde_json::json!([{ "assert": "nesting" }])),
            None,
        ),
        (
            "<label><span><span>A label<input /></span></span></label>",
            Some(serde_json::json!([{ "assert": "nesting", "depth": 3 }])),
            None,
        ),
        (
            "<label><span><span><span>A label<input /></span></span></span></label>",
            Some(serde_json::json!([{ "assert": "nesting", "depth": 4 }])),
            None,
        ),
        (
            "<label><span><span><span><span>A label</span><input /></span></span></span></label>",
            Some(serde_json::json!([{ "assert": "nesting", "depth": 5 }])),
            None,
        ),
        (
            r#"<label><span><span><span><span aria-label="A label" /><input /></span></span></span></label>"#,
            Some(serde_json::json!([{ "assert": "nesting", "depth": 5 }])),
            None,
        ),
        (
            r#"<label><span><span><span><input aria-label="A label" /></span></span></span></label>"#,
            Some(serde_json::json!([{ "assert": "nesting", "depth": 5 }])),
            None,
        ),
        ("<label>foo<meter /></label>", Some(serde_json::json!([{ "assert": "nesting" }])), None),
        ("<label>foo<output /></label>", Some(serde_json::json!([{ "assert": "nesting" }])), None),
        (
            "<label>foo<progress /></label>",
            Some(serde_json::json!([{ "assert": "nesting" }])),
            None,
        ),
        (
            "<label>foo<textarea /></label>",
            Some(serde_json::json!([{ "assert": "nesting" }])),
            None,
        ),
        (
            "<label><span>A label<CustomInput /></span></label>",
            Some(
                serde_json::json!([{ "assert": "nesting", "controlComponents": ["CustomInput"] }]),
            ),
            None,
        ),
        (
            "<label><span>A label<CustomInput /></span></label>",
            Some(serde_json::json!([{ "assert": "nesting" }])),
            Some(component_settings()),
        ),
        (
            "<CustomLabel><span>A label<CustomInput /></span></CustomLabel>",
            Some(
                serde_json::json!([{ "assert": "nesting", "controlComponents": ["CustomInput"], "labelComponents": ["CustomLabel"] }]),
            ),
            None,
        ),
        (
            r#"<CustomLabel><span label="A label"><CustomInput /></span></CustomLabel>"#,
            Some(serde_json::json!([{
                "assert": "nesting",
                "controlComponents": ["CustomInput"],
                "labelComponents": ["CustomLabel"],
                "labelAttributes": ["label"]
            }])),
            None,
        ),
        (
            "<label><span>A label<CustomInput /></span></label>",
            Some(serde_json::json!([{
                "assert": "nesting",
                "controlComponents": ["Custom*"],
            }])),
            None,
        ),
        (
            "<label><span>A label<CustomInput /></span></label>",
            Some(serde_json::json!([{
                "assert": "nesting",
                "controlComponents": ["*Input"],
            }])),
            None,
        ),
        (
            "<label>A label<input /></label>",
            Some(serde_json::json!([{ "assert": "either" }])),
            None,
        ),
        (
            "<label>A label<textarea /></label>",
            Some(serde_json::json!([{ "assert": "either" }])),
            None,
        ),
        (
            r#"<label><img alt="A label" /><input /></label>"#,
            Some(serde_json::json!([{ "assert": "either" }])),
            None,
        ),
        (
            r#"<label><img aria-label="A label" /><input /></label>"#,
            Some(serde_json::json!([{ "assert": "either" }])),
            None,
        ),
        (
            "<label><span>A label<input /></span></label>",
            Some(serde_json::json!([{ "assert": "either" }])),
            None,
        ),
        (
            "<label><span><span>A label<input /></span></span></label>",
            Some(serde_json::json!([{ "assert": "either", "depth": 3 }])),
            None,
        ),
        (
            "<label><span><span><span>A label<input /></span></span></span></label>",
            Some(serde_json::json!([{ "assert": "either", "depth": 4 }])),
            None,
        ),
        (
            "<label><span><span><span><span>A label</span><input /></span></span></span></label>",
            Some(serde_json::json!([{ "assert": "either", "depth": 5 }])),
            None,
        ),
        (
            r#"<label><span><span><span><span aria-label="A label" /><input /></span></span></span></label>"#,
            Some(serde_json::json!([{ "assert": "either", "depth": 5 }])),
            None,
        ),
        (
            r#"<label><span><span><span><input aria-label="A label" /></span></span></span></label>"#,
            Some(serde_json::json!([{ "assert": "either", "depth": 5 }])),
            None,
        ),
        ("<label>foo<meter /></label>", Some(serde_json::json!([{ "assert": "either" }])), None),
        ("<label>foo<output /></label>", Some(serde_json::json!([{ "assert": "either" }])), None),
        ("<label>foo<progress /></label>", Some(serde_json::json!([{ "assert": "either" }])), None),
        ("<label>foo<textarea /></label>", Some(serde_json::json!([{ "assert": "either" }])), None),
        (
            "<label><span>A label<CustomInput /></span></label>",
            Some(serde_json::json!([{ "assert": "either", "controlComponents": ["CustomInput"] }])),
            None,
        ),
        (
            "<label><span>A label<CustomInput /></span></label>",
            Some(serde_json::json!([{ "assert": "either" }])),
            Some(component_settings()),
        ),
        (
            "<CustomLabel><span>A label<CustomInput /></span></CustomLabel>",
            Some(
                serde_json::json!([{ "assert": "either", "controlComponents": ["CustomInput"], "labelComponents": ["CustomLabel"] }]),
            ),
            None,
        ),
        (
            r#"<CustomLabel><span label="A label"><CustomInput /></span></CustomLabel>"#,
            Some(serde_json::json!([{
                "assert": "either",
                "controlComponents": ["CustomInput"],
                "labelComponents": ["CustomLabel"],
                "labelAttributes": ["label"]
            }])),
            None,
        ),
        (
            "<label><span>A label<CustomInput /></span></label>",
            Some(serde_json::json!([{
                "assert": "either",
                "controlComponents": ["Custom*"],
            }])),
            None,
        ),
        (
            "<label><span>A label<CustomInput /></span></label>",
            Some(serde_json::json!([{
                "assert": "either",
                "controlComponents": ["*Input"],
            }])),
            None,
        ),
        (
            r#"<label htmlFor="js_id"><span><span><span>A label<input /></span></span></span></label>"#,
            Some(serde_json::json!([{
                "assert": "both",
                "depth": 4
            }])),
            None,
        ),
        (
            r#"<label htmlFor="js_id" aria-label="A label"><input /></label>"#,
            Some(serde_json::json!([{
                "assert": "both",
            }])),
            None,
        ),
        (
            r#"<label htmlFor="js_id" aria-labelledby="A label"><input /></label>"#,
            Some(serde_json::json!([{
                "assert": "both",
            }])),
            None,
        ),
        (
            r#"<label htmlFor="js_id" aria-labelledby="A label"><textarea /></label>"#,
            Some(serde_json::json!([{
                "assert": "both",
            }])),
            None,
        ),
        (
            r#"<CustomLabel htmlFor="js_id" aria-label="A label"><input /></CustomLabel>"#,
            Some(serde_json::json!([{
                "assert": "both",
                "labelComponents": ["CustomLabel"]
            }])),
            None,
        ),
        (
            r#"<CustomLabel htmlFor="js_id" label="A label"><input /></CustomLabel>"#,
            Some(serde_json::json!([{
                "assert": "both",
                "labelAttributes": ["label"],
                "labelComponents": ["CustomLabel"]
            }])),
            None,
        ),
        (
            r#"<CustomLabel htmlFor="js_id" aria-label="A label"><input /></CustomLabel>"#,
            Some(serde_json::json!([{
                "assert": "both",
            }])),
            Some(component_settings()),
        ),
        (
            r#"<CustomLabel htmlFor="js_id" aria-label="A label"><CustomInput /></CustomLabel>"#,
            Some(serde_json::json!([{
                "assert": "both",
            }])),
            Some(component_settings()),
        ),
        (
            r#"<label htmlFor="js_id" label="A label"><input /></label>"#,
            Some(serde_json::json!([{
                "assert": "both",
                "labelAttributes": ["label"],
            }])),
            None,
        ),
        (
            r#"<label htmlFor="selectInput">Some text<select id="selectInput" /></label>"#,
            Some(serde_json::json!([{
                "assert": "both",
            }])),
            None,
        ),
        (
            "<div />",
            Some(serde_json::json!([{
                "assert": "htmlFor",
            }])),
            None,
        ),
        (
            "<div />",
            Some(serde_json::json!([{
                "assert": "nesting",
            }])),
            None,
        ),
        (
            "<div />",
            Some(serde_json::json!([{
                "assert": "both",
            }])),
            None,
        ),
        (
            "<div />",
            Some(serde_json::json!([{
                "assert": "either",
            }])),
            None,
        ),
        (
            "<CustomElement />",
            Some(serde_json::json!([{
                "assert": "htmlFor",
            }])),
            None,
        ),
        (
            "<CustomElement />",
            Some(serde_json::json!([{
                "assert": "nesting",
            }])),
            None,
        ),
        (
            "<CustomElement />",
            Some(serde_json::json!([{
                "assert": "both",
            }])),
            None,
        ),
        (
            "<CustomElement />",
            Some(serde_json::json!([{
                "assert": "either",
            }])),
            None,
        ),
        (
            r#"<input type="hidden" />"#,
            Some(serde_json::json!([{
                "assert": "htmlFor",
            }])),
            None,
        ),
        (
            r#"<input type="hidden" />"#,
            Some(serde_json::json!([{
                "assert": "nesting",
            }])),
            None,
        ),
        (
            r#"<input type="hidden" />"#,
            Some(serde_json::json!([{
                "assert": "both",
            }])),
            None,
        ),
        (
            r#"<input type="hidden" />"#,
            Some(serde_json::json!([{
                "assert": "either",
            }])),
            None,
        ),
        (
            r#"<div><label htmlFor="js_id"><CustomText /></label><input id="js_id" /></div>"#,
            Some(serde_json::json!([{
                "assert": "htmlFor",
            }])),
            None,
        ),
        (
            "<label><CustomText /><input /></label>",
            Some(serde_json::json!([{
                "assert": "nesting",
            }])),
            None,
        ),
        (
            r#"<div><label htmlFor="js_id"><CustomText /></label><input id="js_id" /></div>"#,
            Some(serde_json::json!([{
                "assert": "either",
            }])),
            None,
        ),
        (
            "<label><CustomText /><input /></label>",
            Some(serde_json::json!([{
                "assert": "either",
            }])),
            None,
        ),
        // ensure `labelAttributes` is sorted for binary search
        (
            r#"<CustomLabel htmlFor="js_id" label="A label" />"#,
            Some(serde_json::json!([{
                "labelComponents": ["CustomLabel"],
                "labelAttributes": ["zzzlabel", "nnnlabel", "label"],
                "assert": "htmlFor"
            }])),
            None,
        ),
        // Issue: <https://github.com/oxc-project/oxc/issues/7849>
        ("<FilesContext.Provider value={{ addAlert, cwdInfo }} />", None, None),
        // Issue: <https://github.com/oxc-project/oxc/issues/11644>
        (
            r#"<label>
                <span>name</span>
                <input
                    defaultValue={
                    actionState.payload?.get("name")?.toString() ?? "Pool party"
                    }
                    name="name"
                    placeholder="name"
                    type="text"
                />
               </label>"#,
            None,
            None,
        ),
        // Test for 'for' attribute with attributes setting
        (
            r#"<label for="js_id">A label</label>"#,
            Some(serde_json::json!([{ "assert": "htmlFor" }])),
            Some(attributes_settings()),
        ),
        (
            r#"<label for="js_id" aria-label="A label" />"#,
            Some(serde_json::json!([{ "assert": "htmlFor" }])),
            Some(attributes_settings()),
        ),
        (
            r#"<label for="js_id">A label</label>"#,
            Some(serde_json::json!([{ "assert": "either" }])),
            Some(attributes_settings()),
        ),
        (
            r#"<label for="js_id" aria-label="A label" />"#,
            Some(serde_json::json!([{ "assert": "either" }])),
            Some(attributes_settings()),
        ),
        (
            r#"<label for="js_id" aria-label="A label"><input /></label>"#,
            Some(serde_json::json!([{ "assert": "both" }])),
            Some(attributes_settings()),
        ),
    ];

    let fail = vec![
        (
            r#"<label htmlFor="js_id"><span><span><span>A label</span></span></span></label>"#,
            Some(serde_json::json!([{
                "assert": "nesting",
                "depth": 4
            }])),
            None,
        ),
        (
            r#"<label htmlFor="js_id" aria-label="A label" />"#,
            Some(serde_json::json!([{
                "assert": "nesting",
            }])),
            None,
        ),
        (
            r#"<label htmlFor="js_id" aria-labelledby="A label" />"#,
            Some(serde_json::json!([{
                "assert": "nesting",
            }])),
            None,
        ),
        (
            r#"<CustomLabel htmlFor="js_id" aria-label="A label" />"#,
            Some(serde_json::json!([{
                "assert": "nesting",
                "labelComponents": ["CustomLabel"]
            }])),
            None,
        ),
        (
            r#"<CustomLabel htmlFor="js_id" label="A label" />"#,
            Some(serde_json::json!([{
                "assert": "nesting",
                "labelAttributes": ["label"],
                "labelComponents": ["CustomLabel"]
            }])),
            None,
        ),
        (
            r#"<CustomLabel htmlFor="js_id" aria-label="A label" />"#,
            Some(serde_json::json!([{
                "assert": "nesting",
            }])),
            Some(component_settings()),
        ),
        (
            r#"<label htmlFor="js_id" label="A label" />"#,
            Some(serde_json::json!([{
                "assert": "nesting",
                "labelAttributes": ["label"],
            }])),
            None,
        ),
        (
            "<label>A label<input /></label>",
            Some(serde_json::json!([{
                "assert": "htmlFor",
            }])),
            None,
        ),
        (
            "<label>A label<textarea /></label>",
            Some(serde_json::json!([{
                "assert": "htmlFor",
            }])),
            None,
        ),
        (
            r#"<label><img alt="A label" /><input /></label>"#,
            Some(serde_json::json!([{
                "assert": "htmlFor",
            }])),
            None,
        ),
        (
            r#"<label><img aria-label="A label" /><input /></label>"#,
            Some(serde_json::json!([{
                "assert": "htmlFor",
            }])),
            None,
        ),
        (
            "<label><span>A label<input /></span></label>",
            Some(serde_json::json!([{
                "assert": "htmlFor",
            }])),
            None,
        ),
        (
            "<label><span><span>A label<input /></span></span></label>",
            Some(serde_json::json!([{
                "assert": "htmlFor",
                "depth": 3
            }])),
            None,
        ),
        (
            "<label><span><span><span>A label<input /></span></span></span></label>\'",
            Some(serde_json::json!([{
                "assert": "htmlFor",
                "depth": 4
            }])),
            None,
        ),
        (
            "<label><span><span><span><span>A label</span><input /></span></span></span></label>",
            Some(serde_json::json!([{
                "assert": "htmlFor",
                "depth": 5
            }])),
            None,
        ),
        (
            r#"<label><span><span><span><span aria-label="A label" /><input /></span></span></span></label>"#,
            Some(serde_json::json!([{
                "assert": "htmlFor",
                "depth": 5
            }])),
            None,
        ),
        (
            r#"<label><span><span><span><input aria-label="A label" /></span></span></span></label>"#,
            Some(serde_json::json!([{
                "assert": "htmlFor",
                "depth": 5
            }])),
            None,
        ),
        (
            "<label><span>A label<CustomInput /></span></label>",
            Some(serde_json::json!([{
                "assert": "htmlFor",
                "controlComponents": ["CustomInput"]
            }])),
            None,
        ),
        (
            "<CustomLabel><span>A label<CustomInput /></span></CustomLabel>",
            Some(serde_json::json!([{
                "assert": "htmlFor",
                "controlComponents": ["CustomInput"],
                "labelComponents": ["CustomLabel"]
            }])),
            None,
        ),
        (
            r#"<CustomLabel><span label="A label"><CustomInput /></span></CustomLabel>"#,
            Some(serde_json::json!([{
                "assert": "htmlFor",
                "controlComponents": ["CustomInput"],
                "labelComponents": ["CustomLabel"],
                "labelAttributes": ["label"]
            }])),
            None,
        ),
        (
            "<label><span>A label<CustomInput /></span></label>",
            Some(serde_json::json!([{
                "assert": "htmlFor",
            }])),
            Some(component_settings()),
        ),
        (
            "<CustomLabel><span>A label<CustomInput /></span></CustomLabel>",
            Some(serde_json::json!([{
                "assert": "htmlFor",
            }])),
            Some(component_settings()),
        ),
        (
            r#"<label htmlFor="js_id" />"#,
            Some(serde_json::json!([{
                "assert": "htmlFor",
            }])),
            None,
        ),
        (
            r#"<label htmlFor="js_id"><input /></label>"#,
            Some(serde_json::json!([{
                "assert": "htmlFor",
            }])),
            None,
        ),
        (
            r#"<label htmlFor="js_id"><textarea /></label>"#,
            Some(serde_json::json!([{
                "assert": "htmlFor",
            }])),
            None,
        ),
        (
            "<label></label>",
            Some(serde_json::json!([{
                "assert": "htmlFor",
            }])),
            None,
        ),
        (
            "<label>A label</label>",
            Some(serde_json::json!([{
                "assert": "htmlFor",
            }])),
            None,
        ),
        (
            "<div><label /><input /></div>",
            Some(serde_json::json!([{
                "assert": "htmlFor",
            }])),
            None,
        ),
        (
            "<div><label>A label</label><input /></div>",
            Some(serde_json::json!([{
                "assert": "htmlFor",
            }])),
            None,
        ),
        (
            r#"<CustomLabel aria-label="A label" />"#,
            Some(serde_json::json!([{
                "assert": "htmlFor",
                "labelComponents": ["CustomLabel"]
            }])),
            None,
        ),
        (
            r#"<CustomLabel label="A label" />"#,
            Some(serde_json::json!([{
                "assert": "htmlFor",
                "labelComponents": ["CustomLabel"],
                "labelAttributes": ["label"]
            }])),
            None,
        ),
        (
            r#"<CustomLabel aria-label="A label" />"#,
            Some(serde_json::json!([{
                "assert": "htmlFor",
            }])),
            Some(component_settings()),
        ),
        (
            r#"<label label="A label" />"#,
            Some(serde_json::json!([{
                "assert": "htmlFor",
                "labelAttributes": ["label"]
            }])),
            None,
        ),
        (
            "<label><span><CustomInput /></span></label>",
            Some(serde_json::json!([{
                "assert": "htmlFor",
                "controlComponents": ["CustomInput"]
            }])),
            None,
        ),
        (
            "<CustomLabel><span><CustomInput /></span></CustomLabel>",
            Some(serde_json::json!([{
                "assert": "htmlFor",
                "controlComponents": ["CustomInput"],
                "labelComponents": ["CustomLabel"],
            }])),
            None,
        ),
        (
            "<CustomLabel><span><CustomInput /></span></CustomLabel>",
            Some(serde_json::json!([{
                "assert": "htmlFor",
                "controlComponents": ["CustomInput"],
                "labelComponents": ["CustomLabel"],
                "labelAttributes": ["label"]
            }])),
            None,
        ),
        (
            "<label><span><CustomInput /></span></label>",
            Some(serde_json::json!([{
                "assert": "htmlFor",
            }])),
            Some(component_settings()),
        ),
        (
            "<CustomLabel><span><CustomInput /></span></CustomLabel>",
            Some(serde_json::json!([{
                "assert": "htmlFor",
            }])),
            Some(component_settings()),
        ),
        (
            r#"<label htmlFor="js_id" />"#,
            Some(serde_json::json!([{
                "assert": "nesting",
            }])),
            None,
        ),
        (
            r#"<label htmlFor="js_id"><input /></label>"#,
            Some(serde_json::json!([{
                "assert": "nesting",
            }])),
            None,
        ),
        (
            r#"<label htmlFor="js_id"><textarea /></label>"#,
            Some(serde_json::json!([{
                "assert": "nesting",
            }])),
            None,
        ),
        (
            "<label></label>",
            Some(serde_json::json!([{
                "assert": "nesting",
            }])),
            None,
        ),
        (
            "<label>A label</label>",
            Some(serde_json::json!([{
                "assert": "nesting",
            }])),
            None,
        ),
        (
            "<div><label /><input /></div>",
            Some(serde_json::json!([{
                "assert": "nesting",
            }])),
            None,
        ),
        (
            "<div><label>A label</label><input /></div>",
            Some(serde_json::json!([{
                "assert": "nesting",
            }])),
            None,
        ),
        (
            r#"<CustomLabel aria-label="A label" />"#,
            Some(serde_json::json!([{
                "assert": "nesting",
                "labelComponents": ["CustomLabel"]
            }])),
            None,
        ),
        (
            r#"<CustomLabel label="A label" />"#,
            Some(serde_json::json!([{
                "assert": "nesting",
                "labelComponents": ["CustomLabel"],
                "labelAttributes": ["label"]
            }])),
            None,
        ),
        (
            r#"<CustomLabel aria-label="A label" />"#,
            Some(serde_json::json!([{
                "assert": "nesting",
            }])),
            Some(component_settings()),
        ),
        (
            r#"<label label="A label" />"#,
            Some(serde_json::json!([{
                "assert": "nesting",
                "labelAttributes": ["label"]
            }])),
            None,
        ),
        (
            "<label><span><CustomInput /></span></label>",
            Some(serde_json::json!([{
                "assert": "nesting",
                "controlComponents": ["CustomInput"]
            }])),
            None,
        ),
        (
            "<CustomLabel><span><CustomInput /></span></CustomLabel>",
            Some(serde_json::json!([{
                "assert": "nesting",
                "controlComponents": ["CustomInput"],
                "labelComponents": ["CustomLabel"],
            }])),
            None,
        ),
        (
            "<CustomLabel><span><CustomInput /></span></CustomLabel>",
            Some(serde_json::json!([{
                "assert": "nesting",
                "controlComponents": ["CustomInput"],
                "labelComponents": ["CustomLabel"],
                "labelAttributes": ["label"]
            }])),
            None,
        ),
        (
            "<label><span><CustomInput /></span></label>",
            Some(serde_json::json!([{
                "assert": "nesting",
            }])),
            Some(component_settings()),
        ),
        (
            "<CustomLabel><span><CustomInput /></span></CustomLabel>",
            Some(serde_json::json!([{
                "assert": "nesting",
            }])),
            Some(component_settings()),
        ),
        (
            r#"<label htmlFor="js_id" />"#,
            Some(serde_json::json!([{
                "assert": "both",
            }])),
            None,
        ),
        (
            r#"<label htmlFor="js_id"><input /></label>"#,
            Some(serde_json::json!([{
                "assert": "both",
            }])),
            None,
        ),
        (
            r#"<label htmlFor="js_id"><textarea /></label>"#,
            Some(serde_json::json!([{
                "assert": "both",
            }])),
            None,
        ),
        (
            "<label></label>",
            Some(serde_json::json!([{
                "assert": "both",
            }])),
            None,
        ),
        (
            "<label>A label</label>",
            Some(serde_json::json!([{
                "assert": "both",
            }])),
            None,
        ),
        (
            "<div><label /><input /></div>",
            Some(serde_json::json!([{
                "assert": "both",
            }])),
            None,
        ),
        (
            "<div><label>A label</label><input /></div>",
            Some(serde_json::json!([{
                "assert": "both",
            }])),
            None,
        ),
        (
            r#"<CustomLabel aria-label="A label" />"#,
            Some(serde_json::json!([{
                "assert": "both",
                "labelComponents": ["CustomLabel"]
            }])),
            None,
        ),
        (
            r#"<CustomLabel label="A label" />"#,
            Some(serde_json::json!([{
                "assert": "both",
                "labelComponents": ["CustomLabel"],
                "labelAttributes": ["label"]
            }])),
            None,
        ),
        (
            r#"<CustomLabel aria-label="A label" />"#,
            Some(serde_json::json!([{
                "assert": "both",
            }])),
            Some(component_settings()),
        ),
        (
            r#"<label label="A label" />"#,
            Some(serde_json::json!([{
                "assert": "both",
                "labelAttributes": ["label"]
            }])),
            None,
        ),
        (
            "<label><span><CustomInput /></span></label>",
            Some(serde_json::json!([{
                "assert": "both",
                "controlComponents": ["CustomInput"]
            }])),
            None,
        ),
        (
            "<CustomLabel><span><CustomInput /></span></CustomLabel>",
            Some(serde_json::json!([{
                "assert": "both",
                "controlComponents": ["CustomInput"],
                "labelComponents": ["CustomLabel"],
            }])),
            None,
        ),
        (
            "<CustomLabel><span><CustomInput /></span></CustomLabel>",
            Some(serde_json::json!([{
                "assert": "both",
                "controlComponents": ["CustomInput"],
                "labelComponents": ["CustomLabel"],
                "labelAttributes": ["label"]
            }])),
            None,
        ),
        (
            "<label><span><CustomInput /></span></label>",
            Some(serde_json::json!([{
                "assert": "both",
            }])),
            Some(component_settings()),
        ),
        (
            "<CustomLabel><span><CustomInput /></span></CustomLabel>",
            Some(serde_json::json!([{
                "assert": "both",
            }])),
            Some(component_settings()),
        ),
        (
            r#"<label htmlFor="js_id" />"#,
            Some(serde_json::json!([{
                "assert": "either",
            }])),
            None,
        ),
        (
            r#"<label htmlFor="js_id"><input /></label>"#,
            Some(serde_json::json!([{
                "assert": "either",
            }])),
            None,
        ),
        (
            r#"<label htmlFor="js_id"><textarea /></label>"#,
            Some(serde_json::json!([{
                "assert": "either",
            }])),
            None,
        ),
        (
            "<label></label>",
            Some(serde_json::json!([{
                "assert": "either",
            }])),
            None,
        ),
        (
            "<label>A label</label>",
            Some(serde_json::json!([{
                "assert": "either",
            }])),
            None,
        ),
        (
            "<div><label /><input /></div>",
            Some(serde_json::json!([{
                "assert": "either",
            }])),
            None,
        ),
        (
            "<div><label>A label</label><input /></div>",
            Some(serde_json::json!([{
                "assert": "either",
            }])),
            None,
        ),
        (
            r#"<CustomLabel aria-label="A label" />"#,
            Some(serde_json::json!([{
                "assert": "either",
                "labelComponents": ["CustomLabel"]
            }])),
            None,
        ),
        (
            r#"<CustomLabel label="A label" />"#,
            Some(serde_json::json!([{
                "assert": "either",
                "labelComponents": ["CustomLabel"],
                "labelAttributes": ["label"]
            }])),
            None,
        ),
        (
            r#"<CustomLabel aria-label="A label" />"#,
            Some(serde_json::json!([{
                "assert": "either",
            }])),
            Some(component_settings()),
        ),
        (
            r#"<label label="A label" />"#,
            Some(serde_json::json!([{
                "assert": "either",
                "labelAttributes": ["label"]
            }])),
            None,
        ),
        (
            "<label><span><CustomInput /></span></label>",
            Some(serde_json::json!([{
                "assert": "either",
                "controlComponents": ["CustomInput"]
            }])),
            None,
        ),
        (
            "<CustomLabel><span><CustomInput /></span></CustomLabel>",
            Some(serde_json::json!([{
                "assert": "either",
                "controlComponents": ["CustomInput"],
                "labelComponents": ["CustomLabel"],
            }])),
            None,
        ),
        (
            "<CustomLabel><span><CustomInput /></span></CustomLabel>",
            Some(serde_json::json!([{
                "assert": "either",
                "controlComponents": ["CustomInput"],
                "labelComponents": ["CustomLabel"],
                "labelAttributes": ["label"]
            }])),
            None,
        ),
        (
            "<label><span><CustomInput /></span></label>",
            Some(serde_json::json!([{
                "assert": "either",
            }])),
            Some(component_settings()),
        ),
        (
            "<CustomLabel><span><CustomInput /></span></CustomLabel>",
            Some(serde_json::json!([{
                "assert": "either",
            }])),
            Some(component_settings()),
        ),
        // ensure `labelComponents` is sorted for binary search
        (
            r#"<CustomLabel aria-label="A label" />"#,
            Some(serde_json::json!([{
                "assert": "either",
                "labelComponents": ["ZZZLabelCustom", "LabelCustom", "CustomLabel"]
            }])),
            None,
        ),
        (
            "<FilesContext.Provider value={{ addAlert, cwdInfo }} />",
            Some(serde_json::json!([{
                "labelComponents": ["FilesContext.Provider"],
            }])),
            None,
        ),
        (
            r#"<label for="js_id">A label</label>"#,
            Some(serde_json::json!([{ "assert": ["htmlFor"] }])),
            None,
        ),
        (
            r#"<label for="js_id" aria-label="A label" />"#,
            Some(serde_json::json!([{ "assert": ["htmlFor"] }])),
            None,
        ),
    ];

    Tester::new(LabelHasAssociatedControl::NAME, LabelHasAssociatedControl::PLUGIN, pass, fail)
        .test_and_snapshot();
}

#[test]
fn test_attribute_values() {
    use crate::tester::Tester;

    let pass = vec![
        (r#"<label aria-label="Name" htmlFor="name" />"#, None, None),
        (r"<label aria-label={label} htmlFor={id} />", None, None),
        (r#"<label aria-label htmlFor="name" />"#, None, None),
        (r#"<label htmlFor="name"><span aria-label /></label>"#, None, None),
        (r#"<label htmlFor="name"><span {...props} /></label>"#, None, None),
        (r#"<label {...props} htmlFor="name" />"#, None, None),
        (r#"<label htmlFor="">Name<input /></label>"#, None, None),
        (r#"<label aria-label="">Name<input /></label>"#, None, None),
        (r#"<label htmlFor=" ">Name</label>"#, None, None),
        (r"<label htmlFor>Name</label>", None, None),
        (r#"<label aria-label={`Name`} htmlFor="name" />"#, None, None),
        (r#"<label aria-label={`${label}`} htmlFor="name" />"#, None, None),
        (r#"<label aria-label={1n} htmlFor="name" />"#, None, None),
        (r"<label htmlFor={1n}>Name</label>", None, None),
        (r#"<label aria-label={"\u0085"} htmlFor="name" />"#, None, None),
        (r#"<label htmlFor="name"><span aria-label={"\u0085"} /></label>"#, None, None),
        (r#"<label htmlFor={"\uFEFF"}>Name</label>"#, None, None),
    ];
    let fail = vec![
        (r#"<label aria-label="" htmlFor="name" />"#, None, None),
        (r#"<label aria-label="  " htmlFor="name" />"#, None, None),
        (r#"<label aria-labelledby="" htmlFor="name" />"#, None, None),
        (r#"<label alt="" htmlFor="name" />"#, None, None),
        (r#"<label aria-label={""} htmlFor="name" />"#, None, None),
        (r#"<label aria-label={"  "} htmlFor="name" />"#, None, None),
        (r#"<label aria-label={``} htmlFor="name" />"#, None, None),
        (r#"<label aria-label={null} htmlFor="name" />"#, None, None),
        (r#"<label aria-label={false} htmlFor="name" />"#, None, None),
        (r#"<label aria-label={0} htmlFor="name" />"#, None, None),
        (r#"<label aria-label={0n} htmlFor="name" />"#, None, None),
        (r#"<label aria-label={"\uFEFF"} htmlFor="name" />"#, None, None),
        (r#"<label htmlFor="name"><span aria-label={"\uFEFF"} /></label>"#, None, None),
        (r#"<label aria-label={undefined} htmlFor="name" />"#, None, None),
        (r#"<label htmlFor="name"><span aria-label={""} /></label>"#, None, None),
        (r#"<label htmlFor="">Name</label>"#, None, None),
        (r#"<label htmlFor={""}>Name</label>"#, None, None),
        (r"<label htmlFor={null}>Name</label>", None, None),
        (r"<label htmlFor={false}>Name</label>", None, None),
        (r"<label htmlFor={0n}>Name</label>", None, None),
        (
            r#"<label htmlFor="">Name<input /></label>"#,
            Some(serde_json::json!([{ "assert": "both" }])),
            None,
        ),
        (
            r#"<label label="" htmlFor="name" />"#,
            Some(serde_json::json!([{
                "labelAttributes": ["label"]
            }])),
            None,
        ),
        (
            r#"<label for="">Name</label>"#,
            None,
            Some(serde_json::json!({"settings": {"jsx-a11y": {
                "attributes": {"for": ["for"]}
            }}})),
        ),
    ];
    Tester::new(LabelHasAssociatedControl::NAME, LabelHasAssociatedControl::PLUGIN, pass, fail)
        .with_snapshot_suffix("attribute_values")
        .test_and_snapshot();
}

#[test]
fn test_falsy_attribute_values() {
    use crate::tester::Tester;

    let pass = vec![
        r#"<label aria-label="true" htmlFor="name" />"#,
        r#"<label aria-label=" false " htmlFor="name" />"#,
        r#"<label aria-label={-1} htmlFor="name" />"#,
        r#"<label aria-label={!false} htmlFor="name" />"#,
        r#"<label aria-label={!!true} htmlFor="name" />"#,
        r#"<label aria-label={~0} htmlFor="name" />"#,
        r#"<label aria-label={+"true"} htmlFor="name" />"#,
        r#"<label aria-label={delete object.label} htmlFor="name" />"#,
    ];
    let fail = vec![
        r#"<label aria-label="false" htmlFor="name" />"#,
        r#"<label aria-label="FaLsE" htmlFor="name" />"#,
        r#"<label aria-label={"false"} htmlFor="name" />"#,
        r#"<label htmlFor="false">Name</label>"#,
        r#"<label htmlFor="name"><span aria-label="false" /></label>"#,
        r#"<label aria-label={-unknown} htmlFor="name" />"#,
        r#"<label aria-label={-0} htmlFor="name" />"#,
        r#"<label aria-label={-0n} htmlFor="name" />"#,
        r#"<label aria-label={+0} htmlFor="name" />"#,
        r#"<label aria-label={!true} htmlFor="name" />"#,
        r#"<label aria-label={!!false} htmlFor="name" />"#,
        r#"<label aria-label={~(-1)} htmlFor="name" />"#,
        r#"<label aria-label={+"false"} htmlFor="name" />"#,
        r"<label htmlFor={void 0}>Name</label>",
        r#"<label aria-label={typeof value} htmlFor="name" />"#,
    ];
    Tester::new(LabelHasAssociatedControl::NAME, LabelHasAssociatedControl::PLUGIN, pass, fail)
        .with_snapshot_suffix("falsy_attribute_values")
        .test_and_snapshot();
}

#[test]
fn test_raw_template_attribute_values() {
    use crate::tester::Tester;

    let pass = vec![
        r#"<label aria-label={`\uFEFF`} htmlFor="name" />"#,
        r#"<label htmlFor="name"><span aria-label={`\uFEFF`} /></label>"#,
        r#"<label aria-label={`\n`} htmlFor="name" />"#,
        r#"<label aria-label={`false`} htmlFor="name" />"#,
        r"<label htmlFor={`\uFEFF`}>Name</label>",
    ];
    let fail = vec![
        r#"<label aria-label={``} htmlFor="name" />"#,
        r#"<label aria-label={`  `} htmlFor="name" />"#,
        r"<label htmlFor={``}>Name</label>",
    ];
    Tester::new(LabelHasAssociatedControl::NAME, LabelHasAssociatedControl::PLUGIN, pass, fail)
        .with_snapshot_suffix("raw_template_attribute_values")
        .test_and_snapshot();
}
