use oxc_macros::declare_oxc_lint;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::rule::{DefaultRuleConfig, Rule};

#[derive(Debug, Default, Clone)]
pub struct NoConfusingVoidExpression(Box<NoConfusingVoidExpressionConfig>);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct NoConfusingVoidExpressionConfig {
    /// Whether to ignore arrow function shorthand that returns void.
    /// When true, allows expressions like `() => someVoidFunction()`.
    pub ignore_arrow_shorthand: bool,
    /// Whether to ignore expressions using the void operator.
    /// When true, allows `void someExpression`.
    pub ignore_void_operator: bool,
    /// Whether to ignore calling functions that are declared to return void.
    /// When true, allows expressions like `x = voidReturningFunction()`.
    pub ignore_void_returning_functions: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule forbids using void expressions in confusing locations such as arrow function returns.
    ///
    /// ### Why is this bad?
    ///
    /// The void operator is useful when you want to execute an expression while evaluating to undefined. However, it can be confusing when used in places where the return value is meaningful, particularly in arrow functions and conditional expressions.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// // arrow function returning void expression
    /// const foo = () => void bar();
    ///
    /// // conditional expression
    /// const result = condition ? void foo() : bar();
    ///
    /// // void in conditional
    /// if (void foo()) {
    ///   // ...
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// // proper use of void
    /// void foo();
    ///
    /// // explicit return statement
    /// const foo = () => {
    ///   bar();
    ///   return;
    /// };
    ///
    /// // statement expression
    /// foo();
    ///
    /// // IIFE with void
    /// void (function() {
    ///   console.log('immediately invoked');
    /// })();
    /// ```
    NoConfusingVoidExpression(tsgolint),
    typescript,
    pedantic,
    pending,
    config = NoConfusingVoidExpressionConfig,
);

impl Rule for NoConfusingVoidExpression {
    fn from_configuration(value: serde_json::Value) -> Self {
        Self(Box::new(
            serde_json::from_value::<DefaultRuleConfig<NoConfusingVoidExpressionConfig>>(value)
                .unwrap_or_default()
                .into_inner(),
        ))
    }

    fn to_configuration(&self) -> Option<Result<serde_json::Value, serde_json::Error>> {
        Some(serde_json::to_value(&*self.0))
    }
}
