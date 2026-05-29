use oxc_macros::declare_oxc_lint;

use crate::{
    context::LintContext,
    rule::Rule,
    rules::shared::padding_around_before_all_blocks::{DOCUMENTATION, run},
    utils::PossibleJestNode,
};

#[derive(Debug, Default, Clone)]
pub struct PaddingAroundBeforeAllBlocks;

declare_oxc_lint!(
    PaddingAroundBeforeAllBlocks,
    jest,
    style,
    fix,
    docs = DOCUMENTATION,
    version = "1.67.0",
);

impl Rule for PaddingAroundBeforeAllBlocks {
    fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        run(jest_node, ctx);
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "beforeAll(() => {});",
        "const thing = 123;\n\nbeforeAll(() => {});",
        "describe('foo', () => {\nbeforeAll(() => {});\n});",
        "const thing = 123;\n\n/* one */\n/* two */\nbeforeAll(() => {});",
    ];

    let fail = vec![
        "const thing = 123;\nbeforeAll(() => {});",
        "const thing = 123;\n/* one */\n/* two */\nbeforeAll(() => {});",
    ];

    let fix = vec![
        ("const thing = 123;\nbeforeAll(() => {});", "const thing = 123;\n\nbeforeAll(() => {});"),
        (
            "const thing = 123;\n/* one */\n/* two */\nbeforeAll(() => {});",
            "const thing = 123;\n\n/* one */\n/* two */\nbeforeAll(() => {});",
        ),
    ];

    Tester::new(
        PaddingAroundBeforeAllBlocks::NAME,
        PaddingAroundBeforeAllBlocks::PLUGIN,
        pass,
        fail,
    )
    .with_jest_plugin(true)
    .expect_fix(fix)
    .test_and_snapshot();
}
