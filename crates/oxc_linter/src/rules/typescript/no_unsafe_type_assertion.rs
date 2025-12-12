use oxc_macros::declare_oxc_lint;

use crate::rule::Rule;

#[derive(Debug, Default, Clone)]
pub struct NoUnsafeTypeAssertion;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows unsafe type assertions that narrow a type.
    ///
    /// ### Why is this bad?
    ///
    /// Type assertions that narrow a type bypass TypeScript's type-checking and can lead to
    /// runtime errors. Type assertions that broaden a type are safe because TypeScript
    /// essentially knows *less* about a type. Instead of using type assertions to narrow a
    /// type, it's better to rely on type guards, which help avoid potential runtime errors
    /// caused by unsafe type assertions.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// function f() {
    ///   return Math.random() < 0.5 ? 42 : 'oops';
    /// }
    /// const z = f() as number;
    ///
    /// const items = [1, '2', 3, '4'];
    /// const number = items[0] as number;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// function f() {
    ///   return Math.random() < 0.5 ? 42 : 'oops';
    /// }
    /// const z = f() as number | string | boolean;
    ///
    /// const items = [1, '2', 3, '4'];
    /// const number = items[0] as number | string | undefined;
    /// ```
    NoUnsafeTypeAssertion(tsgolint),
    typescript,
    suspicious,
    pending,
);

impl Rule for NoUnsafeTypeAssertion {}
