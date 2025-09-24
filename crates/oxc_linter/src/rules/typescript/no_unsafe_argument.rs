use oxc_macros::declare_oxc_lint;

use crate::rule::Rule;

#[derive(Debug, Default, Clone)]
pub struct NoUnsafeArgument;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule disallows calling a function with an argument which is typed as `any`.
    ///
    /// ### Why is this bad?
    ///
    /// The `any` type in TypeScript is a dangerous "escape hatch" from the type system. Using `any` disables most type checking rules and is generally unsafe. When you pass a value typed as `any` to a function, you lose type safety for that function call.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// declare const anyValue: any;
    ///
    /// function takesString(str: string): void {
    ///   console.log(str.length);
    /// }
    ///
    /// takesString(anyValue); // unsafe
    ///
    /// declare function takesNumber(num: number): number;
    /// const result = takesNumber(anyValue); // unsafe
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// declare const stringValue: string;
    /// declare const numberValue: number;
    /// declare const unknownValue: unknown;
    ///
    /// function takesString(str: string): void {
    ///   console.log(str.length);
    /// }
    ///
    /// takesString(stringValue); // safe
    ///
    /// // Type guard to safely use unknown
    /// if (typeof unknownValue === 'string') {
    ///   takesString(unknownValue); // safe after type guard
    /// }
    ///
    /// // Type assertion if you're sure about the type
    /// takesString(unknownValue as string); // explicitly unsafe, but intentional
    /// ```
    NoUnsafeArgument(tsgolint),
    typescript,
    pedantic,
    pending,
);

impl Rule for NoUnsafeArgument {}
