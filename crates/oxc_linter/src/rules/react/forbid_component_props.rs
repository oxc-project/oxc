use fast_glob::glob_match;
use oxc_ast::AstKind;
use oxc_ast::ast::JSXElementName;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_str::CompactStr;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::utils::{get_jsx_element_name, is_react_component_name};
use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    rule::{DefaultRuleConfig, Rule},
};

fn forbid_component_props_diagnostic(
    span: Span,
    prop: &str,
    message: Option<&str>,
) -> OxcDiagnostic {
    if let Some(message) = message {
        return OxcDiagnostic::warn(message.to_string()).with_label(span);
    }
    OxcDiagnostic::warn(format!("Prop \"{prop}\" is forbidden on Components")).with_label(span)
}

#[derive(Debug, Default, Clone)]
struct ForbidOption {
    /// Whether this entry was defined via `propNamePattern` (glob) instead of `propName`.
    is_pattern: bool,
    allowed_for: Vec<CompactStr>,
    allowed_for_patterns: Vec<CompactStr>,
    disallowed_for: Vec<CompactStr>,
    disallowed_for_patterns: Vec<CompactStr>,
    message: Option<String>,
}

impl ForbidOption {
    fn is_forbidden(&self, tag_name: Option<&str>) -> bool {
        let Some(tag) = tag_name else {
            return true;
        };

        let has_disallow =
            !self.disallowed_for.is_empty() || !self.disallowed_for_patterns.is_empty();

        if has_disallow {
            self.is_tag_forbidden_by_disallow(tag)
        } else {
            self.is_tag_forbidden_by_allow(tag)
        }
    }

    fn is_tag_forbidden_by_allow(&self, tag: &str) -> bool {
        if self.allowed_for.iter().any(|a| a.as_str() == tag) {
            return false;
        }
        if self.allowed_for_patterns.is_empty() {
            return true;
        }
        self.allowed_for_patterns.iter().all(|pat| !glob_match(pat.as_str(), tag))
    }

    fn is_tag_forbidden_by_disallow(&self, tag: &str) -> bool {
        if self.disallowed_for.iter().any(|d| d.as_str() == tag) {
            return true;
        }
        if self.disallowed_for_patterns.is_empty() {
            return false;
        }
        self.disallowed_for_patterns.iter().any(|pat| glob_match(pat.as_str(), tag))
    }
}

/// A forbidden prop, either as a plain prop name string or with options.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum ForbidItem {
    /// A prop name string to forbid on all components.
    PropName(CompactStr),
    /// An object with `propName` / `propNamePattern` and allow/disallow lists.
    Object(ForbidItemObject),
}

#[derive(Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ForbidItemObject {
    /// Exact prop name to forbid.
    prop_name: Option<CompactStr>,
    /// Glob pattern to match prop names against.
    prop_name_pattern: Option<CompactStr>,
    /// Component names for which this prop is **allowed** (all others are
    /// forbidden).
    allowed_for: Vec<CompactStr>,
    /// Glob patterns for component names where the prop is **allowed**.
    allowed_for_patterns: Vec<CompactStr>,
    /// Component names for which this prop is **disallowed** (all others are
    /// allowed).
    disallowed_for: Vec<CompactStr>,
    /// Glob patterns for component names where the prop is **disallowed**.
    disallowed_for_patterns: Vec<CompactStr>,
    /// Custom message to display.
    message: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
struct RawForbidItemObject {
    prop_name: Option<CompactStr>,
    prop_name_pattern: Option<CompactStr>,
    allowed_for: Vec<CompactStr>,
    allowed_for_patterns: Vec<CompactStr>,
    disallowed_for: Vec<CompactStr>,
    disallowed_for_patterns: Vec<CompactStr>,
    message: Option<String>,
}

impl<'de> Deserialize<'de> for ForbidItemObject {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = RawForbidItemObject::deserialize(deserializer)?;

        let has_prop_name = raw.prop_name.is_some();
        let has_prop_name_pattern = raw.prop_name_pattern.is_some();

        if has_prop_name == has_prop_name_pattern {
            return Err(serde::de::Error::custom(
                "forbid item object must include exactly one of `propName` or `propNamePattern`",
            ));
        }

        Ok(Self {
            prop_name: raw.prop_name,
            prop_name_pattern: raw.prop_name_pattern,
            allowed_for: raw.allowed_for,
            allowed_for_patterns: raw.allowed_for_patterns,
            disallowed_for: raw.disallowed_for,
            disallowed_for_patterns: raw.disallowed_for_patterns,
            message: raw.message,
        })
    }
}

#[derive(Debug, Default, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct ForbidComponentPropsConfig {
    /// An array specifying the names of props that are forbidden.
    ///
    /// The default value is `["className", "style"]`.
    ///
    /// Each array element can be a string with the property name, or an object with `propName` / `propNamePattern`,
    /// `allowedFor` / `allowedForPatterns`, `disallowedFor` / `disallowedForPatterns`, optional custom `message`
    ///
    /// **Pattern matching**: Uses glob patterns to match prop names and component names.
    /// For example, a `propNamePattern` of `"**-**"` would match any prop name that contains a hyphen, and an `allowedForPatterns` entry of `"*Icon"` would match component names like `SomeIcon` and `AnotherIcon`.
    /// Note that the pattern matching is done in Rust with the fast-glob library, and so may differ
    /// from the JavaScript glob library used by the original ESLint rule.
    ///
    /// Examples:
    ///
    /// - `["error", { "forbid": ["className", "style"] }]`
    /// - `["error", { "forbid": [{ "propName": "className", "message": "Use variant instead" }] }]`
    /// - `["error", { "forbid": [{ "propName": "className", "allowedFor": ["ReactModal"] }] }]`
    /// - `["error", { "forbid": [{ "propNamePattern": "**-**", "disallowedFor": ["Foo"] }] }]`
    forbid: Vec<ForbidItem>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(from = "ForbidComponentPropsConfig")]
pub struct ForbidComponentProps {
    #[serde(skip)]
    #[expect(
        clippy::box_collection,
        reason = "Keeps RuleEnum small, uses same pattern as `react/forbid-dom-props`"
    )]
    forbid: Box<Vec<(CompactStr, ForbidOption)>>,
}

impl Default for ForbidComponentProps {
    fn default() -> Self {
        Self::from(ForbidComponentPropsConfig::default())
    }
}

const DEFAULT_PROPS: &[&str] = &["className", "style"];

impl From<ForbidComponentPropsConfig> for ForbidComponentProps {
    fn from(config: ForbidComponentPropsConfig) -> Self {
        if config.forbid.is_empty() {
            let forbid = DEFAULT_PROPS
                .iter()
                .map(|&name| (CompactStr::new(name), ForbidOption::default()))
                .collect();
            return Self { forbid: Box::new(forbid) };
        }

        let forbid = config
            .forbid
            .into_iter()
            .map(|item| match item {
                ForbidItem::PropName(name) => (name, ForbidOption::default()),
                ForbidItem::Object(obj) => {
                    let (key, is_pattern) = match (obj.prop_name, obj.prop_name_pattern) {
                        (Some(prop_name), None) => (prop_name, false),
                        (None, Some(prop_name_pattern)) => (prop_name_pattern, true),
                        _ => unreachable!(
                            "ForbidItemObject deserialization enforces exactly one matcher key"
                        ),
                    };
                    (
                        key,
                        ForbidOption {
                            is_pattern,
                            allowed_for: obj.allowed_for,
                            allowed_for_patterns: obj.allowed_for_patterns,
                            disallowed_for: obj.disallowed_for,
                            disallowed_for_patterns: obj.disallowed_for_patterns,
                            message: obj.message,
                        },
                    )
                }
            })
            .collect();

        Self { forbid: Box::new(forbid) }
    }
}

impl ForbidComponentProps {
    fn get_prop_options(&self, prop: &str) -> Option<&ForbidOption> {
        for (key, opt) in self.forbid.iter() {
            if opt.is_pattern {
                if glob_match(key.as_str(), prop) {
                    return Some(opt);
                }
            } else if key.as_str() == prop {
                return Some(opt);
            }
        }

        None
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule prevents passing of props to components. This rule only applies to Components (e.g. `<Foo />`) and not DOM nodes (e.g. `<div />`).
    /// By default this rule prevents passing of props that add lots of complexity (`className`, `style`) to Components.
    /// The list of forbidden props can be customized with the forbid option.
    ///
    /// ### Why is this bad?
    ///
    /// This rule checks all JSX elements and verifies that no forbidden props are used on components. This rule is off by default.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <Hello className='foo' />
    /// <Hello style={{color: 'red'}} />
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <Hello name='Joe' />
    /// <div className='foo' />
    /// <div style={{color: 'red'}} />
    /// ```
    ForbidComponentProps,
    react,
    restriction,
    config = ForbidComponentPropsConfig,
    version = "next"
);

impl Rule for ForbidComponentProps {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(jsx_elem) = node.kind() else {
            return;
        };

        let component_name = get_component_name(&jsx_elem.name);
        let Some(name) = component_name else {
            return;
        };

        if !is_react_component_name(name) {
            return;
        }

        let tag_name = get_jsx_element_name(&jsx_elem.name);

        let tag_name_for_check = if component_name.is_some() { Some(tag_name) } else { None };

        for attr_item in &jsx_elem.attributes {
            let Some(attr) = attr_item.as_attribute() else {
                continue;
            };

            let prop_name = attr.name.get_identifier().name.as_str();

            let Some(option) = self.get_prop_options(prop_name) else {
                continue;
            };

            if option.is_forbidden(tag_name_for_check.as_deref()) {
                ctx.diagnostic(forbid_component_props_diagnostic(
                    attr.name.span(),
                    prop_name,
                    option.message.as_deref(),
                ));
            }
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_jsx()
    }
}

/// Returns the rightmost identifier of a JSX element name
/// used to determine whether the element is a React component
fn get_component_name<'a>(name: &'a JSXElementName<'a>) -> Option<&'a str> {
    match name {
        JSXElementName::Identifier(ident) => Some(ident.name.as_str()),
        JSXElementName::IdentifierReference(ident) => Some(ident.name.as_str()),
        JSXElementName::NamespacedName(ns) => Some(ns.name.name.as_str()),
        JSXElementName::MemberExpression(member) => Some(member.property.name.as_str()),
        JSXElementName::ThisExpression(_) => None,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            r#"
                    var First = createReactClass({
                      render: function() {
                        return <div className="foo" />;
                      }
                    });
                  "#,
            None,
        ),
        (
            r#"
                    var First = createReactClass({
                      render: function() {
                        return <div style={{color: "red"}} />;
                      }
                    });
                  "#,
            Some(serde_json::json!([{ "forbid": ["style"] }])),
        ),
        (
            r#"
                    var First = createReactClass({
                      propTypes: externalPropTypes,
                      render: function() {
                        return <Foo bar="baz" />;
                      }
                    });
                  "#,
            None,
        ),
        (
            r#"
                    var First = createReactClass({
                      propTypes: externalPropTypes,
                      render: function() {
                        return <Foo className="bar" />;
                      }
                    });
                  "#,
            Some(serde_json::json!([{ "forbid": ["style"] }])),
        ),
        (
            r#"
                    var First = createReactClass({
                      propTypes: externalPropTypes,
                      render: function() {
                        return <Foo className="bar" />;
                      }
                    });
                  "#,
            Some(serde_json::json!([{ "forbid": ["style", "foo"] }])),
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
            None,
        ),
        (
            r#"
                    class First extends createReactClass {
                      render() {
                        return <this.foo className="bar" />;
                      }
                    }
                  "#,
            Some(serde_json::json!([{ "forbid": ["style"] }])),
        ),
        (
            "
                    const First = (props) => (
                      <this.Foo {...props} />
                    );
                  ",
            None,
        ),
        (
            r#"
                    const item = (<ReactModal className="foo" />);
                  "#,
            Some(
                serde_json::json!([ { "forbid": [ { "propName": "className", "allowedFor": ["ReactModal"], }, ], }, ]),
            ),
        ),
        (
            r#"
                    const item = (<AntdLayout.Content className="antdFoo" />);
                  "#,
            Some(
                serde_json::json!([ { "forbid": [ { "propName": "className", "allowedFor": ["AntdLayout.Content"], }, ], }, ]),
            ),
        ),
        (
            r#"
                    const item = (<this.ReactModal className="foo" />);
                  "#,
            Some(
                serde_json::json!([ { "forbid": [ { "propName": "className", "allowedFor": ["this.ReactModal"], }, ], }, ]),
            ),
        ),
        (
            r#"
                    const item = (<Foo className="foo" />);
                  "#,
            Some(
                serde_json::json!([ { "forbid": [ { "propName": "className", "disallowedFor": ["ReactModal"], }, ], }, ]),
            ),
        ),
        (
            r#"
                    <fbt:param name="Total number of files" number={true} />
                  "#,
            None,
        ),
        (
            r#"
                    const item = (
                      <Foo className="bar">
                        <ReactModal style={{color: "red"}} />
                      </Foo>
                    );
                  "#,
            Some(
                serde_json::json!([ { "forbid": [ { "propName": "className", "disallowedFor": ["OtherModal", "ReactModal"], }, { "propName": "style", "disallowedFor": ["Foo"], }, ], }, ]),
            ),
        ),
        (
            r#"
                    const item = (
                      <Foo className="bar">
                        <ReactModal style={{color: "red"}} />
                      </Foo>
                    );
                  "#,
            Some(
                serde_json::json!([ { "forbid": [ { "propName": "className", "disallowedFor": ["OtherModal", "ReactModal"], }, { "propName": "style", "allowedFor": ["ReactModal"], }, ], }, ]),
            ),
        ),
        (
            r#"
                    const item = (<this.ReactModal className="foo" />);
                  "#,
            Some(
                serde_json::json!([ { "forbid": [ { "propName": "className", "disallowedFor": ["ReactModal"], }, ], }, ]),
            ),
        ),
        (
            r#"
                    const MyComponent = () => (
                      <div aria-label="welcome" />
                    );
                  "#,
            Some(
                serde_json::json!([ { "forbid": [ { "propNamePattern": "**-**", "allowedFor": ["div"], }, ], }, ]),
            ),
        ),
        (
            r#"
                    const rootElement = (
                      <Root>
                        <SomeIcon className="size-lg" />
                        <AnotherIcon className="size-lg" />
                        <SomeSvg className="size-lg" />
                        <UICard className="size-lg" />
                        <UIButton className="size-lg" />
                      </Root>
                    );
                  "#,
            Some(
                serde_json::json!([ { "forbid": [ { "propName": "className", "allowedForPatterns": ["*Icon", "*Svg", "UI*"], }, ], }, ]),
            ),
        ),
        (
            r#"
                    const rootElement = (
                      <Root>
                        <SomeIcon className="size-lg" />
                        <AnotherIcon className="size-lg" />
                        <SomeSvg className="size-lg" />
                        <UICard className="size-lg" />
                        <UIButton className="size-lg" />
                        <ButtonLegacy className="size-lg" />
                      </Root>
                    );
                  "#,
            Some(
                serde_json::json!([ { "forbid": [ { "propName": "className", "allowedFor": ["ButtonLegacy"], "allowedForPatterns": ["*Icon", "*Svg", "UI*"], }, ], }, ]),
            ),
        ),
        (
            r#"
                    const rootElement = (
                      <Root>
                        <SomeIcon className="size-lg" />
                        <AnotherIcon className="size-lg" />
                        <SomeSvg className="size-lg" />
                        <UICard className="size-lg" />
                        <UIButton className="size-lg" />
                      </Root>
                    );
                  "#,
            Some(
                serde_json::json!([ { "forbid": [ { "propName": "className", "disallowedFor": ["Modal"], "disallowedForPatterns": ["*Legacy", "Shared*"], }, ], }, ]),
            ),
        ),
    ];

    let fail = vec![
        (
            r#"
                    var First = createReactClass({
                      propTypes: externalPropTypes,
                      render: function() {
                        return <Foo className="bar" />;
                      }
                    });
                  "#,
            None,
        ),
        (
            r#"
                    var First = createReactClass({
                      propTypes: externalPropTypes,
                      render: function() {
                        return <Foo style={{color: "red"}} />;
                      }
                    });
                  "#,
            None,
        ),
        (
            r#"
                    var First = createReactClass({
                      propTypes: externalPropTypes,
                      render: function() {
                        return <Foo className="bar" />;
                      }
                    });
                  "#,
            Some(serde_json::json!([{ "forbid": ["className", "style"] }])),
        ),
        (
            r#"
                    var First = createReactClass({
                      propTypes: externalPropTypes,
                      render: function() {
                        return <Foo style={{color: "red"}} />;
                      }
                    });
                  "#,
            Some(serde_json::json!([{ "forbid": ["className", "style"] }])),
        ),
        (
            r#"
                    var First = createReactClass({
                      propTypes: externalPropTypes,
                      render: function() {
                        return <Foo style={{color: "red"}} />;
                      }
                    });
                  "#,
            Some(
                serde_json::json!([ { "forbid": [ { "propName": "style", "disallowedFor": ["Foo"], }, ], }, ]),
            ),
        ),
        (
            r#"
                    const item = (<Foo className="foo" />);
                  "#,
            Some(
                serde_json::json!([ { "forbid": [ { "propName": "className", "allowedFor": ["ReactModal"], }, ], }, ]),
            ),
        ),
        (
            r#"
                    const item = (<this.ReactModal className="foo" />);
                  "#,
            Some(
                serde_json::json!([ { "forbid": [ { "propName": "className", "allowedFor": ["ReactModal"], }, ], }, ]),
            ),
        ),
        (
            r#"
                    const item = (<this.ReactModal className="foo" />);
                  "#,
            Some(
                serde_json::json!([ { "forbid": [ { "propName": "className", "disallowedFor": ["this.ReactModal"], }, ], }, ]),
            ),
        ),
        (
            r#"
                    const item = (<ReactModal className="foo" />);
                  "#,
            Some(
                serde_json::json!([ { "forbid": [ { "propName": "className", "disallowedFor": ["ReactModal"], }, ], }, ]),
            ),
        ),
        (
            r#"
                    const item = (<AntdLayout.Content className="antdFoo" />);
                  "#,
            Some(
                serde_json::json!([ { "forbid": [ { "propName": "className", "disallowedFor": ["AntdLayout.Content"], }, ], }, ]),
            ),
        ),
        (
            r#"
                    const item = (<Foo className="foo" />);
                  "#,
            Some(
                serde_json::json!([ { "forbid": [ { "propName": "className", "message": "Please use ourCoolClassName instead of ClassName", }, ], }, ]),
            ),
        ),
        (
            r#"
                    const item = () => (
                      <Foo className="foo">
                        <Bar option="high" />
                      </Foo>
                    );
                  "#,
            Some(
                serde_json::json!([ { "forbid": [ { "propName": "className", "message": "Please use ourCoolClassName instead of ClassName", }, { "propName": "option", "message": "Avoid using option", }, ], }, ]),
            ),
        ),
        (
            r#"
                    const item = () => (
                      <Foo className="foo">
                        <Bar option="high" />
                      </Foo>
                    );
                  "#,
            Some(
                serde_json::json!([ { "forbid": [ { "propName": "className" }, { "propName": "option", "message": "Avoid using option", }, ], }, ]),
            ),
        ),
        (
            "
                    const MyComponent = () => (
                      <Foo kebab-case-prop={123} />
                    );
                  ",
            Some(serde_json::json!([ { "forbid": [ { "propNamePattern": "**-**", }, ], }, ])),
        ),
        (
            "
                    const MyComponent = () => (
                      <Foo kebab-case-prop={123} />
                    );
                  ",
            Some(
                serde_json::json!([ { "forbid": [ { "propNamePattern": "**-**", "message": "Avoid using kebab-case", }, ], }, ]),
            ),
        ),
        (
            r#"
                    const MyComponent = () => (
                      <div>
                        <div aria-label="Hello Akul" />
                        <Foo kebab-case-prop={123} />
                      </div>
                    );
                  "#,
            Some(
                serde_json::json!([ { "forbid": [ { "propNamePattern": "**-**", "allowedFor": ["div"], }, ], }, ]),
            ),
        ),
        (
            r#"
                    const MyComponent = () => (
                      <div>
                        <div aria-label="Hello Akul" />
                        <h1 data-id="my-heading" />
                        <Foo kebab-case-prop={123} />
                      </div>
                    );
                  "#,
            Some(
                serde_json::json!([ { "forbid": [ { "propNamePattern": "**-**", "disallowedFor": ["Foo"], }, ], }, ]),
            ),
        ),
        (
            r#"
                    const rootElement = () => (
                      <Root>
                        <SomeIcon className="size-lg" />
                        <SomeSvg className="size-lg" />
                      </Root>
                    );
                  "#,
            Some(
                serde_json::json!([ { "forbid": [ { "propName": "className", "message": "className available only for icons", "allowedForPatterns": ["*Icon"], }, ], }, ]),
            ),
        ),
        (
            r#"
                    const rootElement = () => (
                      <Root>
                        <UICard style={{backgroundColor: black}}/>
                        <SomeIcon className="size-lg" />
                        <SomeSvg className="size-lg" style={{fill: currentColor}} />
                      </Root>
                    );
                  "#,
            Some(
                serde_json::json!([ { "forbid": [ { "propName": "className", "message": "className available only for icons", "allowedForPatterns": ["*Icon"], }, { "propName": "style", "message": "style available only for SVGs", "allowedForPatterns": ["*Svg"], }, ], }, ]),
            ),
        ),
        (
            r#"
                    const rootElement = (
                      <Root>
                        <SomeIcon className="size-lg" />
                        <AnotherIcon className="size-lg" />
                        <SomeSvg className="size-lg" />
                        <UICard className="size-lg" />
                        <ButtonLegacy className="size-lg" />
                      </Root>
                    );
                  "#,
            Some(
                serde_json::json!([ { "forbid": [ { "propName": "className", "disallowedFor": ["SomeSvg"], "disallowedForPatterns": ["UI*", "*Icon"], "message": "Avoid using className for SomeSvg and components that match the `UI*` and `*Icon` patterns", }, ], }, ]),
            ),
        ),
    ];

    Tester::new(ForbidComponentProps::NAME, ForbidComponentProps::PLUGIN, pass, fail)
        .test_and_snapshot();
}

#[test]
fn invalid_configs_error_in_from_configuration() {
    let missing_both = serde_json::json!([{
        "forbid": [{
            "allowedFor": ["Foo"]
        }]
    }]);
    assert!(ForbidComponentProps::from_configuration(missing_both).is_err());

    let has_both = serde_json::json!([{
        "forbid": [{
            "propName": "className",
            "propNamePattern": "**-**"
        }]
    }]);
    assert!(ForbidComponentProps::from_configuration(has_both).is_err());
}
