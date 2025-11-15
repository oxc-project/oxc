use oxc_macros::declare_oxc_lint;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::rule::{DefaultRuleConfig, Rule};

#[derive(Debug, Clone)]
pub struct ReturnAwait(Box<ReturnAwaitOption>);

impl Default for ReturnAwait {
    fn default() -> Self {
        Self(Box::new(ReturnAwaitOption::InTryCatch))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ReturnAwaitOption {
    /// Require `await` when returning Promises inside try/catch/finally blocks.
    /// This ensures proper error handling and stack traces.
    #[default]
    InTryCatch,
    /// Require `await` before returning Promises in all cases.
    /// Example: `return await Promise.resolve()` is required.
    Always,
    /// Require `await` only when it affects error handling correctness.
    /// Only flags cases where omitting await would change error handling behavior.
    ErrorHandlingCorrectnessOnly,
    /// Disallow `await` before returning Promises in all cases.
    /// Example: `return Promise.resolve()` is required (no await).
    Never,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule enforces consistent returning of awaited values from async functions.
    ///
    /// ### Why is this bad?
    ///
    /// There are different patterns for returning awaited values from async functions. Sometimes you want to await before returning (to handle errors in the current function), and sometimes you want to return the Promise directly (for better performance). This rule helps enforce consistency.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule (depending on configuration):
    /// ```ts
    /// // If configured to require await:
    /// async function fetchData() {
    ///   return fetch('/api/data'); // Should be: return await fetch('/api/data');
    /// }
    ///
    /// async function processData() {
    ///   return someAsyncOperation(); // Should be: return await someAsyncOperation();
    /// }
    ///
    /// // If configured to disallow unnecessary await:
    /// async function fetchData() {
    ///   return await fetch('/api/data'); // Should be: return fetch('/api/data');
    /// }
    ///
    /// async function processData() {
    ///   return await someAsyncOperation(); // Should be: return someAsyncOperation();
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// // When await is required for error handling:
    /// async function fetchData() {
    ///   try {
    ///     return await fetch('/api/data');
    ///   } catch (error) {
    ///     console.error('Fetch failed:', error);
    ///     throw error;
    ///   }
    /// }
    ///
    /// // When returning Promise directly for performance:
    /// async function fetchData() {
    ///   return fetch('/api/data');
    /// }
    ///
    /// // Processing before return requires await:
    /// async function fetchAndProcess() {
    ///   const response = await fetch('/api/data');
    ///   return response.json();
    /// }
    ///
    /// // Multiple async operations:
    /// async function multipleOperations() {
    ///   const data1 = await fetchData1();
    ///   const data2 = await fetchData2();
    ///   return data1 + data2;
    /// }
    /// ```
    ReturnAwait(tsgolint),
    typescript,
    pedantic,
    pending,
    config = ReturnAwaitOption,
);

impl Rule for ReturnAwait {
    fn from_configuration(value: serde_json::Value) -> Self {
        Self(Box::new(
            serde_json::from_value::<DefaultRuleConfig<ReturnAwaitOption>>(value)
                .unwrap_or_default()
                .into_inner(),
        ))
    }

    fn to_configuration(&self) -> Option<Result<serde_json::Value, serde_json::Error>> {
        Some(serde_json::to_value(&*self.0))
    }
}
