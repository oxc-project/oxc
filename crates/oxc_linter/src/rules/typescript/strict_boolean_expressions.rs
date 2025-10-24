use oxc_macros::declare_oxc_lint;

use crate::rule::Rule;

#[derive(Debug, Default, Clone)]
pub struct StrictBooleanExpressions;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow certain types in boolean expressions.
    ///
    /// ### Why is this bad?
    ///
    /// Forbids usage of non-boolean types in expressions where a boolean is expected.
    /// `boolean` and `never` types are always allowed. Additional types which are
    /// considered safe in a boolean context can be configured via options.
    ///
    /// The following nodes are checked:
    ///
    /// - Arguments to the `!`, `&&`, and `||` operators
    /// - The condition in a conditional expression (`cond ? x : y`)
    /// - Conditions for `if`, `for`, `while`, and `do-while` statements.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// const str = 'hello';
    /// if (str) {
    ///   console.log('string');
    /// }
    ///
    /// const num = 42;
    /// if (num) {
    ///   console.log('number');
    /// }
    ///
    /// const obj = { foo: 'bar' };
    /// if (obj) {
    ///   console.log('object');
    /// }
    ///
    /// declare const maybeString: string | undefined;
    /// if (maybeString) {
    ///   console.log(maybeString);
    /// }
    ///
    /// const result = str && num;
    /// const ternary = str ? 'yes' : 'no';
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// const str = 'hello';
    /// if (str !== '') {
    ///   console.log('string');
    /// }
    ///
    /// const num = 42;
    /// if (num !== 0) {
    ///   console.log('number');
    /// }
    ///
    /// const obj = { foo: 'bar' };
    /// if (obj !== null) {
    ///   console.log('object');
    /// }
    ///
    /// declare const maybeString: string | undefined;
    /// if (maybeString !== undefined) {
    ///   console.log(maybeString);
    /// }
    ///
    /// const bool = true;
    /// if (bool) {
    ///   console.log('boolean');
    /// }
    /// ```
    StrictBooleanExpressions(tsgolint),
    typescript,
    pedantic,
    pending,
);

impl Rule for StrictBooleanExpressions {}
