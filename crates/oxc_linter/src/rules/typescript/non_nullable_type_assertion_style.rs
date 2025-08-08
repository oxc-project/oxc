use oxc_macros::declare_oxc_lint;

use crate::rule::Rule;

#[derive(Debug, Default, Clone)]
pub struct NonNullableTypeAssertionStyle;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule prefers a non-null assertion over an explicit type cast for non-nullable types.
    ///
    /// ### Why is this bad?
    ///
    /// When you know that a value cannot be null or undefined, you can use either a non-null assertion (`!`) or a type assertion (`as Type`). The non-null assertion is more concise and clearly communicates the intent that you're asserting the value is not null/undefined.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// declare const value: string | null;
    ///
    /// // Type assertion when non-null assertion would be clearer
    /// const result1 = value as string;
    ///
    /// declare const maybe: number | undefined;
    /// const result2 = maybe as number;
    ///
    /// // In function calls
    /// function takesString(s: string) {
    ///   console.log(s);
    /// }
    ///
    /// takesString(value as string);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// declare const value: string | null;
    ///
    /// // Non-null assertion for non-nullable types
    /// const result1 = value!;
    ///
    /// declare const maybe: number | undefined;
    /// const result2 = maybe!;
    ///
    /// // In function calls
    /// function takesString(s: string) {
    ///   console.log(s);
    /// }
    ///
    /// takesString(value!);
    ///
    /// // Type assertion for actual type changes is still fine
    /// declare const unknown: unknown;
    /// const str = unknown as string; // This is a different type, not just removing null
    /// ```
    NonNullableTypeAssertionStyle(tsgolint),
    typescript,
    restriction,
    pending,
);

impl Rule for NonNullableTypeAssertionStyle {}
