use oxc_macros::declare_oxc_lint;

use crate::rules::shared::padding_around_after_each_blocks::{DOCUMENTATION, run};
use crate::{
    context::LintContext,
    rule::Rule,
};
use crate::rules::PossibleJestNode;

#[derive(Debug, Default, Clone)]
pub struct PaddingAroundAfterEachBlocks;

declare_oxc_lint!(
    PaddingAroundAfterEachBlocks,
    vitest,
    style,
    fix,
    docs = DOCUMENTATION,
    short_description = "Enforce padding around afterEach blocks.",
    version = "next",
);

impl Rule for PaddingAroundAfterEachBlocks {
    fn run_on_jest_node<'a, 'c>(
        &self,
        possible_jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        run(possible_jest_node, ctx);
    }}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "const something = 123;\n\nafterEach(() => {\n//  // more stuff\n});\n\ndescribe('foo', () => {});",
        "const something = 123;\n\nafterEach(() => {\n//  // more stuff\n});",
        "const something = 123;\n\nafterEach(() => {\n//  // more stuff\n})\n\ndescribe('foo', () => {});",
    ];

    let fail = vec![
        "const something = 123;\nafterEach(() => {\n//  // more stuff\n});\ndescribe('foo', () => {});",
        "const something = 123;\n\nafterEach(() => {\n//  // more stuff\n});\ndescribe('foo', () => {});",
        "const something = 123;\nafterEach(() => {\n//  // more stuff\n});\n\ndescribe('foo', () => {});",
    ];

    let fix = vec![
        (
            "const something = 123;\nafterEach(() => {\n//  // more stuff\n});\ndescribe('foo', () => {});",
            "const something = 123;\n\nafterEach(() => {\n//  // more stuff\n});\n\ndescribe('foo', () => {});",
        ),
        (
            "const something = 123;\n\nafterEach(() => {\n//  // more stuff\n})\ndescribe('foo', () => {});",
            "const something = 123;\n\nafterEach(() => {\n//  // more stuff\n})\n\ndescribe('foo', () => {});",
        ),
        (
            "
            const someText = 'abc';
            afterEach(() => {
            });
            describe('someText', () => {
              const something = '123';
              // A comment
              afterEach(() => {
                // stuff
              });
              afterEach(() => {
                // other stuff
              });
            });
            describe('someText2', () => {
              const something = 'xyz';
              afterEach(() => {
                // more stuff
              });
            });
        ",
            "
            const someText = 'abc';

            afterEach(() => {
            });

            describe('someText', () => {
              const something = '123';

              // A comment
              afterEach(() => {
                // stuff
              });

              afterEach(() => {
                // other stuff
              });
            });
            describe('someText2', () => {
              const something = 'xyz';

              afterEach(() => {
                // more stuff
              });
            });
        ",
        ),
    ];

    Tester::new(
        PaddingAroundAfterEachBlocks::NAME,
        PaddingAroundAfterEachBlocks::PLUGIN,
        pass,
        fail,
    )
        .with_vitest_plugin(true)
        .expect_fix(fix)
    .test_and_snapshot();
}
