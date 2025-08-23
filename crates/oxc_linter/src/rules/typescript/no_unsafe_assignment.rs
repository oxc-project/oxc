use oxc_macros::declare_oxc_lint;

use crate::rule::Rule;

#[derive(Debug, Default, Clone)]
pub struct NoUnsafeAssignment;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule disallows assigning a value with type `any` to variables and properties.
    ///
    /// ### Why is this bad?
    ///
    /// The `any` type in TypeScript disables type checking and can lead to runtime errors. When you assign an `any` value to a typed variable, you're essentially bypassing TypeScript's type safety without any guarantees about the actual value.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// declare const anyValue: any;
    ///
    /// const str: string = anyValue; // unsafe assignment
    ///
    /// let num: number;
    /// num = anyValue; // unsafe assignment
    ///
    /// const obj = {
    ///   prop: anyValue as any, // unsafe assignment
    /// };
    ///
    /// interface User {
    ///   name: string;
    ///   age: number;
    /// }
    ///
    /// const user: User = anyValue; // unsafe assignment
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// declare const stringValue: string;
    /// declare const numberValue: number;
    /// declare const unknownValue: unknown;
    ///
    /// const str: string = stringValue; // safe
    ///
    /// let num: number;
    /// num = numberValue; // safe
    ///
    /// // Use type guards with unknown
    /// if (typeof unknownValue === 'string') {
    ///   const str2: string = unknownValue; // safe after type guard
    /// }
    ///
    /// // Explicit any assignment (still not recommended, but intentional)
    /// const anything: any = unknownValue;
    /// ```
    NoUnsafeAssignment(tsgolint),
    typescript,
    pedantic,
    pending,
);

impl Rule for NoUnsafeAssignment {}
