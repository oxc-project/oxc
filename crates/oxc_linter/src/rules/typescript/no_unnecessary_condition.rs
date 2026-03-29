use oxc_macros::declare_oxc_lint;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::rule::{DefaultRuleConfig, Rule};

#[derive(Debug, Default, Clone, Deserialize)]
pub struct NoUnnecessaryCondition(Box<NoUnnecessaryConditionConfig>);

/// Represents the different ways `allowConstantLoopConditions` can be specified in JSON.
/// Can be:
/// - `true` or `false`
/// - A string enum (`"never"`, `"always"`, `"only-allowed-literals"`)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum AllowConstantLoopConditions {
    Boolean(bool),
    Mode(AllowConstantLoopConditionsMode),
}

impl Default for AllowConstantLoopConditions {
    fn default() -> Self {
        Self::Mode(AllowConstantLoopConditionsMode::Never)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum AllowConstantLoopConditionsMode {
    Never,
    Always,
    OnlyAllowedLiterals,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct NoUnnecessaryConditionConfig {
    /// Whether to allow constant loop conditions.
    /// `true` is treated as `"always"`, `false` as `"never"`.
    pub allow_constant_loop_conditions: AllowConstantLoopConditions,
    /// Whether to check type predicate functions.
    pub check_type_predicates: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow conditions that are always truthy, always falsy, or always nullish
    /// based on TypeScript's type information.
    ///
    /// ### Why is this bad?
    ///
    /// Conditions with no possible runtime variation make code harder to read and can
    /// hide logic errors. They often leave dead branches and suggest that the declared
    /// types do not match the intended behavior.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// declare const value: null;
    /// if (value) {
    ///   doWork();
    /// }
    ///
    /// const items: string[] = [];
    /// if (items) {
    ///   doWork();
    /// }
    ///
    /// declare const status: "ready";
    /// if (!status) {
    ///   reportError();
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// declare const maybeUser: User | undefined;
    /// if (maybeUser) {
    ///   doWork(maybeUser);
    /// }
    ///
    /// const items: string[] = [];
    /// if (items.length > 0) {
    ///   doWork();
    /// }
    ///
    /// declare const status: "ready" | "";
    /// if (!status) {
    ///   reportError();
    /// }
    /// ```
    NoUnnecessaryCondition(tsgolint),
    typescript,
    nursery, // TODO(camc314): move to correctness
    config = NoUnnecessaryConditionConfig,
);

impl Rule for NoUnnecessaryCondition {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn to_configuration(&self) -> Option<Result<serde_json::Value, serde_json::Error>> {
        Some(serde_json::to_value(&*self.0))
    }
}
