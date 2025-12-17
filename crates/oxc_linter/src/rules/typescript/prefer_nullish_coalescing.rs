use oxc_macros::declare_oxc_lint;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::rule::{DefaultRuleConfig, Rule};

#[derive(Debug, Default, Clone, Deserialize)]
pub struct PreferNullishCoalescing(Box<PreferNullishCoalescingConfig>);

/// Options for ignoring specific primitive types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct IgnorePrimitivesOptions {
    /// Ignore bigint primitive types.
    pub bigint: bool,
    /// Ignore boolean primitive types.
    pub boolean: bool,
    /// Ignore number primitive types.
    pub number: bool,
    /// Ignore string primitive types.
    pub string: bool,
}

/// Represents the different ways `ignorePrimitives` can be specified in JSON.
/// Can be:
/// - `true` - ignore all primitive types
/// - An object specifying which primitives to ignore
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum IgnorePrimitives {
    /// `"ignorePrimitives": true` - ignore all primitive types
    All(bool),
    /// `"ignorePrimitives": { "string": true, ... }` - ignore specific primitives
    Options(IgnorePrimitivesOptions),
}

impl Default for IgnorePrimitives {
    fn default() -> Self {
        Self::All(false)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct PreferNullishCoalescingConfig {
    /// Unless this is set to `true`, the rule will error on every file whose
    /// `tsconfig.json` does _not_ have the `strictNullChecks` compiler option
    /// (or `strict`) set to `true`.
    ///
    /// It is _not_ recommended to enable this config option.
    pub allow_rule_to_run_without_strict_null_checks_i_know_what_i_am_doing: bool,
    /// Whether to ignore arguments to the `Boolean` constructor.
    pub ignore_boolean_coercion: bool,
    /// Whether to ignore cases that are located within a conditional test.
    pub ignore_conditional_tests: bool,
    /// Whether to ignore any if statements that could be simplified by using
    /// the nullish coalescing operator.
    pub ignore_if_statements: bool,
    /// Whether to ignore any logical or expressions that are part of a mixed
    /// logical expression (with `&&`).
    pub ignore_mixed_logical_expressions: bool,
    /// Whether to ignore any ternary expressions that could be simplified by
    /// using the nullish coalescing operator.
    pub ignore_ternary_tests: bool,
    /// Whether to ignore all (`true`) or some (an object with properties) primitive types.
    pub ignore_primitives: IgnorePrimitives,
}

impl Default for PreferNullishCoalescingConfig {
    fn default() -> Self {
        Self {
            allow_rule_to_run_without_strict_null_checks_i_know_what_i_am_doing: false,
            ignore_boolean_coercion: false,
            ignore_conditional_tests: true,
            ignore_if_statements: false,
            ignore_mixed_logical_expressions: false,
            ignore_ternary_tests: false,
            ignore_primitives: IgnorePrimitives::default(),
        }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce using the nullish coalescing operator (`??`) instead of logical OR (`||`)
    /// or conditional expressions when the left operand might be `null` or `undefined`.
    ///
    /// ### Why is this bad?
    ///
    /// The `||` operator returns the right-hand side when the left-hand side is any
    /// falsy value (`false`, `0`, `''`, `null`, `undefined`, `NaN`). This can lead
    /// to unexpected behavior when you only want to provide a default for `null`
    /// or `undefined`.
    ///
    /// The nullish coalescing operator (`??`) only returns the right-hand side when
    /// the left-hand side is `null` or `undefined`, making the intent clearer and
    /// avoiding bugs with other falsy values.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// declare const x: string | null;
    ///
    /// // Using || when ?? would be more appropriate
    /// const foo = x || 'default';
    ///
    /// // Ternary that could use ??
    /// const bar = x !== null && x !== undefined ? x : 'default';
    /// const baz = x != null ? x : 'default';
    ///
    /// // If statement that could use ??
    /// let value = 'default';
    /// if (x !== null && x !== undefined) {
    ///   value = x;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// declare const x: string | null;
    ///
    /// // Using nullish coalescing
    /// const foo = x ?? 'default';
    ///
    /// // || is fine when you want falsy behavior
    /// declare const y: string;
    /// const bar = y || 'default';
    ///
    /// // Boolean coercion (can be ignored with ignoreBooleanCoercion)
    /// const bool = Boolean(x || y);
    /// ```
    PreferNullishCoalescing(tsgolint),
    typescript,
    pedantic,
    pending,
    config = PreferNullishCoalescingConfig,
);

impl Rule for PreferNullishCoalescing {
    fn from_configuration(value: serde_json::Value) -> Self {
        serde_json::from_value::<DefaultRuleConfig<PreferNullishCoalescing>>(value)
            .unwrap_or_default()
            .into_inner()
    }

    fn to_configuration(&self) -> Option<Result<serde_json::Value, serde_json::Error>> {
        Some(serde_json::to_value(&*self.0))
    }
}
