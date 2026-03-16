use oxc_ast::{AstKind, ast::Argument};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, GetSpan, Span};
use rustc_hash::FxHashMap;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    rule::{DefaultRuleConfig, Rule},
    utils::{get_element_type, is_react_function_call},
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
#[serde(untagged)]
pub enum ForbidItem {
    ElementName(CompactStr),
    ElementWithMessage { element: CompactStr, message: Option<CompactStr> },
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
);

impl Rule for ForbidElements {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::JSXOpeningElement(jsx_el) => {
                let name = &get_element_type(ctx, jsx_el);

                self.add_diagnostic_if_invalid_element(ctx, name, jsx_el.name.span());
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
        ("<button />", Some(serde_json::json!([]))),
        ("<button />", Some(serde_json::json!([{ "forbid": [] }]))),
        ("<Button />", Some(serde_json::json!([{ "forbid": ["button"] }]))),
        ("<Button />", Some(serde_json::json!([{ "forbid": [{ "element": "button" }] }]))),
        ("React.createElement(button)", Some(serde_json::json!([{ "forbid": ["button"] }]))),
        (
            r#"NotReact.createElement("button")"#,
            Some(serde_json::json!([{ "forbid": ["button"] }])),
        ),
        (r#"React.createElement("_thing")"#, Some(serde_json::json!([{ "forbid": ["_thing"] }]))),
        (r#"React.createElement("Modal")"#, Some(serde_json::json!([{ "forbid": ["Modal"] }]))),
        (
            r#"React.createElement("dotted.component")"#,
            Some(serde_json::json!([{ "forbid": ["dotted.component"] }])),
        ),
        ("React.createElement(function() {})", Some(serde_json::json!([{ "forbid": ["button"] }]))),
        ("React.createElement({})", Some(serde_json::json!([{ "forbid": ["button"] }]))),
        ("React.createElement(1)", Some(serde_json::json!([{ "forbid": ["button"] }]))),
        ("React.createElement()", None),
    ];

    let fail = vec![
        ("<button />", Some(serde_json::json!([{ "forbid": ["button"] }]))),
        ("[<Modal />, <button />]", Some(serde_json::json!([{ "forbid": ["button", "Modal"] }]))),
        ("<dotted.component />", Some(serde_json::json!([{ "forbid": ["dotted.component"] }]))),
        (
            "<dotted.Component />",
            Some(
                serde_json::json!([{ "forbid": [{ "element": "dotted.Component", "message": "that ain\"t cool" }] }]),
            ),
        ),
        (
            "<button />",
            Some(
                serde_json::json!([{ "forbid": [{ "element": "button", "message": "use <Button> instead" }] }]),
            ),
        ),
        (
            "<button><input /></button>",
            Some(
                serde_json::json!([{ "forbid": [{ "element": "button" }, { "element": "input" }] }]),
            ),
        ),
        (
            "<button><input /></button>",
            Some(serde_json::json!([{ "forbid": [{ "element": "button" }, "input"] }])),
        ),
        (
            "<button><input /></button>",
            Some(serde_json::json!([{ "forbid": ["input", { "element": "button" }] }])),
        ),
        (
            "<button />",
            Some(
                serde_json::json!([{ "forbid": [{ "element": "button", "message": "use <Button> instead" }, { "element": "button", "message": "use <Button2> instead" } ] }]),
            ),
        ),
        (
            r#"React.createElement("button", {}, child)"#,
            Some(serde_json::json!([{ "forbid": ["button"] }])),
        ),
        (
            r#"[React.createElement(Modal), React.createElement("button")]"#,
            Some(serde_json::json!([{ "forbid": ["button", "Modal"] }])),
        ),
        (
            "React.createElement(dotted.Component)",
            Some(
                serde_json::json!([{ "forbid": [{ "element": "dotted.Component", "message": "that ain\"t cool" }] }]),
            ),
        ),
        (
            "React.createElement(dotted.component)",
            Some(serde_json::json!([{ "forbid": ["dotted.component"] }])),
        ),
        ("React.createElement(_comp)", Some(serde_json::json!([{ "forbid": ["_comp"] }]))),
        (
            r#"React.createElement("button")"#,
            Some(
                serde_json::json!([{ "forbid": [{ "element": "button", "message": "use <Button> instead" }] }]),
            ),
        ),
        (
            r#"React.createElement("button", {}, React.createElement("input"))"#,
            Some(
                serde_json::json!([{ "forbid": [{ "element": "button" }, { "element": "input" }] }]),
            ),
        ),
    ];

    Tester::new(ForbidElements::NAME, ForbidElements::PLUGIN, pass, fail).test_and_snapshot();
}
