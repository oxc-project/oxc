use std::ops::Deref;

use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, de};

use oxc_ast::{
    AstKind,
    ast::{JSXAttributeItem, JSXAttributeValue, JSXChild, JSXElement},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_str::CompactStr;

use crate::{
    AstNode,
    context::LintContext,
    globals::HTML_TAG,
    rule::{DefaultRuleConfig, Rule},
    utils::{
        get_element_type, get_jsx_attribute_name, get_string_literal_prop_value, has_jsx_prop,
        is_hidden_from_screen_reader, is_interactive_element, is_interactive_role,
        is_react_component_name,
    },
};

fn control_has_associated_label_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("A control must be associated with a text label.")
        .with_help(
            "Add a text label to the control element. This can be done by adding text content, an `aria-label` attribute, or an `aria-labelledby` attribute.",
        )
        .with_label(span)
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct ControlHasAssociatedLabel(Box<ControlHasAssociatedLabelConfig>);

/// Elements that are always ignored (cannot reliably determine label source).
const ALWAYS_IGNORE_ELEMENTS: [&str; 1] = ["link"];

const DEFAULT_IGNORE_ELEMENTS: [&str; 7] =
    ["audio", "canvas", "embed", "input", "textarea", "tr", "video"];

const DEFAULT_IGNORE_ROLES: [&str; 10] = [
    "grid",
    "listbox",
    "menu",
    "menubar",
    "radiogroup",
    "row",
    "tablist",
    "toolbar",
    "tree",
    "treegrid",
];

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct ControlHasAssociatedLabelConfig {
    /// Maximum depth to search for an accessible label within the element.
    /// Defaults to `2`.
    #[serde(deserialize_with = "validate_depth")]
    #[schemars(range(max = 25))]
    depth: u8,
    /// Additional attributes to check for accessible label text.
    label_attributes: Vec<CompactStr>,
    /// Custom JSX components to be treated as interactive controls.
    control_components: Vec<CompactStr>,
    /// Elements to ignore.
    /// Defaults to `["audio", "canvas", "embed", "input", "textarea", "tr", "video"]`.
    ignore_elements: Vec<CompactStr>,
    /// Interactive roles to ignore.
    /// Defaults to `["grid", "listbox", "menu", "menubar", "radiogroup", "row", "tablist", "toolbar", "tree", "treegrid"]`.
    ignore_roles: Vec<CompactStr>,
}

fn validate_depth<'de, D>(deserializer: D) -> Result<u8, D::Error>
where
    D: Deserializer<'de>,
{
    let depth = u8::deserialize(deserializer)?;
    if depth > 25 {
        return Err(de::Error::custom("depth must be less than or equal to 25"));
    }
    Ok(depth)
}

impl Deref for ControlHasAssociatedLabel {
    type Target = ControlHasAssociatedLabelConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for ControlHasAssociatedLabelConfig {
    fn default() -> Self {
        Self {
            depth: 2,
            label_attributes: vec![],
            control_components: vec![],
            ignore_elements: DEFAULT_IGNORE_ELEMENTS.into_iter().map(CompactStr::from).collect(),
            ignore_roles: DEFAULT_IGNORE_ROLES.into_iter().map(CompactStr::from).collect(),
        }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce that a control (an interactive element) has a text label.
    ///
    /// ### Why is this bad?
    ///
    /// An interactive element (such as a `<button>`) without an accessible
    /// text label makes it difficult or impossible for users of assistive
    /// technologies to understand the purpose of the control.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <button />
    /// <a href="/path" />
    /// <th />
    /// <div role="button" />
    /// <div role="checkbox" />
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <button>Save</button>
    /// <button aria-label="Save" />
    /// <label>Name <input type="text" /></label>
    /// <a href="/path">Learn more</a>
    /// <th>Column Header</th>
    /// <div role="button">Submit</div>
    /// <div role="checkbox" aria-labelledby="label_id" />
    /// ```
    ControlHasAssociatedLabel,
    jsx_a11y,
    correctness,
    config = ControlHasAssociatedLabelConfig,
    version = "1.65.0",
    short_description = "Enforce that a control (an interactive element) has a text label.",
);

impl Rule for ControlHasAssociatedLabel {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXElement(element) = node.kind() else {
            return;
        };

        let element_type = get_element_type(ctx, &element.opening_element);

        if ALWAYS_IGNORE_ELEMENTS.contains(&element_type.as_ref())
            || self.ignore_elements.iter().any(|e| e.as_str() == element_type.as_ref())
        {
            return;
        }

        let role =
            has_jsx_prop(&element.opening_element, "role").and_then(get_string_literal_prop_value);
        if let Some(role) = role
            && self.ignore_roles.iter().any(|r| r.as_str() == role)
        {
            return;
        }

        if is_hidden_from_screen_reader(ctx, &element.opening_element) {
            return;
        }

        let is_dom_element = HTML_TAG.contains(element_type.as_ref());
        let is_interactive_el = is_interactive_element(&element_type, &element.opening_element);
        let is_interactive_role_el = role.is_some_and(is_interactive_role);
        let is_control_component =
            self.control_components.iter().any(|c| c.as_str() == element_type.as_ref());

        if !(is_interactive_el || is_dom_element && is_interactive_role_el || is_control_component)
        {
            return;
        }

        if !self.may_have_accessible_label(element, ctx) {
            ctx.diagnostic(control_has_associated_label_diagnostic(element.opening_element.span));
        }
    }
}

impl ControlHasAssociatedLabel {
    fn may_have_accessible_label<'a>(
        &self,
        element: &JSXElement<'a>,
        ctx: &LintContext<'a>,
    ) -> bool {
        if self.has_labelling_prop(&element.opening_element.attributes) {
            return true;
        }

        for child in &element.children {
            if self.check_child_for_label(child, 1, ctx) {
                return true;
            }
        }

        false
    }

    fn has_labelling_prop(&self, attributes: &[JSXAttributeItem<'_>]) -> bool {
        let labelling_props: &[&str] = &["alt", "aria-label", "aria-labelledby"];

        attributes.iter().any(|attribute| match attribute {
            JSXAttributeItem::SpreadAttribute(_) => true,
            JSXAttributeItem::Attribute(attr) => {
                let attr_name = get_jsx_attribute_name(&attr.name);
                let is_labelling = labelling_props.iter().any(|p| *p == attr_name.as_ref())
                    || self.label_attributes.iter().any(|p| p.as_str() == attr_name.as_ref());
                if !is_labelling {
                    return false;
                }

                match &attr.value {
                    None => false,
                    Some(JSXAttributeValue::StringLiteral(s)) => {
                        !s.value.as_str().trim().is_empty()
                    }
                    Some(_) => true,
                }
            }
        })
    }

    fn check_child_for_label<'a>(
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
                if self.has_labelling_prop(&element.opening_element.attributes) {
                    return true;
                }

                if element.children.is_empty() {
                    let name = get_element_type(ctx, &element.opening_element);
                    if is_react_component_name(&name)
                        && !self.control_components.iter().any(|c| c.as_str() == name.as_ref())
                    {
                        return true;
                    }
                }

                for child in &element.children {
                    if self.check_child_for_label(child, depth + 1, ctx) {
                        return true;
                    }
                }

                false
            }
            JSXChild::Fragment(fragment) => {
                for child in &fragment.children {
                    if self.check_child_for_label(child, depth + 1, ctx) {
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
fn test_recommended() {
    use crate::tester::Tester;

    // Generated from jsx-eslint/eslint-plugin-jsx-a11y __tests__/src/rules/control-has-associated-label-test.js.
    let pass = vec![
        (
            r"<CustomControl><span><span>Save</span></span></CustomControl>",
            Some(
                serde_json::json!([{"depth":3,"controlComponents":["CustomControl"],"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<CustomControl><span><span label="Save"></span></span></CustomControl>"#,
            Some(
                serde_json::json!([{"depth":3,"controlComponents":["CustomControl"],"labelAttributes":["label"],"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<CustomControl>Save</CustomControl>",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            Some(
                serde_json::json!({"settings":{"jsx-a11y":{"components":{"CustomControl":"button"}}}}),
            ),
        ),
        (
            r"<button>Save</button>",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<button><span>Save</span></button>",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<button><span><span>Save</span></span></button>",
            Some(
                serde_json::json!([{"depth":3,"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<button><span><span><span><span><span><span><span><span>Save</span></span></span></span></span></span></span></span></button>",
            Some(
                serde_json::json!([{"depth":9,"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<button><img alt="Save" /></button>"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<button aria-label="Save" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<button><span aria-label="Save" /></button>"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<button aria-labelledby="js_1" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<button><span aria-labelledby="js_1" /></button>"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<button>{sureWhyNot}</button>",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<button><span><span label="Save"></span></span></button>"#,
            Some(
                serde_json::json!([{"depth":3,"labelAttributes":["label"],"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r##"<a href="#">Save</a>"##,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r##"<area href="#">Save</area>"##,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<link>Save</link>",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<menuitem>Save</menuitem>",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<option>Save</option>",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<th>Save</th>",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="button">Save</div>"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="checkbox">Save</div>"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="columnheader">Save</div>"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="combobox">Save</div>"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="gridcell">Save</div>"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="link">Save</div>"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="menuitem">Save</div>"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="menuitemcheckbox">Save</div>"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="menuitemradio">Save</div>"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="option">Save</div>"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="progressbar">Save</div>"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="radio">Save</div>"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="rowheader">Save</div>"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="searchbox">Save</div>"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="slider">Save</div>"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="spinbutton">Save</div>"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="switch">Save</div>"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="tab">Save</div>"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="textbox">Save</div>"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="treeitem">Save</div>"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="button" aria-label="Save" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="checkbox" aria-label="Save" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="columnheader" aria-label="Save" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="combobox" aria-label="Save" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="gridcell" aria-label="Save" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="link" aria-label="Save" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="menuitem" aria-label="Save" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="menuitemcheckbox" aria-label="Save" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="menuitemradio" aria-label="Save" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="option" aria-label="Save" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="progressbar" aria-label="Save" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="radio" aria-label="Save" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="rowheader" aria-label="Save" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="searchbox" aria-label="Save" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="slider" aria-label="Save" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="spinbutton" aria-label="Save" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="switch" aria-label="Save" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="tab" aria-label="Save" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="textbox" aria-label="Save" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="treeitem" aria-label="Save" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="button" aria-labelledby="js_1" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="checkbox" aria-labelledby="js_1" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="columnheader" aria-labelledby="js_1" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="combobox" aria-labelledby="js_1" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="gridcell" aria-labelledby="Save" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="link" aria-labelledby="js_1" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="menuitem" aria-labelledby="js_1" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="menuitemcheckbox" aria-labelledby="js_1" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="menuitemradio" aria-labelledby="js_1" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="option" aria-labelledby="js_1" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="progressbar" aria-labelledby="js_1" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="radio" aria-labelledby="js_1" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="rowheader" aria-labelledby="js_1" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="searchbox" aria-labelledby="js_1" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="slider" aria-labelledby="js_1" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="spinbutton" aria-labelledby="js_1" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="switch" aria-labelledby="js_1" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="tab" aria-labelledby="js_1" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="textbox" aria-labelledby="js_1" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="treeitem" aria-labelledby="js_1" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<abbr />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<article />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<blockquote />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<br />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<caption />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<dd />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<details />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<dfn />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<dialog />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<dir />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<dl />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<dt />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<fieldset />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<figcaption />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<figure />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<footer />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<form />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<frame />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<h1 />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<h2 />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<h3 />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<h4 />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<h5 />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<h6 />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<hr />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<iframe />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<img />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<label />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<legend />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<li />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<link />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<main />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<mark />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<marquee />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<menu />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<meter />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<nav />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<ol />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<p />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<pre />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<progress />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<ruby />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<section />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<table />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<tbody />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<tfoot />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<thead />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<time />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<ul />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="alert" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="alertdialog" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="application" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="article" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="banner" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="cell" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="complementary" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="contentinfo" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="definition" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="dialog" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="directory" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="document" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="feed" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="figure" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="form" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="group" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="heading" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="img" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="list" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="listitem" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="log" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="main" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="marquee" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="math" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="navigation" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="none" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="note" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="presentation" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="progressbar" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="region" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="rowgroup" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="search" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="status" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="table" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="tabpanel" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="term" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="timer" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="tooltip" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<input />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<input type="button" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<input type="checkbox" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<input type="color" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<input type="date" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<input type="datetime" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<input type="email" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<input type="file" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<input type="hidden" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<input type="hidden" name="bot-field"/>"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<input type="hidden" name="form-name" value="Contact Form"/>"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<input type="image" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<input type="month" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<input type="number" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<input type="password" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<input type="radio" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<input type="range" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<input type="reset" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<input type="search" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<input type="submit" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<input type="tel" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<input type="text" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<label>Foo <input type="text" /></label>"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<input name={field.name} id="foo" type="text" value={field.value} disabled={isDisabled} onChange={changeText(field.onChange, field.name)} onBlur={field.onBlur} />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<input type="time" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<input type="url" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<input type="week" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<audio />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<canvas />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<embed />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<textarea />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<tr />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<video />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="grid" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="listbox" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="menu" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="menubar" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="radiogroup" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="row" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="tablist" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="toolbar" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="tree" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="treegrid" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
    ];

    let fail = vec![
        (
            r"<button />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<button><span /></button>",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<button><img /></button>",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<button><span title="This is not a real label" /></button>"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<button><span><span><span>Save</span></span></span></button>",
            Some(
                serde_json::json!([{"depth":3,"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<CustomControl><span><span></span></span></CustomControl>",
            Some(
                serde_json::json!([{"depth":3,"controlComponents":["CustomControl"],"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<CustomControl></CustomControl>",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            Some(
                serde_json::json!({"settings":{"jsx-a11y":{"components":{"CustomControl":"button"}}}}),
            ),
        ),
        (
            r##"<a href="#" />"##,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r##"<area href="#" />"##,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<menuitem />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<option />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<th />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<td />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="button" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="checkbox" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="columnheader" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="combobox" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="link" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="gridcell" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="menuitem" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="menuitemcheckbox" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="menuitemradio" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="option" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="radio" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="rowheader" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="scrollbar" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="searchbox" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="separator" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="slider" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="spinbutton" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="switch" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="tab" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="textbox" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
    ];

    Tester::new(ControlHasAssociatedLabel::NAME, ControlHasAssociatedLabel::PLUGIN, pass, fail)
        .with_snapshot_suffix("recommended")
        .test_and_snapshot();
}

#[test]
fn test_strict() {
    use crate::tester::Tester;

    // Generated from jsx-eslint/eslint-plugin-jsx-a11y __tests__/src/rules/control-has-associated-label-test.js.
    let pass = vec![
        (
            r"<CustomControl><span><span>Save</span></span></CustomControl>",
            Some(
                serde_json::json!([{"depth":3,"controlComponents":["CustomControl"],"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<CustomControl><span><span label="Save"></span></span></CustomControl>"#,
            Some(
                serde_json::json!([{"depth":3,"controlComponents":["CustomControl"],"labelAttributes":["label"],"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<CustomControl>Save</CustomControl>",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            Some(
                serde_json::json!({"settings":{"jsx-a11y":{"components":{"CustomControl":"button"}}}}),
            ),
        ),
        (
            r"<button>Save</button>",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<button><span>Save</span></button>",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<button><span><span>Save</span></span></button>",
            Some(
                serde_json::json!([{"depth":3,"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<button><span><span><span><span><span><span><span><span>Save</span></span></span></span></span></span></span></span></button>",
            Some(
                serde_json::json!([{"depth":9,"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<button><img alt="Save" /></button>"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<button aria-label="Save" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<button><span aria-label="Save" /></button>"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<button aria-labelledby="js_1" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<button><span aria-labelledby="js_1" /></button>"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<button>{sureWhyNot}</button>",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<button><span><span label="Save"></span></span></button>"#,
            Some(
                serde_json::json!([{"depth":3,"labelAttributes":["label"],"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r##"<a href="#">Save</a>"##,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r##"<area href="#">Save</area>"##,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<link>Save</link>",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<menuitem>Save</menuitem>",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<option>Save</option>",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<th>Save</th>",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="button">Save</div>"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="checkbox">Save</div>"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="columnheader">Save</div>"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="combobox">Save</div>"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="gridcell">Save</div>"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="link">Save</div>"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="menuitem">Save</div>"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="menuitemcheckbox">Save</div>"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="menuitemradio">Save</div>"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="option">Save</div>"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="progressbar">Save</div>"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="radio">Save</div>"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="rowheader">Save</div>"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="searchbox">Save</div>"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="slider">Save</div>"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="spinbutton">Save</div>"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="switch">Save</div>"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="tab">Save</div>"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="textbox">Save</div>"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="treeitem">Save</div>"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="button" aria-label="Save" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="checkbox" aria-label="Save" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="columnheader" aria-label="Save" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="combobox" aria-label="Save" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="gridcell" aria-label="Save" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="link" aria-label="Save" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="menuitem" aria-label="Save" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="menuitemcheckbox" aria-label="Save" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="menuitemradio" aria-label="Save" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="option" aria-label="Save" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="progressbar" aria-label="Save" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="radio" aria-label="Save" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="rowheader" aria-label="Save" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="searchbox" aria-label="Save" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="slider" aria-label="Save" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="spinbutton" aria-label="Save" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="switch" aria-label="Save" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="tab" aria-label="Save" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="textbox" aria-label="Save" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="treeitem" aria-label="Save" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="button" aria-labelledby="js_1" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="checkbox" aria-labelledby="js_1" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="columnheader" aria-labelledby="js_1" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="combobox" aria-labelledby="js_1" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="gridcell" aria-labelledby="Save" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="link" aria-labelledby="js_1" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="menuitem" aria-labelledby="js_1" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="menuitemcheckbox" aria-labelledby="js_1" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="menuitemradio" aria-labelledby="js_1" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="option" aria-labelledby="js_1" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="progressbar" aria-labelledby="js_1" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="radio" aria-labelledby="js_1" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="rowheader" aria-labelledby="js_1" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="searchbox" aria-labelledby="js_1" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="slider" aria-labelledby="js_1" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="spinbutton" aria-labelledby="js_1" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="switch" aria-labelledby="js_1" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="tab" aria-labelledby="js_1" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="textbox" aria-labelledby="js_1" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="treeitem" aria-labelledby="js_1" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<abbr />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<article />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<blockquote />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<br />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<caption />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<dd />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<details />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<dfn />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<dialog />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<dir />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<dl />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<dt />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<fieldset />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<figcaption />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<figure />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<footer />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<form />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<frame />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<h1 />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<h2 />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<h3 />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<h4 />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<h5 />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<h6 />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<hr />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<iframe />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<img />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<label />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<legend />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<li />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<link />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<main />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<mark />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<marquee />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<menu />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<meter />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<nav />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<ol />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<p />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<pre />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<progress />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<ruby />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<section />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<table />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<tbody />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<tfoot />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<thead />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<time />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<ul />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="alert" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="alertdialog" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="application" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="article" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="banner" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="cell" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="complementary" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="contentinfo" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="definition" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="dialog" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="directory" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="document" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="feed" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="figure" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="form" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="group" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="heading" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="img" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="list" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="listitem" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="log" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="main" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="marquee" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="math" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="navigation" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="none" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="note" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="presentation" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="progressbar" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="region" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="rowgroup" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="search" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="status" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="table" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="tabpanel" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="term" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="timer" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="tooltip" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<input />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<input type="button" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<input type="checkbox" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<input type="color" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<input type="date" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<input type="datetime" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<input type="email" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<input type="file" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<input type="hidden" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<input type="hidden" name="bot-field"/>"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<input type="hidden" name="form-name" value="Contact Form"/>"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<input type="image" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<input type="month" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<input type="number" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<input type="password" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<input type="radio" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<input type="range" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<input type="reset" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<input type="search" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<input type="submit" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<input type="tel" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<input type="text" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<label>Foo <input type="text" /></label>"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<input name={field.name} id="foo" type="text" value={field.value} disabled={isDisabled} onChange={changeText(field.onChange, field.name)} onBlur={field.onBlur} />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<input type="time" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<input type="url" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<input type="week" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<audio />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<canvas />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<embed />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<textarea />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<tr />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<video />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="grid" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="listbox" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="menu" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="menubar" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="radiogroup" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="row" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="tablist" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="toolbar" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="tree" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="treegrid" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
    ];

    let fail = vec![
        (
            r"<button />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<button><span /></button>",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<button><img /></button>",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<button><span title="This is not a real label" /></button>"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<button><span><span><span>Save</span></span></span></button>",
            Some(
                serde_json::json!([{"depth":3,"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<CustomControl><span><span></span></span></CustomControl>",
            Some(
                serde_json::json!([{"depth":3,"controlComponents":["CustomControl"],"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<CustomControl></CustomControl>",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            Some(
                serde_json::json!({"settings":{"jsx-a11y":{"components":{"CustomControl":"button"}}}}),
            ),
        ),
        (
            r##"<a href="#" />"##,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r##"<area href="#" />"##,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<menuitem />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<option />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<th />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r"<td />",
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="button" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="checkbox" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="columnheader" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="combobox" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="link" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="gridcell" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="menuitem" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="menuitemcheckbox" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="menuitemradio" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="option" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="radio" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="rowheader" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="scrollbar" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="searchbox" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="separator" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="slider" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="spinbutton" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="switch" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="tab" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
        (
            r#"<div role="textbox" />"#,
            Some(
                serde_json::json!([{"ignoreElements":["audio","canvas","embed","input","textarea","tr","video"],"ignoreRoles":["grid","listbox","menu","menubar","radiogroup","row","tablist","toolbar","tree","treegrid"]}]),
            ),
            None,
        ),
    ];

    Tester::new(ControlHasAssociatedLabel::NAME, ControlHasAssociatedLabel::PLUGIN, pass, fail)
        .with_snapshot_suffix("strict")
        .test_and_snapshot();
}

#[test]
fn test_no_config() {
    use crate::tester::Tester;

    // Generated from jsx-eslint/eslint-plugin-jsx-a11y __tests__/src/rules/control-has-associated-label-test.js.
    let pass = vec![
        (r#"<label>Name <input type="text" /></label>"#, None, None),
        (r"<input />", None, None),
        (r#"<input type="text" />"#, None, None),
        (r#"<input type="hidden" />"#, None, None),
        (r#"<input type="text" aria-hidden="true" />"#, None, None),
        (r"<audio />", None, None),
        (r"<canvas />", None, None),
        (r"<embed />", None, None),
        (r"<textarea />", None, None),
        (r"<tr />", None, None),
        (r"<video />", None, None),
        (r#"<div role="grid" />"#, None, None),
        (r#"<div role="listbox" />"#, None, None),
        (r#"<div role="menu" />"#, None, None),
        (r#"<div role="menubar" />"#, None, None),
        (r#"<div role="radiogroup" />"#, None, None),
        (r#"<div role="row" />"#, None, None),
        (r#"<div role="tablist" />"#, None, None),
        (r#"<div role="toolbar" />"#, None, None),
        (r#"<div role="tree" />"#, None, None),
        (r#"<div role="treegrid" />"#, None, None),
    ];

    let fail = vec![
        (r"<button />", None, None),
        (r##"<a href="#" />"##, None, None),
        (r#"<div role="button" />"#, None, None),
    ];

    Tester::new(ControlHasAssociatedLabel::NAME, ControlHasAssociatedLabel::PLUGIN, pass, fail)
        .with_snapshot_suffix("no_config")
        .test_and_snapshot();
}
