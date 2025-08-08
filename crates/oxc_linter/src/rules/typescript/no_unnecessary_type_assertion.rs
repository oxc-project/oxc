use oxc_macros::declare_oxc_lint;

use crate::rule::Rule;

#[derive(Debug, Default, Clone)]
pub struct NoUnnecessaryTypeAssertion;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule disallows type assertions that do not change the type of an expression.
    ///
    /// ### Why is this bad?
    ///
    /// Type assertions that don't actually change the type of an expression are unnecessary and can be safely removed. They add visual noise without providing any benefit and may indicate confusion about TypeScript's type system.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// const str: string = 'hello';
    /// const redundant = str as string; // unnecessary, str is already string
    ///
    /// function getString(): string {
    ///   return 'hello';
    /// }
    /// const result = getString() as string; // unnecessary, getString() already returns string
    ///
    /// const num = 42;
    /// const alsoRedundant = num as 42; // unnecessary if TypeScript can infer literal type
    ///
    /// // Unnecessary assertion to wider type
    /// const literal = 'hello' as string;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// const unknown: unknown = 'hello';
    /// const str = unknown as string; // necessary to narrow type
    ///
    /// const element = document.getElementById('myElement') as HTMLInputElement; // necessary for specific element type
    ///
    /// const obj = { name: 'John' };
    /// const name = obj.name as const; // necessary for literal type
    ///
    /// // No assertion needed
    /// const str2: string = 'hello';
    /// const num: number = 42;
    /// ```
    NoUnnecessaryTypeAssertion(tsgolint),
    typescript,
    suspicious,
    pending,
);

impl Rule for NoUnnecessaryTypeAssertion {}
