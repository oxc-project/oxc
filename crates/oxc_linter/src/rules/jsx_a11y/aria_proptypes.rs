use lazy_static::lazy_static;
use oxc_ast::{
    ast::{Expression, JSXAttributeItem, JSXAttributeValue},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use std::collections::HashMap;

use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    globals::VALID_ARIA_PROPS,
    rule::Rule,
    utils::{get_attribute_name, get_prop_value, parse_jsx_value},
    AstNode,
};

fn aria_proptypes_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("eslint-plugin-jsx-a11y(aria-proptypes):").with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct AriaProptypes;

declare_oxc_lint!(
    /// ### What it does
    ///
    ///
    /// ### Why is this bad?
    ///
    ///
    /// ### Example
    /// ```javascript
    /// ```
    AriaProptypes,
    correctness
);

impl Rule for AriaProptypes {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::JSXAttributeItem(attr_item) = node.kind() {
            let prop_value = get_prop_value(attr_item);
            match attr_item {
                JSXAttributeItem::Attribute(attr) => {
                    let attr_name = get_attribute_name(&attr.name).to_lowercase();
                    if !attr_name.starts_with("aria-") || !VALID_ARIA_PROPS.contains(&attr_name) {
                        return;
                    }
                    if let Some(definition) = ARIA_PROPERTY_DEFINITIONS.get(&*attr_name) {
                        if let Some(value) = prop_value {
                            let valid = match &definition.property_type {
                                ARIAPropertyType::Boolean => validate_boolean(value),
                                ARIAPropertyType::Id | ARIAPropertyType::String => {
                                    get_string_value(value).is_some()
                                }
                                ARIAPropertyType::TriState => {
                                    validate_boolean(value)
                                        || get_string_value(value).map_or(false, |v| v == "mixed")
                                }
                                ARIAPropertyType::Integer | ARIAPropertyType::Number => {
                                    parse_jsx_value(value).is_ok()
                                }
                                ARIAPropertyType::Token(permitted_values) => {
                                    validate_token(value, permitted_values)
                                }
                                ARIAPropertyType::IdList => validate_idlist(value),
                                ARIAPropertyType::TokenList(permitted_values) => {
                                    validate_tokenlist(value, permitted_values)
                                }
                            };

                            if valid || (definition.allow_undefined && is_undefined(value)) {
                                return;
                            } else {
                                ctx.diagnostic(aria_proptypes_diagnostic(attr.span));
                            }
                        }
                    }
                }
                JSXAttributeItem::SpreadAttribute(_) => return,
            }
        }
    }
}

/// valid when the value is a boolean literal or a string literal of "true" or "false"
fn validate_boolean(value: &JSXAttributeValue) -> bool {
    match value {
        JSXAttributeValue::StringLiteral(s) => s.value == "true" || s.value == "false",
        JSXAttributeValue::ExpressionContainer(container) => {
            match container.expression.as_expression() {
                Some(Expression::BooleanLiteral(_)) => true,
                Some(Expression::StringLiteral(s)) => s.value == "true" || s.value == "false",
                _ => false,
            }
        }
        _ => false,
    }
}

fn validate_token(value: &JSXAttributeValue, permitted_values: &[TokenValue]) -> bool {
    match value {
        JSXAttributeValue::StringLiteral(s) => permitted_values.iter().any(|token| match token {
            TokenValue::Str(val) => s.value.eq_ignore_ascii_case(val),
            _ => false,
        }),
        JSXAttributeValue::ExpressionContainer(container) => {
            match container.expression.as_expression() {
                Some(Expression::StringLiteral(s)) => {
                    permitted_values.iter().any(|token| match token {
                        TokenValue::Str(val) => s.value.eq_ignore_ascii_case(val),
                        _ => false,
                    })
                }
                _ => false,
            }
        }
        _ => false,
    }
}

fn validate_idlist(value: &JSXAttributeValue) -> bool {
    match value {
        JSXAttributeValue::StringLiteral(s) => !s.value.is_empty(),
        JSXAttributeValue::ExpressionContainer(container) => {
            match container.expression.as_expression() {
                Some(Expression::StringLiteral(s)) => !s.value.is_empty(),
                _ => false,
            }
        }
        _ => false,
    }
}

fn validate_tokenlist(value: &JSXAttributeValue, permitted_values: &[TokenValue]) -> bool {
    let mut token_values = match value {
        JSXAttributeValue::StringLiteral(s) => s.value.split_whitespace(),
        JSXAttributeValue::ExpressionContainer(container) => {
            match container.expression.as_expression() {
                Some(Expression::StringLiteral(s)) => s.value.split_whitespace(),
                _ => return false,
            }
        }
        _ => return false,
    };

    token_values.all(|token| {
        permitted_values.iter().any(|permitted_token| match permitted_token {
            TokenValue::Str(val) => token.eq_ignore_ascii_case(val),
            _ => false,
        })
    })
}

fn get_string_value(value: &JSXAttributeValue) -> Option<String> {
    match value {
        JSXAttributeValue::StringLiteral(s) => Some(s.value.to_string()),
        JSXAttributeValue::ExpressionContainer(container) => {
            match container.expression.as_expression() {
                Some(Expression::StringLiteral(s)) => Some(s.value.to_string()),
                _ => None,
            }
        }
        _ => None,
    }
}

fn is_undefined(value: &JSXAttributeValue) -> bool {
    match value {
        JSXAttributeValue::StringLiteral(s) => s.value == "undefined",
        JSXAttributeValue::ExpressionContainer(container) => {
            if let Some(Expression::StringLiteral(s)) = container.expression.as_expression() {
                s.value == "undefined"
            } else {
                false
            }
        }
        _ => false,
    }
}

enum TokenValue {
    Str(String),
    Bool(bool),
}

enum ARIAPropertyType {
    Id,
    Boolean,
    String,
    Integer,
    Number,
    Token(Vec<TokenValue>),
    TokenList(Vec<TokenValue>),
    IdList,
    TriState,
}

struct ARIAPropertyDefinition {
    property_type: ARIAPropertyType,
    allow_undefined: bool,
}

type ARIAPropertyDefinitions = HashMap<&'static str, ARIAPropertyDefinition>;

lazy_static! {
    /// Creates a map of ARIA property definitions based on NPM package [`aria-query`](https://github.com/A11yance/aria-query).
    ///
    /// This function is used to validate ARIA properties in JSX, aligning with
    /// `eslint-plugin-jsx-a11y`'s aria-properties rule implementation found at:
    /// <https://github.com/A11yance/aria-query/blob/main/src/ariaPropsMap.js>
    ///
    /// Returns a map of ARIA property definitions for use in validation.
    static ref ARIA_PROPERTY_DEFINITIONS: ARIAPropertyDefinitions = {
        let mut definitions = ARIAPropertyDefinitions::new();
        for &prop in VALID_ARIA_PROPS.iter() {
            let definiton = match prop {
                "aria-activedescendant" => ARIAPropertyDefinition {
                    property_type: ARIAPropertyType::Id,
                    allow_undefined: false,
                },
                "aria-atomic" => ARIAPropertyDefinition {
                    property_type: ARIAPropertyType::Boolean,
                    allow_undefined: false,
                },
                "aria-complete" => ARIAPropertyDefinition {
                    property_type: ARIAPropertyType::Token(vec![
                        TokenValue::Str("inline".to_string()),
                        TokenValue::Str("list".to_string()),
                        TokenValue::Str("none".to_string()),
                        TokenValue::Str("both".to_string()),
                    ]),
                    allow_undefined: false,
                },
                "aria-braillelabel" => ARIAPropertyDefinition {
                    property_type: ARIAPropertyType::String,
                    allow_undefined: false,
                },
                "aria-brailleroledescription" => ARIAPropertyDefinition {
                    property_type: ARIAPropertyType::String,
                    allow_undefined: false,
                },
                "aria-busy" => ARIAPropertyDefinition {
                    property_type: ARIAPropertyType::Boolean,
                    allow_undefined: false,
                },
                "aria-checked" => ARIAPropertyDefinition {
                    property_type: ARIAPropertyType::TriState,
                    allow_undefined: false,
                },
                "aria-colcount" => ARIAPropertyDefinition {
                    property_type: ARIAPropertyType::Integer,
                    allow_undefined: false,
                },
                "aria-colindex" => ARIAPropertyDefinition {
                    property_type: ARIAPropertyType::Integer,
                    allow_undefined: false,
                },
                "aria-colspan" => ARIAPropertyDefinition {
                    property_type: ARIAPropertyType::Integer,
                    allow_undefined: false,
                },
                "aria-controls" => ARIAPropertyDefinition {
                    property_type: ARIAPropertyType::IdList,
                    allow_undefined: false,
                },
                "aria-current" => ARIAPropertyDefinition {
                    property_type: ARIAPropertyType::Token(vec![
                        TokenValue::Str("page".to_string()),
                        TokenValue::Str("step".to_string()),
                        TokenValue::Str("location".to_string()),
                        TokenValue::Str("date".to_string()),
                        TokenValue::Str("time".to_string()),
                        TokenValue::Bool(true),
                        TokenValue::Bool(false),
                    ]),
                    allow_undefined: false,
                },
                "aria-describedby" => ARIAPropertyDefinition {
                    property_type: ARIAPropertyType::IdList,
                    allow_undefined: false,
                },
                "aria-description" => ARIAPropertyDefinition {
                    property_type: ARIAPropertyType::String,
                    allow_undefined: false,
                },
                "aria-details" => ARIAPropertyDefinition {
                    property_type: ARIAPropertyType::Id,
                    allow_undefined: false,
                },
                "aria-disabled" => ARIAPropertyDefinition {
                    property_type: ARIAPropertyType::Boolean,
                    allow_undefined: false,
                },
                "aria-dropeffect" => ARIAPropertyDefinition {
                    property_type: ARIAPropertyType::TokenList(vec![
                        TokenValue::Str("copy".to_string()),
                        TokenValue::Str("execute".to_string()),
                        TokenValue::Str("link".to_string()),
                        TokenValue::Str("move".to_string()),
                        TokenValue::Str("none".to_string()),
                        TokenValue::Str("popup".to_string()),
                    ]),
                    allow_undefined: false,
                },
                "aria-errormessage" => ARIAPropertyDefinition {
                    property_type: ARIAPropertyType::Id,
                    allow_undefined: false,
                },
                "aria-expanded" => ARIAPropertyDefinition {
                    property_type: ARIAPropertyType::Boolean,
                    allow_undefined: true,
                },
                "aria-flowto" => ARIAPropertyDefinition {
                    property_type: ARIAPropertyType::IdList,
                    allow_undefined: false,
                },
                "aria-grabbed" => ARIAPropertyDefinition {
                    property_type: ARIAPropertyType::Boolean,
                    allow_undefined: true,
                },
                "aria-haspopup" => ARIAPropertyDefinition {
                    property_type: ARIAPropertyType::Token(vec![
                        TokenValue::Bool(true),
                        TokenValue::Bool(false),
                        TokenValue::Str("menu".to_string()),
                        TokenValue::Str("listbox".to_string()),
                        TokenValue::Str("tree".to_string()),
                        TokenValue::Str("grid".to_string()),
                        TokenValue::Str("dialog".to_string()),
                    ]),
                    allow_undefined: false,
                },
                "aria-hidden" => ARIAPropertyDefinition {
                    property_type: ARIAPropertyType::Boolean,
                    allow_undefined: true,
                },
                "aria-invalid" => ARIAPropertyDefinition {
                    property_type: ARIAPropertyType::Token(vec![
                        TokenValue::Bool(true),
                        TokenValue::Bool(false),
                        TokenValue::Str("grammar".to_string()),
                        TokenValue::Str("spelling".to_string()),
                    ]),
                    allow_undefined: false,
                },
                "aria-keyshortcuts" => ARIAPropertyDefinition {
                    property_type: ARIAPropertyType::String,
                    allow_undefined: false,
                },
                "aria-label" => ARIAPropertyDefinition {
                    property_type: ARIAPropertyType::String,
                    allow_undefined: false,
                },
                "aria-labelledby" => ARIAPropertyDefinition {
                    property_type: ARIAPropertyType::IdList,
                    allow_undefined: false,
                },
                "aria-level" => ARIAPropertyDefinition {
                    property_type: ARIAPropertyType::Integer,
                    allow_undefined: false,
                },
                "aria-live" => ARIAPropertyDefinition {
                    property_type: ARIAPropertyType::Token(vec![
                        TokenValue::Str("off".to_string()),
                        TokenValue::Str("assertive".to_string()),
                        TokenValue::Str("polite".to_string()),
                    ]),
                    allow_undefined: false,
                },
                "aria-modal" => ARIAPropertyDefinition {
                    property_type: ARIAPropertyType::Boolean,
                    allow_undefined: false,
                },
                "aria-multiline" => ARIAPropertyDefinition {
                    property_type: ARIAPropertyType::Boolean,
                    allow_undefined: false,
                },
                "aria-multiselectable" => ARIAPropertyDefinition {
                    property_type: ARIAPropertyType::Boolean,
                    allow_undefined: false,
                },
                "aria-orientation" => ARIAPropertyDefinition {
                    property_type: ARIAPropertyType::Token(vec![
                        TokenValue::Str("horizontal".to_string()),
                        TokenValue::Str("vertical".to_string()),
                        TokenValue::Str("undefined".to_string()),
                    ]),
                    allow_undefined: false,
                },
                "aria-owns" => ARIAPropertyDefinition {
                    property_type: ARIAPropertyType::IdList,
                    allow_undefined: false,
                },
                "aria-placeholder" => ARIAPropertyDefinition {
                    property_type: ARIAPropertyType::String,
                    allow_undefined: false,
                },
                "aria-posinset" => ARIAPropertyDefinition {
                    property_type: ARIAPropertyType::Integer,
                    allow_undefined: false,
                },
                "aria-pressed" => ARIAPropertyDefinition {
                    property_type: ARIAPropertyType::TriState,
                    allow_undefined: false,
                },
                "aria-readonly" => ARIAPropertyDefinition {
                    property_type: ARIAPropertyType::Boolean,
                    allow_undefined: false,
                },
                "aria-relevant" => ARIAPropertyDefinition {
                    property_type: ARIAPropertyType::TokenList(vec![
                        TokenValue::Str("additions".to_string()),
                        TokenValue::Str("all".to_string()),
                        TokenValue::Str("removals".to_string()),
                        TokenValue::Str("text".to_string()),
                    ]),
                    allow_undefined: false,
                },
                "aria-required" => ARIAPropertyDefinition {
                    property_type: ARIAPropertyType::Boolean,
                    allow_undefined: false,
                },
                "aria-roledescription" => ARIAPropertyDefinition {
                    property_type: ARIAPropertyType::String,
                    allow_undefined: false,
                },
                "aria-rowcount" => ARIAPropertyDefinition {
                    property_type: ARIAPropertyType::Integer,
                    allow_undefined: false,
                },
                "aria-rowindex" => ARIAPropertyDefinition {
                    property_type: ARIAPropertyType::Integer,
                    allow_undefined: false,
                },
                "aria-rowspan" => ARIAPropertyDefinition {
                    property_type: ARIAPropertyType::Integer,
                    allow_undefined: false,
                },
                "aria-selected" => ARIAPropertyDefinition {
                    property_type: ARIAPropertyType::Boolean,
                    allow_undefined: true,
                },
                "aria-setsize" => ARIAPropertyDefinition {
                    property_type: ARIAPropertyType::Integer,
                    allow_undefined: false,
                },
                "aria-sort" => ARIAPropertyDefinition {
                    property_type: ARIAPropertyType::Token(vec![
                        TokenValue::Str("ascending".to_string()),
                        TokenValue::Str("descending".to_string()),
                        TokenValue::Str("none".to_string()),
                        TokenValue::Str("other".to_string()),
                    ]),
                    allow_undefined: false,
                },
                "aria-valuemax" => ARIAPropertyDefinition {
                    property_type: ARIAPropertyType::Number,
                    allow_undefined: false,
                },
                "aria-valuemin" => ARIAPropertyDefinition {
                    property_type: ARIAPropertyType::Number,
                    allow_undefined: false,
                },
                "aria-valuenow" => ARIAPropertyDefinition {
                    property_type: ARIAPropertyType::Number,
                    allow_undefined: false,
                },
                "aria-valuetext" => ARIAPropertyDefinition {
                    property_type: ARIAPropertyType::String,
                    allow_undefined: false,
                },
                _ => ARIAPropertyDefinition {
                    property_type: ARIAPropertyType::String,
                    allow_undefined: false,
                }
            };
            definitions.insert(prop, definiton);
        }

        return definitions;
    };
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "<div aria-foo=\"true\" />",
        "<div abcaria-foo=\"true\" />",
        "<div aria-hidden={true} />",
        "<div aria-hidden=\"true\" />",
        "<div aria-hidden={\"false\"} />",
        "<div aria-hidden={!false} />",
        "<div aria-hidden />",
        "<div aria-hidden={false} />",
        r#"<div aria-hidden={!true} />"#,
        r#"<div aria-hidden={!\"yes\"} />"#,
        r#"<div aria-hidden={foo} />"#,
        r#"<div aria-hidden={foo.bar} />"#,
        r#"<div aria-hidden={null} />"#,
        r#"<div aria-hidden={undefined} />"#,
        r#"<div aria-hidden={<div />} />"#,
        r#"<div aria-label=\"Close\" />"#,
        r#"<div aria-label={`Close`} />"#,
        r#"<div aria-label={foo} />"#,
        r#"<div aria-label={foo.bar} />"#,
        r#"<div aria-label={null} />"#,
        r#"<div aria-label={undefined} />"#,
        r#"<input aria-invalid={error ? \"true\" : \"false\"} />"#,
        r#"<input aria-invalid={undefined ? \"true\" : \"false\"} />"#,
        r#"<div aria-checked={true} />"#,
        r#"<div aria-checked=\"true\" />"#,
        r#"<div aria-checked={\"false\"} />"#,
        r#"<div aria-checked={!false} />"#,
        r#"<div aria-checked />"#,
        r#"<div aria-checked={false} />"#,
        r#"<div aria-checked={!true} />"#,
        r#"<div aria-checked={!\"yes\"} />"#,
        r#"<div aria-checked={foo} />"#,
        r#"<div aria-checked={foo.bar} />"#,
        r#"<div aria-checked=\"mixed\" />"#,
        r#"<div aria-checked={`mixed`} />"#,
        r#"<div aria-checked={null} />"#,
        r#"<div aria-checked={undefined} />"#,
        r#"<div aria-level={123} />"#,
        r#"<div aria-level={-123} />"#,
        r#"<div aria-level={+123} />"#,
        r#"<div aria-level={~123} />"#,
        r#"<div aria-level={\"123\"} />"#,
        r#"<div aria-level={`123`} />"#,
        r#"<div aria-level=\"123\" />"#,
        r#"<div aria-level={foo} />"#,
        r#"<div aria-level={foo.bar} />"#,
        r#"<div aria-level={null} />"#,
        r#"<div aria-level={undefined} />"#,
        r#"<div aria-valuemax={123} />"#,
        r#"<div aria-valuemax={-123} />"#,
        r#"<div aria-valuemax={+123} />"#,
        r#"<div aria-valuemax={~123} />"#,
        r#"<div aria-valuemax={\"123\"} />"#,
        r#"<div aria-valuemax={`123`} />"#,
        r#"<div aria-valuemax=\"123\" />"#,
        r#"<div aria-valuemax={foo} />"#,
        r#"<div aria-valuemax={foo.bar} />"#,
        r#"<div aria-valuemax={null} />"#,
        r#"<div aria-valuemax={undefined} />"#,
        r#"<div aria-sort=\"ascending\" />"#,
        r#"<div aria-sort=\"ASCENDING\" />"#,
        r#"<div aria-sort={\"ascending\"} />"#,
        r#"<div aria-sort={`ascending`} />"#,
        r#"<div aria-sort=\"descending\" />"#,
        r#"<div aria-sort={\"descending\"} />"#,
        r#"<div aria-sort={`descending`} />"#,
        r#"<div aria-sort=\"none\" />"#,
        r#"<div aria-sort={\"none\"} />"#,
        r#"<div aria-sort={`none`} />"#,
        r#"<div aria-sort=\"other\" />"#,
        r#"<div aria-sort={\"other\"} />"#,
        r#"<div aria-sort={`other`} />"#,
        r#"<div aria-sort={foo} />"#,
        r#"<div aria-sort={foo.bar} />"#,
        r#"<div aria-invalid={true} />"#,
        r#"<div aria-invalid=\"true\" />"#,
        r#"<div aria-invalid={false} />"#,
        r#"<div aria-invalid=\"false\" />"#,
        r#"<div aria-invalid=\"grammar\" />"#,
        r#"<div aria-invalid=\"spelling\" />"#,
        r#"<div aria-invalid={null} />"#,
        r#"<div aria-invalid={undefined} />"#,
        r#"<div aria-relevant=\"additions\" />"#,
        r#"<div aria-relevant={\"additions\"} />"#,
        r#"<div aria-relevant={`additions`} />"#,
        r#"<div aria-relevant=\"additions removals\" />"#,
        r#"<div aria-relevant=\"additions additions\" />"#,
        r#"<div aria-relevant={\"additions removals\"} />"#,
        r#"<div aria-relevant={`additions removals`} />"#,
        r#"<div aria-relevant=\"additions removals text\" />"#,
        r#"<div aria-relevant={\"additions removals text\"} />"#,
        r#"<div aria-relevant={`additions removals text`} />"#,
        r#"<div aria-relevant=\"additions removals text all\" />"#,
        r#"<div aria-relevant={\"additions removals text all\"} />"#,
        r#"<div aria-relevant={`removals additions text all`} />"#,
        r#"<div aria-relevant={foo} />"#,
        r#"<div aria-relevant={foo.bar} />"#,
        r#"<div aria-relevant={null} />"#,
        r#"<div aria-relevant={undefined} />"#,
        r#"<div aria-activedescendant=\"ascending\" />"#,
        r#"<div aria-activedescendant=\"ASCENDING\" />"#,
        r#"<div aria-activedescendant={\"ascending\"} />"#,
        r#"<div aria-activedescendant={`ascending`} />"#,
        r#"<div aria-activedescendant=\"descending\" />"#,
        r#"<div aria-activedescendant={\"descending\"} />"#,
        r#"<div aria-activedescendant={`descending`} />"#,
        r#"<div aria-activedescendant=\"none\" />"#,
        r#"<div aria-activedescendant={\"none\"} />"#,
        r#"<div aria-activedescendant={`none`} />"#,
        r#"<div aria-activedescendant=\"other\" />"#,
        r#"<div aria-activedescendant={\"other\"} />"#,
        r#"<div aria-activedescendant={`other`} />"#,
        r#"<div aria-activedescendant={foo} />"#,
        r#"<div aria-activedescendant={foo.bar} />"#,
        r#"<div aria-activedescendant={null} />"#,
        r#"<div aria-activedescendant={undefined} />"#,
        r#"<div aria-labelledby=\"additions\" />"#,
        r#"<div aria-labelledby={\"additions\"} />"#,
        r#"<div aria-labelledby={`additions`} />"#,
        r#"<div aria-labelledby=\"additions removals\" />"#,
        r#"<div aria-labelledby=\"additions additions\" />"#,
        r#"<div aria-labelledby={\"additions removals\"} />"#,
        r#"<div aria-labelledby={`additions removals`} />"#,
        r#"<div aria-labelledby=\"additions removals text\" />"#,
        r#"<div aria-labelledby={\"additions removals text\"} />"#,
        r#"<div aria-labelledby={`additions removals text`} />"#,
        r#"<div aria-labelledby=\"additions removals text all\" />"#,
        r#"<div aria-labelledby={\"additions removals text all\"} />"#,
        r#"<div aria-labelledby={`removals additions text all`} />"#,
        r#"<div aria-labelledby={foo} />"#,
        r#"<div aria-labelledby={foo.bar} />"#,
        r#"<div aria-labelledby={null} />"#,
        r#"<div aria-labelledby={undefined} />"#,
    ];

    let fail = vec![
        r#"<div aria-hidden=\"yes\" />"#,
        r#"<div aria-hidden=\"no\" />"#,
        r#"<div aria-hidden={1234} />"#,
        r#"<div aria-hidden={`${abc}`} />"#,
        r#"<div aria-label />"#,
        r#"<div aria-label={true} />"#,
        r#"<div aria-label={false} />"#,
        r#"<div aria-label={1234} />"#,
        r#"<div aria-label={!true} />"#,
        r#"<div aria-checked=\"yes\" />"#,
        r#"<div aria-checked=\"no\" />"#,
        r#"<div aria-checked={1234} />"#,
        r#"<div aria-checked={`${abc}`} />"#,
        r#"<div aria-level=\"yes\" />"#,
        r#"<div aria-level=\"no\" />"#,
        r#"<div aria-level={`abc`} />"#,
        r#"<div aria-level={true} />"#,
        r#"<div aria-level />"#,
        r#"<div aria-level={\"false\"} />"#,
        r#"<div aria-level={!\"false\"} />"#,
        r#"<div aria-valuemax=\"yes\" />"#,
        r#"<div aria-valuemax=\"no\" />"#,
        r#"<div aria-valuemax={`abc`} />"#,
        r#"<div aria-valuemax={true} />"#,
        r#"<div aria-valuemax />"#,
        r#"<div aria-valuemax={\"false\"} />"#,
        r#"<div aria-valuemax={!\"false\"} />"#,
        r#"<div aria-sort=\"\" />"#,
        r#"<div aria-sort=\"descnding\" />"#,
        r#"<div aria-sort />"#,
        r#"<div aria-sort={true} />"#,
        r#"<div aria-sort={\"false\"} />"#,
        r#"<div aria-sort=\"ascending descending\" />"#,
        r#"<div aria-relevant=\"\" />"#,
        r#"<div aria-relevant=\"foobar\" />"#,
        r#"<div aria-relevant />"#,
        r#"<div aria-relevant={true} />"#,
        r#"<div aria-relevant={\"false\"} />"#,
        r#"<div aria-relevant=\"additions removalss\" />"#,
        r#"<div aria-relevant=\"additions removalss \" />"#,
    ];

    Tester::new(AriaProptypes::NAME, pass, fail).test_and_snapshot();
}
