use oxc_macros::declare_oxc_lint;

use crate::rule::Rule;

#[derive(Debug, Default, Clone)]
pub struct NoUnsafeReturn;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule disallows returning a value with type `any` from a function.
    ///
    /// ### Why is this bad?
    ///
    /// The `any` type in TypeScript disables type checking. When you return a value typed as `any` from a function, you're essentially passing the type-safety problem to the caller without providing any guarantees about what the function actually returns.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// declare const anyValue: any;
    ///
    /// function getString(): string {
    ///   return anyValue; // unsafe return
    /// }
    ///
    /// const getNumber = (): number => anyValue; // unsafe return
    ///
    /// function processData(): { name: string; age: number } {
    ///   return anyValue; // unsafe return
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// declare const stringValue: string;
    /// declare const numberValue: number;
    /// declare const unknownValue: unknown;
    ///
    /// function getString(): string {
    ///   return stringValue; // safe
    /// }
    ///
    /// const getNumber = (): number => numberValue; // safe
    ///
    /// function processUnknown(): unknown {
    ///   return unknownValue; // safe - explicitly returning unknown
    /// }
    ///
    /// // Type guard to safely return
    /// function safeGetString(): string | null {
    ///   if (typeof unknownValue === 'string') {
    ///     return unknownValue; // safe after type guard
    ///   }
    ///   return null;
    /// }
    /// ```
    NoUnsafeReturn(tsgolint),
    typescript,
    pedantic,
    pending,
);

impl Rule for NoUnsafeReturn {}
