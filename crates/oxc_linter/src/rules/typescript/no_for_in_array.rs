use oxc_macros::declare_oxc_lint;

use crate::rule::Rule;

#[derive(Debug, Default, Clone)]
pub struct NoForInArray;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule disallows iterating over an array with a for-in loop.
    ///
    /// ### Why is this bad?
    ///
    /// A for-in loop iterates over the enumerable properties of an object, which includes the array indices, but also includes any enumerable properties added to the array prototype or the array instance. This is almost never what you want when iterating over an array.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// const arr = [1, 2, 3];
    ///
    /// for (const i in arr) {
    ///   console.log(arr[i]);
    /// }
    ///
    /// for (const i in arr) {
    ///   console.log(i, arr[i]);
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// const arr = [1, 2, 3];
    ///
    /// // Use for-of to iterate over array values
    /// for (const value of arr) {
    ///   console.log(value);
    /// }
    ///
    /// // Use regular for loop with index
    /// for (let i = 0; i < arr.length; i++) {
    ///   console.log(i, arr[i]);
    /// }
    ///
    /// // Use forEach
    /// arr.forEach((value, index) => {
    ///   console.log(index, value);
    /// });
    ///
    /// // for-in is fine for objects
    /// const obj = { a: 1, b: 2 };
    /// for (const key in obj) {
    ///   console.log(key, obj[key]);
    /// }
    /// ```
    NoForInArray(tsgolint),
    typescript,
    correctness,
    pending,
);

impl Rule for NoForInArray {}
