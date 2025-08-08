use oxc_macros::declare_oxc_lint;

use crate::rule::Rule;

#[derive(Debug, Default, Clone)]
pub struct NoMisusedSpread;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule disallows spreading syntax in places where it doesn't make sense or could cause runtime errors.
    ///
    /// ### Why is this bad?
    ///
    /// The spread operator can be misused in ways that might not be immediately obvious but can cause runtime errors or unexpected behavior. This rule helps catch common misuses.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// // Spreading a non-iterable value in an array
    /// const num = 42;
    /// const arr = [...num]; // Runtime error: num is not iterable
    ///
    /// // Spreading a Promise in an array
    /// const promise = Promise.resolve([1, 2, 3]);
    /// const arr2 = [...promise]; // Runtime error: Promise is not iterable
    ///
    /// // Spreading non-object in object literal
    /// const str = 'hello';
    /// const obj = { ...str }; // Creates { '0': 'h', '1': 'e', ... } which might be unexpected
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// // Spreading arrays
    /// const arr1 = [1, 2, 3];
    /// const arr2 = [...arr1];
    ///
    /// // Spreading objects
    /// const obj1 = { a: 1, b: 2 };
    /// const obj2 = { ...obj1 };
    ///
    /// // Spreading resolved Promise
    /// const promise = Promise.resolve([1, 2, 3]);
    /// const arr3 = [...(await promise)];
    ///
    /// // Using Array.from for non-iterables if needed
    /// const str = 'hello';
    /// const arr4 = Array.from(str); // ['h', 'e', 'l', 'l', 'o']
    /// ```
    NoMisusedSpread(tsgolint),
    typescript,
    correctness,
    pending,
);

impl Rule for NoMisusedSpread {}
