use oxc_macros::declare_oxc_lint;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    rule::{DefaultRuleConfig, Rule},
    utils::{TypeOrValueSpecifier, default_true},
};

#[derive(Debug, Default, Clone)]
pub struct OnlyThrowError(Box<OnlyThrowErrorConfig>);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct OnlyThrowErrorConfig {
    /// An array of type or value specifiers for additional types that are allowed to be thrown.
    /// Use this to allow throwing custom error types.
    pub allow: Vec<TypeOrValueSpecifier>,
    /// Whether to allow throwing values typed as `any`.
    #[serde(default = "default_true")]
    pub allow_throwing_any: bool,
    /// Whether to allow throwing values typed as `unknown`.
    #[serde(default = "default_true")]
    pub allow_throwing_unknown: bool,
}

impl Default for OnlyThrowErrorConfig {
    fn default() -> Self {
        Self { allow: Vec::new(), allow_throwing_any: true, allow_throwing_unknown: true }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule disallows throwing non-Error values.
    ///
    /// ### Why is this bad?
    ///
    /// It's considered good practice to only throw Error objects (or subclasses of Error). This is because Error objects automatically capture a stack trace, which is useful for debugging. Additionally, some tools and environments expect thrown values to be Error objects.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// throw 'error'; // throwing string
    ///
    /// throw 42; // throwing number
    ///
    /// throw true; // throwing boolean
    ///
    /// throw { message: 'error' }; // throwing plain object
    ///
    /// throw null; // throwing null
    ///
    /// throw undefined; // throwing undefined
    ///
    /// const error = 'Something went wrong';
    /// throw error; // throwing non-Error variable
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// throw new Error('Something went wrong');
    ///
    /// throw new TypeError('Invalid type');
    ///
    /// throw new RangeError('Value out of range');
    ///
    /// // Custom Error subclasses
    /// class CustomError extends Error {
    ///   constructor(message: string) {
    ///     super(message);
    ///     this.name = 'CustomError';
    ///   }
    /// }
    /// throw new CustomError('Custom error occurred');
    ///
    /// // Variables that are Error objects
    /// const error = new Error('Error message');
    /// throw error;
    /// ```
    OnlyThrowError(tsgolint),
    typescript,
    pedantic,
    pending,
    config = OnlyThrowErrorConfig,
);

impl Rule for OnlyThrowError {
    fn from_configuration(value: serde_json::Value) -> Self {
        Self(Box::new(
            serde_json::from_value::<DefaultRuleConfig<OnlyThrowErrorConfig>>(value)
                .unwrap_or_default()
                .into_inner(),
        ))
    }

    fn to_configuration(&self) -> Option<Result<serde_json::Value, serde_json::Error>> {
        Some(serde_json::to_value(&*self.0))
    }
}
