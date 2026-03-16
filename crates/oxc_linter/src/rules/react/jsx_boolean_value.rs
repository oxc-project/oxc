use oxc_ast::{
    AstKind,
    ast::{Expression, JSXAttributeItem, JSXAttributeName, JSXAttributeValue},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, Span};
use rustc_hash::FxHashSet;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    rule::{Rule, TupleRuleConfig},
    utils::get_prop_value,
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

#[derive(Debug, Default, Clone, JsonSchema, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum EnforceBooleanAttribute {
    /// All boolean attributes must have explicit values.
    Always,
    /// All boolean attributes must omit values that are set to `true`.
    #[default]
    Never,
}

#[derive(Debug, Default, Clone, JsonSchema, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct JsxBooleanValueOptions {
    /// List of attribute names that should always have explicit boolean values.
    /// Only necessary when main mode is `"never"`.
    always: FxHashSet<CompactStr>,
    /// List of attribute names that should never have explicit boolean values.
    /// Only necessary when main mode is `"always"`.
    never: FxHashSet<CompactStr>,
    /// If `true`, treats `prop={false}` as equivalent to the prop being `undefined`.
    /// When combined with `"never"` mode, this will enforce that the attribute is omitted entirely.
    ///
    /// ```jsx
    /// // With "assumeUndefinedIsFalse": true
    /// <App foo={false} />; // Incorrect
    /// <App />;             // Correct
    /// ```
    ///
    /// This option does nothing in `"always"` mode.
    assume_undefined_is_false: bool,
}

#[derive(Debug, Default, Clone, JsonSchema, Deserialize, Serialize)]
#[serde(default)]
pub struct JsxBooleanValueConfig(EnforceBooleanAttribute, JsxBooleanValueOptions);

#[derive(Debug, Default, Clone, Deserialize)]
pub struct JsxBooleanValue(Box<JsxBooleanValueConfig>);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce a consistent boolean attribute style in your code.
    ///
    /// ### Why is this bad?
    ///
    /// In JSX, you can set a boolean attribute to `true` or omit it.
    /// This rule will enforce a consistent style for boolean attributes.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule with default `"never"` mode:
    /// ```jsx
    /// const Hello = <Hello personal={true} />;
    /// ```
    ///
    /// Examples of **correct** code for this rule with default `"never"` mode:
    /// ```jsx
    /// const Hello = <Hello personal />;
    ///
    /// const Foo = <Foo isSomething={false} />;
    /// ```
    ///
    /// Examples of **incorrect** code for this rule with `"always"` mode:
    /// ```jsx
    /// const Hello = <Hello personal />;
    /// ```
    ///
    /// Examples of **correct** code for this rule with `"always"` mode:
    /// ```jsx
    /// const Hello = <Hello personal={true} />;
    /// ```
    JsxBooleanValue,
    react,
    style,
    fix,
    config = JsxBooleanValueConfig,
);

impl Rule for JsxBooleanValue {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<TupleRuleConfig<Self>>(value).map(TupleRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(jsx_opening_elem) = node.kind() else { return };

        let JsxBooleanValueConfig(mode, options) = &*self.0;

        for attr in &jsx_opening_elem.attributes {
            let JSXAttributeItem::Attribute(jsx_attr) = attr else { continue };
            let JSXAttributeName::Identifier(ident) = &jsx_attr.name else { continue };

            match get_prop_value(attr) {
                None => {
                    if Self::is_always(mode, options, ident.name.as_str()) {
                        ctx.diagnostic_with_fix(
                            boolean_value_always_diagnostic(&ident.name, ident.span),
                            |fixer| fixer.insert_text_after(&ident.span, "={true}"),
                        );
                    }
                }
                Some(JSXAttributeValue::ExpressionContainer(container)) => {
                    if let Some(expr) = container.expression.as_expression()
                        && let Expression::BooleanLiteral(expr) = expr.without_parentheses()
                    {
                        if expr.value && Self::is_never(mode, options, ident.name.as_str()) {
                            let span = Span::new(ident.span.end, jsx_attr.span.end);
                            ctx.diagnostic_with_fix(
                                boolean_value_diagnostic(&ident.name, span),
                                |fixer| fixer.delete_range(span),
                            );
                        }

                        if !expr.value
                            && Self::is_never(mode, options, ident.name.as_str())
                            && options.assume_undefined_is_false
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
                _ => {}
            }
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_jsx()
    }
}

impl JsxBooleanValue {
    /// Returns true if the attribute should always have an explicit boolean value.
    fn is_always(
        mode: &EnforceBooleanAttribute,
        options: &JsxBooleanValueOptions,
        prop_name: &str,
    ) -> bool {
        match mode {
            // When mode is "always", all attributes should have explicit values except those in `never`
            EnforceBooleanAttribute::Always => !options.never.contains(prop_name),
            // When mode is "never", only attributes in `always` should have explicit values
            EnforceBooleanAttribute::Never => options.always.contains(prop_name),
        }
    }

    /// Returns true if the attribute should never have an explicit boolean value.
    fn is_never(
        mode: &EnforceBooleanAttribute,
        options: &JsxBooleanValueOptions,
        prop_name: &str,
    ) -> bool {
        match mode {
            // When mode is "never", all attributes should omit values except those in `always`
            EnforceBooleanAttribute::Never => !options.always.contains(prop_name),
            // When mode is "always", only attributes in `never` should omit values
            EnforceBooleanAttribute::Always => options.never.contains(prop_name),
        }
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
            Some(
                serde_json::json!(["always", { "assumeUndefinedIsFalse": true, "never": ["baz", "bak"] }]),
            ),
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
            Some(
                serde_json::json!(["always", { "assumeUndefinedIsFalse": true, "never": ["baz", "bak"] }]),
            ),
        ),
        ("<App foo />", "<App foo={true} />", Some(serde_json::json!(["always"]))),
    ];

    Tester::new(JsxBooleanValue::NAME, JsxBooleanValue::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
