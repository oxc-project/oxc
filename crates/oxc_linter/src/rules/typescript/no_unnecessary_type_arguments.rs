use oxc_macros::declare_oxc_lint;

use crate::rule::Rule;

#[derive(Debug, Default, Clone)]
pub struct NoUnnecessaryTypeArguments;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule disallows type arguments that are identical to the default type parameter.
    ///
    /// ### Why is this bad?
    ///
    /// Explicit type arguments that are the same as their default values are unnecessary and add visual noise to the code. TypeScript will infer these types automatically.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// function identity<T = string>(arg: T): T {
    ///   return arg;
    /// }
    ///
    /// // Unnecessary type argument - string is the default
    /// const result = identity<string>('hello');
    ///
    /// interface Container<T = number> {
    ///   value: T;
    /// }
    ///
    /// // Unnecessary type argument - number is the default
    /// const container: Container<number> = { value: 42 };
    ///
    /// class MyClass<T = boolean> {
    ///   constructor(public value: T) {}
    /// }
    ///
    /// // Unnecessary type argument - boolean is the default
    /// const instance = new MyClass<boolean>(true);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// function identity<T = string>(arg: T): T {
    ///   return arg;
    /// }
    ///
    /// // Using default type
    /// const result1 = identity('hello');
    ///
    /// // Using different type
    /// const result2 = identity<number>(42);
    ///
    /// interface Container<T = number> {
    ///   value: T;
    /// }
    ///
    /// // Using default type
    /// const container1: Container = { value: 42 };
    ///
    /// // Using different type
    /// const container2: Container<string> = { value: 'hello' };
    /// ```
    NoUnnecessaryTypeArguments(tsgolint),
    typescript,
    suspicious,
    pending,
);

impl Rule for NoUnnecessaryTypeArguments {}
