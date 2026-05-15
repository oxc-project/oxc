use oxc_macros::declare_oxc_lint;

use crate::{
    context::LintContext,
    rule::Rule,
    rules::shared::padding_around_after_all_blocks::{DOCUMENTATION, run},
    utils::PossibleJestNode,
};

#[derive(Debug, Default, Clone)]
pub struct PaddingAroundAfterAllBlocks;

declare_oxc_lint!(
    PaddingAroundAfterAllBlocks,
    jest,
    style,
    fix,
    docs = DOCUMENTATION,
    version = "1.59.0",
);

impl Rule for PaddingAroundAfterAllBlocks {
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
