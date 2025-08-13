use oxc_macros::declare_oxc_lint;

use crate::rule::Rule;

#[derive(Debug, Default, Clone)]
pub struct NoUnnecessaryBooleanLiteralCompare;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule disallows unnecessary equality comparisons with boolean literals.
    ///
    /// ### Why is this bad?
    ///
    /// Comparing boolean values to boolean literals is unnecessary when the comparison can be eliminated. These comparisons make code more verbose without adding value.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// declare const someCondition: boolean;
    ///
    /// if (someCondition === true) {
    ///   // ...
    /// }
    ///
    /// if (someCondition === false) {
    ///   // ...
    /// }
    ///
    /// if (someCondition !== true) {
    ///   // ...
    /// }
    ///
    /// if (someCondition !== false) {
    ///   // ...
    /// }
    ///
    /// const result = someCondition == true;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// declare const someCondition: boolean;
    ///
    /// if (someCondition) {
    ///   // ...
    /// }
    ///
    /// if (!someCondition) {
    ///   // ...
    /// }
    ///
    /// // Comparisons with non-boolean types are allowed
    /// declare const someValue: unknown;
    /// if (someValue === true) {
    ///   // ...
    /// }
    /// ```
    NoUnnecessaryBooleanLiteralCompare(tsgolint),
    typescript,
    suspicious,
    pending,
);

impl Rule for NoUnnecessaryBooleanLiteralCompare {}
