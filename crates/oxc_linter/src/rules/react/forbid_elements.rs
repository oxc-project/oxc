use oxc_ast::{AstKind, ast::Argument};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_str::CompactStr;
use rustc_hash::FxHashMap;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    rule::{DefaultRuleConfig, Rule},
    utils::{get_jsx_element_name, is_react_function_call},
};

fn forbid_elements_diagnostic(
    element: &str,
    help: Option<&CompactStr>,
    span: Span,
) -> OxcDiagnostic {
    if let Some(help) = help {
        return OxcDiagnostic::warn(format!("<{element}> is forbidden."))
            .with_help(help.to_string())
            .with_label(span);
    }

    OxcDiagnostic::warn(format!("<{element}> is forbidden.")).with_label(span)
}

#[derive(Debug, Default, Clone, Deserialize, JsonSchema)]
#[serde(from = "ForbidElementsConfig")]
pub struct ForbidElements {
    // Map from element name to optional message for diagnostics.
    forbid: Box<FxHashMap<CompactStr, Option<CompactStr>>>,
}

impl From<ForbidElementsConfig> for ForbidElements {
    fn from(config: ForbidElementsConfig) -> Self {
        // Convert Vec to FxHashMap.
        // Later entries will override earlier ones for duplicates.
        let mut forbid = FxHashMap::default();
        for item in config.forbid {
            match item {
                ForbidItem::ElementName(element) => {
                    forbid.insert(element, None);
                }
                ForbidItem::ElementWithMessage { element, message } => {
                    forbid.insert(element, message);
                }
            }
        }
        Self { forbid: Box::new(forbid) }
    }
}

/// A forbidden element, either as a plain element name or with a custom message.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(untagged, deny_unknown_fields)]
pub enum ForbidItem {
    ElementName(CompactStr),
    ElementWithMessage {
        /// The element name to forbid.
        element: CompactStr,
        /// The message to display when this element is found
        message: Option<CompactStr>,
    },
}

// Raw config for deserialization.
#[derive(Debug, Default, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct ForbidElementsConfig {
    /// List of forbidden elements, with optional messages for display with lint violations.
    ///
    /// Examples:
    ///
    /// - `["error, { "forbid": ["button"] }]`
    /// - `["error, { "forbid": [{ "element": "button", "message": "Use <Button> instead." }] }]`
    /// - `["error, { "forbid": [{ "element": "input" }] }]`
    forbid: Vec<ForbidItem>,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Allows you to configure a list of forbidden elements and to specify their desired replacements.
    ///
    /// ### Why is this bad?
    ///
    /// You may want to forbid usage of certain elements in favor of others, e.g.
    /// forbid all `<div />` and use `<Box />` instead.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// // ["error", { "forbid": ["button"] }]
    /// <button />
    /// React.createElement('button');
    ///
    /// // ["error", { "forbid": ["Modal"] }]
    /// <Modal />
    /// React.createElement(Modal);
    ///
    /// // ["error", { "forbid": ["Namespaced.Element"] }]
    /// <Namespaced.Element />
    /// React.createElement(Namespaced.Element);
    ///
    /// // ["error", { "forbid": [{ "element": "button", "message": "use <Button> instead" }, "input"] }]
    /// <div><button /><input /></div>
    /// React.createElement('div', {}, React.createElement('button', {}, React.createElement('input')));
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// // ["error", { "forbid": ["button"] }]
    /// <Button />
    ///
    /// // ["error", { "forbid": [{ "element": "button" }] }]
    /// <Button />
    /// ```
    ForbidElements,
    react,
    restriction,
    config = ForbidElementsConfig,
    version = "0.16.11",
    short_description = "Allows you to configure a list of forbidden elements and to specify their desired replacements.",
);

impl Rule for ForbidElements {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::JSXOpeningElement(jsx_el) => {
                // Use the literal JSX element name here, matching `jsx-ast-utils/elementType`
                // used by eslint-plugin-react. Do NOT use `get_element_type`, which applies
                // the jsx-a11y `components` alias map and polymorphic prop — those settings are
                // scoped to jsx-a11y rules and must not influence `forbid-elements`.
                let name = get_jsx_element_name(&jsx_el.name);

                self.add_diagnostic_if_invalid_element(ctx, &name, jsx_el.name.span());
            }
            AstKind::CallExpression(call_expr) => {
                if !is_react_function_call(call_expr, r"createElement") {
                    return;
                }

                let Some(argument) = call_expr.arguments.first() else {
                    return;
                };

                match argument {
                    Argument::Identifier(it) => {
                        if !is_valid_identifier(&it.name) {
                            return;
                        }
                        self.add_diagnostic_if_invalid_element(ctx, it.name.as_str(), it.span);
                    }
                    Argument::StringLiteral(str) => {
                        if !is_valid_literal(&str.value) {
                            return;
                        }
                        self.add_diagnostic_if_invalid_element(ctx, str.value.as_str(), str.span);
                    }
                    Argument::StaticMemberExpression(member_expression) => {
                        let Some(it) = member_expression.object.get_identifier_reference() else {
                            return;
                        };
                        let name = format!("{}.{}", it.name, member_expression.property.name);
                        self.add_diagnostic_if_invalid_element(ctx, &name, member_expression.span);
                    }
                    _ => {}
                }
            }
            _ => (),
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_jsx() && !self.forbid.is_empty()
    }
}

impl ForbidElements {
    fn add_diagnostic_if_invalid_element(&self, ctx: &LintContext, name: &str, span: Span) {
        if let Some(message) = self.forbid.get(name) {
            ctx.diagnostic(forbid_elements_diagnostic(name, message.as_ref(), span));
        }
    }
}

// Match /^[A-Z_]/
// https://github.com/jsx-eslint/eslint-plugin-react/blob/master/lib/rules/forbid-elements.js#L109
fn is_valid_identifier(str: &str) -> bool {
    str.chars().next().is_some_and(|c| c.is_uppercase() || c == '_')
}

// Match /^[a-z][^.]*$/
// https://github.com/jsx-eslint/eslint-plugin-react/blob/master/lib/rules/forbid-elements.js#L111
fn is_valid_literal(str: &str) -> bool {
    str.chars().next().is_some_and(char::is_lowercase) && !str.contains('.')
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("<button />", Some(serde_json::json!([])), None),
        ("<button />", Some(serde_json::json!([{ "forbid": [] }])), None),
        ("<Button />", Some(serde_json::json!([{ "forbid": ["button"] }])), None),
        ("<Button />", Some(serde_json::json!([{ "forbid": [{ "element": "button" }] }])), None),
        ("React.createElement(button)", Some(serde_json::json!([{ "forbid": ["button"] }])), None),
        (
            r#"NotReact.createElement("button")"#,
            Some(serde_json::json!([{ "forbid": ["button"] }])),
            None,
        ),
        (
            r#"React.createElement("_thing")"#,
            Some(serde_json::json!([{ "forbid": ["_thing"] }])),
            None,
        ),
        (
            r#"React.createElement("Modal")"#,
            Some(serde_json::json!([{ "forbid": ["Modal"] }])),
            None,
        ),
        (
            r#"React.createElement("dotted.component")"#,
            Some(serde_json::json!([{ "forbid": ["dotted.component"] }])),
            None,
        ),
        (
            "React.createElement(function() {})",
            Some(serde_json::json!([{ "forbid": ["button"] }])),
            None,
        ),
        ("React.createElement({})", Some(serde_json::json!([{ "forbid": ["button"] }])), None),
        ("React.createElement(1)", Some(serde_json::json!([{ "forbid": ["button"] }])), None),
        ("React.createElement()", None, None),
        // A custom component aliased to a DOM element via the jsx-a11y `components`
        // setting must NOT be treated as that DOM element here: `<Link>` is not `<a>`.
        // This setting is scoped to jsx-a11y rules only.
        (
            r#"<Link href="https://example.com">hi</Link>"#,
            Some(serde_json::json!([{ "forbid": ["a"] }])),
            Some(
                serde_json::json!({ "settings": { "jsx-a11y": { "components": { "Link": "a" } } } }),
            ),
        ),
    ];

    let fail = vec![
        ("<button />", Some(serde_json::json!([{ "forbid": ["button"] }])), None),
        (
            "[<Modal />, <button />]",
            Some(serde_json::json!([{ "forbid": ["button", "Modal"] }])),
            None,
        ),
        (
            "<dotted.component />",
            Some(serde_json::json!([{ "forbid": ["dotted.component"] }])),
            None,
        ),
        (
            "<dotted.Component />",
            Some(
                serde_json::json!([{ "forbid": [{ "element": "dotted.Component", "message": "that ain\"t cool" }] }]),
            ),
            None,
        ),
        (
            "<button />",
            Some(
                serde_json::json!([{ "forbid": [{ "element": "button", "message": "use <Button> instead" }] }]),
            ),
            None,
        ),
        (
            "<button><input /></button>",
            Some(
                serde_json::json!([{ "forbid": [{ "element": "button" }, { "element": "input" }] }]),
            ),
            None,
        ),
        (
            "<button><input /></button>",
            Some(serde_json::json!([{ "forbid": [{ "element": "button" }, "input"] }])),
            None,
        ),
        (
            "<button><input /></button>",
            Some(serde_json::json!([{ "forbid": ["input", { "element": "button" }] }])),
            None,
        ),
        (
            "<button />",
            Some(
                serde_json::json!([{ "forbid": [{ "element": "button", "message": "use <Button> instead" }, { "element": "button", "message": "use <Button2> instead" } ] }]),
            ),
            None,
        ),
        (
            r#"React.createElement("button", {}, child)"#,
            Some(serde_json::json!([{ "forbid": ["button"] }])),
            None,
        ),
        (
            r#"[React.createElement(Modal), React.createElement("button")]"#,
            Some(serde_json::json!([{ "forbid": ["button", "Modal"] }])),
            None,
        ),
        (
            "React.createElement(dotted.Component)",
            Some(
                serde_json::json!([{ "forbid": [{ "element": "dotted.Component", "message": "that ain\"t cool" }] }]),
            ),
            None,
        ),
        (
            "React.createElement(dotted.component)",
            Some(serde_json::json!([{ "forbid": ["dotted.component"] }])),
            None,
        ),
        ("React.createElement(_comp)", Some(serde_json::json!([{ "forbid": ["_comp"] }])), None),
        (
            r#"React.createElement("button")"#,
            Some(
                serde_json::json!([{ "forbid": [{ "element": "button", "message": "use <Button> instead" }] }]),
            ),
            None,
        ),
        (
            r#"React.createElement("button", {}, React.createElement("input"))"#,
            Some(
                serde_json::json!([{ "forbid": [{ "element": "button" }, { "element": "input" }] }]),
            ),
            None,
        ),
        // A literal `<a>` is still forbidden even when a `components` alias exists for it.
        (
            r#"<a href="https://example.com">hi</a>"#,
            Some(serde_json::json!([{ "forbid": ["a"] }])),
            Some(
                serde_json::json!({ "settings": { "jsx-a11y": { "components": { "Link": "a" } } } }),
            ),
        ),
        // Custom components are still forbidden by their real name.
        ("<Foo />", Some(serde_json::json!([{ "forbid": ["Foo"] }])), None),
    ];

    Tester::new(ForbidElements::NAME, ForbidElements::PLUGIN, pass, fail).test_and_snapshot();
}
