use oxc_macros::declare_oxc_lint;

use crate::{
    context::LintContext,
    rule::Rule,
    rules::{
        PossibleJestNode,
        shared::no_test_prefixes::{DOCUMENTATION, run_on_jest_node},
    },
};

#[derive(Debug, Default, Clone)]
pub struct NoTestPrefixes;

declare_oxc_lint!(NoTestPrefixes, vitest, style, fix, docs = DOCUMENTATION, version = "0.0.7",);

impl Rule for NoTestPrefixes {
    fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        run_on_jest_node(jest_node, ctx);
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let mut pass = vec![
        ("describe('foo', function () {})", None),
        ("it('foo', function () {})", None),
        ("it.concurrent('foo', function () {})", None),
        ("test('foo', function () {})", None),
        ("test.concurrent('foo', function () {})", None),
        ("describe.only('foo', function () {})", None),
        ("it.only('foo', function () {})", None),
        ("it.each()('foo', function () {})", None),
        ("it.each``('foo', function () {})", None),
        ("test.only('foo', function () {})", None),
        ("test.each()('foo', function () {})", None),
        ("test.each``('foo', function () {})", None),
        ("describe.skip('foo', function () {})", None),
        ("it.skip('foo', function () {})", None),
        ("test.skip('foo', function () {})", None),
        ("foo()", None),
        ("[1,2,3].forEach()", None),
    ];

    let mut fail = vec![
        ("fdescribe('foo', function () {})", None),
        ("xdescribe.each([])('foo', function () {})", None),
        ("fit('foo', function () {})", None),
        ("xdescribe('foo', function () {})", None),
        ("xit('foo', function () {})", None),
        ("xtest('foo', function () {})", None),
        ("xit.each``('foo', function () {})", None),
        ("xtest.each``('foo', function () {})", None),
        ("xit.each([])('foo', function () {})", None),
        ("xtest.each([])('foo', function () {})", None),
        (
            "
                import { xit } from 'vitest';
                xit('foo', function () {})
            ",
            None,
        ),
        (
            "
                import { xit as skipThis } from 'vitest';
                skipThis('foo', function () {})
            ",
            None,
        ),
        (
            "
                import { fit as onlyThis } from 'vitest';
                onlyThis('foo', function () {})
            ",
            None,
        ),
    ];

    let pass_vitest = vec![
        ("describe(\"foo\", function () {})", None),
        ("it(\"foo\", function () {})", None),
        ("it.concurrent(\"foo\", function () {})", None),
        ("test(\"foo\", function () {})", None),
        ("test.concurrent(\"foo\", function () {})", None),
        ("describe.only(\"foo\", function () {})", None),
        ("it.only(\"foo\", function () {})", None),
        ("it.each()(\"foo\", function () {})", None),
    ];

    let fail_vitest = vec![
        ("fdescribe(\"foo\", function () {})", None),
        ("xdescribe.each([])(\"foo\", function () {})", None),
        ("fit(\"foo\", function () {})", None),
        ("xdescribe(\"foo\", function () {})", None),
        ("xit(\"foo\", function () {})", None),
        ("xtest(\"foo\", function () {})", None),
        ("xit.each``(\"foo\", function () {})", None),
        ("xtest.each``(\"foo\", function () {})", None),
        ("xit.each([])(\"foo\", function () {})", None),
        ("xtest.each([])(\"foo\", function () {})", None),
    ];

    pass.extend(pass_vitest);
    fail.extend(fail_vitest);

    let fix = vec![
        ("xdescribe('foo', () => {})", "describe.skip('foo', () => {})"),
        ("fdescribe('foo', () => {})", "describe.only('foo', () => {})"),
        ("xtest('foo', () => {})", "test.skip('foo', () => {})"),
        // NOTE(@DonIsaac): is this intentional?
        // ("ftest('foo', () => {})", "test.only('foo', () => {})"),
        ("xit('foo', () => {})", "it.skip('foo', () => {})"),
        ("fit('foo', () => {})", "it.only('foo', () => {})"),
    ];

    Tester::new(NoTestPrefixes::NAME, NoTestPrefixes::PLUGIN, pass, fail)
        .with_vitest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
