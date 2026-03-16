use oxc_macros::declare_oxc_lint;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::rule::{DefaultRuleConfig, Rule};

#[derive(Debug, Default, Clone, Deserialize)]
pub struct PreferStringStartsEndsWith(Box<PreferStringStartsEndsWithConfig>);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum AllowSingleElementEquality {
    Always,
    Never,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct PreferStringStartsEndsWithConfig {
    /// Whether equality checks against the first/last character are allowed.
    pub allow_single_element_equality: Option<AllowSingleElementEquality>,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefer `startsWith` and `endsWith` over manual string boundary checks.
    ///
    /// ### Why is this bad?
    ///
    /// Boundary checks written with `slice`, `indexOf`, regex anchors, or manual indexing are
    /// harder to read and maintain than `startsWith`/`endsWith`.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// value.slice(0, 3) === 'foo';
    /// value.slice(-3) === 'bar';
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// value.startsWith('foo');
    /// value.endsWith('bar');
    /// ```
    PreferStringStartsEndsWith(tsgolint),
    typescript,
    nursery,
    config = PreferStringStartsEndsWithConfig,
);

impl Rule for PreferStringStartsEndsWith {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn to_configuration(&self) -> Option<Result<serde_json::Value, serde_json::Error>> {
        Some(serde_json::to_value(&*self.0))
    }
}
