use oxc_macros::declare_oxc_lint;

use crate::rule::Rule;

#[derive(Debug, Default, Clone)]
pub struct NoDeprecated;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow using code marked as `@deprecated`.
    ///
    /// ### Why is this bad?
    ///
    /// The JSDoc `@deprecated` tag can be used to document some piece of code
    /// being deprecated. It's best to avoid using code marked as deprecated.
    /// This rule reports on any references to code marked as `@deprecated`.
    ///
    /// TypeScript recognizes the `@deprecated` tag, allowing editors to visually
    /// indicate deprecated code — usually with a strikethrough. However, TypeScript
    /// doesn't report type errors for deprecated code on its own.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// /** @deprecated Use apiV2 instead. */
    /// declare function apiV1(): Promise<string>;
    /// declare function apiV2(): Promise<string>;
    ///
    /// await apiV1(); // Using deprecated function
    ///
    /// import { parse } from 'node:url';
    /// // 'parse' is deprecated. Use the WHATWG URL API instead.
    /// const url = parse('/foo');
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// /** @deprecated Use apiV2 instead. */
    /// declare function apiV1(): Promise<string>;
    /// declare function apiV2(): Promise<string>;
    ///
    /// await apiV2(); // Using non-deprecated function
    ///
    /// // Modern Node.js API, uses `new URL()`
    /// const url2 = new URL('/foo', 'http://www.example.com');
    /// ```
    NoDeprecated(tsgolint),
    typescript,
    pedantic
);

impl Rule for NoDeprecated {}
