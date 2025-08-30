use oxc_macros::declare_oxc_lint;

use crate::rule::Rule;

#[derive(Debug, Default, Clone)]
pub struct NoUnsafeEnumComparison;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule disallows comparing an enum value with a non-enum value.
    ///
    /// ### Why is this bad?
    ///
    /// Enum values should only be compared with other values of the same enum type or their underlying literal values in a type-safe manner. Comparing enums with unrelated values can lead to unexpected behavior and defeats the purpose of using enums for type safety.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// enum Status {
    ///   Open = 'open',
    ///   Closed = 'closed',
    /// }
    ///
    /// enum Color {
    ///   Red = 'red',
    ///   Blue = 'blue',
    /// }
    ///
    /// declare const status: Status;
    /// declare const color: Color;
    /// declare const str: string;
    ///
    /// // Comparing enum with different enum
    /// if (status === color) {} // unsafe
    ///
    /// // Comparing enum with string (unless it's a literal that matches)
    /// if (status === str) {} // unsafe
    ///
    /// // Comparing with arbitrary value
    /// if (status === 'unknown') {} // unsafe
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// enum Status {
    ///   Open = 'open',
    ///   Closed = 'closed',
    /// }
    ///
    /// declare const status: Status;
    ///
    /// // Comparing with same enum values
    /// if (status === Status.Open) {} // safe
    ///
    /// // Comparing with the correct literal type
    /// if (status === 'open') {} // safe
    ///
    /// // Using enum methods
    /// if (Object.values(Status).includes(someValue)) {} // safe way to check
    /// ```
    NoUnsafeEnumComparison(tsgolint),
    typescript,
    suspicious,
    pending,
);

impl Rule for NoUnsafeEnumComparison {}
