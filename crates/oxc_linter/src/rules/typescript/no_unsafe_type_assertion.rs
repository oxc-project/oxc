use oxc_macros::declare_oxc_lint;

use crate::rule::Rule;

#[derive(Debug, Default, Clone)]
pub struct NoUnsafeTypeAssertion;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule disallows type assertions using the `any` type.
    ///
    /// ### Why is this bad?
    ///
    /// Type assertions using `any` completely bypass TypeScript's type system and can lead to runtime errors. They should be avoided in favor of more specific type assertions or proper type guards.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// declare const value: unknown;
    ///
    /// const str = value as any; // unsafe type assertion
    ///
    /// const obj = value as any as string; // double assertion through any
    ///
    /// function processValue(input: unknown) {
    ///   const processed = input as any; // unsafe
    ///   return processed.someProperty;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// declare const value: unknown;
    ///
    /// // Use specific type assertions
    /// const str = value as string; // more specific assertion
    ///
    /// // Use type guards
    /// if (typeof value === 'string') {
    ///   const str2 = value; // safe, no assertion needed
    /// }
    ///
    /// // Use proper interface assertions
    /// interface User {
    ///   name: string;
    ///   age: number;
    /// }
    ///
    /// const user = value as User; // specific type assertion
    ///
    /// // Use unknown for truly unknown values
    /// const unknown: unknown = value;
    /// ```
    NoUnsafeTypeAssertion(tsgolint),
    typescript,
    suspicious,
    pending,
);

impl Rule for NoUnsafeTypeAssertion {}
