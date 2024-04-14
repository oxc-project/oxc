use oxc_ast::{
    ast::{Argument, Expression, JSXAttributeItem, ObjectPropertyKind},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{get_element_type, get_jsx_attribute_name, is_create_element_call},
    AstNode,
};

#[derive(Debug, Error, Diagnostic)]
enum CheckedRequiresOnchangeOrReadonlyDiagnostic {
    #[error("eslint-plugin-react(checked-requires-onchange-or-readonly): `checked` should be used with either `onChange` or `readOnly`.")]
    #[diagnostic(severity(warning), help("Add either `onChange` or `readOnly`."))]
    MissingProperty(#[label] Span),

    #[error("eslint-plugin-react(checked-requires-onchange-or-readonly): Use either `checked` or `defaultChecked`, but not both.")]
    #[diagnostic(severity(warning), help("Remove either `checked` or `defaultChecked`."))]
    ExclusiveCheckedAttribute(#[label] Span, #[label] Span),
}

#[derive(Debug, Default, Clone)]
pub struct CheckedRequiresOnchangeOrReadonly {
    ignore_missing_properties: bool,
    ignore_exclusive_checked_attribute: bool,
}

declare_oxc_lint!(
    /// ### What it does
    /// This rule enforces onChange or readonly attribute for checked property of input elements.
    /// It also warns when checked and defaultChecked properties are used together.
    ///
    /// ### Example
    /// ```javascript
    /// // Bad
    /// <input type="checkbox" checked />
    /// <input type="checkbox" checked defaultChecked />
    /// <input type="radio" checked defaultChecked />
    ///
    /// React.createElement('input', { checked: false });
    /// React.createElement('input', { type: 'checkbox', checked: true });
    /// React.createElement('input', { type: 'checkbox', checked: true, defaultChecked: true });
    ///
    /// // Good
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
    pedantic
);

impl Rule for CheckedRequiresOnchangeOrReadonly {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::JSXOpeningElement(jsx_opening_el) => {
                let Some(element_type) = get_element_type(ctx, jsx_opening_el) else { return };
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
                    if !self.ignore_exclusive_checked_attribute {
                        if let Some(default_checked_span) = default_checked_span {
                            ctx.diagnostic(
                                CheckedRequiresOnchangeOrReadonlyDiagnostic::ExclusiveCheckedAttribute(
                                    checked_span,
                                    default_checked_span,
                                ),
                            );
                        }
                    }

                    if !self.ignore_missing_properties && is_missing_property {
                        ctx.diagnostic(
                            CheckedRequiresOnchangeOrReadonlyDiagnostic::MissingProperty(
                                checked_span,
                            ),
                        );
                    }
                }
            }
            AstKind::CallExpression(call_expr) => {
                if !is_create_element_call(call_expr) {
                    return;
                }

                let Some(Argument::Expression(Expression::StringLiteral(element_name))) =
                    call_expr.arguments.first()
                else {
                    return;
                };

                if element_name.value != "input" {
                    return;
                }

                let Some(Argument::Expression(Expression::ObjectExpression(obj_expr))) =
                    call_expr.arguments.get(1)
                else {
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
                    if !self.ignore_exclusive_checked_attribute {
                        if let Some(default_checked_span) = default_checked_span {
                            ctx.diagnostic(
                                CheckedRequiresOnchangeOrReadonlyDiagnostic::ExclusiveCheckedAttribute(
                                    checked_span,
                                    default_checked_span,
                                ),
                            );
                        }
                    }

                    if !self.ignore_missing_properties && is_missing_property {
                        ctx.diagnostic(
                            CheckedRequiresOnchangeOrReadonlyDiagnostic::MissingProperty(
                                checked_span,
                            ),
                        );
                    }
                }
            }
            _ => {}
        }
    }

    fn from_configuration(value: serde_json::Value) -> Self {
        let value = value.as_array().and_then(|arr| arr.first()).and_then(|val| val.as_object());

        Self {
            ignore_missing_properties: value
                .and_then(|val| {
                    val.get("ignoreMissingProperties").and_then(serde_json::Value::as_bool)
                })
                .unwrap_or(false),
            ignore_exclusive_checked_attribute: value
                .and_then(|val| {
                    val.get("ignoreExclusiveCheckedAttribute").and_then(serde_json::Value::as_bool)
                })
                .unwrap_or(false),
        }
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

    Tester::new(CheckedRequiresOnchangeOrReadonly::NAME, pass, fail).test_and_snapshot();
}
