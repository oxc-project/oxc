use oxc_macros::declare_oxc_lint;

use crate::rule::Rule;

#[derive(Debug, Default, Clone)]
pub struct SwitchExhaustivenessCheck;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule requires switch statements to be exhaustive when switching on union types.
    ///
    /// ### Why is this bad?
    ///
    /// When switching on a union type, it's important to handle all possible cases to avoid runtime errors. TypeScript can help ensure exhaustiveness, but only if the switch statement is properly structured with a default case that TypeScript can analyze.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// type Status = 'pending' | 'approved' | 'rejected';
    ///
    /// function handleStatus(status: Status) {
    ///   switch (status) {
    ///     case 'pending':
    ///       return 'Waiting for approval';
    ///     case 'approved':
    ///       return 'Request approved';
    ///     // Missing 'rejected' case
    ///   }
    /// }
    ///
    /// enum Color {
    ///   Red,
    ///   Green,
    ///   Blue,
    /// }
    ///
    /// function getColorName(color: Color) {
    ///   switch (color) {
    ///     case Color.Red:
    ///       return 'red';
    ///     case Color.Green:
    ///       return 'green';
    ///     // Missing Color.Blue case
    ///   }
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// type Status = 'pending' | 'approved' | 'rejected';
    ///
    /// function handleStatus(status: Status) {
    ///   switch (status) {
    ///     case 'pending':
    ///       return 'Waiting for approval';
    ///     case 'approved':
    ///       return 'Request approved';
    ///     case 'rejected':
    ///       return 'Request rejected';
    ///   }
    /// }
    ///
    /// // Or with default case for exhaustiveness checking
    /// function handleStatusWithDefault(status: Status) {
    ///   switch (status) {
    ///     case 'pending':
    ///       return 'Waiting for approval';
    ///     case 'approved':
    ///       return 'Request approved';
    ///     case 'rejected':
    ///       return 'Request rejected';
    ///     default:
    ///       const _exhaustiveCheck: never = status;
    ///       return _exhaustiveCheck;
    ///   }
    /// }
    ///
    /// enum Color {
    ///   Red,
    ///   Green,
    ///   Blue,
    /// }
    ///
    /// function getColorName(color: Color) {
    ///   switch (color) {
    ///     case Color.Red:
    ///       return 'red';
    ///     case Color.Green:
    ///       return 'green';
    ///     case Color.Blue:
    ///       return 'blue';
    ///     default:
    ///       const _exhaustiveCheck: never = color;
    ///       return _exhaustiveCheck;
    ///   }
    /// }
    /// ```
    SwitchExhaustivenessCheck(tsgolint),
    typescript,
    pedantic,
    pending,
);

impl Rule for SwitchExhaustivenessCheck {}
