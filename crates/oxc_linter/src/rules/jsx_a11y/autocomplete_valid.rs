use oxc_ast::{
    AstKind,
    ast::{JSXAttributeItem, JSXAttributeValue},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_str::CompactStr;
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
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
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
    recommended,
    config = AutocompleteValidConfig,
    version = "0.2.0",
    short_description = "Enforces that an element's autocomplete attribute must be a valid value.",
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
    "additional-name",
    "country",
    "country-name",
    "current-password",
    "email",
    "family-name",
    "given-name",
    "honorific-prefix",
    "honorific-suffix",
    "impp",
    "language",
    "name",
    "nickname",
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
    "tel-local-prefix",
    "tel-local-suffix",
    "tel-national",
    "transaction-amount",
    "transaction-currency",
    "url",
    "username",
    "webauthn",
];

const BILLING_OR_SHIPPING: [&str; 2] = ["billing", "shipping"];

/// Contact-type tokens may precede a contact field per the WHATWG autofill grammar.
const CONTACT_GROUPING_PREFIXES: [&str; 5] = ["home", "work", "mobile", "fax", "pager"];

const CONTACT_FIELDS: [&str; 10] = [
    "tel",
    "tel-country-code",
    "tel-national",
    "tel-area-code",
    "tel-local",
    "tel-local-prefix",
    "tel-local-suffix",
    "tel-extension",
    "email",
    "impp",
];

fn is_valid_autocomplete_value(value: &str) -> bool {
    let mut parts: Vec<&str> = value.split_whitespace().collect();

    // `webauthn` is only allowed as the final token.
    let stripped_webauthn = parts.len() > 1 && parts.last() == Some(&"webauthn");
    if stripped_webauthn {
        parts.pop();
    }

    let mut index = 0;
    if parts.get(index).is_some_and(|token| token.starts_with("section-")) {
        index += 1;
    }
    if parts.get(index).is_some_and(|token| BILLING_OR_SHIPPING.contains(token)) {
        index += 1;
    }
    if parts.get(index).is_some_and(|token| CONTACT_GROUPING_PREFIXES.contains(token)) {
        index += 1;
        // A contact grouping only applies to contact fields.
        return parts.get(index).is_some_and(|field| CONTACT_FIELDS.contains(field))
            && index + 1 == parts.len();
    }

    let Some(&field) = parts.get(index) else {
        return false;
    };
    if index + 1 != parts.len() {
        return false;
    }
    if index == 0 {
        // A lone field token; `on`, `off`, and `webauthn` cannot combine
        // with a trailing `webauthn`.
        VALID_AUTOCOMPLETE_VALUES.contains(field)
            && !(stripped_webauthn && matches!(field, "on" | "off" | "webauthn"))
    } else {
        // After a `section-` or billing modifier the token must be a field name.
        VALID_AUTOCOMPLETE_VALUES.contains(field) && !matches!(field, "on" | "off" | "webauthn")
    }
}

impl Rule for AutocompleteValid {
    fn from_configuration(value: Value) -> Result<Self, serde_json::error::Error> {
        DefaultRuleConfig::<Self>::from_value(value).map(DefaultRuleConfig::into_inner)
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
            let Some(value) = autocomplete_values.value.as_str() else {
                ctx.diagnostic(autocomplete_valid_diagnostic(
                    attr.span,
                    ctx.source_range(autocomplete_values.span),
                ));
                return;
            };
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
        ("<input type='text' autocomplete='billing family-name' />;", None, None),
        ("<input type='text' autocomplete='section-blue shipping street-address' />;", None, None),
        ("<input type='text' autocomplete='section-somewhere shipping work email' />;", None, None),
        ("<input type='text' autocomplete='given-name' />;", None, None),
        ("<input type='text' autocomplete='additional-name' />;", None, None),
        ("<input type='text' autocomplete='family-name' />;", None, None),
        ("<input type='text' autocomplete='honorific-prefix' />;", None, None),
        ("<input type='text' autocomplete='honorific-suffix' />;", None, None),
        ("<input type='text' autocomplete='nickname' />;", None, None),
        ("<input type='text' autocomplete='tel-local-prefix' />;", None, None),
        ("<input type='text' autocomplete='tel-local-suffix' />;", None, None),
        ("<input type='text' autocomplete='section-billing email' />;", None, None),
        ("<input type='text' autocomplete='work email' />;", None, None),
        ("<input type='text' autocomplete='home tel' />;", None, None),
        ("<input type='text' autocomplete='billing name' />;", None, None),
        ("<input type='text' autocomplete='shipping tel' />;", None, None),
        ("<input type='text' autocomplete='mobile tel-extension' />;", None, None),
        ("<input type='text' autocomplete='fax impp' />;", None, None),
        ("<input type='text' autocomplete='pager tel-local' />;", None, None),
        ("<input type='text' autocomplete='current-password webauthn' />;", None, None),
        (
            "<input type='text' autocomplete='section-a shipping street-address webauthn' />;",
            None,
            None,
        ),
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
        ("<input type='text' autocomplete='work name' />;", None, None),
        ("<input type='text' autocomplete='shipping section-a street-address' />;", None, None),
        ("<input type='text' autocomplete='section-a billing' />;", None, None),
        ("<input type='text' autocomplete='on webauthn' />;", None, None),
        ("<input type='text' autocomplete='billing on' />;", None, None),
        ("<input type='text' autocomplete='email webauthn webauthn' />;", None, None),
        (
            "<Bar autocomplete='baz'></Bar>;",
            Some(serde_json::json!([{ "inputComponents": ["Bar"] }])),
            None,
        ),
        ("<Input type='text' autocomplete='baz' />;", None, Some(settings())),
    ];

    Tester::new(AutocompleteValid::NAME, AutocompleteValid::PLUGIN, pass, fail).test_and_snapshot();
}
