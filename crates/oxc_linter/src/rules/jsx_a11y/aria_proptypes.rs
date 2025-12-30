use cow_utils::CowUtils;

use oxc_ast::{
    AstKind,
    ast::{JSXAttributeValue, JSXExpression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_ecmascript::{ToBoolean, WithoutGlobalReferenceInformation};
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, Span};
use oxc_syntax::operator::UnaryOperator;

use crate::{
    AstNode, context::LintContext, globals::AriaProperty, rule::Rule, utils::get_jsx_attribute_name,
};

fn aria_proptypes_diagnostic(
    span: Span,
    prop_name: &str,
    prop_type: &AriaPropType,
) -> OxcDiagnostic {
    let valid_prop_message = generate_valid_prop_message(prop_type);
    OxcDiagnostic::warn(format!("This is not a valid ARIA state and property value for '{prop_name}'."))
    .with_help(format!(
        "The valid value for '{prop_name}' is: {valid_prop_message}.\nYou can find a list of valid ARIA state and property values at https://www.w3.org/TR/wai-aria/#x6-7-definitions-of-states-and-properties-all-aria-attributes"
    ))
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct AriaProptypes;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces that elements do not use invalid ARIA state and property values.
    ///
    /// ### Why is this bad?
    ///
    /// Using invalid ARIA state and property values can mislead screen readers and other assistive technologies.
    /// It may cause the accessibility features of the website to fail, making it difficult for users with disabilities to use the site effectively.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <div aria-level="yes" />
    /// <div aria-relevant="additions removalss" />
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <div aria-label="foo" />
    /// <div aria-labelledby="foo bar" />
    /// <div aria-checked={false} />
    /// <div aria-invalid="grammar" />
    /// ```
    AriaProptypes,
    jsx_a11y,
    correctness,
);

impl Rule for AriaProptypes {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXAttribute(attr) = node.kind() else {
            return;
        };
        let name = get_jsx_attribute_name(&attr.name);
        let name = name.cow_to_ascii_lowercase();
        let Ok(aria_prop_name) = AriaProperty::try_from(name.as_ref()) else { return };
        let aria_prop_type = get_aria_prop_type(aria_prop_name);
        let Some(aria_prop_value_string) = &attr.value else {
            if !allow_none_value(&aria_prop_type) {
                let diagnostic = aria_proptypes_diagnostic(attr.span, &name, &aria_prop_type);
                ctx.diagnostic(diagnostic);
                return;
            }
            return;
        };

        if !is_valid_value_for_aria_prop_type(&aria_prop_type, aria_prop_value_string) {
            let diagnostic = aria_proptypes_diagnostic(attr.span, &name, &aria_prop_type);
            ctx.diagnostic(diagnostic);
        }
    }
}

// ref: https://www.w3.org/TR/wai-aria-1.2/#propcharacteristic_value
#[derive(Debug, Clone, PartialEq)]
enum AriaPropType {
    Id,
    Boolean,
    String,
    Tristate,
    Integer,
    IdList,
    OptionalBoolean,
    Token(Vec<CompactStr>),
    TokenList(Vec<CompactStr>),
    Number,
}

// Whether the attribute value is allowed to be None such as <button aria-expanded>
fn allow_none_value(aria_prop_type: &AriaPropType) -> bool {
    match aria_prop_type {
        AriaPropType::Boolean
        | AriaPropType::OptionalBoolean
        | AriaPropType::Tristate
        | AriaPropType::String => true,
        AriaPropType::Token(tokens) | AriaPropType::TokenList(tokens) => {
            // allow None value if "true" is allowed for its tokens
            tokens.iter().any(|token| token == "true")
        }
        _ => false,
    }
}

fn is_valid_value_for_aria_prop_type(
    aria_prop_type: &AriaPropType,
    value: &JSXAttributeValue,
) -> bool {
    if !is_target_literal_value(value) {
        return true;
    }
    match aria_prop_type {
        AriaPropType::Boolean | AriaPropType::OptionalBoolean => {
            let Some(value_string) = parse_aria_prop_value_as_string(value, true) else {
                return false;
            };
            matches!(value_string.as_str(), "true" | "false")
        }
        AriaPropType::Tristate => {
            let Some(value_string) = parse_aria_prop_value_as_string(value, true) else {
                return false;
            };
            matches!(value_string.as_str(), "true" | "false" | "mixed")
        }
        AriaPropType::String | AriaPropType::Id => {
            // Template literals with expressions always produce strings at runtime
            if let JSXAttributeValue::ExpressionContainer(container) = value
                && let JSXExpression::TemplateLiteral(t) = &container.expression
                && t.single_quasi().is_none()
            {
                return true;
            }
            parse_aria_prop_value_as_string(value, false).is_some()
        }
        AriaPropType::Integer | AriaPropType::Number => {
            if let Some(value_string) = parse_aria_prop_value_as_string(value, false) {
                return value_string.parse::<f64>().is_ok();
            }
            match value {
                JSXAttributeValue::ExpressionContainer(container) => {
                    matches!(container.expression, JSXExpression::NumericLiteral(_))
                }
                _ => false,
            }
        }
        AriaPropType::IdList => {
            // Template literals with expressions always produce strings at runtime and are
            // valid for ID list ARIA props (e.g., `${id}-label` or `${id}-label ${id}-help-text`).
            if let JSXAttributeValue::ExpressionContainer(container) = value
                && let JSXExpression::TemplateLiteral(t) = &container.expression
                && t.single_quasi().is_none()
            {
                return true;
            }

            let Some(value_string) = parse_aria_prop_value_as_string(value, false) else {
                return false;
            };

            value_string.split_whitespace().next().is_some()
        }
        AriaPropType::Token(valid_tokens) => {
            let Some(value_string) = parse_aria_prop_value_as_string(value, true) else {
                return false;
            };
            valid_tokens.iter().any(|valid_token| valid_token == &value_string)
        }
        AriaPropType::TokenList(valid_tokens) => {
            let Some(value) = parse_aria_prop_value_as_string(value, true) else {
                return false;
            };
            // Each token must be in valid_tokens
            let mut count = 0;
            for token in value.split_whitespace() {
                if !valid_tokens.iter().any(|valid_token| valid_token == token) {
                    return false;
                }
                count += 1;
            }
            count > 0
        }
    }
}

fn parse_aria_prop_value_as_string(
    value: &JSXAttributeValue,
    boolean_as_string: bool, // whether to convert boolean literal to string
) -> Option<CompactStr> {
    match value {
        JSXAttributeValue::StringLiteral(string_lit) => {
            Some(string_lit.value.cow_to_lowercase().into())
        }
        JSXAttributeValue::ExpressionContainer(container) => match &container.expression {
            JSXExpression::StringLiteral(string_lit) => {
                Some(string_lit.value.cow_to_lowercase().into())
            }
            JSXExpression::TemplateLiteral(template_lit) => {
                Some(template_lit.single_quasi()?.cow_to_lowercase().into())
            }
            JSXExpression::BooleanLiteral(bool_lit) => {
                if boolean_as_string {
                    Some(bool_lit.value.to_string().into())
                } else {
                    None
                }
            }
            JSXExpression::UnaryExpression(unary)
                if boolean_as_string && unary.operator == UnaryOperator::LogicalNot =>
            {
                let value = !unary.argument.to_boolean(&WithoutGlobalReferenceInformation)?;
                Some(value.to_string().into())
            }
            _ => None,
        },
        _ => None,
    }
}

fn is_target_literal_value(value: &JSXAttributeValue) -> bool {
    match value {
        JSXAttributeValue::StringLiteral(_) => true,
        JSXAttributeValue::ExpressionContainer(container) => match &container.expression {
            JSXExpression::StringLiteral(_)
            | JSXExpression::BooleanLiteral(_)
            | JSXExpression::NumericLiteral(_)
            | JSXExpression::BigIntLiteral(_)
            | JSXExpression::TemplateLiteral(_) => true,
            JSXExpression::UnaryExpression(unary) => {
                // Check if unary `!` expression can be statically evaluated (e.g., `!true`, `!"string"`)
                unary.operator == UnaryOperator::LogicalNot
                    && unary.argument.to_boolean(&WithoutGlobalReferenceInformation).is_some()
            }
            _ => false, // null literal always pass this rule
        },
        _ => false,
    }
}

// ref: https://github.com/A11yance/aria-query/blob/main/src/ariaPropsMap.js
fn get_aria_prop_type(prop_name: AriaProperty) -> AriaPropType {
    match prop_name {
        AriaProperty::ActiveDescendant | AriaProperty::Details | AriaProperty::ErrorMessage => {
            AriaPropType::Id
        }
        AriaProperty::Atomic
        | AriaProperty::Busy
        | AriaProperty::Disabled
        | AriaProperty::Modal
        | AriaProperty::Multiline
        | AriaProperty::Multiselectable
        | AriaProperty::Readonly
        | AriaProperty::Required => AriaPropType::Boolean,
        AriaProperty::BrailleLabel
        | AriaProperty::BrailleRoleDescription
        | AriaProperty::Description
        | AriaProperty::KeyShortcuts
        | AriaProperty::Label
        | AriaProperty::Placeholder
        | AriaProperty::RoleDescription
        | AriaProperty::ValueText => AriaPropType::String,
        AriaProperty::Checked | AriaProperty::Pressed => AriaPropType::Tristate,
        AriaProperty::ColCount
        | AriaProperty::ColIndex
        | AriaProperty::ColSpan
        | AriaProperty::Level
        | AriaProperty::PosInSet
        | AriaProperty::RowCount
        | AriaProperty::RowIndex
        | AriaProperty::RowSpan
        | AriaProperty::SetSize => AriaPropType::Integer,
        AriaProperty::Controls
        | AriaProperty::DescribedBy
        | AriaProperty::FlowTo
        | AriaProperty::LabelledBy
        | AriaProperty::Owns => AriaPropType::IdList,
        AriaProperty::Expanded
        | AriaProperty::Grabbed
        | AriaProperty::Hidden
        | AriaProperty::Selected => AriaPropType::OptionalBoolean,
        AriaProperty::ValueMax | AriaProperty::ValueMin | AriaProperty::ValueNow => {
            AriaPropType::Number
        }
        AriaProperty::AutoComplete => {
            AriaPropType::Token(vec!["none".into(), "inline".into(), "list".into(), "both".into()])
        }
        AriaProperty::Current => AriaPropType::Token(vec![
            "page".into(),
            "step".into(),
            "location".into(),
            "date".into(),
            "time".into(),
            "true".into(),
            "false".into(),
        ]),
        AriaProperty::HasPopup => AriaPropType::Token(vec![
            "false".into(),
            "true".into(),
            "menu".into(),
            "listbox".into(),
            "tree".into(),
            "grid".into(),
            "dialog".into(),
        ]),
        AriaProperty::Invalid => AriaPropType::Token(vec![
            "grammar".into(),
            "false".into(),
            "spelling".into(),
            "true".into(),
        ]),
        AriaProperty::Live => {
            AriaPropType::Token(vec!["assertive".into(), "off".into(), "polite".into()])
        }
        AriaProperty::Orientation => {
            AriaPropType::Token(vec!["horizontal".into(), "undefined".into(), "vertical".into()])
        }
        AriaProperty::Sort => AriaPropType::Token(vec![
            "ascending".into(),
            "descending".into(),
            "none".into(),
            "other".into(),
        ]),
        AriaProperty::DropEffect => AriaPropType::TokenList(vec![
            "copy".into(),
            "execute".into(),
            "link".into(),
            "move".into(),
            "none".into(),
            "popup".into(),
        ]),
        AriaProperty::Relevant => AriaPropType::TokenList(vec![
            "additions".into(),
            "all".into(),
            "removals".into(),
            "text".into(),
        ]),
    }
}

fn generate_valid_prop_message(prop_type: &AriaPropType) -> String {
    match prop_type {
        AriaPropType::Boolean | AriaPropType::OptionalBoolean => "'true' or 'false'".to_string(),
        AriaPropType::Tristate => "'true', 'false', or 'mixed'".to_string(),
        AriaPropType::String => "a string value".to_string(),
        AriaPropType::Integer => "an integer value".to_string(),
        AriaPropType::Number => "a number value".to_string(),
        AriaPropType::Id => "a single element ID".to_string(),
        AriaPropType::IdList => "a space-separated list of element IDs".to_string(),
        AriaPropType::Token(tokens) => {
            format!("one of the following tokens: {}", tokens.join(", "))
        }
        AriaPropType::TokenList(tokens) => {
            format!("a space-separated list of the following tokens: {}", tokens.join(", "))
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"<div aria-foo="true" />"#,
        r#"<div abcaria-foo="true" />"#,
        "<div aria-hidden={true} />",
        r#"<div aria-hidden="true" />"#,
        r#"<div aria-hidden={"false"} />"#,
        "<div aria-hidden={!false} />",
        "<div aria-hidden />",
        "<div aria-hidden={false} />",
        "<div aria-hidden={!!foo} />",
        "<div aria-hidden={!true} />",
        r#"<div aria-hidden={!"yes"} />"#,
        "<div aria-hidden={foo} />",
        "<div aria-hidden={foo.bar} />",
        "<div aria-hidden={null} />",
        "<div aria-hidden={undefined} />",
        "<div aria-hidden={<div />} />",
        r#"<div aria-label="Close" />"#,
        r#"<div aria-label="" />"#,
        "<div aria-label />",
        "<div aria-label={`Close`} />",
        "<div aria-label={`Hello ${foo}`} />",
        "<div aria-label={`foo-${id}`} />",
        "<div aria-label={`foo-${id} bar`} />",
        "<div aria-label={foo} />",
        "<div aria-label={foo.bar} />",
        "<div aria-label={null} />",
        "<div aria-label={undefined} />",
        r#"<input aria-invalid={error ? "true" : "false"} />"#,
        r#"<input aria-invalid={undefined ? "true" : "false"} />"#,
        "<div aria-checked={true} />",
        "<div aria-checked={!!foo} />",
        r#"<div aria-checked="true" />"#,
        r#"<div aria-checked={"false"} />"#,
        "<div aria-checked={!false} />",
        "<div aria-checked />",
        "<div aria-checked={false} />",
        "<div aria-checked={!true} />",
        r#"<div aria-checked={!"yes"} />"#,
        "<div aria-checked={foo} />",
        "<div aria-checked={foo.bar} />",
        r#"<div aria-checked="mixed" />"#,
        "<div aria-checked={`mixed`} />",
        "<div aria-checked={null} />",
        "<div aria-checked={undefined} />",
        "<div aria-level={123} />",
        "<div aria-level={-123} />",
        "<div aria-level={+123} />",
        "<div aria-level={~123} />",
        r#"<div aria-level={"123"} />"#,
        "<div aria-level={`123`} />",
        r#"<div aria-level="123" />"#,
        "<div aria-level={foo} />",
        "<div aria-level={foo.bar} />",
        "<div aria-level={null} />",
        "<div aria-level={undefined} />",
        "<div aria-valuemax={123} />",
        "<div aria-valuemax={-123} />",
        "<div aria-valuemax={+123} />",
        "<div aria-valuemax={~123} />",
        r#"<div aria-valuemax={"123"} />"#,
        "<div aria-valuemax={`123`} />",
        r#"<div aria-valuemax="123" />"#,
        "<div aria-valuemax={foo} />",
        "<div aria-valuemax={foo.bar} />",
        "<div aria-valuemax={null} />",
        "<div aria-valuemax={undefined} />",
        r#"<div aria-sort="ascending" />"#,
        r#"<div aria-sort="ASCENDING" />"#,
        r#"<div aria-sort={"ascending"} />"#,
        "<div aria-sort={`ascending`} />",
        r#"<div aria-sort="descending" />"#,
        r#"<div aria-sort={"descending"} />"#,
        "<div aria-sort={`descending`} />",
        r#"<div aria-sort="none" />"#,
        r#"<div aria-sort={"none"} />"#,
        "<div aria-sort={`none`} />",
        r#"<div aria-sort="other" />"#,
        r#"<div aria-sort={"other"} />"#,
        "<div aria-sort={`other`} />",
        "<div aria-sort={foo} />",
        "<div aria-sort={foo.bar} />",
        "<div aria-invalid={true} />",
        r#"<div aria-invalid="true" />"#,
        "<div aria-invalid={false} />",
        r#"<div aria-invalid="false" />"#,
        r#"<div aria-invalid="grammar" />"#,
        r#"<div aria-invalid="spelling" />"#,
        "<div aria-invalid={null} />",
        "<div aria-invalid={undefined} />",
        r#"<div aria-relevant="additions" />"#,
        r#"<div aria-relevant={"additions"} />"#,
        "<div aria-relevant={`additions`} />",
        r#"<div aria-relevant="additions removals" />"#,
        r#"<div aria-relevant="additions additions" />"#,
        r#"<div aria-relevant={"additions removals"} />"#,
        "<div aria-relevant={`additions removals`} />",
        r#"<div aria-relevant="additions removals text" />"#,
        r#"<div aria-relevant={"additions removals text"} />"#,
        "<div aria-relevant={`additions removals text`} />",
        r#"<div aria-relevant="additions removals text all" />"#,
        r#"<div aria-relevant={"additions removals text all"} />"#,
        "<div aria-relevant={`removals additions text all`} />",
        "<div aria-relevant={foo} />",
        "<div aria-relevant={foo.bar} />",
        "<div aria-relevant={null} />",
        "<div aria-relevant={undefined} />",
        r#"<div aria-activedescendant="ascending" />"#,
        r#"<div aria-activedescendant="ASCENDING" />"#,
        r#"<div aria-activedescendant={"ascending"} />"#,
        "<div aria-activedescendant={`ascending`} />",
        r#"<div aria-activedescendant="descending" />"#,
        r#"<div aria-activedescendant={"descending"} />"#,
        "<div aria-activedescendant={`descending`} />",
        r#"<div aria-activedescendant="none" />"#,
        r#"<div aria-activedescendant={"none"} />"#,
        "<div aria-activedescendant={`none`} />",
        r#"<div aria-activedescendant="other" />"#,
        r#"<div aria-activedescendant={"other"} />"#,
        "<div aria-activedescendant={`other`} />",
        "<div aria-activedescendant={foo} />",
        "<div aria-activedescendant={foo.bar} />",
        "<div aria-activedescendant={null} />",
        "<div aria-activedescendant={undefined} />",
        r#"<div aria-labelledby="additions" />"#,
        r#"<div aria-labelledby={"additions"} />"#,
        "<div aria-labelledby={`additions`} />",
        r#"<div aria-labelledby="additions removals" />"#,
        r#"<div aria-labelledby="additions additions" />"#,
        r#"<div aria-labelledby={"additions removals"} />"#,
        "<div aria-labelledby={`additions removals`} />",
        r#"<div aria-labelledby="additions removals text" />"#,
        r#"<div aria-labelledby={"additions removals text"} />"#,
        "<div aria-labelledby={`additions removals text`} />",
        r#"<div aria-labelledby="additions removals text all" />"#,
        r#"<div aria-labelledby={"additions removals text all"} />"#,
        "<div aria-labelledby={`removals additions text all`} />",
        "<div aria-labelledby={foo} />",
        "<div aria-labelledby={foo.bar} />",
        "<div aria-labelledby={null} />",
        "<div aria-labelledby={undefined} />",
        // Ensure that template literals with expressions are allowed for idlist aria props.
        "<div aria-labelledby={`${id}-label`} />",
        "<div aria-labelledby={`${id}`} />",
        "<div aria-labelledby={`${id}-label ${id}-help-text`} />",
        "<div aria-describedby={`${id}-label`} />",
        "<div aria-describedby={`${id}`} />",
        "<div aria-describedby={`${foo.bar}`} />",
        "<div aria-describedby={`${id}-label ${id}-help-text`} />",
        "<div aria-describedby={`${foo.bar}-label ${foo.bar}-help-text`} />",
    ];

    let fail = vec![
        r#"<div aria-hidden="yes" />"#,
        r#"<div aria-hidden="no" />"#,
        "<div aria-hidden={1234} />",
        "<div aria-hidden={`${abc}`} />",
        "<div aria-label={true} />",
        "<div aria-label={false} />",
        "<div aria-label={1234} />",
        "<div aria-label={!true} />",
        r#"<div aria-checked="yes" />"#,
        r#"<div aria-checked="no" />"#,
        "<div aria-checked={1234} />",
        "<div aria-checked={`${abc}`} />",
        r#"<div aria-level="yes" />"#,
        r#"<div aria-level="no" />"#,
        "<div aria-level={`abc`} />",
        "<div aria-level={true} />",
        "<div aria-level />",
        r#"<div aria-level={"false"} />"#,
        r#"<div aria-level={!"false"} />"#,
        r#"<div aria-valuemax="yes" />"#,
        r#"<div aria-valuemax="no" />"#,
        "<div aria-valuemax={`abc`} />",
        "<div aria-valuemax={true} />",
        "<div aria-valuemax />",
        r#"<div aria-valuemax={"false"} />"#,
        r#"<div aria-valuemax={!"false"} />"#,
        r#"<div aria-sort="" />"#,
        r#"<div aria-sort="descnding" />"#,
        "<div aria-sort />",
        "<div aria-sort={true} />",
        r#"<div aria-sort={"false"} />"#,
        r#"<div aria-sort="ascending descending" />"#,
        r#"<div aria-relevant="" />"#,
        r#"<div aria-relevant="foobar" />"#,
        "<div aria-relevant />",
        "<div aria-relevant={true} />",
        r#"<div aria-relevant={"false"} />"#,
        r#"<div aria-relevant="additions removalss" />"#,
        r#"<div aria-relevant="additions removalss " />"#,
        // Fails because these should not allow boolean values or numbers.
        "<div aria-labelledby={true} />",
        "<div aria-labelledby={false} />",
        "<div aria-labelledby={123} />",
        // Fails because this is a string, and so not interpolated.
        r#"<div aria-hidden={"!!foo"} />"#,
    ];

    Tester::new(AriaProptypes::NAME, AriaProptypes::PLUGIN, pass, fail).test_and_snapshot();
}
