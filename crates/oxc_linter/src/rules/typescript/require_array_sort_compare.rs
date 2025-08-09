use oxc_macros::declare_oxc_lint;

use crate::rule::Rule;

#[derive(Debug, Default, Clone)]
pub struct RequireArraySortCompare;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule requires Array.sort() to be called with a comparison function.
    ///
    /// ### Why is this bad?
    ///
    /// When Array.sort() is called without a comparison function, it converts elements to strings and sorts them lexicographically. This often leads to unexpected results, especially with numbers where `[1, 10, 2].sort()` returns `[1, 10, 2]` instead of `[1, 2, 10]`.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// const numbers = [3, 1, 4, 1, 5];
    /// numbers.sort(); // Lexicographic sort, not numeric
    ///
    /// const mixedArray = ['10', '2', '1'];
    /// mixedArray.sort(); // Might be intended, but explicit compareFn is clearer
    ///
    /// [3, 1, 4].sort(); // Will sort as strings: ['1', '3', '4']
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// const numbers = [3, 1, 4, 1, 5];
    ///
    /// // Numeric sort
    /// numbers.sort((a, b) => a - b);
    ///
    /// // Reverse numeric sort
    /// numbers.sort((a, b) => b - a);
    ///
    /// // String sort (explicit)
    /// const strings = ['banana', 'apple', 'cherry'];
    /// strings.sort((a, b) => a.localeCompare(b));
    ///
    /// // Custom object sorting
    /// interface Person {
    ///   name: string;
    ///   age: number;
    /// }
    ///
    /// const people: Person[] = [
    ///   { name: 'Alice', age: 30 },
    ///   { name: 'Bob', age: 25 },
    /// ];
    ///
    /// people.sort((a, b) => a.age - b.age);
    /// people.sort((a, b) => a.name.localeCompare(b.name));
    /// ```
    RequireArraySortCompare(tsgolint),
    typescript,
    correctness,
    pending,
);

impl Rule for RequireArraySortCompare {}
