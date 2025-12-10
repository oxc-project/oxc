use oxc_ast::{
    AstKind,
    ast::{JSXAttributeItem, JSXAttributeValue},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, Span};
use rustc_hash::FxHashSet;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;
use std::ops::Deref;

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
    utils::{get_element_type, has_jsx_prop_ignore_case},
};

fn autocomplete_valid_diagnostic(span: Span, autocomplete: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("`{autocomplete}` is not a valid value for `autocomplete`."))
        .with_help(format!("Change `{autocomplete}` to a valid value for `autocomplete`."))
        .with_label(span)
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct AutocompleteValid(Box<AutocompleteValidConfig>);

#[derive(Debug, Clone, PartialEq, Eq, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AutocompleteValidConfig {
    /// List of custom component names that should be treated as input elements.
    input_components: FxHashSet<CompactStr>,
}

impl Deref for AutocompleteValid {
    type Target = AutocompleteValidConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for AutocompleteValidConfig {
    fn default() -> Self {
        Self { input_components: FxHashSet::from_iter([CompactStr::new("input")]) }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces that an element's autocomplete attribute must be a valid value.
    ///
    /// ### Why is this bad?
    ///
    /// Incorrectly using the autocomplete attribute may decrease the accessibility of the website for users.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <input autocomplete="invalid-value" />
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <input autocomplete="name" />
    /// ```
    AutocompleteValid,
    jsx_a11y,
    correctness,
    config = AutocompleteValidConfig
);

static VALID_AUTOCOMPLETE_VALUES: phf::Set<&'static str> = phf::phf_set![
    "address-level1",
    "address-level2",
    "address-level3",
    "address-level4",
    "address-line1",
    "address-line2",
    "address-line3",
    "bday",
    "bday-day",
    "bday-month",
    "bday-year",
    "cc-additional-name",
    "cc-csc",
    "cc-exp",
    "cc-exp-month",
    "cc-exp-year",
    "cc-family-name",
    "cc-given-name",
    "cc-name",
    "cc-number",
    "cc-type",
    "country",
    "country-name",
    "current-password",
    "email",
    "impp",
    "language",
    "name",
    "new-password",
    "off",
    "on",
    "one-time-code",
    "organization",
    "organization-title",
    "photo",
    "postal-code",
    "sex",
    "street-address",
    "tel",
    "tel-area-code",
    "tel-country-code",
    "tel-extension",
    "tel-local",
    "tel-national",
    "transaction-amount",
    "transaction-currency",
    "url",
    "username",
    "webauthn",
];

static BILLING_AND_SHIPPING: [&str; 11] = [
    "address-level1",
    "address-level2",
    "address-level3",
    "address-level4",
    "address-line1",
    "address-line2",
    "address-line3",
    "country",
    "country-name",
    "postal-code",
    "street-address",
];

fn is_valid_autocomplete_value(value: &str) -> bool {
    let parts: Vec<&str> = value.split_whitespace().collect();

    match parts.len() {
        1 => VALID_AUTOCOMPLETE_VALUES.contains(parts[0]),
        2 if ["billing", "shipping"].contains(&parts[0]) => {
            BILLING_AND_SHIPPING.contains(&parts[1])
        }
        _ => false,
    }
}

impl Rule for AutocompleteValid {
    fn from_configuration(config: Value) -> Self {
        serde_json::from_value::<DefaultRuleConfig<AutocompleteValid>>(config)
            .unwrap_or_default()
            .into_inner()
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::JSXOpeningElement(jsx_el) = node.kind() {
            let name = &get_element_type(ctx, jsx_el);

            if !self.input_components.contains(name.as_ref()) {
                return;
            }

            let Some(autocomplete_prop) = has_jsx_prop_ignore_case(jsx_el, "autocomplete") else {
                return;
            };
            let attr = match autocomplete_prop {
                JSXAttributeItem::Attribute(attr) => attr,
                JSXAttributeItem::SpreadAttribute(_) => return,
            };
            let Some(JSXAttributeValue::StringLiteral(autocomplete_values)) = &attr.value else {
                return;
            };
            let value = &autocomplete_values.value;
            if !is_valid_autocomplete_value(value) {
                ctx.diagnostic(autocomplete_valid_diagnostic(attr.span, value));
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    fn settings() -> serde_json::Value {
        serde_json::json!({
            "settings": { "jsx-a11y": {
                "components": {
                    "Input": "input",
                }
            } }
        })
    }

    let pass = vec![
        ("<input type='text' />;", None, None),
        ("<input type='text' autocomplete='name' />;", None, None),
        // ("<input type='text' autocomplete='' />;", None, None),
        ("<input type='text' autocomplete='off' />;", None, None),
        ("<input type='text' autocomplete='on' />;", None, None),
        // ("<input type='text' autocomplete='billing family-name' />;", None, None),
        // ("<input type='text' autocomplete='section-blue shipping street-address' />;", None, None),
        // ("<input type='text' autocomplete='section-somewhere shipping work email' />;", None, None),
        ("<input type='text' autocomplete />;", None, None),
        ("<input type='text' autocomplete={autocompl} />;", None, None),
        ("<input type='text' autocomplete={autocompl || 'name'} />;", None, None),
        ("<input type='text' autocomplete={autocompl || 'foo'} />;", None, None),
        ("<Foo autocomplete='bar'></Foo>;", None, None),
        // ("<input type={isEmail ? 'email' : 'text'} autocomplete='none' />;", None, None),
        ("<Input type='text' autocomplete='name' />", None, Some(settings())),
        ("<Input type='text' autocomplete='baz' />", None, None),
        ("<input type='date' autocomplete='email' />;", None, None),
        ("<input type='number' autocomplete='url' />;", None, None),
        ("<input type='month' autocomplete='tel' />;", None, None),
        (
            "<Foo type='month' autocomplete='tel'></Foo>;",
            Some(serde_json::json!([{ "inputComponents": ["Foo"] }])),
            None,
        ),
    ];

    let fail = vec![
        ("<input type='text' autocomplete='foo' />;", None, None),
        ("<input type='text' autocomplete='name invalid' />;", None, None),
        ("<input type='text' autocomplete='invalid name' />;", None, None),
        ("<input type='text' autocomplete='home url' />;", None, None),
        (
            "<Bar autocomplete='baz'></Bar>;",
            Some(serde_json::json!([{ "inputComponents": ["Bar"] }])),
            None,
        ),
        ("<Input type='text' autocomplete='baz' />;", None, Some(settings())),
    ];

    Tester::new(AutocompleteValid::NAME, AutocompleteValid::PLUGIN, pass, fail).test_and_snapshot();
}
