use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use regex::Regex;

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Error, Diagnostic)]
#[error("Some tests seem to be commented")]
#[diagnostic(severity(warning), help("Remove or uncomment this comment"))]
struct NoCommentedOutTestsDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoCommentedOutTests;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule raises a warning about commented out tests. It's similar to
    /// no-disabled-tests rule.
    ///
    /// ### Why is this bad?
    ///
    /// You may forget to uncomment the test. This rule raises a warning about commented out tests. It's similar to
    /// no-disabled-tests rule.
    ///
    /// ### Example
    ///
    /// ```javascript
    /// // describe('foo', () => {});
    /// // it('foo', () => {});
    /// // test('foo', () => {});
    ///
    /// // describe.skip('foo', () => {});
    /// // it.skip('foo', () => {});
    /// // test.skip('foo', () => {});
    /// ```
    NoCommentedOutTests,
    suspicious
);

impl Rule for NoCommentedOutTests {
    fn run_once(&self, ctx: &LintContext) {
        let comments = ctx.semantic().trivias().comments();
        let source_text = ctx.semantic().source_text();
        //  /^\s*[xf]?(test|it|describe)(\.\w+|\[['"]\w+['"]\])?\s*\(/mu
        let re =
            Regex::new(r#"(?mu)^\s*[xf]?(test|it|describe)(\.\w+|\[['"]\w+['"]\])?\s*\("#).unwrap();
        let comment_with_test_list = comments.iter().filter_map(|(start, comment)| {
            let start = *start;
            let end = comment.end();
            let comment_text = &source_text[(start as usize)..(end as usize)];
            if re.is_match(comment_text) {
                Some(Span::new(start, end))
            } else {
                None
            }
        });

        for span in comment_with_test_list {
            ctx.diagnostic(NoCommentedOutTestsDiagnostic(span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("// foo('bar', function () {})", None),
        ("describe('foo', function () {})", None),
        ("it('foo', function () {})", None),
        ("describe.only('foo', function () {})", None),
        ("it.only('foo', function () {})", None),
        ("it.concurrent('foo', function () {})", None),
        ("test('foo', function () {})", None),
        ("test.only('foo', function () {})", None),
        ("test.concurrent('foo', function () {})", None),
        ("var appliedSkip = describe.skip; appliedSkip.apply(describe)", None),
        ("var calledSkip = it.skip; calledSkip.call(it)", None),
        ("({ f: function () {} }).f()", None),
        ("(a || b).f()", None),
        ("itHappensToStartWithIt()", None),
        ("testSomething()", None),
        ("// latest(dates)", None),
        ("// TODO: unify with Git implementation from Shipit (?)", None),
        ("#!/usr/bin/env node", None),
        (
            r#"
              import { pending } from "actions"
              test("foo", () => { expect(pending()).toEqual({}) })
            "#,
            None,
        ),
        (
            r#"
              const { pending } = require("actions")
              test("foo", () => { expect(pending()).toEqual({}) })
            "#,
            None,
        ),
        (
            r#"
              test("foo", () => {
                const pending = getPending()
                expect(pending()).toEqual({})
              }) 
            "#,
            None,
        ),
        (
            r#"
              test("foo", () => { expect(pending()).toEqual({}) })
              function pending() { return {} } 
            "#,
            None,
        ),
    ];

    let fail = vec![
        ("// fdescribe('foo', function () {})", None),
        ("// describe['skip']('foo', function () {})", None),
        ("// describe['skip']('foo', function () {})", None),
        ("// it.skip('foo', function () {})", None),
        ("// it.only('foo', function () {})", None),
        ("// it.concurrent('foo', function () {})", None),
        ("// it['skip']('foo', function () {})", None),
        ("// test.skip('foo', function () {})", None),
        ("// test.concurrent('foo', function () {})", None),
        ("// test['skip']('foo', function () {})", None),
        ("// xdescribe('foo', function () {})", None),
        ("// xit('foo', function () {})", None),
        ("// fit('foo', function () {})", None),
        ("// xtest('foo', function () {})", None),
        (
            r#"
              // test(
              //   "foo", function () {}
              // )
            "#,
            None,
        ),
        (
            r#"
              /* test
                (
                  "foo", function () {}
                )
              */
            "#,
            None,
        ),
        ("// it('has title but no callback')", None),
        ("// it()", None),
        ("// test.someNewMethodThatMightBeAddedInTheFuture()", None),
        ("// test['someNewMethodThatMightBeAddedInTheFuture']()", None),
        ("// test('has title but no callback')", None),
        (
            r#"
        	        foo()
        	        /*
        	          describe('has title but no callback', () => {})
        	        */
        	        bar()
        	      "#,
            None,
        ),
    ];

    Tester::new(NoCommentedOutTests::NAME, pass, fail).test_and_snapshot();
}
