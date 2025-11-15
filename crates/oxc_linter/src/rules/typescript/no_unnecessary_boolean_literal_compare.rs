use oxc_macros::declare_oxc_lint;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    rule::{DefaultRuleConfig, Rule},
    utils::default_true,
};

#[derive(Debug, Default, Clone)]
pub struct NoUnnecessaryBooleanLiteralCompare(Box<NoUnnecessaryBooleanLiteralCompareConfig>);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct NoUnnecessaryBooleanLiteralCompareConfig {
    /// Whether to allow comparing nullable boolean expressions to `false`.
    /// When false, `x === false` where x is `boolean | null` will be flagged.
    #[serde(default = "default_true")]
    pub allow_comparing_nullable_booleans_to_false: bool,
    /// Whether to allow comparing nullable boolean expressions to `true`.
    /// When false, `x === true` where x is `boolean | null` will be flagged.
    #[serde(default = "default_true")]
    pub allow_comparing_nullable_booleans_to_true: bool,
    /// Whether to allow this rule to run without `strictNullChecks` enabled.
    /// This is not recommended as the rule may produce incorrect results.
    pub allow_rule_to_run_without_strict_null_checks_i_know_what_i_am_doing: bool,
}

impl Default for NoUnnecessaryBooleanLiteralCompareConfig {
    fn default() -> Self {
        Self {
            allow_comparing_nullable_booleans_to_false: true,
            allow_comparing_nullable_booleans_to_true: true,
            allow_rule_to_run_without_strict_null_checks_i_know_what_i_am_doing: false,
        }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule disallows unnecessary equality comparisons with boolean literals.
    ///
    /// ### Why is this bad?
    ///
    /// Comparing boolean values to boolean literals is unnecessary when the comparison can be eliminated. These comparisons make code more verbose without adding value.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// declare const someCondition: boolean;
    ///
    /// if (someCondition === true) {
    ///   // ...
    /// }
    ///
    /// if (someCondition === false) {
    ///   // ...
    /// }
    ///
    /// if (someCondition !== true) {
    ///   // ...
    /// }
    ///
    /// if (someCondition !== false) {
    ///   // ...
    /// }
    ///
    /// const result = someCondition == true;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// declare const someCondition: boolean;
    ///
    /// if (someCondition) {
    ///   // ...
    /// }
    ///
    /// if (!someCondition) {
    ///   // ...
    /// }
    ///
    /// // Comparisons with non-boolean types are allowed
    /// declare const someValue: unknown;
    /// if (someValue === true) {
    ///   // ...
    /// }
    /// ```
    NoUnnecessaryBooleanLiteralCompare(tsgolint),
    typescript,
    suspicious,
    pending,
    config = NoUnnecessaryBooleanLiteralCompareConfig,
);

impl Rule for NoUnnecessaryBooleanLiteralCompare {
    fn from_configuration(value: serde_json::Value) -> Self {
        Self(Box::new(
            serde_json::from_value::<DefaultRuleConfig<NoUnnecessaryBooleanLiteralCompareConfig>>(
                value,
            )
            .unwrap_or_default()
            .into_inner(),
        ))
    }

    fn to_configuration(&self) -> Option<Result<serde_json::Value, serde_json::Error>> {
        Some(serde_json::to_value(&*self.0))
    }
}
