use oxc_ast::{
    AstKind,
    ast::{Argument, JSXAttributeItem, ObjectPropertyKind},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
    utils::{get_element_type, get_jsx_attribute_name, is_create_element_call},
};

fn missing_property(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("`checked` should be used with either `onChange` or `readOnly`.")
        .with_help("Add either `onChange` or `readOnly`.")
        .with_label(span)
}

fn exclusive_checked_attribute(checked_span: Span, default_checked_span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Use either `checked` or `defaultChecked`, but not both.")
        .with_help("Remove either `checked` or `defaultChecked`.")
        .with_labels([checked_span, default_checked_span])
}

#[derive(Debug, Default, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CheckedRequiresOnchangeOrReadonly {
    /// Ignore the requirement to provide either `onChange` or `readOnly` when the `checked` prop is present.
    ignore_missing_properties: bool,
    /// Ignore the restriction that `checked` and `defaultChecked` should not be used together.
    ignore_exclusive_checked_attribute: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule enforces onChange or readonly attribute for checked property of input elements.
    /// It also warns when checked and defaultChecked properties are used together.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <input type="checkbox" checked />
    /// <input type="checkbox" checked defaultChecked />
    /// <input type="radio" checked defaultChecked />
    ///
    /// React.createElement('input', { checked: false });
    /// React.createElement('input', { type: 'checkbox', checked: true });
    /// React.createElement('input', { type: 'checkbox', checked: true, defaultChecked: true });
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <input type="checkbox" checked onChange={() => {}} />
    /// <input type="checkbox" checked readOnly />
    /// <input type="checkbox" checked onChange readOnly />
    /// <input type="checkbox" defaultChecked />
    ///
    /// React.createElement('input', { type: 'checkbox', checked: true, onChange() {} });
    /// React.createElement('input', { type: 'checkbox', checked: true, readOnly: true });
    /// React.createElement('input', { type: 'checkbox', checked: true, onChange() {}, readOnly: true });
    /// React.createElement('input', { type: 'checkbox', defaultChecked: true });
    /// ```
    CheckedRequiresOnchangeOrReadonly,
    react,
    pedantic,
    config = CheckedRequiresOnchangeOrReadonly,
);

impl Rule for CheckedRequiresOnchangeOrReadonly {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::JSXOpeningElement(jsx_opening_el) => {
                let element_type = get_element_type(ctx, jsx_opening_el);

                if element_type != "input" {
                    return;
                }

                let (checked_span, default_checked_span, is_missing_property) =
                    jsx_opening_el.attributes.iter().fold(
                        (None, None, true),
                        |(checked_span, default_checked_span, is_missing_property), attr| {
                            if let JSXAttributeItem::Attribute(jsx_attr) = attr {
                                let name = get_jsx_attribute_name(&jsx_attr.name);
                                (
                                    if name == "checked" {
                                        Some(jsx_attr.span)
                                    } else {
                                        checked_span
                                    },
                                    if default_checked_span.is_none() && name == "defaultChecked" {
                                        Some(jsx_attr.span)
                                    } else {
                                        default_checked_span
                                    },
                                    is_missing_property
                                        && !(name == "onChange" || name == "readOnly"),
                                )
                            } else {
                                (checked_span, default_checked_span, is_missing_property)
                            }
                        },
                    );

                if let Some(checked_span) = checked_span {
                    if !self.ignore_exclusive_checked_attribute
                        && let Some(default_checked_span) = default_checked_span
                    {
                        ctx.diagnostic(exclusive_checked_attribute(
                            checked_span,
                            default_checked_span,
                        ));
                    }

                    if !self.ignore_missing_properties && is_missing_property {
                        ctx.diagnostic(missing_property(checked_span));
                    }
                }
            }
            AstKind::CallExpression(call_expr) => {
                if !is_create_element_call(call_expr) {
                    return;
                }

                let Some(Argument::StringLiteral(element_name)) = call_expr.arguments.first()
                else {
                    return;
                };

                if element_name.value != "input" {
                    return;
                }

                let Some(Argument::ObjectExpression(obj_expr)) = call_expr.arguments.get(1) else {
                    return;
                };

                let (checked_span, default_checked_span, is_missing_property) =
                    obj_expr.properties.iter().fold(
                        (None, None, true),
                        |(checked_span, default_checked_span, is_missing_property), prop| {
                            if let ObjectPropertyKind::ObjectProperty(object_prop) = prop {
                                if let Some(name) = object_prop.key.static_name() {
                                    (
                                        if checked_span.is_none() && name == "checked" {
                                            Some(object_prop.span)
                                        } else {
                                            checked_span
                                        },
                                        if default_checked_span.is_none()
                                            && name == "defaultChecked"
                                        {
                                            Some(object_prop.span)
                                        } else {
                                            default_checked_span
                                        },
                                        is_missing_property
                                            && !(name == "onChange" || name == "readOnly"),
                                    )
                                } else {
                                    (checked_span, default_checked_span, is_missing_property)
                                }
                            } else {
                                (checked_span, default_checked_span, is_missing_property)
                            }
                        },
                    );

                if let Some(checked_span) = checked_span {
                    if !self.ignore_exclusive_checked_attribute
                        && let Some(default_checked_span) = default_checked_span
                    {
                        ctx.diagnostic(exclusive_checked_attribute(
                            checked_span,
                            default_checked_span,
                        ));
                    }

                    if !self.ignore_missing_properties && is_missing_property {
                        ctx.diagnostic(missing_property(checked_span));
                    }
                }
            }
            _ => {}
        }
    }

    fn from_configuration(value: serde_json::Value) -> Self {
        serde_json::from_value::<DefaultRuleConfig<CheckedRequiresOnchangeOrReadonly>>(value)
            .unwrap_or_default()
            .into_inner()
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (r"<input type='checkbox' />", None),
        (r"<input type='checkbox' onChange={noop} />", None),
        (r"<input type='checkbox' readOnly />", None),
        (r"<input type='checkbox' checked onChange={noop} />", None),
        (r"<input type='checkbox' checked={true} onChange={noop} />", None),
        (r"<input type='checkbox' checked={false} onChange={noop} />", None),
        (r"<input type='checkbox' checked readOnly />", None),
        (r"<input type='checkbox' checked={true} readOnly />", None),
        (r"<input type='checkbox' checked={false} readOnly />", None),
        (r"<input type='checkbox' defaultChecked />", None),
        (r"React.createElement('input')", None),
        (r"React.createElement('input', { checked: true, onChange: noop })", None),
        (r"React.createElement('input', { checked: false, onChange: noop })", None),
        (r"React.createElement('input', { checked: true, readOnly: true })", None),
        (r"React.createElement('input', { checked: true, onChange: noop, readOnly: true })", None),
        (r"React.createElement('input', { checked: foo, onChange: noop, readOnly: true })", None),
        (
            r"<input type='checkbox' checked />",
            Some(serde_json::json!([{ "ignoreMissingProperties": true }])),
        ),
        (
            r"<input type='checkbox' checked={true} />",
            Some(serde_json::json!([{ "ignoreMissingProperties": true }])),
        ),
        (
            r"<input type='checkbox' onChange={noop} checked defaultChecked />",
            Some(serde_json::json!([{ "ignoreExclusiveCheckedAttribute": true }])),
        ),
        (
            r"<input type='checkbox' onChange={noop} checked={true} defaultChecked />",
            Some(serde_json::json!([{ "ignoreExclusiveCheckedAttribute": true }])),
        ),
        (
            r"<input type='checkbox' onChange={noop} checked defaultChecked />",
            Some(
                serde_json::json!([{ "ignoreMissingProperties": true, "ignoreExclusiveCheckedAttribute": true }]),
            ),
        ),
        (r"<span/>", None),
        (r"React.createElement('span')", None),
        (r"(()=>{})()", None),
    ];

    let fail = vec![
        (r"<input type='radio' checked />", None),
        (r"<input type='radio' checked={true} />", None),
        (r"<input type='checkbox' checked />", None),
        (r"<input type='checkbox' checked={true} />", None),
        (r"<input type='checkbox' checked={condition ? true : false} />", None),
        (r"<input type='checkbox' checked defaultChecked />", None),
        (r"React.createElement('input', { checked: false })", None),
        (r"React.createElement('input', { checked: true, defaultChecked: true })", None),
        (
            r"<input type='checkbox' checked defaultChecked />",
            Some(serde_json::json!([{ "ignoreMissingProperties": true }])),
        ),
        (
            r"<input type='checkbox' checked defaultChecked />",
            Some(serde_json::json!([{ "ignoreExclusiveCheckedAttribute": true }])),
        ),
        (
            r"<input type='checkbox' checked defaultChecked />",
            Some(
                serde_json::json!([{ "ignoreMissingProperties": false, "ignoreExclusiveCheckedAttribute": false }]),
            ),
        ),
    ];

    Tester::new(
        CheckedRequiresOnchangeOrReadonly::NAME,
        CheckedRequiresOnchangeOrReadonly::PLUGIN,
        pass,
        fail,
    )
    .test_and_snapshot();
}
