use oxc_macros::declare_oxc_lint;

use crate::rule::Rule;

#[derive(Debug, Default, Clone)]
pub struct NoArrayDelete;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule disallows using the delete operator on array values.
    ///
    /// ### Why is this bad?
    ///
    /// When using the delete operator on an array, the element is not actually removed, but instead the array slot is turned into undefined. This is usually not the intended behavior. Instead, you should use methods like Array.prototype.splice() to properly remove elements from an array.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// declare const arr: number[];
    /// delete arr[0];
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// declare const arr: number[];
    /// arr.splice(0, 1);
    ///
    /// // or with a filter
    /// const filteredArr = arr.filter((_, index) => index !== 0);
    ///
    /// // delete on object is allowed
    /// declare const obj: { a?: number };
    /// delete obj.a;
    /// ```
    NoArrayDelete(tsgolint),
    typescript,
    correctness,
    pending,
);

impl Rule for NoArrayDelete {}
