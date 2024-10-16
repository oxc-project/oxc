use oxc_ast::{
    ast::{Expression, JSXAttributeItem, JSXAttributeName, JSXAttributeValue},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, Span};
use rustc_hash::FxHashSet;

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    utils::get_prop_value,
    AstNode,
};

fn boolean_value_diagnostic(attr: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Value must be omitted for boolean attribute {attr:?}"))
        .with_label(span)
}

fn boolean_value_always_diagnostic(attr: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Value must be set for boolean attribute {attr:?}"))
        .with_label(span)
}

fn boolean_value_undefined_false_diagnostic(attr: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Value must be omitted for `false` attribute {attr:?}"))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct JsxBooleanValue(Box<JsxBooleanValueConfig>);

#[derive(Debug, Default, Clone)]
pub enum EnforceBooleanAttribute {
    Always,
    #[default]
    Never,
}

#[derive(Debug, Default, Clone)]
pub struct JsxBooleanValueConfig {
    pub enforce_boolean_attribute: EnforceBooleanAttribute,
    pub exceptions: FxHashSet<CompactStr>,
    pub assume_undefined_is_false: bool,
}

impl std::ops::Deref for JsxBooleanValue {
    type Target = JsxBooleanValueConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce a consistent boolean attribute style in your code.
    ///
    /// ### Example
    /// ```jsx
    /// const Hello = <Hello personal={true} />;
    /// ```
    JsxBooleanValue,
    style,
    fix,
);

impl Rule for JsxBooleanValue {
    fn from_configuration(value: serde_json::Value) -> Self {
        let enforce_boolean_attribute = value
            .get(0)
            .and_then(serde_json::Value::as_str)
            .map_or_else(EnforceBooleanAttribute::default, |value| match value {
                "always" => EnforceBooleanAttribute::Always,
                _ => EnforceBooleanAttribute::Never,
            });

        let config = value.get(1);
        let assume_undefined_is_false = config
            .and_then(|c| c.get("assumeUndefinedIsFalse"))
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);

        // The exceptions are the inverse of the default, specifying both always and
        // never in the rule configuration is not allowed and ignored.
        let attribute_name = match enforce_boolean_attribute {
            EnforceBooleanAttribute::Never => "always",
            EnforceBooleanAttribute::Always => "never",
        };

        let exceptions = config
            .and_then(|c| c.get(attribute_name))
            .and_then(serde_json::Value::as_array)
            .map(|v| v.iter().filter_map(serde_json::Value::as_str).map(CompactStr::from).collect())
            .unwrap_or_default();

        Self(Box::new(JsxBooleanValueConfig {
            enforce_boolean_attribute,
            exceptions,
            assume_undefined_is_false,
        }))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(jsx_opening_elem) = node.kind() else { return };

        for attr in &jsx_opening_elem.attributes {
            let JSXAttributeItem::Attribute(jsx_attr) = attr else { continue };
            let JSXAttributeName::Identifier(ident) = &jsx_attr.name else { continue };

            match get_prop_value(attr) {
                None => {
                    if self.is_always(ident.name.as_str()) {
                        ctx.diagnostic_with_fix(
                            boolean_value_always_diagnostic(&ident.name, ident.span),
                            |fixer| fixer.insert_text_after(&ident.span, "={true}"),
                        );
                    }
                }
                Some(JSXAttributeValue::ExpressionContainer(container)) => {
                    if let Some(expr) = container.expression.as_expression() {
                        if let Expression::BooleanLiteral(expr) = expr.without_parentheses() {
                            if expr.value && self.is_never(ident.name.as_str()) {
                                let span = Span::new(ident.span.end, jsx_attr.span.end);
                                ctx.diagnostic_with_fix(
                                    boolean_value_diagnostic(&ident.name, span),
                                    |fixer| fixer.delete_range(span),
                                );
                            }

                            if !expr.value
                                && self.is_never(ident.name.as_str())
                                && self.assume_undefined_is_false
                            {
                                ctx.diagnostic_with_fix(
                                    boolean_value_undefined_false_diagnostic(
                                        &ident.name,
                                        jsx_attr.span,
                                    ),
                                    |fixer| fixer.delete(&jsx_attr.span),
                                );
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_jsx()
    }
}

impl JsxBooleanValue {
    fn is_always(&self, prop_name: &str) -> bool {
        let is_exception = self.exceptions.contains(prop_name);
        if matches!(self.enforce_boolean_attribute, EnforceBooleanAttribute::Always) {
            return !is_exception;
        }
        is_exception
    }

    fn is_never(&self, prop_name: &str) -> bool {
        let is_exception = self.exceptions.contains(prop_name);
        if matches!(self.enforce_boolean_attribute, EnforceBooleanAttribute::Never) {
            return !is_exception;
        }
        is_exception
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("<App foo />;", Some(serde_json::json!(["never"]))),
        ("<App foo bar={true} />;", Some(serde_json::json!(["always", { "never": ["foo"] }]))),
        ("<App foo />;", None),
        ("<App foo={true} />;", Some(serde_json::json!(["always"]))),
        ("<App foo={true} bar />;", Some(serde_json::json!(["never", { "always": ["foo"] }]))),
        ("<App />;", Some(serde_json::json!(["never", { "assumeUndefinedIsFalse": true }]))),
        (
            "<App foo={false} />;",
            Some(
                serde_json::json!(["never", { "assumeUndefinedIsFalse": true, "always": ["foo"] }]),
            ),
        ),
    ];

    let fail = vec![
        ("<App foo={true} />;", Some(serde_json::json!(["never"]))),
        (
            "<App foo={true} bar={true} baz={true} />;",
            Some(serde_json::json!(["always", { "never": ["foo", "bar"] }])),
        ),
        ("<App foo={true} />;", None),
        ("<App foo = {true} />;", None),
        ("<App foo />;", Some(serde_json::json!(["always"]))),
        ("<App foo bar baz />;", Some(serde_json::json!(["never", { "always": ["foo", "bar"] }]))),
        (
            "<App foo={false} bak={false} />;",
            Some(serde_json::json!(["never", { "assumeUndefinedIsFalse": true }])),
        ),
        (
            "<App foo={true} bar={false} baz={false} bak={false} />;",
            Some(serde_json::json!([
            "always",
                { "assumeUndefinedIsFalse": true, "never": ["baz", "bak"] },
              ])),
        ),
        (
            "<App foo={true} bar={true} baz />;",
            Some(serde_json::json!(["always", { "never": ["foo", "bar"] }])),
        ),
    ];

    let fix = vec![
        ("<App foo = {true} />", "<App foo />", None),
        (
            "<App foo={false} bak={false} />;",
            "<App   />;",
            Some(serde_json::json!(["never", { "assumeUndefinedIsFalse": true }])),
        ),
        (
            "<App foo={true} bak={false} />;",
            "<App foo  />;",
            Some(serde_json::json!(["never", { "assumeUndefinedIsFalse": true }])),
        ),
        (
            "<App foo={true} bar={false} baz={false} bak={false} />;",
            "<App foo={true} bar={false}   />;",
            Some(serde_json::json!([
            "always",
                { "assumeUndefinedIsFalse": true, "never": ["baz", "bak"] },
              ])),
        ),
        ("<App foo />", "<App foo={true} />", Some(serde_json::json!(["always"]))),
    ];

    Tester::new(JsxBooleanValue::NAME, pass, fail).expect_fix(fix).test_and_snapshot();
}
