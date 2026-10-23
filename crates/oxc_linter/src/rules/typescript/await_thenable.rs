use oxc_macros::declare_oxc_lint;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::rule::{DefaultRuleConfig, Rule};

#[derive(Debug, Default, Clone, Deserialize)]
pub struct AwaitThenable(Box<AwaitThenableConfig>);

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct AwaitThenableConfig {
    /// Allow Promise aggregators such as `Promise.all` to receive mixed arrays or
    /// iterables containing at least one potentially Promise-like value.
    /// Inputs containing only synchronous values are still reported.
    pub allow_mixed_promise_arrays: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule disallows awaiting a value that is not a Thenable.
    ///
    /// ### Why is this bad?
    ///
    /// While it is valid JavaScript to await a non-Promise-like value (it will resolve immediately), this practice can be confusing for readers who are not aware of this behavior. It can also be a sign of a programmer error, such as forgetting to add parentheses to call a function that returns a Promise.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```
    /// await 12;
    /// await (() => {});
    ///
    /// // non-Promise values
    /// await Math.random;
    /// await { then() {} };
    ///
    /// // this is not a Promise - it's a function that returns a Promise
    /// declare const getPromise: () => Promise<string>;
    /// await getPromise;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```
    /// await Promise.resolve('value');
    /// await Promise.reject(new Error());
    ///
    /// // Promise-like values
    /// await {
    ///   then(onfulfilled, onrejected) {
    ///     onfulfilled('value');
    ///   },
    /// };
    ///
    /// // this is a Promise - produced by calling a function
    /// declare const getPromise: () => Promise<string>;
    /// await getPromise();
    /// ```
    AwaitThenable(tsgolint),
    typescript,
    correctness,
    recommended,
    suggestion,
    config = AwaitThenableConfig,
    version = "1.12.0",
    short_description = "This rule disallows awaiting a value that is not a Thenable.",
);

impl Rule for AwaitThenable {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        DefaultRuleConfig::<Self>::from_value(value).map(DefaultRuleConfig::into_inner)
    }

    fn to_configuration(&self) -> Option<Result<serde_json::Value, serde_json::Error>> {
        Some(serde_json::to_value(&*self.0))
    }
}
