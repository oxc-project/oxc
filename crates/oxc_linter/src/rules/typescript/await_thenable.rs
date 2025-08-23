use oxc_macros::declare_oxc_lint;

use crate::rule::Rule;

#[derive(Debug, Default, Clone)]
pub struct AwaitThenable;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule disallows awaiting a value that is not a Thenable.
    ///
    /// ### Why is this bad?
    ///
    /// While it is valid JavaScript to await a non-Promise-like value (it will resolve immediately), this practice can be confusing for readers who are not aware of this behavior. It can also be a sign of a programmer error, such as forgetting to add parentheses to call a function that returns a Promise.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// await 12;
    /// await (() => {});
    ///
    /// // non-Promise values
    /// await Math.random;
    /// await { then() {} };
    ///
    /// // this is not a Promise - it's a function that returns a Promise
    /// declare const getPromise: () => Promise<string>;
    /// await getPromise;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// await Promise.resolve('value');
    /// await Promise.reject(new Error());
    ///
    /// // Promise-like values
    /// await {
    ///   then(onfulfilled, onrejected) {
    ///     onfulfilled('value');
    ///   },
    /// };
    ///
    /// // this is a Promise - produced by calling a function
    /// declare const getPromise: () => Promise<string>;
    /// await getPromise();
    /// ```
    AwaitThenable(tsgolint),
    typescript,
    correctness,
    pending,
);

impl Rule for AwaitThenable {}
