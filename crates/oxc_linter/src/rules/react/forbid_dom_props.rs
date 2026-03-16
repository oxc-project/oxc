use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, Span};
use rustc_hash::{FxHashMap, FxHashSet};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    rule::{DefaultRuleConfig, Rule},
    utils::is_react_component_name,
};

fn forbid_dom_props_diagnostic(
    span: Span,
    property: &str,
    message: Option<&String>,
) -> OxcDiagnostic {
    if let Some(message) = message {
        return OxcDiagnostic::warn(message.clone()).with_label(span);
    }
    OxcDiagnostic::warn(format!("Prop \"{property}\" is forbidden on DOM Nodes")).with_label(span)
}

#[derive(Debug, Default, Clone)]
struct ForbidPropOptions {
    disallowed_for: FxHashSet<CompactStr>,
    message: Option<String>,
}

#[derive(Debug, Default, Clone, Deserialize, JsonSchema)]
#[serde(from = "ForbidDomPropsConfig")]
pub struct ForbidDomProps {
    #[serde(skip)]
    forbid: Box<FxHashMap<CompactStr, ForbidPropOptions>>,
}

impl From<ForbidDomPropsConfig> for ForbidDomProps {
    fn from(config: ForbidDomPropsConfig) -> Self {
        let mut forbid = FxHashMap::default();
        for item in config.forbid {
            match item {
                ForbidDomPropsItem::PropName(prop_name) => {
                    forbid.insert(prop_name, ForbidPropOptions::default());
                }
                ForbidDomPropsItem::PropWithOptions(PropWithOptions {
                    prop_name,
                    disallowed_for,
                    message,
                }) => {
                    forbid.insert(
                        prop_name,
                        ForbidPropOptions {
                            disallowed_for: disallowed_for
                                .unwrap_or_default()
                                .into_iter()
                                .collect(),
                            message,
                        },
                    );
                }
            }
        }
        Self { forbid: Box::new(forbid) }
    }
}

/// A forbidden prop, either as a plain prop name string or with options.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum ForbidDomPropsItem {
    /// A prop name to forbid on all DOM elements.
    PropName(CompactStr),
    /// A prop with optional `disallowedFor` DOM node list and custom `message`.
    PropWithOptions(PropWithOptions),
}

/// A prop with optional `disallowedFor` DOM node list and custom `message`.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PropWithOptions {
    /// The name of the prop to forbid.
    prop_name: CompactStr,
    /// A list of DOM element names (e.g. `["div", "span"]`) on which this
    /// prop is forbidden. If empty or omitted, the prop is forbidden on all
    /// DOM elements.
    disallowed_for: Option<Vec<CompactStr>>,
    /// A custom message to display when this prop is used.
    message: Option<String>,
}

/// Configuration for the `forbid-dom-props` rule.
#[derive(Debug, Default, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct ForbidDomPropsConfig {
    /// An array of prop names or objects that are forbidden on DOM elements.
    ///
    /// Each array element can be a string with the property name, or an object
    /// with `propName`, an optional `disallowedFor` array of DOM node names,
    /// and an optional custom `message`.
    ///
    /// Examples:
    ///
    /// - `["error", { "forbid": ["id", "style"] }]`
    /// - `["error", { "forbid": [{ "propName": "className", "message": "Use class instead" }] }]`
    /// - `["error", { "forbid": [{ "propName": "style", "disallowedFor": ["div", "span"] }] }]`
    forbid: Vec<ForbidDomPropsItem>,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule prevents passing of props to elements. This rule only applies to DOM Nodes (e.g. <div />) and not Components (e.g. <Component />). The list of forbidden props can be customized with the forbid option.
    ///
    /// ### Why is this bad?
    ///
    /// This rule checks all JSX elements and verifies that no forbidden props are used on DOM Nodes. This rule is off by default.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// // [1, { "forbid": ["id"] }]
    /// <div id='Joe' />
    ///
    /// // [1, { "forbid": ["style"] }]
    /// <div style={{color: 'red'}} />
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// // [1, { "forbid": ["id"] }]
    /// <Hello id='foo' />
    ///
    /// // [1, { "forbid": ["id"] }]
    /// <Hello id={{color: 'red'}} />
    /// ```
    ForbidDomProps,
    react,
    restriction,
    config = ForbidDomPropsConfig,
);

impl Rule for ForbidDomProps {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::JSXOpeningElement(jsx_elem) = node.kind() {
            let Some(tag_name) = jsx_elem.name.get_identifier_name() else {
                return;
            };

            if is_react_component_name(&tag_name) {
                return;
            }

            for attr_item in &jsx_elem.attributes {
                let Some(attr) = attr_item.as_attribute() else {
                    continue;
                };

                let Some(attr_ident) = attr.name.as_identifier() else {
                    continue;
                };

                let prop_name = attr_ident.name.as_str();
                if let Some(options) = self.forbid.get(prop_name) {
                    if !options.disallowed_for.is_empty()
                        && !options.disallowed_for.contains(tag_name.as_str())
                    {
                        continue;
                    }
                    ctx.diagnostic(forbid_dom_props_diagnostic(
                        attr_ident.span,
                        prop_name,
                        options.message.as_ref(),
                    ));
                }
            }
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_jsx() && !self.forbid.is_empty()
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            r#"
                    const First = (props) => (
                      <div name="foo" />
                    );
                  "#,
            None,
        ),
        (
            r#"
                    var First = createReactClass({
                      render: function() {
                        return <Foo id="foo" />;
                      }
                    });
                  "#,
            Some(serde_json::json!([{ "forbid": ["id"] }])),
        ),
        (
            r#"
                    var First = createReactClass({
                      propTypes: externalPropTypes,
                      render: function() {
                        return <Foo id="bar" style={{color: "red"}} />;
                      }
                    });
                  "#,
            Some(serde_json::json!([{ "forbid": ["style", "id"] }])),
        ),
        (
            r#"
                    var First = createReactClass({
                      propTypes: externalPropTypes,
                      render: function() {
                        return <this.Foo bar="baz" />;
                      }
                    });
                  "#,
            Some(serde_json::json!([{ "forbid": ["id"] }])),
        ),
        (
            r#"
                    class First extends createReactClass {
                      render() {
                        return <this.foo id="bar" />;
                      }
                    }
                  "#,
            Some(serde_json::json!([{ "forbid": ["id"] }])),
        ),
        (
            "
                    const First = (props) => (
                      <this.Foo {...props} />
                    );
                  ",
            Some(serde_json::json!([{ "forbid": ["id"] }])),
        ),
        (
            r#"
                    const First = (props) => (
                      <fbt:param name="name">{props.name}</fbt:param>
                    );
                  "#,
            Some(serde_json::json!([{ "forbid": ["id"] }])),
        ),
        (
            r#"
                    const First = (props) => (
                      <div name="foo" />
                    );
                  "#,
            Some(serde_json::json!([{ "forbid": ["id"] }])),
        ),
        (
            r#"
                    const First = (props) => (
                      <div otherProp="bar" />
                    );
                  "#,
            Some(
                serde_json::json!([{ "forbid": [{"propName": "otherProp","disallowedFor": ["span"],},],},]),
            ),
        ),
    ];

    let fail = vec![
        (
            r#"
                    var First = createReactClass({
                      propTypes: externalPropTypes,
                      render: function() {
                        return <div id="bar" />;
                      }
                    });
                  "#,
            Some(serde_json::json!([{ "forbid": ["id"] }])),
        ),
        (
            r#"
                    class First extends createReactClass {
                      render() {
                        return <div id="bar" />;
                      }
                    }
                  "#,
            Some(serde_json::json!([{ "forbid": ["id"] }])),
        ),
        (
            r#"
                    const First = (props) => (
                      <div id="foo" />
                    );
                  "#,
            Some(serde_json::json!([{ "forbid": ["id"] }])),
        ),
        (
            r#"
                    const First = (props) => (
                      <div className="foo" />
                    );
                  "#,
            Some(
                serde_json::json!([{"forbid": [{ "propName": "className", "message": "Please use class instead of ClassName" }],},]),
            ),
        ),
        (
            r#"
                    const First = (props) => (
                      <div className="foo">
                        <div otherProp="bar" />
                      </div>
                    );
                  "#,
            Some(
                serde_json::json!([{"forbid": [{ "propName": "className", "message": "Please use class instead of ClassName" },{ "propName": "otherProp", "message": "Avoid using otherProp" },],},]),
            ),
        ),
        (
            r#"
                    const First = (props) => (
                      <div className="foo">
                        <div otherProp="bar" />
                      </div>
                    );
                  "#,
            Some(
                serde_json::json!([{"forbid": [{ "propName": "className" },{ "propName": "otherProp", "message": "Avoid using otherProp" },],},]),
            ),
        ),
        (
            r#"
                    const First = (props) => (
                      <form accept='file'>
                        <input type="file" id="videoFile" accept="video/*" />
                        <input type="hidden" name="fullname" />
                      </form>
                    );
                  "#,
            Some(
                serde_json::json!([{"forbid": [{"propName": "accept", "disallowedFor": ["form"],"message": "Avoid using the accept attribute on <form>",}],},]),
            ),
        ),
        (
            r#"
                    const First = (props) => (
                      <div className="foo">
                        <input className="boo" />
                        <span className="foobar">Foobar</span>
                        <div otherProp="bar" className="forbiddenClassname" />
                      </div>
                    );
                  "#,
            Some(
                serde_json::json!([{"forbid": [{"propName": "className","disallowedFor": ["div", "span"],"message": "Please use class instead of ClassName",},{ "propName": "otherProp", "message": "Avoid using otherProp" },],},]),
            ),
        ),
    ];

    Tester::new(ForbidDomProps::NAME, ForbidDomProps::PLUGIN, pass, fail).test_and_snapshot();
}
