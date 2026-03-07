use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_react_children_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("`React.Children` should not be used.")
        .with_help("Children passed to this React component should be handled in a different way, see the docs for alternatives.")
        .with_label(span)
        .with_note("https://react.dev/reference/react/Children")
}

#[derive(Debug, Default, Clone)]
pub struct NoReactChildren;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// FIXME: Briefly describe the rule's purpose.
    ///
    /// ### Why is this bad?
    ///
    /// FIXME: Explain why violating this rule is problematic.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// FIXME: Add at least one example of code that violates the rule.
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// FIXME: Add at least one example of code that is allowed with the rule.
    /// ```
    NoReactChildren,
    react,
    nursery, // TODO: change category to `correctness`, `suspicious`, `pedantic`, `perf`, `restriction`, or `style`
             // See <https://oxc.rs/docs/contribute/linter.html#rule-category> for details
    pending, // TODO: describe fix capabilities. Remove or set to `none` if no fix can be done,
             // keep at 'pending' if you think one could be added but don't know how.
             // Options are 'fix', 'fix_dangerous', 'suggestion', and 'conditional_fix_suggestion'
);

impl Rule for NoReactChildren {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {}
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![];

    let fail = vec![];

    Tester::new(NoReactChildren::NAME, NoReactChildren::PLUGIN, pass, fail).test_and_snapshot();
}
