use oxc_macros::declare_oxc_lint;

use crate::rule::Rule;

#[derive(Debug, Default, Clone)]
pub struct NoMisusedPromises;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule forbids providing Promises to logical locations such as if statements in places where the TypeScript compiler allows them but they are not handled properly. These situations can often arise due to a missing await keyword or just a misunderstanding of the way async functions are handled/awaited.
    ///
    /// ### Why is this bad?
    ///
    /// Misused promises can cause crashes or other unexpected behavior, unless there are possibly some global unhandled promise handlers registered.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// // Promises in conditionals:
    /// const promise = Promise.resolve('value');
    /// if (promise) {
    ///   // Do something
    /// }
    ///
    /// // Promises where `void` return was expected:
    /// [1, 2, 3].forEach(async value => {
    ///   await fetch(`/${value}`);
    /// });
    ///
    /// // Spreading Promises:
    /// const getData = () => fetch('/');
    /// console.log({ foo: 42, ...getData() });
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// // Awaiting the Promise to get its value in a conditional:
    /// const promise = Promise.resolve('value');
    /// if (await promise) {
    ///   // Do something
    /// }
    ///
    /// // Using a `for-of` with `await` inside (instead of `forEach`):
    /// for (const value of [1, 2, 3]) {
    ///   await fetch(`/${value}`);
    /// }
    ///
    /// // Spreading data returned from Promise, instead of the Promise itself:
    /// const getData = () => fetch('/');
    /// console.log({ foo: 42, ...(await getData()) });
    /// ```
    NoMisusedPromises(tsgolint),
    typescript,
    pedantic,
    pending,
);

impl Rule for NoMisusedPromises {}
