use oxc_macros::declare_oxc_lint;

use crate::rule::Rule;

#[derive(Debug, Default, Clone)]
pub struct NoDuplicateTypeConstituents;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule disallows duplicate constituents of union or intersection types.
    ///
    /// ### Why is this bad?
    ///
    /// Duplicate constituents in union and intersection types serve no purpose and can make code harder to read. They are likely a mistake.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// type T1 = 'A' | 'A';
    ///
    /// type T2 = A | A | B;
    ///
    /// type T3 = { a: string } & { a: string };
    ///
    /// type T4 = [A, A];
    ///
    /// type T5 =
    ///   | 'foo'
    ///   | 'bar'
    ///   | 'foo';
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// type T1 = 'A' | 'B';
    ///
    /// type T2 = A | B | C;
    ///
    /// type T3 = { a: string } & { b: string };
    ///
    /// type T4 = [A, B];
    ///
    /// type T5 =
    ///   | 'foo'
    ///   | 'bar'
    ///   | 'baz';
    /// ```
    NoDuplicateTypeConstituents(tsgolint),
    typescript,
    correctness,
    pending,
);

impl Rule for NoDuplicateTypeConstituents {}
