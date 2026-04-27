use oxc_macros::declare_oxc_lint;

use crate::{
    context::LintContext,
    rule::Rule,
    rules::shared::prefer_todo::{DOCUMENTATION, run},
    utils::PossibleJestNode,
};

#[derive(Debug, Default, Clone)]
pub struct PreferTodo;

declare_oxc_lint!(PreferTodo, vitest, style, fix, docs = DOCUMENTATION, version = "0.0.16",);

impl Rule for PreferTodo {
    fn run_on_jest_node<'a, 'c>(
        &self,
        possible_jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        run(possible_jest_node, ctx);
    }
}

#[test]
fn tests() {
    use crate::tester::Tester;

    let pass = vec![
        ("test()", None),
        ("test.concurrent()", None),
        ("test.todo('i need to write this test correct');", None),
        ("test(obj)", None),
        ("test.concurrent(obj)", None),
        ("fit('foo')", None),
        ("fit.concurrent('foo')", None),
        ("xit('foo')", None),
        ("test('foo', 1)", None),
        ("test('stub', () => expect(1).toBe(1));", None),
        ("test.concurrent('stub', () => expect(1).toBe(1));", None),
        (
            "
                supportsDone && params.length < test.length
                    ? done => test(...params, done)
                    : () => test(...params);
            ",
            None,
        ),
    ];

    let fail = vec![
        ("test('i need to write this test');", None),
        ("test('i need to write this test',);", None),
        ("test(`i need to write this test`);", None),
        ("it('foo', function () {})", None),
        ("it('foo', () => {})", None),
        ("test.skip('i need to write this test', () => {});", None),
        ("test.skip('i need to write this test', function() {});", None),
        (r#"test["skip"]('i need to write this test', function() {});"#, None),
        ("test[`skip`]('i need to write this test', function() {});", None),
    ];

    let fix = vec![
        (
            "test.skip('i need to write this test');",
            "test.todo('i need to write this test');",
            None,
        ),
        ("test('i need to write this test',);", "test.todo('i need to write this test',);", None),
        ("test(`i need to write this test`);", "test.todo(`i need to write this test`);", None),
        ("it.skip('foo', function () {})", "it.todo('foo')", None),
        ("it(`i need to write this test`, () => {})", "it.todo(`i need to write this test`)", None),
        ("it('foo', function () {})", "it.todo('foo')", None),
        ("it('foo', () => {})", "it.todo('foo')", None),
        (
            "test.skip('i need to write this test', () => {});",
            "test.todo('i need to write this test');",
            None,
        ),
        (
            "test.skip('i need to write this test', function() {});",
            "test.todo('i need to write this test');",
            None,
        ),
        (
            "test['skip']('i need to write this test', function() {});",
            "test['todo']('i need to write this test');",
            None,
        ),
        (
            "test['skip']('i need to write this test', () => {});",
            "test['todo']('i need to write this test');",
            None,
        ),
        (
            "test['skip']('i need to write this test');",
            "test['todo']('i need to write this test');",
            None,
        ),
    ];

    Tester::new(PreferTodo::NAME, PreferTodo::PLUGIN, pass, fail)
        .with_vitest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
