use oxc_macros::declare_oxc_lint;

use crate::rule::Rule;

#[derive(Debug, Default, Clone)]
pub struct PromiseFunctionAsync;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule requires any function or method that returns a Promise to be marked as async.
    ///
    /// ### Why is this bad?
    ///
    /// Functions that return Promises should typically be marked as `async` to make their asynchronous nature clear and to enable the use of `await` within them. This makes the code more readable and helps prevent common mistakes with Promise handling.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// // Function returning Promise without async
    /// function fetchData(): Promise<string> {
    ///   return fetch('/api/data').then(res => res.text());
    /// }
    ///
    /// // Method returning Promise without async
    /// class DataService {
    ///   getData(): Promise<any> {
    ///     return fetch('/api/data').then(res => res.json());
    ///   }
    /// }
    ///
    /// // Arrow function returning Promise without async
    /// const processData = (): Promise<void> => {
    ///   return Promise.resolve();
    /// };
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// // Async function
    /// async function fetchData(): Promise<string> {
    ///   const response = await fetch('/api/data');
    ///   return response.text();
    /// }
    ///
    /// // Async method
    /// class DataService {
    ///   async getData(): Promise<any> {
    ///     const response = await fetch('/api/data');
    ///     return response.json();
    ///   }
    /// }
    ///
    /// // Async arrow function
    /// const processData = async (): Promise<void> => {
    ///   await someAsyncOperation();
    /// };
    ///
    /// // Functions that don't return Promise are fine
    /// function syncFunction(): string {
    ///   return 'hello';
    /// }
    ///
    /// // Functions returning Promise-like but not actual Promise
    /// function createThenable(): { then: Function } {
    ///   return { then: () => {} };
    /// }
    /// ```
    PromiseFunctionAsync(tsgolint),
    typescript,
    restriction,
    pending,
);

impl Rule for PromiseFunctionAsync {}
