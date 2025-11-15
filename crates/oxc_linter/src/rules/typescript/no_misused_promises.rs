use oxc_macros::declare_oxc_lint;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    rule::{DefaultRuleConfig, Rule},
    utils::default_true,
};

fn default_checks_void_return() -> ChecksVoidReturn {
    ChecksVoidReturn::Boolean(true)
}

#[derive(Debug, Default, Clone)]
pub struct NoMisusedPromises(Box<NoMisusedPromisesConfig>);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
#[expect(clippy::struct_field_names)]
pub struct NoMisusedPromisesConfig {
    /// Whether to check if Promises are used in conditionals.
    /// When true, disallows using Promises in conditions where a boolean is expected.
    #[serde(default = "default_true")]
    pub checks_conditionals: bool,
    /// Whether to check if Promises are used in spread syntax.
    /// When true, disallows spreading Promise values.
    #[serde(default = "default_true")]
    pub checks_spreads: bool,
    /// Configuration for checking if Promises are returned in contexts expecting void.
    /// Can be a boolean to enable/disable all checks, or an object for granular control.
    #[serde(default = "default_checks_void_return")]
    pub checks_void_return: ChecksVoidReturn,
}

impl Default for NoMisusedPromisesConfig {
    fn default() -> Self {
        Self {
            checks_conditionals: true,
            checks_spreads: true,
            checks_void_return: ChecksVoidReturn::Boolean(true),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum ChecksVoidReturn {
    Boolean(bool),
    Options(ChecksVoidReturnOptions),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct ChecksVoidReturnOptions {
    /// Whether to check Promise-returning functions passed as arguments to void-returning functions.
    #[serde(default = "default_true")]
    pub arguments: bool,
    /// Whether to check Promise-returning functions in JSX attributes expecting void.
    #[serde(default = "default_true")]
    pub attributes: bool,
    /// Whether to check Promise-returning methods that override void-returning inherited methods.
    #[serde(default = "default_true")]
    pub inherited_methods: bool,
    /// Whether to check Promise-returning functions assigned to object properties expecting void.
    #[serde(default = "default_true")]
    pub properties: bool,
    /// Whether to check Promise values returned from void-returning functions.
    #[serde(default = "default_true")]
    pub returns: bool,
    /// Whether to check Promise-returning functions assigned to variables typed as void-returning.
    #[serde(default = "default_true")]
    pub variables: bool,
}

impl Default for ChecksVoidReturnOptions {
    fn default() -> Self {
        Self {
            arguments: true,
            attributes: true,
            inherited_methods: true,
            properties: true,
            returns: true,
            variables: true,
        }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule forbids providing Promises to logical locations such as if statements in places where the TypeScript compiler allows them but they are not handled properly. These situations can often arise due to a missing await keyword or just a misunderstanding of the way async functions are handled/awaited.
    ///
    /// ### Why is this bad?
    ///
    /// Misused promises can cause crashes or other unexpected behavior, unless there are possibly some global unhandled promise handlers registered.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// // Promises in conditionals:
    /// const promise = Promise.resolve('value');
    /// if (promise) {
    ///   // Do something
    /// }
    ///
    /// // Promises where `void` return was expected:
    /// [1, 2, 3].forEach(async value => {
    ///   await fetch(`/${value}`);
    /// });
    ///
    /// // Spreading Promises:
    /// const getData = () => fetch('/');
    /// console.log({ foo: 42, ...getData() });
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// // Awaiting the Promise to get its value in a conditional:
    /// const promise = Promise.resolve('value');
    /// if (await promise) {
    ///   // Do something
    /// }
    ///
    /// // Using a `for-of` with `await` inside (instead of `forEach`):
    /// for (const value of [1, 2, 3]) {
    ///   await fetch(`/${value}`);
    /// }
    ///
    /// // Spreading data returned from Promise, instead of the Promise itself:
    /// const getData = () => fetch('/');
    /// console.log({ foo: 42, ...(await getData()) });
    /// ```
    NoMisusedPromises(tsgolint),
    typescript,
    pedantic,
    pending,
    config = NoMisusedPromisesConfig,
);

impl Rule for NoMisusedPromises {
    fn from_configuration(value: serde_json::Value) -> Self {
        Self(Box::new(
            serde_json::from_value::<DefaultRuleConfig<NoMisusedPromisesConfig>>(value)
                .unwrap_or_default()
                .into_inner(),
        ))
    }

    fn to_configuration(&self) -> Option<Result<serde_json::Value, serde_json::Error>> {
        Some(serde_json::to_value(&*self.0))
    }
}
