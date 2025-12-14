use oxc_macros::declare_oxc_lint;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::rule::{DefaultRuleConfig, Rule};

#[derive(Debug, Default, Clone, Deserialize)]
pub struct PreferPromiseRejectErrors(Box<PreferPromiseRejectErrorsConfig>);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct PreferPromiseRejectErrorsConfig {
    /// Whether to allow calling `Promise.reject()` with no arguments.
    pub allow_empty_reject: bool,
    /// Whether to allow rejecting Promises with values typed as `any`.
    pub allow_throwing_any: bool,
    /// Whether to allow rejecting Promises with values typed as `unknown`.
    pub allow_throwing_unknown: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule enforces passing an Error object to Promise.reject().
    ///
    /// ### Why is this bad?
    ///
    /// It's considered good practice to only reject promises with Error objects. This is because Error objects automatically capture a stack trace, which is useful for debugging. Additionally, some tools and environments expect rejection reasons to be Error objects.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// Promise.reject('error'); // rejecting with string
    ///
    /// Promise.reject(42); // rejecting with number
    ///
    /// Promise.reject(true); // rejecting with boolean
    ///
    /// Promise.reject({ message: 'error' }); // rejecting with plain object
    ///
    /// Promise.reject(null); // rejecting with null
    ///
    /// Promise.reject(); // rejecting with undefined
    ///
    /// const error = 'Something went wrong';
    /// Promise.reject(error); // rejecting with non-Error variable
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// Promise.reject(new Error('Something went wrong'));
    ///
    /// Promise.reject(new TypeError('Invalid type'));
    ///
    /// Promise.reject(new RangeError('Value out of range'));
    ///
    /// // Custom Error subclasses
    /// class CustomError extends Error {
    ///   constructor(message: string) {
    ///     super(message);
    ///     this.name = 'CustomError';
    ///   }
    /// }
    /// Promise.reject(new CustomError('Custom error occurred'));
    ///
    /// // Variables that are Error objects
    /// const error = new Error('Error message');
    /// Promise.reject(error);
    /// ```
    PreferPromiseRejectErrors(tsgolint),
    typescript,
    pedantic,
    pending,
    config = PreferPromiseRejectErrorsConfig,
);

impl Rule for PreferPromiseRejectErrors {
    fn from_configuration(value: serde_json::Value) -> Self {
        serde_json::from_value::<DefaultRuleConfig<PreferPromiseRejectErrors>>(value)
            .unwrap_or_default()
            .into_inner()
    }

    fn to_configuration(&self) -> Option<Result<serde_json::Value, serde_json::Error>> {
        Some(serde_json::to_value(&*self.0))
    }
}
