use oxc_macros::declare_oxc_lint;

use crate::rule::Rule;

#[derive(Debug, Default, Clone)]
pub struct ReturnAwait;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule enforces consistent returning of awaited values from async functions.
    ///
    /// ### Why is this bad?
    ///
    /// There are different patterns for returning awaited values from async functions. Sometimes you want to await before returning (to handle errors in the current function), and sometimes you want to return the Promise directly (for better performance). This rule helps enforce consistency.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule (depending on configuration):
    /// ```ts
    /// // If configured to require await:
    /// async function fetchData() {
    ///   return fetch('/api/data'); // Should be: return await fetch('/api/data');
    /// }
    ///
    /// async function processData() {
    ///   return someAsyncOperation(); // Should be: return await someAsyncOperation();
    /// }
    ///
    /// // If configured to disallow unnecessary await:
    /// async function fetchData() {
    ///   return await fetch('/api/data'); // Should be: return fetch('/api/data');
    /// }
    ///
    /// async function processData() {
    ///   return await someAsyncOperation(); // Should be: return someAsyncOperation();
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// // When await is required for error handling:
    /// async function fetchData() {
    ///   try {
    ///     return await fetch('/api/data');
    ///   } catch (error) {
    ///     console.error('Fetch failed:', error);
    ///     throw error;
    ///   }
    /// }
    ///
    /// // When returning Promise directly for performance:
    /// async function fetchData() {
    ///   return fetch('/api/data');
    /// }
    ///
    /// // Processing before return requires await:
    /// async function fetchAndProcess() {
    ///   const response = await fetch('/api/data');
    ///   return response.json();
    /// }
    ///
    /// // Multiple async operations:
    /// async function multipleOperations() {
    ///   const data1 = await fetchData1();
    ///   const data2 = await fetchData2();
    ///   return data1 + data2;
    /// }
    /// ```
    ReturnAwait(tsgolint),
    typescript,
    pedantic,
    pending,
);

impl Rule for ReturnAwait {}
