use oxc_macros::declare_oxc_lint;

use crate::rule::{DefaultRuleConfig, Rule};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Deserialize)]
pub struct NoMeaninglessVoidOperator(Box<NoMeaninglessVoidOperatorConfig>);

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct NoMeaninglessVoidOperatorConfig {
    /// Whether to check `void` applied to expressions of type `never`.
    pub check_never: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule disallows the void operator when its argument is already of type void or undefined.
    ///
    /// ### Why is this bad?
    ///
    /// The void operator is useful when you want to execute an expression and force it to evaluate to undefined. However, using void on expressions that are already of type void or undefined is meaningless and adds unnecessary complexity to the code.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// function foo(): void {
    ///   return;
    /// }
    ///
    /// void foo(); // meaningless, foo() already returns void
    ///
    /// void undefined; // meaningless, undefined is already undefined
    ///
    /// async function bar() {
    ///   void (await somePromise); // meaningless if somePromise resolves to void
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// function getValue(): number {
    ///   return 42;
    /// }
    ///
    /// void getValue(); // meaningful, converts number to void
    ///
    /// void console.log('hello'); // meaningful, console.log returns undefined but we want to be explicit
    ///
    /// function processData() {
    ///   // some processing
    /// }
    ///
    /// processData(); // no void needed since we don't care about return value
    /// ```
    NoMeaninglessVoidOperator(tsgolint),
    typescript,
    correctness,
    pending,
    config = NoMeaninglessVoidOperatorConfig,
);

impl Rule for NoMeaninglessVoidOperator {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        Ok(serde_json::from_value::<DefaultRuleConfig<Self>>(value)
            .unwrap_or_default()
            .into_inner())
    }

    fn to_configuration(&self) -> Option<Result<serde_json::Value, serde_json::Error>> {
        Some(serde_json::to_value(&*self.0))
    }
}
