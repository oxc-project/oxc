use oxc_macros::declare_oxc_lint;

use crate::rule::Rule;

#[derive(Debug, Default, Clone)]
pub struct NoUnnecessaryTypeParameters;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow type parameters that are declared but not meaningfully used.
    ///
    /// ### Why is this bad?
    ///
    /// Unnecessary type parameters make signatures noisier and harder to understand, and they
    /// often hide opportunities to simplify APIs.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// function parseYAML<T>(input: string): T {
    ///   return input as any as T;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// function parseYAML(input: string): unknown {
    ///   return input;
    /// }
    ///
    /// function identity<T>(value: T): T {
    ///   return value;
    /// }
    /// ```
    NoUnnecessaryTypeParameters(tsgolint),
    typescript,
    nursery,
);

impl Rule for NoUnnecessaryTypeParameters {}
