use oxc_macros::declare_oxc_lint;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::rule::{DefaultRuleConfig, Rule};

#[derive(Debug, Default, Clone, Deserialize)]
pub struct NoUnsafeMemberAccess(Box<NoUnsafeMemberAccessConfig>);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct NoUnsafeMemberAccessConfig {
    /// Whether to allow `?.` optional chains on `any` values.
    /// When `true`, optional chaining on `any` values will not be flagged.
    /// Default is `false`.
    pub allow_optional_chaining: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule disallows member access on a value with type `any`.
    ///
    /// ### Why is this bad?
    ///
    /// The `any` type in TypeScript disables type checking. When you access a member (property or method) on a value typed as `any`, TypeScript cannot verify that the member exists or what type it has. This can lead to runtime errors.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// declare const anyValue: any;
    ///
    /// anyValue.foo; // unsafe member access
    ///
    /// anyValue.bar.baz; // unsafe nested member access
    ///
    /// anyValue['key']; // unsafe computed member access
    ///
    /// const result = anyValue.method(); // unsafe method access
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// declare const obj: { foo: string; bar: { baz: number } };
    /// declare const unknownValue: unknown;
    ///
    /// obj.foo; // safe
    ///
    /// obj.bar.baz; // safe
    ///
    /// obj['foo']; // safe
    ///
    /// // Type guard for unknown
    /// if (typeof unknownValue === 'object' && unknownValue !== null && 'foo' in unknownValue) {
    ///   console.log(unknownValue.foo); // safe after type guard
    /// }
    ///
    /// // Explicit type assertion if needed
    /// (anyValue as { foo: string }).foo; // explicitly unsafe but intentional
    /// ```
    NoUnsafeMemberAccess(tsgolint),
    typescript,
    pedantic,
    pending,
    config = NoUnsafeMemberAccessConfig,
);

impl Rule for NoUnsafeMemberAccess {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        Ok(serde_json::from_value::<DefaultRuleConfig<Self>>(value)
            .unwrap_or_default()
            .into_inner())
    }

    fn to_configuration(&self) -> Option<Result<serde_json::Value, serde_json::Error>> {
        Some(serde_json::to_value(&*self.0))
    }
}
