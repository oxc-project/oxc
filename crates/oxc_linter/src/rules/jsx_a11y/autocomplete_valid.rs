use crate::{context::LintContext, rule::Rule, utils::has_jsx_prop_lowercase, AstNode};
use oxc_ast::{
    ast::{JSXAttributeItem, JSXAttributeValue},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{self, Error},
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use phf::{phf_map, phf_set};
#[derive(Debug, Error, Diagnostic)]
#[error(
    "eslint-plugin-jsx-a11y(autocomplete-valid): `{autocomplete}` is not a valid value for autocomplete."
)]
#[diagnostic(severity(warning), help("Change `{autocomplete}` to a valid value for autocomplete."))]
struct AutocompleteValidDiagnostic {
    #[label]
    pub span: Span,
    pub autocomplete: String,
}

#[derive(Debug, Default, Clone)]
pub struct AutocompleteValid;
declare_oxc_lint!(
    /// ### What it does
    /// Enforces that an element's autocomplete attribute must be a valid value.
    ///
    /// ### Why is this bad?
    /// Incorrectly using the autocomplete attribute may decrease the accessibility of the website for users.
    ///
    /// ### Example
    /// ```javascript
    /// // Bad
    /// <input autocomplete="invalid-value" />
    ///
    /// // Good
    /// <input autocomplete="name" />
    /// ```
    AutocompleteValid,
    correctness
);

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
            .map_or(false, |valid_suffixes| valid_suffixes.contains(parts[1])),
        _ => false,
    }
}

impl Rule for AutocompleteValid {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::JSXOpeningElement(jsx_el) = node.kind() {
            let autocomplete_prop = match has_jsx_prop_lowercase(jsx_el, "autocomplete") {
                Some(autocomplete_prop) => autocomplete_prop,
                None => return,
            };
            let attr = match autocomplete_prop {
                JSXAttributeItem::Attribute(attr) => attr,
                JSXAttributeItem::SpreadAttribute(_) => return,
            };
            let autocomplete_values = match &attr.value {
                Some(JSXAttributeValue::StringLiteral(autocomplete_values)) => autocomplete_values,
                _ => return,
            };
            let value = autocomplete_values.value.to_string();
            if !is_valid_autocomplete_value(&value) {
                ctx.diagnostic(AutocompleteValidDiagnostic {
                    span: attr.span,
                    autocomplete: value,
                });
            }
        }
    }
}

#[test]
fn test() {
    use crate::rules::AutocompleteValid;
    use crate::tester::Tester;

    fn config() -> serde_json::Value {
        serde_json::json!([{
          "inputComponents": [ "Bar" ]
        }])
    }

    fn settings() -> serde_json::Value {
        serde_json::json!({
            "jsx-a11y": {
                "components": {
                    "Input": "input",
                }
            }
        })
    }

    let pass = vec![
        ("<input type='text' />;", None, None, None),
        ("<input type='text' autocomplete='name' />;", None, None, None),
        ("<input type='text' autocomplete='off' />", None, None, None),
        ("<input type='text' autocomplete='on' />;", None, None, None),
        ("<input type='text' autocomplete='shipping street-address' />;", None, None, None),
        ("<input type='text' autocomplete />;", None, None, None),
        ("<input type='text' autocomplete={autocompl} />;", None, None, None),
        ("<input type='text' autocomplete={autocompl || 'name'} />;", None, None, None),
        ("<input type='text' autocomplete={autocompl || 'foo'} />;", None, None, None),
        ("<input type={isEmail ? 'email' : 'text'} autocomplete='off' />;", None, None, None),
        ("<Input type='text' autocomplete='name' />", None, Some(settings()), None),
    ];

    let fail = vec![
        ("<input type='text' autocomplete='foo' />;", None, None, None),
        ("<input type='text' autocomplete='name invalid' />;", None, None, None),
        ("<input type='text' autocomplete='invalid name' />;", None, None, None),
        ("<input type='text' autocomplete='home url' />;", Some(config()), None, None),
        ("<Bar autocomplete='baz'></Bar>;", None, None, None),
        ("<Input type='text' autocomplete='baz' />;", None, Some(settings()), None),
    ];

    Tester::new(AutocompleteValid::NAME, pass, fail).test_and_snapshot();
}
