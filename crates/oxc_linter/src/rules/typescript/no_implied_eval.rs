use oxc_macros::declare_oxc_lint;

use crate::rule::Rule;

#[derive(Debug, Default, Clone)]
pub struct NoImpliedEval;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule disallows the use of eval-like methods.
    ///
    /// ### Why is this bad?
    ///
    /// It's considered a good practice to avoid using eval() in JavaScript. There are security and performance implications involved with doing so, which is why many linters recommend disallowing eval(). However, there are some other ways to pass a string and have it interpreted as JavaScript code that have similar concerns.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// setTimeout('alert("Hi!");', 100);
    ///
    /// setInterval('alert("Hi!");', 100);
    ///
    /// setImmediate('alert("Hi!")');
    ///
    /// window.setTimeout('count = 5', 10);
    ///
    /// window.setInterval('foo = bar', 10);
    ///
    /// const fn = new Function('a', 'b', 'return a + b');
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// setTimeout(() => {
    ///   alert('Hi!');
    /// }, 100);
    ///
    /// setInterval(() => {
    ///   alert('Hi!');
    /// }, 100);
    ///
    /// setImmediate(() => {
    ///   alert('Hi!');
    /// });
    ///
    /// const fn = (a: number, b: number) => a + b;
    /// ```
    NoImpliedEval(tsgolint),
    typescript,
    correctness,
    pending,
);

impl Rule for NoImpliedEval {}
