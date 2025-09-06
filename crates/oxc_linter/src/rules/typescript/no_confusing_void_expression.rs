use oxc_macros::declare_oxc_lint;

use crate::rule::Rule;

#[derive(Debug, Default, Clone)]
pub struct NoConfusingVoidExpression;

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
);

impl Rule for NoConfusingVoidExpression {}
