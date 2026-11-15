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
            AstKind::ReturnStatement(_) => {}
            _ => return,
        }
        run(node, ctx);
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

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
        (
            "
                function myHelper() {
                    return expect(1).toBe(1);
                }
            ",
            None,
        ),
        (
            "
                it('one', () => {
                    function myHelper() {
                        return expect(1).toBe(1);
                    }
                });
            ",
            None,
        ),
        (
            r"const test = (name, callback) => callback();
            test('foo', () => { return expect(1).toBe(1); });",
            None,
        ),
        (
            r"import { test as baseTest } from 'other';
            const customTest = baseTest.extend({});
            customTest('foo', () => { return expect(1).toBe(1); });",
            None,
        ),
        (
            r"import { test as baseTest } from 'vitest';
            const customTest = baseTest.extend({});
            customTest('foo', () => {
                function helper() {
                    return expect(1).toBe(1);
                }
                helper();
            });",
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
        (
            r"import { test as baseTest } from 'vitest';
            const extendedTest = baseTest.extend({});
            const customTest = extendedTest.extend({});
            customTest.only('foo', () => {
                return expect(1).toBe(1);
            });",
            None,
        ),
        (
            r"import { it as baseTest } from 'vitest';
            const customTest = baseTest.extend({});
            function callback() {
                return expect(1).toBe(1);
            }
            customTest.each([1])('foo', callback);",
            None,
        ),
    ];

    Tester::new(NoTestReturnStatement::NAME, NoTestReturnStatement::PLUGIN, pass, fail)
        .with_vitest_plugin(true)
        .test_and_snapshot();
}
