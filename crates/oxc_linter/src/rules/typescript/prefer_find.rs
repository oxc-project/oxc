use oxc_macros::declare_oxc_lint;

use crate::rule::Rule;

#[derive(Debug, Default, Clone)]
pub struct PreferFind;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefer `.find(...)` over `.filter(...)[0]` for retrieving a single element.
    ///
    /// ### Why is this bad?
    ///
    /// `.filter(...)[0]` builds an intermediate array and is less clear about intent.
    /// `.find(...)` directly expresses that only the first matching element is needed.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// const first = list.filter(item => item.active)[0];
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// const first = list.find(item => item.active);
    /// ```
    PreferFind(tsgolint),
    typescript,
    nursery,
);

impl Rule for PreferFind {}
