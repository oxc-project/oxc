use oxc_macros::declare_oxc_lint;

use crate::rule::Rule;

#[derive(Debug, Default, Clone)]
pub struct PreferReduceTypeParameter;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule prefers using a type parameter for the accumulator in Array.reduce instead of casting.
    ///
    /// ### Why is this bad?
    ///
    /// Array.reduce can be called with a generic type parameter to specify the type of the accumulator. This is preferred over casting the result because it provides better type safety and is more explicit about the intended type.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// const numbers = [1, 2, 3];
    ///
    /// // Casting the result
    /// const sum = numbers.reduce((acc, val) => acc + val, 0) as number;
    ///
    /// // Using type assertion on accumulator
    /// const result = [1, 2, 3].reduce((acc: string[], curr) => {
    ///   acc.push(curr.toString());
    ///   return acc;
    /// }, [] as string[]);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// const numbers = [1, 2, 3];
    ///
    /// // Using type parameter
    /// const sum = numbers.reduce<number>((acc, val) => acc + val, 0);
    ///
    /// // Type parameter for complex types
    /// const result = [1, 2, 3].reduce<string[]>((acc, curr) => {
    ///   acc.push(curr.toString());
    ///   return acc;
    /// }, []);
    ///
    /// // When TypeScript can infer the type, no parameter needed
    /// const simpleSum = numbers.reduce((acc, val) => acc + val, 0);
    ///
    /// // Object accumulator with type parameter
    /// interface Count {
    ///   [key: string]: number;
    /// }
    ///
    /// const counts = ['a', 'b', 'a'].reduce<Count>((acc, item) => {
    ///   acc[item] = (acc[item] || 0) + 1;
    ///   return acc;
    /// }, {});
    /// ```
    PreferReduceTypeParameter(tsgolint),
    typescript,
    style,
    pending,
);

impl Rule for PreferReduceTypeParameter {}
