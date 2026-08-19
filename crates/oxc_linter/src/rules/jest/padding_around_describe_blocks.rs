use oxc_macros::declare_oxc_lint;

use crate::{
    context::LintContext,
    rule::Rule,
    rules::shared::padding_around_describe_blocks::{DOCUMENTATION, run},
    utils::PossibleJestNode,
};

#[derive(Debug, Default, Clone)]
pub struct PaddingAroundDescribeBlocks;

declare_oxc_lint!(
    PaddingAroundDescribeBlocks,
    jest,
    style,
    fix,
    docs = DOCUMENTATION,
    // TODO: confirm this matches the actual next-unreleased crate version at build time
    // (crates/oxc_linter/Cargo.toml was at 1.79.0 in this checkout).
    version = "1.80.0",
);

impl Rule for PaddingAroundDescribeBlocks {
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
        "describe('foo', () => {});",
        "const thing = 123;\n\ndescribe('foo', () => {});",
        "describe('foo', () => {\ndescribe('bar', () => {});\n});",
        "const thing = 123;\n\n/* one */\n/* two */\ndescribe('foo', () => {});",
        "const thing = 123;\n\nfdescribe('foo', () => {});",
        "const thing = 123;\n\nxdescribe('foo', () => {});",
        "const thing = 123;\n\ndescribe.each([1, 2])('foo %i', () => {});",
        "const thing = 123;\n\ndescribe.only('foo', () => {});",
        "const thing = 123;\n\ndescribe.skip('foo', () => {});",
        "describe('foo', () => {});\n\ndescribe('bar', () => {});",
    ];

    let fail = vec![
        "const thing = 123;\ndescribe('foo', () => {});",
        "const thing = 123;\n/* one */\n/* two */\ndescribe('foo', () => {});",
        "const thing = 123;\nfdescribe('foo', () => {});",
        "const thing = 123;\nxdescribe('foo', () => {});",
        "const thing = 123;\ndescribe.each([1, 2])('foo %i', () => {});",
        "const thing = 123;\ndescribe.only('foo', () => {});",
        "const thing = 123;\ndescribe.skip('foo', () => {});",
        "describe('foo', () => {});\ndescribe('bar', () => {});",
    ];

    let fix = vec![
        (
            "const thing = 123;\ndescribe('foo', () => {});",
            "const thing = 123;\n\ndescribe('foo', () => {});",
        ),
        (
            "const thing = 123;\n/* one */\n/* two */\ndescribe('foo', () => {});",
            "const thing = 123;\n\n/* one */\n/* two */\ndescribe('foo', () => {});",
        ),
        (
            "describe('foo', () => {});\ndescribe('bar', () => {});",
            "describe('foo', () => {});\n\ndescribe('bar', () => {});",
        ),
    ];

    Tester::new(PaddingAroundDescribeBlocks::NAME, PaddingAroundDescribeBlocks::PLUGIN, pass, fail)
        .with_jest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
