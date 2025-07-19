use oxc_macros::declare_oxc_lint;

use crate::rule::Rule;

#[derive(Debug, Default, Clone)]
pub struct NoFloatingPromises;

// See <https://github.com/oxc-project/oxc/issues/6050> for documentation details.
declare_oxc_lint!(
    /// ### What it does
    ///
    /// Briefly describe the rule's purpose.
    ///
    /// ### Why is this bad?
    ///
    /// Explain why violating this rule is problematic.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// FIXME: Tests will fail if examples are missing or syntactically incorrect.
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// FIXME: Tests will fail if examples are missing or syntactically incorrect.
    /// ```
    NoFloatingPromises,
    typescript,
    correctness,
    pending,
);

impl Rule for NoFloatingPromises {
    fn should_run(&self, _ctx: &crate::context::ContextHost) -> bool {
        false
    }
}
