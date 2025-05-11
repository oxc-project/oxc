use lazy_regex::Regex;
use oxc_ast::{AstKind, ast::Argument};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, GetSpan, Span};
use serde::Deserialize;
use serde_json::Value;

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    rule::Rule,
    utils::{get_element_type, is_react_function_call},
};

fn forbid_elements_diagnostic(
    element: &CompactStr,
    help: Option<CompactStr>,
    span: Span,
) -> OxcDiagnostic {
    if let Some(help) = help {
        return OxcDiagnostic::warn(format!("<{element}> is forbidden."))
            .with_help(help)
            .with_label(span);
    }

    OxcDiagnostic::warn(format!("<{element}> is forbidden.")).with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct ForbidElements(Box<ForbidElementsConfig>);

impl std::ops::Deref for ForbidElements {
    type Target = ForbidElementsConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Default, Clone)]
pub struct ForbidElementsConfig {
    forbid_elements: Vec<ForbidElement>,
}

#[derive(Debug, Clone, Deserialize)]
struct ForbidElement {
    element: CompactStr,
    message: Option<CompactStr>,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Allows you to configure a list of forbidden elements and to specify their desired replacements.
    ///
    /// ### Why is this bad?
    ///
    /// You may want to forbid usage of certain elements in favor of others, (e.g. forbid all <div /> and use <Box /> instead)
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// // [1, { "forbid": ["button"] }]
    /// <button />
    /// React.createElement('button');
    ///
    /// // [1, { "forbid": ["Modal"] }]
    /// <Modal />
    /// React.createElement(Modal);
    ///
    /// // [1, { "forbid": ["Namespaced.Element"] }]
    /// <Namespaced.Element />
    /// React.createElement(Namespaced.Element);
    ///
    /// // [1, { "forbid": [{ "element": "button", "message": "use <Button> instead" }, "input"] }]
    /// <div><button /><input /></div>
    /// React.createElement('div', {}, React.createElement('button', {}, React.createElement('input')));
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// // [1, { "forbid": ["button"] }]
    /// <Button />
    ///
    /// // [1, { "forbid": [{ "element": "button" }] }]
    /// <Button />
    /// ```
    ForbidElements,
    react,
    restriction,
);

fn add_configuration_forbid_from_object(
    forbid_elements: &mut Vec<ForbidElement>,
    forbid_value: &serde_json::Value,
) {
    let Some(forbid_array) = forbid_value.as_array() else {
        return;
    };

    for forbid_value in forbid_array {
        match forbid_value {
            Value::String(element_name) => forbid_elements
                .push(ForbidElement { element: CompactStr::new(element_name), message: None }),
            Value::Object(_) => {
                if let Ok(forbid_element) =
                    serde_json::from_value::<ForbidElement>(forbid_value.clone())
                {
                    forbid_elements.push(forbid_element)
                }
            }
            _ => (),
        }
    }
}

impl Rule for ForbidElements {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::JSXOpeningElement(jsx_el) => {
                let name = &get_element_type(ctx, jsx_el);

                self.add_diagnostic_if_invalid_element(
                    ctx,
                    &CompactStr::new(name),
                    jsx_el.name.span(),
                );
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
                        if !Regex::new(r"^[A-Z_]").unwrap().is_match(it.name.as_str()) {
                            return;
                        }
                        self.add_diagnostic_if_invalid_element(
                            ctx,
                            &CompactStr::new(it.name.as_str()),
                            it.span,
                        );
                    }
                    Argument::StringLiteral(str) => {
                        if !Regex::new(r"^[a-z][^.]*$").unwrap().is_match(str.value.as_str()) {
                            return;
                        }
                        self.add_diagnostic_if_invalid_element(
                            ctx,
                            &CompactStr::new(str.value.as_str()),
                            str.span,
                        );
                    }
                    Argument::StaticMemberExpression(member_expression) => {
                        let Some(it) = member_expression.object.get_identifier_reference() else {
                            return;
                        };
                        self.add_diagnostic_if_invalid_element(
                            ctx,
                            &CompactStr::new(
                                format!("{}.{}", it.name, member_expression.property.name).as_str(),
                            ),
                            member_expression.span,
                        );
                    }
                    _ => {}
                }
            }
            _ => (),
        }
    }

    fn from_configuration(value: serde_json::Value) -> Self {
        let mut forbid_elements: Vec<ForbidElement> = Vec::new();

        match &value {
            Value::Array(configs) => {
                for config in configs {
                    match config {
                        Value::Object(obj) => {
                            if let Some(forbid_value) = obj.get("forbid") {
                                add_configuration_forbid_from_object(
                                    &mut forbid_elements,
                                    forbid_value,
                                );
                            }
                        }
                        _ => (),
                    }
                }
            }
            Value::Object(obj) => {
                if let Some(forbid_value) = obj.get("forbid") {
                    add_configuration_forbid_from_object(&mut forbid_elements, forbid_value);
                }
            }
            _ => {}
        }

        Self(Box::new(ForbidElementsConfig { forbid_elements }))
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_jsx()
    }
}

impl ForbidElements {
    fn add_diagnostic_if_invalid_element(&self, ctx: &LintContext, name: &CompactStr, span: Span) {
        for forbid_element in &self.forbid_elements {
            if forbid_element.element.as_str() != name.as_str() {
                continue;
            }

            ctx.diagnostic(forbid_elements_diagnostic(
                &forbid_element.element,
                forbid_element.message.clone(),
                span,
            ));
        }
    }
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
        // (r#"createElement("button")"#, Some(serde_json::json!([{ "forbid": ["button"] }]))),
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
                serde_json::json!([        {          "forbid": [            { "element": "dotted.Component", "message": "that ain\"t cool" },          ],        },      ]),
            ),
        ),
        (
            "<button />",
            Some(
                serde_json::json!([        {          "forbid": [            { "element": "button", "message": "use <Button> instead" },          ],        },      ]),
            ),
        ),
        (
            "<button><input /></button>",
            Some(
                serde_json::json!([        {          "forbid": [            { "element": "button" },            { "element": "input" },          ],        },      ]),
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
                serde_json::json!([        {          "forbid": [            { "element": "button", "message": "use <Button> instead" },            { "element": "button", "message": "use <Button2> instead" },          ],        },      ]),
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
                serde_json::json!([        {          "forbid": [            { "element": "dotted.Component", "message": "that ain\"t cool" },          ],        },      ]),
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
                serde_json::json!([        {          "forbid": [            { "element": "button", "message": "use <Button> instead" },          ],        },      ]),
            ),
        ),
        (
            r#"React.createElement("button", {}, React.createElement("input"))"#,
            Some(
                serde_json::json!([        {          "forbid": [            { "element": "button" }, { "element": "input" },          ],        },      ]),
            ),
        ),
    ];

    Tester::new(ForbidElements::NAME, ForbidElements::PLUGIN, pass, fail).test_and_snapshot();
}
