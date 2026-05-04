use oxc_macros::declare_oxc_lint;

use crate::{
    context::LintContext,
    rule::Rule,
    rules::shared::no_focused_tests::{DOCUMENTATION, run},
    utils::PossibleJestNode,
};

#[derive(Debug, Default, Clone)]
pub struct NoFocusedTests;

declare_oxc_lint!(NoFocusedTests, jest, correctness, fix, docs = DOCUMENTATION, version = "0.0.8",);

impl Rule for NoFocusedTests {
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
        ("describe()", None),
        ("it()", None),
        ("describe.skip()", None),
        ("it.skip()", None),
        ("test()", None),
        ("test.skip()", None),
        ("var appliedOnly = describe.only; appliedOnly.apply(describe)", None),
        ("var calledOnly = it.only; calledOnly.call(it)", None),
        ("it.each()()", None),
        ("it.each`table`()", None),
        ("test.each()()", None),
        ("test.each`table`()", None),
        ("test.concurrent()", None),
    ];

    let fail = vec![
        ("describe.only()", None),
        // TODO: this need set setting like `settings: { jest: { globalAliases: { describe: ['context'] } } },`
        // ("context.only()", None),
        ("describe.only.each()()", None),
        ("describe.only.each`table`()", None),
        ("describe[\"only\"]()", None),
        ("it.only()", None),
        ("it.concurrent.only.each``()", None),
        ("it.only.each()()", None),
        ("it.only.each`table`()", None),
        ("it[\"only\"]()", None),
        ("test.only()", None),
        ("test.concurrent.only.each()()", None),
        ("test.only.each()()", None),
        ("test.only.each`table`()", None),
        ("test[\"only\"]()", None),
        ("fdescribe()", None),
        ("fit()", None),
        ("fit.each()()", None),
        ("fit.each`table`()", None),
    ];

    let fix = vec![
        ("describe.only('foo', () => {})", "describe('foo', () => {})", None),
        ("describe['only']('foo', () => {})", "describe('foo', () => {})", None),
        ("fdescribe('foo', () => {})", "describe('foo', () => {})", None),
    ];

    Tester::new(NoFocusedTests::NAME, NoFocusedTests::PLUGIN, pass, fail)
        .with_jest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
