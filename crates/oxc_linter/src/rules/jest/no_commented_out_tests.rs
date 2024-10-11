use lazy_static::lazy_static;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use regex::Regex;

use crate::{context::LintContext, rule::Rule};

fn no_commented_out_tests_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Some tests seem to be commented")
        .with_help("Remove or uncomment this comment")
        .with_label(span)
}

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
    /// You may forget to uncomment some tests. This rule raises a warning about commented out tests. It's similar to
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
    ///
    /// This rule is compatible with [eslint-plugin-vitest](https://github.com/veritem/eslint-plugin-vitest/blob/main/docs/rules/no-commented-out-tests.md),
    /// to use it, add the following configuration to your `.eslintrc.json`:
    ///
    /// ```json
    /// {
    ///   "rules": {
    ///      "vitest/no-commented-out-tests": "error"
    ///   }
    /// }
    /// ```
    NoCommentedOutTests,
    suspicious
);

impl Rule for NoCommentedOutTests {
    fn run_once(&self, ctx: &LintContext) {
        lazy_static! {
        //  /^\s*[xf]?(test|it|describe)(\.\w+|\[['"]\w+['"]\])?\s*\(/mu
            static ref RE: Regex =
            Regex::new(r#"(?mu)^\s*[xf]?(test|it|describe)(\.\w+|\[['"]\w+['"]\])?\s*\("#).unwrap();
        }
        let comments = ctx.semantic().comments();
        let source_text = ctx.semantic().source_text();
        let commented_tests = comments.iter().filter_map(|comment| {
            let text = comment.span.source_text(source_text);
            if RE.is_match(text) {
                Some(comment.span)
            } else {
                None
            }
        });

        for span in commented_tests {
            ctx.diagnostic(no_commented_out_tests_diagnostic(span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let mut pass = vec![
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

    let mut fail = vec![
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
            r"
              foo()
              /*
                describe('has title but no callback', () => {})
              */
              bar()
            ",
            None,
        ),
    ];

    let pass_vitest = vec![
        r#"// foo("bar", function () {})"#,
        r#"describe("foo", function () {})"#,
        r#"it("foo", function () {})"#,
        r#"describe.only("foo", function () {})"#,
        r#"it.only("foo", function () {})"#,
        r#"it.concurrent("foo", function () {})"#,
        r#"test("foo", function () {})"#,
        r#"test.only("foo", function () {})"#,
        r#"test.concurrent("foo", function () {})"#,
        r"var appliedSkip = describe.skip; appliedSkip.apply(describe)",
        r"var calledSkip = it.skip; calledSkip.call(it)",
        r"({ f: function () {} }).f()",
        r"(a || b).f()",
        r"itHappensToStartWithIt()",
        r"testSomething()",
        r"// latest(dates)",
        r"// TODO: unify with Git implementation from Shipit (?)",
        "#!/usr/bin/env node#",
    ];

    let fail_vitest = vec![
        r"// describe(\'foo\', function () {})\'",
        r#"// test.concurrent("foo", function () {})"#,
        r#"// test["skip"]("foo", function () {})"#,
        r#"// xdescribe("foo", function () {})"#,
        r#"// xit("foo", function () {})"#,
        r#"// fit("foo", function () {})"#,
        r#"
            // test(
            //   "foo", function () {}
            // )
        "#,
    ];

    pass.extend(pass_vitest.into_iter().map(|x| (x, None)));
    fail.extend(fail_vitest.into_iter().map(|x| (x, None)));

    Tester::new(NoCommentedOutTests::NAME, pass, fail)
        .with_jest_plugin(true)
        .with_vitest_plugin(true)
        .test_and_snapshot();
}
