use oxc_ast::AstKind;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;

use crate::{
    context::LintContext,
    rule::Rule,
    rules::shared::no_test_return_statement::{DOCUMENTATION, run},
};

#[derive(Debug, Default, Clone)]
pub struct NoTestReturnStatement;

declare_oxc_lint!(NoTestReturnStatement, vitest, style, docs = DOCUMENTATION, version = "0.2.0",);

impl Rule for NoTestReturnStatement {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::CallExpression(_) | AstKind::Function(_) => {}
            _ => return,
        }
        run(node, ctx);
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    // Note: Both Jest and Vitest share the same unit tests

    let pass = vec![
        ("it('noop', () => {});", None),
        ("test('noop', () => {});", None),
        ("test('one', () => expect(1).toBe(1));", None),
        ("test('empty')", None),
        (
            "
                test('one', () => {
                    expect(1).toBe(1);
                });
            ",
            None,
        ),
        (
            "
                it('one', function () {
                    expect(1).toBe(1);
                });
            ",
            None,
        ),
        (
            "
                it('one', myTest);
                function myTest() {
                    expect(1).toBe(1);
                }
            ",
            None,
        ),
        (
            "
                it('one', () => expect(1).toBe(1));
                function myHelper() {}
            ",
            None,
        ),
    ];

    let fail = vec![
        (
            "
                test('one', () => {
                   return expect(1).toBe(1);
                });
            ",
            None,
        ),
        (
            "
                it('one', function () {
                    return expect(1).toBe(1);
                });
            ",
            None,
        ),
        (
            "
                it.skip('one', function () {
                    return expect(1).toBe(1);
                });
            ",
            None,
        ),
        (
            "
                it.each``('one', function () {
                    return expect(1).toBe(1);
                });
            ",
            None,
        ),
        (
            "
                it.each()('one', function () {
                    return expect(1).toBe(1);
                });
            ",
            None,
        ),
        (
            "
                it.only.each``('one', function () {
                    return expect(1).toBe(1);
                });
            ",
            None,
        ),
        (
            "
                it.only.each()('one', function () {
                    return expect(1).toBe(1);
                });
            ",
            None,
        ),
        (
            "
                it('one', myTest);
                function myTest () {
                    return expect(1).toBe(1);
                }
            ",
            None,
        ),
    ];

    Tester::new(NoTestReturnStatement::NAME, NoTestReturnStatement::PLUGIN, pass, fail)
        .with_vitest_plugin(true)
        .test_and_snapshot();
}
