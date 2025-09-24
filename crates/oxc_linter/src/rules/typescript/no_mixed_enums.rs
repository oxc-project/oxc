use oxc_macros::declare_oxc_lint;

use crate::rule::Rule;

#[derive(Debug, Default, Clone)]
pub struct NoMixedEnums;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule disallows enums from having both string and numeric members.
    ///
    /// ### Why is this bad?
    ///
    /// TypeScript enums can have string, numeric, or computed members. Having mixed string and numeric members in the same enum can lead to confusion and unexpected runtime behavior due to how TypeScript compiles enums.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// enum Status {
    ///   Open = 1,
    ///   Closed = 'closed',
    /// }
    ///
    /// enum Direction {
    ///   Up = 'up',
    ///   Down = 2,
    ///   Left = 'left',
    ///   Right = 4,
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// // All numeric
    /// enum Status {
    ///   Open = 1,
    ///   Closed = 2,
    /// }
    ///
    /// // All string
    /// enum Direction {
    ///   Up = 'up',
    ///   Down = 'down',
    ///   Left = 'left',
    ///   Right = 'right',
    /// }
    ///
    /// // Auto-incremented numeric
    /// enum Color {
    ///   Red,
    ///   Green,
    ///   Blue,
    /// }
    /// ```
    NoMixedEnums(tsgolint),
    typescript,
    pedantic,
    pending,
);

impl Rule for NoMixedEnums {}
