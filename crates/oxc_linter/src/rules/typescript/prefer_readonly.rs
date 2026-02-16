use oxc_macros::declare_oxc_lint;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::rule::{DefaultRuleConfig, Rule};

#[derive(Debug, Default, Clone, Deserialize)]
pub struct PreferReadonly(Box<PreferReadonlyConfig>);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct PreferReadonlyConfig {
    /// Restrict checks to members immediately initialized with inline lambda values.
    pub only_inline_lambdas: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Require class members that are never reassigned to be marked `readonly`.
    ///
    /// ### Why is this bad?
    ///
    /// Members that never change should be declared `readonly` to make class invariants explicit
    /// and prevent accidental mutation.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// class Counter {
    ///   private value = 0;
    ///
    ///   getValue() {
    ///     return this.value;
    ///   }
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// class Counter {
    ///   private readonly value = 0;
    ///
    ///   getValue() {
    ///     return this.value;
    ///   }
    /// }
    /// ```
    PreferReadonly(tsgolint),
    typescript,
    nursery,
    config = PreferReadonlyConfig,
);

impl Rule for PreferReadonly {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn to_configuration(&self) -> Option<Result<serde_json::Value, serde_json::Error>> {
        Some(serde_json::to_value(&*self.0))
    }
}
