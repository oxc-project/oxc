use crate::rule::Rule;
use oxc_macros::declare_oxc_lint;

#[derive(Debug, Default, Clone)]
pub struct NoUselessDefaultAssignment;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow default assignments that can never be used.
    ///
    /// ### Why is this bad?
    ///
    /// A default assignment is redundant when the value can never be `undefined`.
    /// This adds runtime logic and noise without changing behavior.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// [1, 2, 3].map((a = 0) => a + 1);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// [1, 2, 3].map((a) => a + 1);
    /// ```
    NoUselessDefaultAssignment(tsgolint),
    typescript,
    nursery,
);

impl Rule for NoUselessDefaultAssignment {}
