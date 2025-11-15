use oxc_macros::declare_oxc_lint;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    rule::{DefaultRuleConfig, Rule},
    utils::default_true,
};

#[derive(Debug, Default, Clone)]
pub struct PromiseFunctionAsync(Box<PromiseFunctionAsyncConfig>);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct PromiseFunctionAsyncConfig {
    /// Whether to allow functions returning `any` type without requiring `async`.
    #[serde(default = "default_true")]
    pub allow_any: bool,
    /// A list of Promise type names that are allowed without requiring `async`.
    /// Example: `["SpecialPromise"]` to allow functions returning `SpecialPromise` without `async`.
    pub allowed_promise_names: Vec<String>,
    /// Whether to check arrow functions for missing `async` keyword.
    #[serde(default = "default_true")]
    pub check_arrow_functions: bool,
    /// Whether to check function declarations for missing `async` keyword.
    #[serde(default = "default_true")]
    pub check_function_declarations: bool,
    /// Whether to check function expressions for missing `async` keyword.
    #[serde(default = "default_true")]
    pub check_function_expressions: bool,
    /// Whether to check method declarations for missing `async` keyword.
    #[serde(default = "default_true")]
    pub check_method_declarations: bool,
}

impl Default for PromiseFunctionAsyncConfig {
    fn default() -> Self {
        Self {
            allow_any: true,
            allowed_promise_names: Vec::new(),
            check_arrow_functions: true,
            check_function_declarations: true,
            check_function_expressions: true,
            check_method_declarations: true,
        }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule requires any function or method that returns a Promise to be marked as async.
    ///
    /// ### Why is this bad?
    ///
    /// Functions that return Promises should typically be marked as `async` to make their asynchronous nature clear and to enable the use of `await` within them. This makes the code more readable and helps prevent common mistakes with Promise handling.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// // Function returning Promise without async
    /// function fetchData(): Promise<string> {
    ///   return fetch('/api/data').then(res => res.text());
    /// }
    ///
    /// // Method returning Promise without async
    /// class DataService {
    ///   getData(): Promise<any> {
    ///     return fetch('/api/data').then(res => res.json());
    ///   }
    /// }
    ///
    /// // Arrow function returning Promise without async
    /// const processData = (): Promise<void> => {
    ///   return Promise.resolve();
    /// };
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// // Async function
    /// async function fetchData(): Promise<string> {
    ///   const response = await fetch('/api/data');
    ///   return response.text();
    /// }
    ///
    /// // Async method
    /// class DataService {
    ///   async getData(): Promise<any> {
    ///     const response = await fetch('/api/data');
    ///     return response.json();
    ///   }
    /// }
    ///
    /// // Async arrow function
    /// const processData = async (): Promise<void> => {
    ///   await someAsyncOperation();
    /// };
    ///
    /// // Functions that don't return Promise are fine
    /// function syncFunction(): string {
    ///   return 'hello';
    /// }
    ///
    /// // Functions returning Promise-like but not actual Promise
    /// function createThenable(): { then: Function } {
    ///   return { then: () => {} };
    /// }
    /// ```
    PromiseFunctionAsync(tsgolint),
    typescript,
    restriction,
    pending,
    config = PromiseFunctionAsyncConfig,
);

impl Rule for PromiseFunctionAsync {
    fn from_configuration(value: serde_json::Value) -> Self {
        Self(Box::new(
            serde_json::from_value::<DefaultRuleConfig<PromiseFunctionAsyncConfig>>(value)
                .unwrap_or_default()
                .into_inner(),
        ))
    }

    fn to_configuration(&self) -> Option<Result<serde_json::Value, serde_json::Error>> {
        Some(serde_json::to_value(&*self.0))
    }
}
