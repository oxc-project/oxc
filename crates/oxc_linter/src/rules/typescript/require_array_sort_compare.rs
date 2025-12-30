use oxc_macros::declare_oxc_lint;

use crate::rule::{DefaultRuleConfig, Rule};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Deserialize)]
pub struct RequireArraySortCompare(Box<RequireArraySortCompareConfig>);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct RequireArraySortCompareConfig {
    /// Whether to ignore arrays in which all elements are strings.
    pub ignore_string_arrays: bool,
}

impl Default for RequireArraySortCompareConfig {
    fn default() -> Self {
        Self { ignore_string_arrays: true }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule requires Array.sort() to be called with a comparison function.
    ///
    /// ### Why is this bad?
    ///
    /// When Array.sort() is called without a comparison function, it converts elements to strings and sorts them lexicographically. This often leads to unexpected results, especially with numbers where `[1, 10, 2].sort()` returns `[1, 10, 2]` instead of `[1, 2, 10]`.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// const numbers = [3, 1, 4, 1, 5];
    /// numbers.sort(); // Lexicographic sort, not numeric
    ///
    /// const mixedArray = ['10', '2', '1'];
    /// mixedArray.sort(); // Might be intended, but explicit compareFn is clearer
    ///
    /// [3, 1, 4].sort(); // Will sort as strings: ['1', '3', '4']
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// const numbers = [3, 1, 4, 1, 5];
    ///
    /// // Numeric sort
    /// numbers.sort((a, b) => a - b);
    ///
    /// // Reverse numeric sort
    /// numbers.sort((a, b) => b - a);
    ///
    /// // String sort (explicit)
    /// const strings = ['banana', 'apple', 'cherry'];
    /// strings.sort((a, b) => a.localeCompare(b));
    ///
    /// // Custom object sorting
    /// interface Person {
    ///   name: string;
    ///   age: number;
    /// }
    ///
    /// const people: Person[] = [
    ///   { name: 'Alice', age: 30 },
    ///   { name: 'Bob', age: 25 },
    /// ];
    ///
    /// people.sort((a, b) => a.age - b.age);
    /// people.sort((a, b) => a.name.localeCompare(b.name));
    /// ```
    RequireArraySortCompare(tsgolint),
    typescript,
    correctness,
    pending,
    config = RequireArraySortCompareConfig,
);

impl Rule for RequireArraySortCompare {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        Ok(serde_json::from_value::<DefaultRuleConfig<Self>>(value)
            .unwrap_or_default()
            .into_inner())
    }

    fn to_configuration(&self) -> Option<Result<serde_json::Value, serde_json::Error>> {
        Some(serde_json::to_value(&*self.0))
    }
}
