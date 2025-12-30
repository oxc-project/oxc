use oxc_macros::declare_oxc_lint;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    rule::{DefaultRuleConfig, Rule},
    utils::TypeOrValueSpecifier,
};

#[derive(Debug, Default, Clone, Deserialize)]
pub struct NoMisusedSpread(Box<NoMisusedSpreadConfig>);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct NoMisusedSpreadConfig {
    /// An array of type or value specifiers that are allowed to be spread
    /// even if they would normally be flagged as misused.
    pub allow: Vec<TypeOrValueSpecifier>,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule disallows spreading syntax in places where it doesn't make sense or could cause runtime errors.
    ///
    /// ### Why is this bad?
    ///
    /// The spread operator can be misused in ways that might not be immediately obvious but can cause runtime errors or unexpected behavior. This rule helps catch common misuses.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// // Spreading a non-iterable value in an array
    /// const num = 42;
    /// const arr = [...num]; // Runtime error: num is not iterable
    ///
    /// // Spreading a Promise in an array
    /// const promise = Promise.resolve([1, 2, 3]);
    /// const arr2 = [...promise]; // Runtime error: Promise is not iterable
    ///
    /// // Spreading non-object in object literal
    /// const str = 'hello';
    /// const obj = { ...str }; // Creates { '0': 'h', '1': 'e', ... } which might be unexpected
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// // Spreading arrays
    /// const arr1 = [1, 2, 3];
    /// const arr2 = [...arr1];
    ///
    /// // Spreading objects
    /// const obj1 = { a: 1, b: 2 };
    /// const obj2 = { ...obj1 };
    ///
    /// // Spreading resolved Promise
    /// const promise = Promise.resolve([1, 2, 3]);
    /// const arr3 = [...(await promise)];
    ///
    /// // Using Array.from for non-iterables if needed
    /// const str = 'hello';
    /// const arr4 = Array.from(str); // ['h', 'e', 'l', 'l', 'o']
    /// ```
    NoMisusedSpread(tsgolint),
    typescript,
    correctness,
    pending,
    config = NoMisusedSpreadConfig,
);

impl Rule for NoMisusedSpread {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        Ok(serde_json::from_value::<DefaultRuleConfig<Self>>(value)
            .unwrap_or_default()
            .into_inner())
    }

    fn to_configuration(&self) -> Option<Result<serde_json::Value, serde_json::Error>> {
        Some(serde_json::to_value(&*self.0))
    }
}
