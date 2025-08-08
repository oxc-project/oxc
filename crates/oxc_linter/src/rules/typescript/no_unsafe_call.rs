use oxc_macros::declare_oxc_lint;

use crate::rule::Rule;

#[derive(Debug, Default, Clone)]
pub struct NoUnsafeCall;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule disallows calling a value with type `any`.
    ///
    /// ### Why is this bad?
    ///
    /// The `any` type in TypeScript disables type checking. When you call a value typed as `any`, TypeScript cannot verify that it's actually a function, what parameters it expects, or what it returns. This can lead to runtime errors.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// declare const anyValue: any;
    ///
    /// anyValue(); // unsafe call
    ///
    /// anyValue(1, 2, 3); // unsafe call
    ///
    /// const result = anyValue('hello'); // unsafe call
    ///
    /// // Chained unsafe calls
    /// anyValue().then().catch(); // unsafe
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// declare const fn: () => void;
    /// declare const fnWithParams: (a: number, b: string) => boolean;
    /// declare const unknownValue: unknown;
    ///
    /// fn(); // safe
    ///
    /// const result = fnWithParams(1, 'hello'); // safe
    ///
    /// // Type guard for unknown
    /// if (typeof unknownValue === 'function') {
    ///   unknownValue(); // safe after type guard
    /// }
    ///
    /// // Explicit type assertion if you're certain
    /// (anyValue as () => void)(); // explicitly unsafe but intentional
    /// ```
    NoUnsafeCall(tsgolint),
    typescript,
    pedantic,
    pending,
);

impl Rule for NoUnsafeCall {}
