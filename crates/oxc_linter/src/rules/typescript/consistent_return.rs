use oxc_macros::declare_oxc_lint;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::rule::{DefaultRuleConfig, Rule};

#[derive(Debug, Default, Clone, Deserialize)]
pub struct ConsistentReturn(Box<ConsistentReturnConfig>);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct ConsistentReturnConfig {
    /// Treat explicit `return undefined` as equivalent to an unspecified return.
    pub treat_undefined_as_unspecified: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce consistent return behavior in functions.
    ///
    /// ### Why is this bad?
    ///
    /// Mixing value-returning and non-value-returning code paths makes control flow harder to
    /// reason about and frequently indicates a bug.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// function maybe(flag: boolean): number {
    ///   if (flag) {
    ///     return 1;
    ///   }
    ///   return;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// function maybe(flag: boolean): number {
    ///   if (flag) {
    ///     return 1;
    ///   }
    ///   return 0;
    /// }
    /// ```
    ConsistentReturn(tsgolint),
    typescript,
    nursery,
    config = ConsistentReturnConfig,
);

impl Rule for ConsistentReturn {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn to_configuration(&self) -> Option<Result<serde_json::Value, serde_json::Error>> {
        Some(serde_json::to_value(&*self.0))
    }
}
