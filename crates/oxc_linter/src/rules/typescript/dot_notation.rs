use oxc_macros::declare_oxc_lint;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::rule::{DefaultRuleConfig, Rule};

#[derive(Debug, Default, Clone, Deserialize)]
pub struct DotNotation(Box<DotNotationConfig>);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
#[expect(clippy::struct_field_names)]
pub struct DotNotationConfig {
    /// Allow bracket notation for properties covered by an index signature.
    pub allow_index_signature_property_access: bool,
    /// Allow bracket notation for ES3 keyword property names (for example `obj["class"]`).
    pub allow_keywords: bool,
    /// Regex pattern for property names that are allowed to use bracket notation.
    pub allow_pattern: String,
    /// Allow bracket notation for private class members.
    pub allow_private_class_property_access: bool,
    /// Allow bracket notation for protected class members.
    pub allow_protected_class_property_access: bool,
}

impl Default for DotNotationConfig {
    fn default() -> Self {
        Self {
            allow_index_signature_property_access: false,
            allow_keywords: true,
            allow_pattern: String::new(),
            allow_private_class_property_access: false,
            allow_protected_class_property_access: false,
        }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce dot notation whenever property access can be written safely as `obj.prop`.
    ///
    /// ### Why is this bad?
    ///
    /// Dot notation is generally more readable and concise than bracket notation for static
    /// property names.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// obj['name'];
    /// foo['bar'];
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// obj.name;
    /// foo.bar;
    ///
    /// obj[key];
    /// obj['not-an-identifier'];
    /// ```
    DotNotation(tsgolint),
    typescript,
    nursery,
    config = DotNotationConfig,
);

impl Rule for DotNotation {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn to_configuration(&self) -> Option<Result<serde_json::Value, serde_json::Error>> {
        Some(serde_json::to_value(&*self.0))
    }
}
