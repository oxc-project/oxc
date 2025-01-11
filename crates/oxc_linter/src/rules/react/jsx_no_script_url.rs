use lazy_static::lazy_static;
use regex::Regex;
use rustc_hash::FxHashMap;
use serde_json::Value;

use oxc_ast::{ast::JSXAttributeItem, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, GetSpan, Span};

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    AstNode,
};

fn jsx_no_script_url_diagnostic(span: Span) -> OxcDiagnostic {
    // See <https://oxc.rs/docs/contribute/linter/adding-rules.html#diagnostics> for details
    OxcDiagnostic::warn("A future version of React will block javascript: URLs as a security precaution.")
        .with_help("Use event handlers instead if you can. If you need to generate unsafe HTML, try using dangerouslySetInnerHTML instead.")
        .with_label(span)
}

lazy_static! {
    static ref JS_SCRIPT_REGEX: Regex =
        Regex::new(r"(j|J)[\r\n\t]*(a|A)[\r\n\t]*(v|V)[\r\n\t]*(a|A)[\r\n\t]*(s|S)[\r\n\t]*(c|C)[\r\n\t]*(r|R)[\r\n\t]*(i|I)[\r\n\t]*(p|P)[\r\n\t]*(t|T)[\r\n\t]*:").unwrap();
}

#[derive(Debug, Default, Clone)]
pub struct JsxNoScriptUrl(Box<JsxNoScriptUrlConfig>);

#[derive(Debug, Default, Clone)]
pub struct JsxNoScriptUrlConfig {
    include_from_settings: bool,
    components: FxHashMap<String, Vec<String>>,
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
    /// Disallow usage of `javascript:` URLs
    ///
    /// ### Why is this bad?
    ///
    /// URLs starting with `javascript:` are a dangerous attack surface because it’s easy to accidentally include unsanitized output in a tag like `<a href>` and create a security hole.
    /// In React 16.9 any URLs starting with `javascript:` scheme log a warning.
    /// In a future major release, React will throw an error if it encounters a `javascript:` URL.
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
    pending
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

    fn from_configuration(value: Value) -> Self {
        let mut components: FxHashMap<String, Vec<String>> = FxHashMap::default();
        if let Some(arr) = value.get(0).and_then(Value::as_array) {
            for component in arr {
                let name = component.get("name").and_then(Value::as_str).unwrap_or("").to_string();
                let props =
                    component.get("props").and_then(Value::as_array).map_or(vec![], |array| {
                        array
                            .iter()
                            .map(|prop| prop.as_str().map_or(String::new(), String::from))
                            .collect::<Vec<String>>()
                    });
                components.insert(name, props);
            }
            Self(Box::new(JsxNoScriptUrlConfig {
                include_from_settings: value.get(1).is_some_and(|conf| {
                    conf.get("includeFromSettings").and_then(Value::as_bool).is_some_and(|v| v)
                }),
                components,
            }))
        } else {
            Self(Box::new(JsxNoScriptUrlConfig {
                include_from_settings: value.get(0).is_some_and(|conf| {
                    conf.get("includeFromSettings").and_then(Value::as_bool).is_some_and(|v| v)
                }),
                components: FxHashMap::default(),
            }))
        }
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
            Some(serde_json::json!([        [{ "name": "Foo", "props": ["to", "href"] }],      ])),
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
            Some(serde_json::json!([        [{ "name": "Foo", "props": ["to", "href"] }],      ])),
            None,
        ),
        (
            r#"<Foo href="javascript:"></Foo>"#,
            Some(serde_json::json!([        [{ "name": "Foo", "props": ["to", "href"] }],      ])),
            None,
        ),
        (
            r#"<a href="javascript:void(0)"></a>"#,
            Some(serde_json::json!([        [{ "name": "Foo", "props": ["to", "href"] }],      ])),
            None,
        ),
        (
            r#"<Foo to="javascript:"></Foo>"#,
            Some(
                serde_json::json!([        [{ "name": "Bar", "props": ["to", "href"] }],        { "includeFromSettings": true },      ]),
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
                serde_json::json!([        [{ "name": "Bar", "props": ["link"] }],        { "includeFromSettings": true },      ]),
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
            Some(serde_json::json!([        [{ "name": "Bar", "props": ["link"] }],      ])),
            Some(
                serde_json::json!({ "settings": {"react": {"linkComponents": [{ "name": "Foo", "linkAttribute": ["to", "href"] }]}} }),
            ),
        ),
    ];

    Tester::new(JsxNoScriptUrl::NAME, JsxNoScriptUrl::PLUGIN, pass, fail).test_and_snapshot();
}
