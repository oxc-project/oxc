use oxc_macros::declare_oxc_lint;

use crate::rule::Rule;

#[derive(Debug, Default, Clone)]
pub struct NoFloatingPromises;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule disallows "floating" Promises in TypeScript code, which is a Promise that is created without any code to handle its resolution or rejection.
    ///
    /// This rule will report Promise-valued statements that are not treated in one of the following ways:
    ///
    /// - Calling its `.then()` with two arguments
    /// - Calling its `.catch()` with one argument
    /// - `await`ing it
    /// - `return`ing it
    /// - `void`ing it
    ///
    /// This rule also reports when an Array containing Promises is created and not properly handled. The main way to resolve this is by using one of the Promise concurrency methods to create a single Promise, then handling that according to the procedure above. These methods include:
    ///
    /// - `Promise.all()`
    /// - `Promise.allSettled()`
    /// - `Promise.any()`
    /// - `Promise.race()`
    ///
    /// ### Why is this bad?
    ///
    /// Floating Promises can cause several issues, such as improperly sequenced operations, ignored Promise rejections, and more.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// const promise = new Promise((resolve, reject) => resolve('value'));
    /// promise;
    ///
    /// async function returnsPromise() {
    ///   return 'value';
    /// }
    /// returnsPromise().then(() => {});
    ///
    /// Promise.reject('value').catch();
    ///
    /// Promise.reject('value').finally();
    ///
    /// [1, 2, 3].map(async x => x + 1);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// const promise = new Promise((resolve, reject) => resolve('value'));
    /// await promise;
    ///
    /// async function returnsPromise() {
    ///   return 'value';
    /// }
    ///
    /// void returnsPromise();
    ///
    /// returnsPromise().then(
    ///   () => {},
    ///   () => {},
    /// );
    ///
    /// Promise.reject('value').catch(() => {});
    ///
    /// await Promise.reject('value').finally(() => {});
    ///
    /// await Promise.all([1, 2, 3].map(async x => x + 1));
    /// ```
    NoFloatingPromises(tsgolint),
    typescript,
    suspicious,
    pending,
);

impl Rule for NoFloatingPromises {}
