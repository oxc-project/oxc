use std::ops::Deref;

use globset::{Glob, GlobSet, GlobSetBuilder};
use oxc_ast::{
    ast::{JSXAttributeItem, JSXAttributeValue, JSXChild, JSXElement},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, Span};

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{get_element_type, get_jsx_attribute_name, has_jsx_prop, is_react_component_name},
    AstNode,
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

#[derive(Debug, Clone)]
pub struct LabelHasAssociatedControlConfig {
    depth: u8,
    assert: Assert,
    label_components: Vec<CompactStr>,
    label_attributes: Vec<CompactStr>,
    control_components: GlobSet,
}

#[derive(Debug, Clone)]
enum Assert {
    HtmlFor,
    Nesting,
    Both,
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
            control_components: GlobSet::empty(),
        }
    }
}

declare_oxc_lint!(
    /// ### What it does
    /// Enforce that a label tag has a text label and an associated control.
    ///
    /// ### Why is this bad?
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
);

impl Rule for LabelHasAssociatedControl {
    fn from_configuration(value: serde_json::Value) -> Self {
        let mut config = LabelHasAssociatedControlConfig::default();

        let mut control_builder = GlobSetBuilder::new();
        control_builder.add(Glob::new("input").unwrap());
        control_builder.add(Glob::new("meter").unwrap());
        control_builder.add(Glob::new("output").unwrap());
        control_builder.add(Glob::new("progress").unwrap());
        control_builder.add(Glob::new("select").unwrap());
        control_builder.add(Glob::new("textarea").unwrap());

        let Some(options) = value.get(0) else {
            config.control_components = control_builder.build().unwrap();
            return Self(Box::new(config));
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
        {
            if let Some(mut components) = label_components
                .iter()
                .map(serde_json::Value::as_str)
                .map(|component| component.map(CompactStr::from))
                .collect::<Option<Vec<CompactStr>>>()
            {
                config.label_components.append(&mut components);
            }
        }

        if let Some(label_attributes) =
            options.get("labelAttributes").and_then(serde_json::Value::as_array)
        {
            if let Some(mut attributes) = label_attributes
                .iter()
                .map(serde_json::Value::as_str)
                .map(|attribute| attribute.map(CompactStr::from))
                .collect::<Option<Vec<CompactStr>>>()
            {
                config.label_attributes.append(&mut attributes);
            }
        }

        if let Some(control_components) =
            options.get("controlComponents").and_then(serde_json::Value::as_array)
        {
            control_components.iter().map(serde_json::Value::as_str).for_each(|component| {
                let Some(component) = component else {
                    return;
                };

                let Ok(glob) = Glob::new(component) else {
                    return;
                };

                control_builder.add(glob);
            });
        }

        config.control_components = if let Ok(controls) = control_builder.build() {
            controls
        } else {
            let mut control_builder = GlobSetBuilder::new();
            control_builder.add(Glob::new("input").unwrap());
            control_builder.add(Glob::new("meter").unwrap());
            control_builder.add(Glob::new("output").unwrap());
            control_builder.add(Glob::new("progress").unwrap());
            control_builder.add(Glob::new("select").unwrap());
            control_builder.add(Glob::new("textarea").unwrap());
            control_builder.build().unwrap()
        };

        config.label_components.sort_unstable();
        config.label_components.dedup();

        config.label_attributes.sort_unstable();
        config.label_attributes.dedup();

        Self(Box::new(config))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXElement(element) = node.kind() else {
            return;
        };

        let element_type = get_element_type(ctx, &element.opening_element);

        if self.label_components.binary_search(&element_type.into()).is_err() {
            return;
        }

        let has_html_for = has_jsx_prop(&element.opening_element, "htmlFor").is_some();
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
        };

        ctx.diagnostic(label_has_associated_control_diagnostic(element.opening_element.span));
    }
}

impl LabelHasAssociatedControl {
    fn has_accessible_label<'a>(&self, root: &JSXElement<'a>, ctx: &LintContext<'a>) -> bool {
        if root.opening_element.attributes.iter().any(|attribute| match attribute {
            JSXAttributeItem::Attribute(attr) => {
                let attr_name = get_jsx_attribute_name(&attr.name);
                self.label_attributes.binary_search(&attr_name.into()).is_ok()
            }
            JSXAttributeItem::SpreadAttribute(_) => true,
        }) {
            return true;
        }

        for child in &root.children {
            if self.search_for_accessible_label(child, 1, ctx) {
                return true;
            }
        }

        false
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
                if self.control_components.is_match(element_type.to_string()) {
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
                let has_labelling_prop =
                    element.opening_element.attributes.iter().any(|attr| match attr {
                        JSXAttributeItem::Attribute(attribute) => {
                            self.label_attributes.iter().any(|labelling_prop| {
                                attribute.is_identifier(labelling_prop)
                                    && attribute.value.as_ref().is_some_and(|attribute_value| {
                                        match attribute_value {
                                            JSXAttributeValue::StringLiteral(literal) => {
                                                !literal.value.as_str().trim().is_empty()
                                            }
                                            _ => true,
                                        }
                                    })
                            })
                        }
                        JSXAttributeItem::SpreadAttribute(_) => true,
                    });

                if has_labelling_prop {
                    return true;
                }

                if element.children.is_empty() {
                    let name = get_element_type(ctx, &element.opening_element);
                    if is_react_component_name(&name)
                        && !self.control_components.is_match(name.to_string())
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
    ];

    Tester::new(LabelHasAssociatedControl::NAME, LabelHasAssociatedControl::PLUGIN, pass, fail)
        .test_and_snapshot();
}
