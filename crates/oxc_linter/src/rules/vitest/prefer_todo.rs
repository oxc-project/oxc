use oxc_ast::AstKind;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    rules::shared::prefer_todo::{DOCUMENTATION, run},
};

#[derive(Debug, Default, Clone)]
pub struct PreferTodo;

declare_oxc_lint!(PreferTodo, vitest, style, fix, docs = DOCUMENTATION, version = "0.0.16",);

impl Rule for PreferTodo {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(_) = node.kind() else { return };
        run(node, ctx);
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.frameworks().is_test()
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
        // Non-`skip` modifiers and call-chain forms cannot be safely auto-fixed,
        // so they are treated as valid code.
        ("test.only('focused test', () => {});", None),
        ("test.only('focused test', function () {});", None),
        ("test.fails('is expected to fail', () => {});", None),
        ("test.fails('is expected to fail', function () {});", None),
        ("test.concurrent('unimplemented');", None),
        ("test.each([1, 2])('case %i', () => {});", None),
        ("test.runIf(true)('runs on this platform', () => {});", None),
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
        ("test('i need to write this test');", "test.todo('i need to write this test');"),
        ("test('i need to write this test',);", "test.todo('i need to write this test',);"),
        ("test(`i need to write this test`);", "test.todo(`i need to write this test`);"),
        ("it.skip('foo', function () {})", "it.todo('foo')"),
        (
            "test.skip('i need to write this test', () => {});",
            "test.todo('i need to write this test');",
        ),
        (
            "test.skip('i need to write this test', function() {});",
            "test.todo('i need to write this test');",
        ),
        (
            "test['skip']('i need to write this test', function() {});",
            "test['todo']('i need to write this test');",
        ),
        (
            "test['skip']('i need to write this test', () => {});",
            "test['todo']('i need to write this test');",
        ),
        (
            "test['skip']('i need to write this test');",
            "test['todo']('i need to write this test');",
        ),
    ];

    Tester::new(PreferTodo::NAME, PreferTodo::PLUGIN, pass, fail)
        .with_vitest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
