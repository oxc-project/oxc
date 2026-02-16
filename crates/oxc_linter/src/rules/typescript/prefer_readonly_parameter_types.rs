use oxc_macros::declare_oxc_lint;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    rule::{DefaultRuleConfig, Rule},
    utils::TypeOrValueSpecifier,
};

#[derive(Debug, Default, Clone, Deserialize)]
pub struct PreferReadonlyParameterTypes(Box<PreferReadonlyParameterTypesConfig>);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct PreferReadonlyParameterTypesConfig {
    /// Type/value specifiers that should be exempt from this rule.
    pub allow: Vec<TypeOrValueSpecifier>,
    /// Whether to check constructor parameter properties.
    pub check_parameter_properties: bool,
    /// Whether to ignore parameters without explicit type annotations.
    pub ignore_inferred_types: bool,
    /// Whether mutable methods should be treated as readonly members.
    pub treat_methods_as_readonly: bool,
}

impl Default for PreferReadonlyParameterTypesConfig {
    fn default() -> Self {
        Self {
            allow: Vec::new(),
            check_parameter_properties: true,
            ignore_inferred_types: false,
            treat_methods_as_readonly: false,
        }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Require function and method parameters to use readonly-compatible types.
    ///
    /// ### Why is this bad?
    ///
    /// Mutable parameter types make accidental mutation easier and weaken function contracts.
    /// Readonly parameter types communicate intent and improve API safety.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// function update(items: string[]) {
    ///   items.push('x');
    /// }
    ///
    /// function consume(obj: { value: string }) {
    ///   obj.value = obj.value.trim();
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// function update(items: readonly string[]) {
    ///   return items.length;
    /// }
    ///
    /// function consume(obj: Readonly<{ value: string }>) {
    ///   return obj.value;
    /// }
    /// ```
    PreferReadonlyParameterTypes(tsgolint),
    typescript,
    nursery,
    config = PreferReadonlyParameterTypesConfig,
);

impl Rule for PreferReadonlyParameterTypes {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn to_configuration(&self) -> Option<Result<serde_json::Value, serde_json::Error>> {
        Some(serde_json::to_value(&*self.0))
    }
}
