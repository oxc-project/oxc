use lazy_regex::{Lazy, Regex, lazy_regex};
use rustc_hash::FxHashMap;
use schemars::{
    JsonSchema, SchemaGenerator,
    schema::{
        ArrayValidation, InstanceType, Schema, SchemaObject, SingleOrVec, SubschemaValidation,
    },
};
use serde::Deserialize;
use serde_json::Value;

use oxc_ast::{AstKind, ast::JSXAttributeItem};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_str::CompactStr;

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    rule::{MixedTupleRuleConfig, Rule},
};

fn jsx_no_script_url_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("React 19 disallows `javascript:` URLs as a security precaution.")
        .with_help("Use event handlers instead if you can.")
        .with_label(span)
}

static JS_SCRIPT_REGEX: Lazy<Regex> = lazy_regex!(
    r"(j|J)[\r\n\t]*(a|A)[\r\n\t]*(v|V)[\r\n\t]*(a|A)[\r\n\t]*(s|S)[\r\n\t]*(c|C)[\r\n\t]*(r|R)[\r\n\t]*(i|I)[\r\n\t]*(p|P)[\r\n\t]*(t|T)[\r\n\t]*:"
);

#[derive(Debug, Default, Clone)]
pub struct JsxNoScriptUrl(Box<JsxNoScriptUrlConfig>);

#[derive(Debug, Default, Clone)]
pub struct JsxNoScriptUrlConfig {
    include_from_settings: bool,
    components: FxHashMap<String, Vec<String>>,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(transparent)]
struct JsxNoScriptUrlComponents(Vec<JsxNoScriptUrlComponent>);

impl JsxNoScriptUrlComponents {
    fn into_map(self) -> FxHashMap<String, Vec<String>> {
        self.0.into_iter().map(|component| (component.name, component.props)).collect()
    }
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct JsxNoScriptUrlComponent {
    /// Component name.
    name: String,
    /// List of properties that should be validated.
    props: Vec<String>,
}

#[derive(Debug, Default, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
struct JsxNoScriptUrlOptions {
    /// Whether to include components from settings.
    include_from_settings: bool,
}

#[derive(Debug, Default, Clone, Deserialize)]
struct JsxNoScriptUrlRuleConfig(
    MixedTupleRuleConfig<JsxNoScriptUrlComponents, JsxNoScriptUrlOptions>,
);

impl JsonSchema for JsxNoScriptUrlRuleConfig {
    fn schema_name() -> String {
        "JsxNoScriptUrlRuleConfig".to_string()
    }

    fn is_referenceable() -> bool {
        false
    }

    fn json_schema(r#gen: &mut SchemaGenerator) -> Schema {
        let components = r#gen.subschema_for::<Vec<JsxNoScriptUrlComponent>>();
        let options = r#gen.subschema_for::<JsxNoScriptUrlOptions>();

        SchemaObject {
            subschemas: Some(Box::new(SubschemaValidation {
                any_of: Some(vec![
                    config_tuple_schema(vec![components, options.clone()], 1),
                    options,
                ]),
                ..Default::default()
            })),
            ..Default::default()
        }
        .into()
    }
}

fn config_tuple_schema(items: Vec<Schema>, min_items: u32) -> Schema {
    let max_items = u32::try_from(items.len()).expect("rule config tuple should fit in u32");

    SchemaObject {
        instance_type: Some(InstanceType::Array.into()),
        array: Some(Box::new(ArrayValidation {
            items: Some(SingleOrVec::Vec(items)),
            min_items: Some(min_items),
            max_items: Some(max_items),
            ..Default::default()
        })),
        ..Default::default()
    }
    .into()
}

impl std::ops::Deref for JsxNoScriptUrl {
    type Target = JsxNoScriptUrlConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow usage of `javascript:` URLs.
    ///
    /// ### Why is this bad?
    ///
    /// URLs starting with `javascript:` are a dangerous attack surface because it’s easy to accidentally
    /// include unsanitized output in a tag like `<a href>` and create a security hole.
    ///
    /// Starting in React 16.9, any URLs starting with `javascript:` log a warning.
    ///
    /// In React 19, `javascript:` URLs are
    /// [disallowed entirely](https://react.dev/blog/2024/04/25/react-19-upgrade-guide#other-breaking-changes).
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <a href="javascript:void(0)">Test</a>
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <Foo test="javascript:void(0)" />
    /// ```
    JsxNoScriptUrl,
    react,
    suspicious,
    pending,
    config = JsxNoScriptUrlRuleConfig,
    version = "0.13.2",
    short_description = "Disallow usage of `javascript:` URLs.",
);

fn is_link_attribute(tag_name: &str, prop_value_literal: String, ctx: &LintContext) -> bool {
    tag_name == "a"
        || ctx.settings().react.get_link_component_attrs(tag_name).is_some_and(
            |link_component_attrs| {
                link_component_attrs.contains(&CompactStr::from(prop_value_literal))
            },
        )
}

impl JsxNoScriptUrl {
    fn is_link_tag(&self, tag_name: &str, ctx: &LintContext) -> bool {
        if !self.include_from_settings {
            return tag_name == "a";
        }
        if tag_name == "a" {
            return true;
        }
        ctx.settings().react.get_link_component_attrs(tag_name).is_some()
    }
}

impl Rule for JsxNoScriptUrl {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::JSXOpeningElement(element) = node.kind() {
            let Some(component_name) = element.name.get_identifier_name() else {
                return;
            };
            if let Some(link_props) = self.components.get(component_name.as_str()) {
                for jsx_attribute in &element.attributes {
                    if let JSXAttributeItem::Attribute(attr) = jsx_attribute {
                        let Some(prop_value) = &attr.value else {
                            return;
                        };
                        if prop_value.as_string_literal().is_some_and(|val| {
                            link_props.contains(&attr.name.get_identifier().name.to_string())
                                && JS_SCRIPT_REGEX.captures(&val.value).is_some()
                        }) {
                            ctx.diagnostic(jsx_no_script_url_diagnostic(attr.span()));
                        }
                    }
                }
            } else if self.is_link_tag(component_name.as_str(), ctx) {
                for jsx_attribute in &element.attributes {
                    if let JSXAttributeItem::Attribute(attr) = jsx_attribute {
                        let Some(prop_value) = &attr.value else {
                            return;
                        };
                        if prop_value.as_string_literal().is_some_and(|val| {
                            is_link_attribute(
                                component_name.as_str(),
                                attr.name.get_identifier().name.to_string(),
                                ctx,
                            ) && JS_SCRIPT_REGEX.captures(&val.value).is_some()
                        }) {
                            ctx.diagnostic(jsx_no_script_url_diagnostic(attr.span()));
                        }
                    }
                }
            }
        }
    }

    fn from_configuration(value: Value) -> Result<Self, serde_json::error::Error> {
        let JsxNoScriptUrlRuleConfig(MixedTupleRuleConfig(components, options)) =
            serde_json::from_value::<JsxNoScriptUrlRuleConfig>(value)?;

        Ok(Self(Box::new(JsxNoScriptUrlConfig {
            include_from_settings: options.include_from_settings,
            components: components.into_map(),
        })))
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_jsx()
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (r#"<a href="https://reactjs.org"></a>"#, None, None),
        (r#"<a href="mailto:foo@bar.com"></a>"#, None, None),
        (r##"<a href="#"></a>"##, None, None),
        (r#"<a href=""></a>"#, None, None),
        (r#"<a name="foo"></a>"#, None, None),
        (r#"<a href={"javascript:"}></a>"#, None, None),
        (r#"<Foo href="javascript:"></Foo>"#, None, None),
        ("<a href />", None, None),
        (
            r#"<Foo other="javascript:"></Foo>"#,
            Some(serde_json::json!([ [{ "name": "Foo", "props": ["to", "href"] }] ])),
            None,
        ),
        (
            r#"<Foo href="javascript:"></Foo>"#,
            None,
            Some(
                serde_json::json!({ "settings": {"react": {"linkComponents": [{ "name": "Foo", "linkAttribute": ["to", "href"] }]} } }),
            ),
        ),
        (
            r#"<Foo other="javascript:"></Foo>"#,
            Some(serde_json::json!([[], { "includeFromSettings": true }])),
            Some(
                serde_json::json!({ "settings": {"react": {"linkComponents": [{ "name": "Foo", "linkAttribute": ["to", "href"] }]} } }),
            ),
        ),
        (
            r#"<Foo href="javascript:"></Foo>"#,
            Some(serde_json::json!([[], { "includeFromSettings": false }])),
            Some(
                serde_json::json!({ "settings": {"react": {"linkComponents": [{ "name": "Foo", "linkAttribute": ["to", "href"] }]} } }),
            ),
        ),
    ];

    let fail = vec![
        (r#"<a href="javascript:"></a>"#, None, None),
        (r#"<a href="javascript:void(0)"></a>"#, None, None),
        (
            r#"<a href="j


			a
v	ascript:"></a>"#,
            None,
            None,
        ),
        (
            r#"<Foo to="javascript:"></Foo>"#,
            Some(serde_json::json!([ [{ "name": "Foo", "props": ["to", "href"] }] ])),
            None,
        ),
        (
            r#"<Foo href="javascript:"></Foo>"#,
            Some(serde_json::json!([ [{ "name": "Foo", "props": ["to", "href"] }] ])),
            None,
        ),
        (
            r#"<a href="javascript:void(0)"></a>"#,
            Some(serde_json::json!([ [{ "name": "Foo", "props": ["to", "href"] }] ])),
            None,
        ),
        (
            r#"<Foo to="javascript:"></Foo>"#,
            Some(
                serde_json::json!([ [{ "name": "Bar", "props": ["to", "href"] }], { "includeFromSettings": true } ]),
            ),
            Some(
                serde_json::json!({ "settings": {"react": {"linkComponents": [{ "name": "Foo", "linkAttribute": "to" }]}}}),
            ),
        ),
        (
            r#"<Foo href="javascript:"></Foo>"#,
            Some(serde_json::json!([{ "includeFromSettings": true }])),
            Some(
                serde_json::json!({ "settings": {"react": {"linkComponents": [{ "name": "Foo", "linkAttribute": ["to", "href"] }]} }}),
            ),
        ),
        (
            r#"
			      <div>
			        <Foo href="javascript:"></Foo>
			        <Bar link="javascript:"></Bar>
			      </div>
			    "#,
            Some(
                serde_json::json!([ [{ "name": "Bar", "props": ["link"] }], { "includeFromSettings": true } ]),
            ),
            Some(
                serde_json::json!({ "settings": {"react": {"linkComponents": [{ "name": "Foo", "linkAttribute": ["to", "href"] }]}} }),
            ),
        ),
        (
            r#"
			      <div>
			        <Foo href="javascript:"></Foo>
			        <Bar link="javascript:"></Bar>
			      </div>
			    "#,
            Some(serde_json::json!([ [{ "name": "Bar", "props": ["link"] }] ])),
            Some(
                serde_json::json!({ "settings": {"react": {"linkComponents": [{ "name": "Foo", "linkAttribute": ["to", "href"] }]}} }),
            ),
        ),
    ];

    Tester::new(JsxNoScriptUrl::NAME, JsxNoScriptUrl::PLUGIN, pass, fail).test_and_snapshot();
}
