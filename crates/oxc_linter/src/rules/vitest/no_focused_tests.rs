use oxc_macros::declare_oxc_lint;

use crate::{
    context::LintContext,
    rule::Rule,
    rules::shared::no_focused_tests::{DOCUMENTATION, run},
    utils::PossibleJestNode,
};

#[derive(Debug, Default, Clone)]
pub struct NoFocusedTests;

declare_oxc_lint!(
    NoFocusedTests,
    vitest,
    correctness,
    fix,
    docs = DOCUMENTATION,
    version = "0.0.8",
);

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

    let mut pass = vec![
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
        ("fdescribe()", None),
        ("fit()", None),
        ("fit.each()()", None),
        ("fit.each`table`()", None),
    ];

    let mut fail = vec![
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
    ];

    let mut fix = vec![
        ("describe.only('foo', () => {})", "describe('foo', () => {})", None),
        ("describe['only']('foo', () => {})", "describe('foo', () => {})", None),
    ];

    let pass_vitest = vec![
        (r#"it("test", () => {});"#, None),
        (r#"describe("test group", () => {});"#, None),
        (r#"it("test", () => {});"#, None),
        (r#"describe("test group", () => {});"#, None),
    ];

    let fail_vitest = vec![
        (
            r#"
            import { it } from 'vitest';
            it.only("test", () => {});
            "#,
            None,
        ),
        (r#"describe.only("test", () => {});"#, None),
        (r#"test.only("test", () => {});"#, None),
        (r#"it.only.each([])("test", () => {});"#, None),
        (r#"test.only.each``("test", () => {});"#, None),
        (r#"it.only.each``("test", () => {});"#, None),
    ];

    let fix_vitest = vec![
        (r#"it.only("test", () => {});"#, r#"it("test", () => {});"#, None),
        (r#"describe.only("test", () => {});"#, r#"describe("test", () => {});"#, None),
        (r#"test.only("test", () => {});"#, r#"test("test", () => {});"#, None),
        (r#"it.only.each([])("test", () => {});"#, r#"it.each([])("test", () => {});"#, None),
        (r#"test.only.each``("test", () => {});"#, r#"test.each``("test", () => {});"#, None),
        (r#"it.only.each``("test", () => {});"#, r#"it.each``("test", () => {});"#, None),
    ];

    pass.extend(pass_vitest);
    fail.extend(fail_vitest);
    fix.extend(fix_vitest);

    Tester::new(NoFocusedTests::NAME, NoFocusedTests::PLUGIN, pass, fail)
        .with_vitest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
