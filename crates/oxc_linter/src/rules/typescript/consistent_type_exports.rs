use oxc_macros::declare_oxc_lint;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::rule::{DefaultRuleConfig, Rule};

#[derive(Debug, Default, Clone, Deserialize)]
pub struct ConsistentTypeExports(Box<ConsistentTypeExportsConfig>);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct ConsistentTypeExportsConfig {
    /// Enables an autofix strategy that rewrites mixed exports using inline `type` specifiers.
    pub fix_mixed_exports_with_inline_type_specifier: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce using `export type` for exports that are only used as types.
    ///
    /// ### Why is this bad?
    ///
    /// Mixing type-only exports with value exports without `export type` makes module intent
    /// harder to read and can cause unnecessary runtime export surface.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// type Foo = { bar: string };
    /// export { Foo };
    ///
    /// export { TypeOnly, value } from "./mod";
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// type Foo = { bar: string };
    /// export type { Foo };
    ///
    /// export type { TypeOnly } from "./mod";
    /// export { value } from "./mod";
    /// ```
    ConsistentTypeExports(tsgolint),
    typescript,
    nursery,
    config = ConsistentTypeExportsConfig,
);

impl Rule for ConsistentTypeExports {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn to_configuration(&self) -> Option<Result<serde_json::Value, serde_json::Error>> {
        Some(serde_json::to_value(&*self.0))
    }
}
