use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, Span};
use rustc_hash::{FxHashMap, FxHashSet};
use serde_json::Value;

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    rule::Rule,
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
pub struct ForbidDomPropsConfig {
    forbid: FxHashMap<CompactStr, ForbidOptions>,
}
#[derive(Debug, Clone)]
enum ForbidOptions {
    AsStrings(()),
    AsObjects(ForbidObject),
}

#[derive(Debug, Clone)]
pub struct ForbidObject {
    disallowed_for: FxHashSet<CompactStr>,
    message: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub struct ForbidDomProps(Box<ForbidDomPropsConfig>);

impl std::ops::Deref for ForbidDomProps {
    type Target = ForbidDomPropsConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
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
    ///
    /// ### Options
    ///
    /// #### forbid
    ///
    /// An array of strings, with the names of props that are forbidden. The default value of this option [].
    /// Each array element can either be a string with the property name or object specifying the property name, an optional custom message, and a DOM nodes disallowed list (e.g. <div />)
    ///
    /// `{"propName": "someProp", "disallowedFor": ["DOMNode", "AnotherDOMNode"], "message": "Avoid using someProp" }`
    ForbidDomProps,
    react,
    restriction,
);

impl Rule for ForbidDomProps {
    fn from_configuration(value: serde_json::Value) -> Self {
        let mut forbid_map: FxHashMap<CompactStr, ForbidOptions> = FxHashMap::default();

        if let Some(config) = value.get(0)
            && let Some(forbid_array) = config.get("forbid").and_then(Value::as_array)
        {
            for item in forbid_array {
                match item {
                    Value::String(prop_name) => {
                        forbid_map.insert(CompactStr::new(prop_name), ForbidOptions::AsStrings(()));
                    }
                    Value::Object(obj) => {
                        if let Some(prop_name) =
                            obj.get("propName").and_then(Value::as_str).map(CompactStr::from)
                        {
                            let message =
                                obj.get("message").and_then(Value::as_str).map(String::from);

                            let disallowed_for: FxHashSet<CompactStr> = obj
                                .get("disallowedFor")
                                .and_then(Value::as_array)
                                .map(|arr| {
                                    arr.iter()
                                        .filter_map(Value::as_str)
                                        .map(CompactStr::from)
                                        .collect()
                                })
                                .unwrap_or_default();

                            forbid_map.insert(
                                prop_name,
                                ForbidOptions::AsObjects(ForbidObject { disallowed_for, message }),
                            );
                        }
                    }
                    _ => {}
                }
            }
        }

        Self(Box::new(ForbidDomPropsConfig { forbid: forbid_map }))
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
                let forbid = self.forbid.get(prop_name);

                if let Some(options) = forbid {
                    match options {
                        ForbidOptions::AsStrings(()) => {
                            ctx.diagnostic(forbid_dom_props_diagnostic(
                                attr_ident.span,
                                prop_name,
                                None,
                            ));
                        }
                        ForbidOptions::AsObjects(forbid_object) => {
                            if !forbid_object.disallowed_for.is_empty()
                                && !forbid_object.disallowed_for.contains(tag_name.as_str())
                            {
                                continue;
                            }

                            ctx.diagnostic(forbid_dom_props_diagnostic(
                                attr_ident.span,
                                prop_name,
                                forbid_object.message.as_ref(),
                            ));
                        }
                    }
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
