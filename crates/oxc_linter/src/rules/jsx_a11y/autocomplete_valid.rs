use oxc_ast::{
    ast::{JSXAttributeItem, JSXAttributeValue},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, Span};
use phf::{phf_map, phf_set};
use rustc_hash::FxHashSet;
use serde_json::Value;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{get_element_type, has_jsx_prop_ignore_case},
    AstNode,
};

fn autocomplete_valid_diagnostic(span: Span, autocomplete: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("`{autocomplete}` is not a valid value for autocomplete."))
        .with_help(format!("Change `{autocomplete}` to a valid value for autocomplete."))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct AutocompleteValid(Box<AutocompleteValidConfig>);
declare_oxc_lint!(
    /// ### What it does
    /// Enforces that an element's autocomplete attribute must be a valid value.
    ///
    /// ### Why is this bad?
    /// Incorrectly using the autocomplete attribute may decrease the accessibility of the website for users.
    ///
    /// ### Example
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
    correctness
);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutocompleteValidConfig {
    input_components: FxHashSet<CompactStr>,
}

impl std::ops::Deref for AutocompleteValid {
    type Target = AutocompleteValidConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::default::Default for AutocompleteValidConfig {
    fn default() -> Self {
        Self { input_components: FxHashSet::from_iter([CompactStr::new("input")]) }
    }
}

static VALID_AUTOCOMPLETE_VALUES: phf::Set<&'static str> = phf_set! {
    "on",
    "name",
    "email",
    "username",
    "new-password",
    "current-password",
    "one-time-code",
    "off",
    "organization-title",
    "organization",
    "street-address",
    "address-line1",
    "address-line2",
    "address-line3",
    "address-level4",
    "address-level3",
    "address-level2",
    "address-level1",
    "country",
    "country-name",
    "postal-code",
    "cc-name",
    "cc-given-name",
    "cc-additional-name",
    "cc-family-name",
    "cc-number",
    "cc-exp",
    "cc-exp-month",
    "cc-exp-year",
    "cc-csc",
    "cc-type",
    "transaction-currency",
    "transaction-amount",
    "language",
    "bday",
    "bday-day",
    "bday-month",
    "bday-year",
    "sex",
    "tel",
    "tel-country-code",
    "tel-national",
    "tel-area-code",
    "tel-local",
    "tel-extension",
    "impp",
    "url",
    "photo",
    "webauthn",
};

static BILLING: phf::Set<&'static str> = phf_set! {
    "street-address",
    "address-line1",
    "address-line2",
    "address-line3",
    "address-level4",
    "address-level3",
    "address-level2",
    "address-level1",
    "country",
    "country-name",
    "postal-code",
};

static SHIPPING: phf::Set<&'static str> = phf_set! {
    "street-address",
    "address-line1",
    "address-line2",
    "address-line3",
    "address-level4",
    "address-level3",
    "address-level2",
    "address-level1",
    "country",
    "country-name",
    "postal-code",
};

static VALID_AUTOCOMPLETE_COMBINATIONS: phf::Map<&'static str, &'static phf::Set<&'static str>> = phf_map! {
    "billing" => &BILLING,
    "shipping" => &SHIPPING,
};

fn is_valid_autocomplete_value(value: &str) -> bool {
    let parts: Vec<&str> = value.split_whitespace().collect();
    match parts.len() {
        1 => VALID_AUTOCOMPLETE_VALUES.contains(parts[0]),
        2 => VALID_AUTOCOMPLETE_COMBINATIONS
            .get(parts[0])
            .is_some_and(|valid_suffixes| valid_suffixes.contains(parts[1])),
        _ => false,
    }
}

impl Rule for AutocompleteValid {
    fn from_configuration(config: Value) -> Self {
        config
            .get(0)
            .and_then(|c| c.get("inputComponents"))
            .and_then(Value::as_array)
            .map(|components| {
                components
                    .iter()
                    .filter_map(Value::as_str)
                    .map(CompactStr::from)
                    .chain(Some("input".into()))
                    .collect()
            })
            .map(|input_components| Self(Box::new(AutocompleteValidConfig { input_components })))
            .unwrap_or_default()
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
