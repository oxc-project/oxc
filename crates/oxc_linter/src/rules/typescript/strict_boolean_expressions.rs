use oxc_macros::declare_oxc_lint;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    rule::{DefaultRuleConfig, Rule},
    utils::default_true,
};

#[derive(Debug, Default, Clone)]
pub struct StrictBooleanExpressions(Box<StrictBooleanExpressionsConfig>);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct StrictBooleanExpressionsConfig {
    /// Whether to allow `any` type in boolean contexts.
    pub allow_any: bool,
    /// Whether to allow nullable boolean types (e.g., `boolean | null`) in boolean contexts.
    pub allow_nullable_boolean: bool,
    /// Whether to allow nullable number types (e.g., `number | null`) in boolean contexts.
    pub allow_nullable_number: bool,
    /// Whether to allow nullable string types (e.g., `string | null`) in boolean contexts.
    pub allow_nullable_string: bool,
    /// Whether to allow nullable enum types in boolean contexts.
    pub allow_nullable_enum: bool,
    /// Whether to allow nullable object types in boolean contexts.
    #[serde(default = "default_true")]
    pub allow_nullable_object: bool,
    /// Whether to allow string types in boolean contexts (checks for non-empty strings).
    #[serde(default = "default_true")]
    pub allow_string: bool,
    /// Whether to allow number types in boolean contexts (checks for non-zero numbers).
    #[serde(default = "default_true")]
    pub allow_number: bool,
    /// Whether to allow this rule to run without `strictNullChecks` enabled.
    /// This is not recommended as the rule may produce incorrect results.
    pub allow_rule_to_run_without_strict_null_checks_i_know_what_i_am_doing: bool,
}

impl Default for StrictBooleanExpressionsConfig {
    fn default() -> Self {
        Self {
            allow_any: false,
            allow_nullable_boolean: false,
            allow_nullable_number: false,
            allow_nullable_string: false,
            allow_nullable_enum: false,
            allow_nullable_object: true,
            allow_string: true,
            allow_number: true,
            allow_rule_to_run_without_strict_null_checks_i_know_what_i_am_doing: false,
        }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow certain types in boolean expressions.
    ///
    /// ### Why is this bad?
    ///
    /// Forbids usage of non-boolean types in expressions where a boolean is expected.
    /// `boolean` and `never` types are always allowed. Additional types which are
    /// considered safe in a boolean context can be configured via options.
    ///
    /// The following nodes are checked:
    ///
    /// - Arguments to the `!`, `&&`, and `||` operators
    /// - The condition in a conditional expression (`cond ? x : y`)
    /// - Conditions for `if`, `for`, `while`, and `do-while` statements.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// const str = 'hello';
    /// if (str) {
    ///   console.log('string');
    /// }
    ///
    /// const num = 42;
    /// if (num) {
    ///   console.log('number');
    /// }
    ///
    /// const obj = { foo: 'bar' };
    /// if (obj) {
    ///   console.log('object');
    /// }
    ///
    /// declare const maybeString: string | undefined;
    /// if (maybeString) {
    ///   console.log(maybeString);
    /// }
    ///
    /// const result = str && num;
    /// const ternary = str ? 'yes' : 'no';
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// const str = 'hello';
    /// if (str !== '') {
    ///   console.log('string');
    /// }
    ///
    /// const num = 42;
    /// if (num !== 0) {
    ///   console.log('number');
    /// }
    ///
    /// const obj = { foo: 'bar' };
    /// if (obj !== null) {
    ///   console.log('object');
    /// }
    ///
    /// declare const maybeString: string | undefined;
    /// if (maybeString !== undefined) {
    ///   console.log(maybeString);
    /// }
    ///
    /// const bool = true;
    /// if (bool) {
    ///   console.log('boolean');
    /// }
    /// ```
    StrictBooleanExpressions(tsgolint),
    typescript,
    pedantic,
    pending,
    config = StrictBooleanExpressionsConfig,
);

impl Rule for StrictBooleanExpressions {
    fn from_configuration(value: serde_json::Value) -> Self {
        Self(Box::new(
            serde_json::from_value::<DefaultRuleConfig<StrictBooleanExpressionsConfig>>(value)
                .unwrap_or_default()
                .into_inner(),
        ))
    }

    fn to_configuration(&self) -> Option<Result<serde_json::Value, serde_json::Error>> {
        Some(serde_json::to_value(&*self.0))
    }
}
