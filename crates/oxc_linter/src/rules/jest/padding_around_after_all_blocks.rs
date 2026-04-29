use oxc_ast::AstKind;
use oxc_macros::declare_oxc_lint;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{
        JestGeneralFnKind, ParsedGeneralJestFnCall, PossibleJestNode, parse_general_jest_fn_call,
        report_missing_padding_before_jest_block,
    },
};

#[derive(Debug, Default, Clone)]
pub struct PaddingAroundAfterAllBlocks;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule enforces a line of padding before and after 1 or more
    /// `afterAll` statements.
    ///
    /// ### Why is this bad?
    ///
    /// Inconsistent formatting of code can make the code more difficult to read
    /// and follow. This rule helps ensure that `afterAll` blocks are visually
    /// separated from the rest of the code, making them easier to identify while
    /// looking through test files.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// const thing = 123;
    /// afterAll(() => {});
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const thing = 123;
    ///
    /// afterAll(() => {});
    /// ```
    PaddingAroundAfterAllBlocks,
    jest,
    style,
    fix,
    version = "1.59.0",
);

impl Rule for PaddingAroundAfterAllBlocks {
    fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        let node = jest_node.node;
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };
        let Some(jest_fn_call) = parse_general_jest_fn_call(call_expr, jest_node, ctx) else {
            return;
        };
        let ParsedGeneralJestFnCall { kind, name, .. } = &jest_fn_call;
        let Some(kind) = kind.to_general() else {
            return;
        };
        if kind != JestGeneralFnKind::Hook {
            return;
        }
        if name != "afterAll" {
            return;
        }
        report_missing_padding_before_jest_block(node, ctx, name);
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "afterAll(() => {});",
        "const thing = 123;\n\nafterAll(() => {});",
        "describe('foo', () => {\nafterAll(() => {});\n});",
        "const thing = 123;\n\n/* one */\n/* two */\nafterAll(() => {});",
    ];

    let fail = vec![
        "const thing = 123;\nafterAll(() => {});",
        "const thing = 123;\n/* one */\n/* two */\nafterAll(() => {});",
    ];

    let fix = vec![
        ("const thing = 123;\nafterAll(() => {});", "const thing = 123;\n\nafterAll(() => {});"),
        (
            "const thing = 123;\n/* one */\n/* two */\nafterAll(() => {});",
            "const thing = 123;\n\n/* one */\n/* two */\nafterAll(() => {});",
        ),
    ];

    Tester::new(PaddingAroundAfterAllBlocks::NAME, PaddingAroundAfterAllBlocks::PLUGIN, pass, fail)
        .with_jest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
