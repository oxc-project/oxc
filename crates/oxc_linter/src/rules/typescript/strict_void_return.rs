use oxc_macros::declare_oxc_lint;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::rule::{DefaultRuleConfig, Rule};

#[derive(Debug, Default, Clone, Deserialize)]
pub struct StrictVoidReturn(Box<StrictVoidReturnConfig>);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct StrictVoidReturnConfig {
    /// Allow callbacks that return `any` in places that expect a `void` callback.
    pub allow_return_any: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow returning non-void values where a `void` return is expected.
    ///
    /// ### Why is this bad?
    ///
    /// Returning values from `void` contexts can hide logic errors and make callback APIs
    /// behave unexpectedly.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// declare function run(cb: () => void): void;
    ///
    /// run(() => 'value');
    /// run(async () => 123);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// declare function run(cb: () => void): void;
    ///
    /// run(() => {
    ///   doWork();
    /// });
    ///
    /// run(() => undefined);
    /// ```
    StrictVoidReturn(tsgolint),
    typescript,
    nursery,
    config = StrictVoidReturnConfig,
);

impl Rule for StrictVoidReturn {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn to_configuration(&self) -> Option<Result<serde_json::Value, serde_json::Error>> {
        Some(serde_json::to_value(&*self.0))
    }
}
