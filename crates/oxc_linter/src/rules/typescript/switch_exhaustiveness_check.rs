use oxc_macros::declare_oxc_lint;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::rule::{DefaultRuleConfig, Rule};

#[derive(Debug, Default, Clone, Deserialize)]
pub struct SwitchExhaustivenessCheck(Box<SwitchExhaustivenessCheckConfig>);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct SwitchExhaustivenessCheckConfig {
    /// Whether to allow default cases on switches that are not exhaustive.
    /// When false, requires exhaustive switch statements without default cases.
    pub allow_default_case_for_exhaustive_switch: bool,
    /// Whether to consider `default` cases exhaustive for union types.
    /// When true, a switch statement with a `default` case is considered exhaustive
    /// even if not all union members are handled explicitly.
    pub consider_default_exhaustive_for_unions: bool,
    /// Regular expression pattern that when matched in a default case comment,
    /// will suppress the exhaustiveness check.
    /// Example: `"@skip-exhaustive-check"` to allow `default: // @skip-exhaustive-check`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_case_comment_pattern: Option<String>,
    /// Whether to require default cases on switches over union types that are not exhaustive.
    /// When true, switches with non-exhaustive union types must have a default case.
    pub require_default_for_non_union: bool,
}

impl Default for SwitchExhaustivenessCheckConfig {
    fn default() -> Self {
        Self {
            allow_default_case_for_exhaustive_switch: true,
            consider_default_exhaustive_for_unions: false,
            default_case_comment_pattern: None,
            require_default_for_non_union: false,
        }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule requires switch statements to be exhaustive when switching on union types.
    ///
    /// ### Why is this bad?
    ///
    /// When switching on a union type, it's important to handle all possible cases to avoid runtime errors. TypeScript can help ensure exhaustiveness, but only if the switch statement is properly structured with a default case that TypeScript can analyze.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// type Status = 'pending' | 'approved' | 'rejected';
    ///
    /// function handleStatus(status: Status) {
    ///   switch (status) {
    ///     case 'pending':
    ///       return 'Waiting for approval';
    ///     case 'approved':
    ///       return 'Request approved';
    ///     // Missing 'rejected' case
    ///   }
    /// }
    ///
    /// enum Color {
    ///   Red,
    ///   Green,
    ///   Blue,
    /// }
    ///
    /// function getColorName(color: Color) {
    ///   switch (color) {
    ///     case Color.Red:
    ///       return 'red';
    ///     case Color.Green:
    ///       return 'green';
    ///     // Missing Color.Blue case
    ///   }
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// type Status = 'pending' | 'approved' | 'rejected';
    ///
    /// function handleStatus(status: Status) {
    ///   switch (status) {
    ///     case 'pending':
    ///       return 'Waiting for approval';
    ///     case 'approved':
    ///       return 'Request approved';
    ///     case 'rejected':
    ///       return 'Request rejected';
    ///   }
    /// }
    ///
    /// // Or with default case for exhaustiveness checking
    /// function handleStatusWithDefault(status: Status) {
    ///   switch (status) {
    ///     case 'pending':
    ///       return 'Waiting for approval';
    ///     case 'approved':
    ///       return 'Request approved';
    ///     case 'rejected':
    ///       return 'Request rejected';
    ///     default:
    ///       const _exhaustiveCheck: never = status;
    ///       return _exhaustiveCheck;
    ///   }
    /// }
    ///
    /// enum Color {
    ///   Red,
    ///   Green,
    ///   Blue,
    /// }
    ///
    /// function getColorName(color: Color) {
    ///   switch (color) {
    ///     case Color.Red:
    ///       return 'red';
    ///     case Color.Green:
    ///       return 'green';
    ///     case Color.Blue:
    ///       return 'blue';
    ///     default:
    ///       const _exhaustiveCheck: never = color;
    ///       return _exhaustiveCheck;
    ///   }
    /// }
    /// ```
    SwitchExhaustivenessCheck(tsgolint),
    typescript,
    pedantic,
    pending,
    config = SwitchExhaustivenessCheckConfig,
);

impl Rule for SwitchExhaustivenessCheck {
    fn from_configuration(value: serde_json::Value) -> Self {
        serde_json::from_value::<DefaultRuleConfig<SwitchExhaustivenessCheck>>(value)
            .unwrap_or_default()
            .into_inner()
    }

    fn to_configuration(&self) -> Option<Result<serde_json::Value, serde_json::Error>> {
        Some(serde_json::to_value(&*self.0))
    }
}
