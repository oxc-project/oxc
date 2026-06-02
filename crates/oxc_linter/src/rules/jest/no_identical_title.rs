use oxc_macros::declare_oxc_lint;

use crate::{
    context::LintContext,
    rule::Rule,
    rules::shared::no_identical_title::{DOCUMENTATION, run_once},
};

#[derive(Debug, Default, Clone)]
pub struct NoIdenticalTitle;

declare_oxc_lint!(NoIdenticalTitle, jest, style, docs = DOCUMENTATION, version = "0.0.14",);

impl Rule for NoIdenticalTitle {
    fn run_once(&self, ctx: &LintContext) {
        run_once(ctx);
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("it(); it();", None),
        ("describe(); describe();", None),
        ("describe('foo', () => {}); it('foo', () => {});", None),
        (
            "
              describe('foo', () => {
                it('works', () => {});
              });
            ",
            None,
        ),
        (
            "
              it('one', () => {});
              it('two', () => {});
            ",
            None,
        ),
        (
            "
              describe('foo', () => {});
              describe('foe', () => {});
            ",
            None,
        ),
        (
            "
              it(`one`, () => {});
              it(`two`, () => {});
            ",
            None,
        ),
        (
            "
              describe(`foo`, () => {});
              describe(`foe`, () => {});
            ",
            None,
        ),
        (
            "
              describe('foo', () => {
                test('this', () => {});
                test('that', () => {});
              });
            ",
            None,
        ),
        (
            "
              test.concurrent('this', () => {});
              test.concurrent('that', () => {});
            ",
            None,
        ),
        (
            "
              test.concurrent('this', () => {});
              test.only.concurrent('that', () => {});
            ",
            None,
        ),
        (
            "
              test.only.concurrent('this', () => {});
              test.concurrent('that', () => {});
            ",
            None,
        ),
        (
            "
              test.only.concurrent('this', () => {});
              test.only.concurrent('that', () => {});
            ",
            None,
        ),
        (
            "
              test.only('this', () => {});
              test.only('that', () => {});
            ",
            None,
        ),
        (
            "
              describe('foo', () => {
                it('works', () => {});

                describe('foe', () => {
                  it('works', () => {});
                });
              });
            ",
            None,
        ),
        (
            "
              describe('foo', () => {
                describe('foe', () => {
                  it('works', () => {});
                });

                it('works', () => {});
              });
            ",
            None,
        ),
        ("describe('foo', () => describe('foe', () => {}));", None),
        (
            "
              describe('foo', () => {
                describe('foe', () => {});
              });

              describe('foe', () => {});
            ",
            None,
        ),
        ("test('number' + n, function() {});", None),
        ("test('number' + n, function() {}); test('number' + n, function() {});", None),
        // ("it(`${n}`, function() {});", None),
        // ("it(`${n}`, function() {}); it(`${n}`, function() {});", None),
        (
            "
              describe('a class named ' + myClass.name, () => {
                describe('#myMethod', () => {});
              });

              describe('something else', () => {});
            ",
            None,
        ),
        (
            "
              describe('my class', () => {
                describe('#myMethod', () => {});
                describe('a class named ' + myClass.name, () => {});
              });
            ",
            None,
        ),
        (
            "
              const test = { content: () => 'foo' };
              test.content(`something that is not from jest`, () => {});
              test.content(`something that is not from jest`, () => {});
            ",
            None,
        ),
        (
            "
              const describe = { content: () => 'foo' };
              describe.content(`something that is not from jest`, () => {});
              describe.content(`something that is not from jest`, () => {});
            ",
            None,
        ),
        (
            "
              describe.each`
                description
                ${'b'}
              `('$description', () => {});

              describe.each`
                description
                ${'a'}
              `('$description', () => {});
            ",
            None,
        ),
        (
            "
              describe('top level', () => {
                describe.each``('nested each', () => {
                  describe.each``('nested nested each', () => {});
                });

                describe('nested', () => {});
              });
            ",
            None,
        ),
        (
            "
              describe.each``('my title', value => {});
              describe.each``('my title', value => {});
              describe.each([])('my title', value => {});
              describe.each([])('my title', value => {});
            ",
            None,
        ),
        (
            "
              describe.each([])('when the value is %s', value => {});
              describe.each([])('when the value is %s', value => {});
            ",
            None,
        ),
    ];

    let fail = vec![
        (
            "
              describe('foo', () => {
                it('works', () => {});
                it('works', () => {});
              });
            ",
            None,
        ),
        (
            "
              it('works', () => {});
              it('works', () => {});
            ",
            None,
        ),
        (
            "
              test.only('this', () => {});
              test('this', () => {});
            ",
            None,
        ),
        (
            "
              xtest('this', () => {});
              test('this', () => {});
            ",
            None,
        ),
        (
            "
              test.only('this', () => {});
              test.only('this', () => {});
            ",
            None,
        ),
        (
            "
              test.concurrent('this', () => {});
              test.concurrent('this', () => {});
            ",
            None,
        ),
        (
            "
              test.only('this', () => {});
              test.concurrent('this', () => {});
            ",
            None,
        ),
        (
            "
              describe('foo', () => {});
              describe('foo', () => {});
            ",
            None,
        ),
        (
            "
              describe('foo', () => {});
              xdescribe('foo', () => {});
            ",
            None,
        ),
        (
            "
              fdescribe('foo', () => {});
              describe('foo', () => {});
            ",
            None,
        ),
        (
            "
              describe('foo', () => {
                describe('foe', () => {});
              });
              describe('foo', () => {});
            ",
            None,
        ),
        (
            "
              describe('foo', () => {
                it(`catches backticks with the same title`, () => {});
                it(`catches backticks with the same title`, () => {});
              });
            ",
            None,
        ),
        // (
        //     "
        //       context('foo', () => {
        //         describe('foe', () => {});
        //       });
        //       describe('foo', () => {});
        //     ",
        //     None,
        // ),
    ];

    Tester::new(NoIdenticalTitle::NAME, NoIdenticalTitle::PLUGIN, pass, fail)
        .with_jest_plugin(true)
        .test_and_snapshot();
}
