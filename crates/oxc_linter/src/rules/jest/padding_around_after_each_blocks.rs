use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::utils::{
    JestGeneralFnKind, ParsedGeneralJestFnCall, parse_general_jest_fn_call,
    report_missing_padding_after_jest_block, report_missing_padding_before_jest_block,
};
use crate::{
    AstNode,
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::Rule,
    utils::PossibleJestNode,
};

fn padding_around_after_each_blocks_diagnostic(span: Span) -> OxcDiagnostic {
    // See <https://oxc.rs/docs/contribute/linter/adding-rules.html#diagnostics> for details
    OxcDiagnostic::warn("Should be an imperative statement about what is wrong.")
        .with_help("Should be a command-like statement that tells the user how to fix the issue.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PaddingAroundAfterEachBlocks;

// See <https://github.com/oxc-project/oxc/issues/6050> for documentation details.
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
    PaddingAroundAfterEachBlocks,
    jest,
    style, // TODO: change category to `correctness`, `suspicious`, `pedantic`, `perf`, `restriction`, or `style`
             // See <https://oxc.rs/docs/contribute/linter.html#rule-category> for details
    fix, // TODO: describe fix capabilities. Remove or set to `none` if no fix can be done,
             // keep at 'pending' if you think one could be added but don't know how.
             // Options are 'fix', 'fix_dangerous', 'suggestion', and 'conditional_fix_suggestion'
    version = "next",
    short_description = "FIXME: One-sentence description of the rule.",
);

impl Rule for PaddingAroundAfterEachBlocks {
    fn run_on_jest_node<'a, 'c>(
        &self,
        possible_jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        let node = possible_jest_node.node;
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };
        let Some(jest_fn_call) = parse_general_jest_fn_call(call_expr, possible_jest_node, ctx)
        else {
            return;
        };
        let ParsedGeneralJestFnCall { kind, name, .. } = &jest_fn_call;
        let Some(kind) = kind.to_general() else {
            return;
        };
        if kind != JestGeneralFnKind::Hook {
            return;
        }
        if name != "afterEach" {
            return;
        }
        report_missing_padding_before_jest_block(node, ctx, name);
        report_missing_padding_after_jest_block(node, ctx, name);
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "const something = 123;\n\nafterEach(() => {\n//  // more stuff\n});\n\ndescribe('foo', () => {});",
    ];

    let fail = vec![
        "const something = 123;\nafterEach(() => {\n//  // more stuff\n});\ndescribe('foo', () => {});",
        "const something = 123;\n\nafterEach(() => {\n//  // more stuff\n});\ndescribe('foo', () => {});",
        "const something = 123;\nafterEach(() => {\n//  // more stuff\n});\n\ndescribe('foo', () => {});",
    ];

    let fix = vec![(
        "const something = 123;\nafterEach(() => {\n//  // more stuff\n});\ndescribe('foo', () => {});",
        "const something = 123;\n\nafterEach(() => {\n//  // more stuff\n});\n\ndescribe('foo', () => {});",
    )];

    Tester::new(
        PaddingAroundAfterEachBlocks::NAME,
        PaddingAroundAfterEachBlocks::PLUGIN,
        pass,
        fail,
    )
    .with_jest_plugin(true)
    .expect_fix(fix)
    .test_and_snapshot();
}
