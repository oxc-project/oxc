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
pub struct PaddingAroundBeforeEachBlocks;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule enforces a line of padding before and after 1 or more
    /// `beforeEach` statements.
    ///
    /// ### Why is this bad?
    ///
    /// Inconsistent formatting of code can make the code more difficult to read
    /// and follow. This rule helps ensure that `beforeEach` blocks are visually
    /// separated from the rest of the code, making them easier to identify while
    /// looking through test files.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// const thing = 123;
    /// beforeEach(() => {});
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const thing = 123;
    ///
    /// beforeEach(() => {});
    /// ```
    PaddingAroundBeforeEachBlocks,
    jest,
    style,
    fix,
    version = "1.62.0",
);

impl Rule for PaddingAroundBeforeEachBlocks {
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
        if name != "beforeEach" {
            return;
        }
        report_missing_padding_before_jest_block(node, ctx, name);
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "beforeEach(() => {});",
        "const thing = 123;\n\nbeforeEach(() => {});",
        "describe('foo', () => {\nbeforeEach(() => {});\n});",
        "const thing = 123;\n\n/* one */\n/* two */\nbeforeEach(() => {});",
        "afterEach(() => {});\n\nbeforeEach(() => {});",
        "describe('foo', () => {\n  beforeAll(() => {});\n\n  beforeEach(() => {});\n});",
    ];

    let fail = vec![
        "const thing = 123;\nbeforeEach(() => {});",
        "const thing = 123;\n/* one */\n/* two */\nbeforeEach(() => {});",
        "afterEach(() => {});\nbeforeEach(() => {});",
        "describe('foo', () => {\n  beforeAll(() => {});\n  beforeEach(() => {});\n});",
    ];

    let fix = vec![
        (
            "const thing = 123;\nbeforeEach(() => {});",
            "const thing = 123;\n\nbeforeEach(() => {});",
        ),
        (
            "const thing = 123;\n/* one */\n/* two */\nbeforeEach(() => {});",
            "const thing = 123;\n\n/* one */\n/* two */\nbeforeEach(() => {});",
        ),
        (
            "afterEach(() => {});\nbeforeEach(() => {});",
            "afterEach(() => {});\n\nbeforeEach(() => {});",
        ),
    ];

    Tester::new(
        PaddingAroundBeforeEachBlocks::NAME,
        PaddingAroundBeforeEachBlocks::PLUGIN,
        pass,
        fail,
    )
    .with_jest_plugin(true)
    .expect_fix(fix)
    .test_and_snapshot();
}
