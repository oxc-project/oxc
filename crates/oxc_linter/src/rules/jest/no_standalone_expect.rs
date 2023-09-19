use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("")]
#[diagnostic(severity(warning), help(""))]
struct NoStandaloneExpectDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoStandaloneExpect;

declare_oxc_lint!(
    /// ### What it does
    ///
    ///
    /// ### Why is this bad?
    ///
    ///
    /// ### Example
    /// ```javascript
    /// ```
    NoStandaloneExpect,
    correctness
);

impl Rule for NoStandaloneExpect {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {}
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("expect.any(String)", None),
("expect.extend({})", None),
("describe(\"a test\", () => { it(\"an it\", () => {expect(1).toBe(1); }); });", None),
("describe(\"a test\", () => { it(\"an it\", () => { const func = () => { expect(1).toBe(1); }; }); });", None),
("describe(\"a test\", () => { const func = () => { expect(1).toBe(1); }; });", None),
("describe(\"a test\", () => { function func() { expect(1).toBe(1); }; });", None),
("describe(\"a test\", () => { const func = function(){ expect(1).toBe(1); }; });", None),
("it(\"an it\", () => expect(1).toBe(1))", None),
("const func = function(){ expect(1).toBe(1); };", None),
("const func = () => expect(1).toBe(1);", None),
("{}", None),
("it.each([1, true])(\"trues\", value => { expect(value).toBe(true); });", None),
("it.each([1, true])(\"trues\", value => { expect(value).toBe(true); }); it(\"an it\", () => { expect(1).toBe(1) });", None),
("
				it.each`
					num   | value
					${1} | ${true}
				`('trues', ({ value }) => {
					expect(value).toBe(true);
				});
				", None),
("it.only(\"an only\", value => { expect(value).toBe(true); });", None),
("it.concurrent(\"an concurrent\", value => { expect(value).toBe(true); });", None),
("describe.each([1, true])(\"trues\", value => { it(\"an it\", () => expect(value).toBe(true) ); });", None),
("
					describe('scenario', () => {
					const t = Math.random() ? it.only : it;
					t('testing', () => expect(true));
					});
				", Some(serde_json::json!([{ "additionalTestBlockFunctions": ['t'] }]))),
(r#"
					each([
					[1, 1, 2],
					[1, 2, 3],
					[2, 1, 3],
					]).test('returns the result of adding %d to %d', (a, b, expected) => {
					expect(a + b).toBe(expected);
					});
			"#, Some(serde_json::json!([{ "additionalTestBlockFunctions": ["each.test"] }])))
    ];

    let fail = vec![
        ("(() => {})('testing', () => expect(true).toBe(false))", None),
("expect.hasAssertions()", None),
("expect().hasAssertions()", None),
("
			        describe('scenario', () => {
			          const t = Math.random() ? it.only : it;
			          t('testing', () => expect(true).toBe(false));
			        });
			      ", None),
("
			        describe('scenario', () => {
			          const t = Math.random() ? it.only : it;
			          t('testing', () => expect(true).toBe(false));
			        });
			      ", Some(serde_json::json!([{ "additionalTestBlockFunctions": "undefined" }]))),
("
			        each([
			          [1, 1, 2],
			          [1, 2, 3],
			          [2, 1, 3],
			        ]).test('returns the result of adding %d to %d', (a, b, expected) => {
			          expect(a + b).toBe(expected);
			        });
			      ", None),
("
			        each([
			          [1, 1, 2],
			          [1, 2, 3],
			          [2, 1, 3],
			        ]).test('returns the result of adding %d to %d', (a, b, expected) => {
			          expect(a + b).toBe(expected);
			        });
			      ", Some(serde_json::json!([{ "additionalTestBlockFunctions": ["each"] }]))),
("
			        each([
			          [1, 1, 2],
			          [1, 2, 3],
			          [2, 1, 3],
			        ]).test('returns the result of adding %d to %d', (a, b, expected) => {
			          expect(a + b).toBe(expected);
			        });
			      ", Some(serde_json::json!([{ "additionalTestBlockFunctions": ["test"] }]))),
("describe(\"a test\", () => { expect(1).toBe(1); });", None),
("describe(\"a test\", () => expect(1).toBe(1));", None),
("describe(\"a test\", () => { const func = () => { expect(1).toBe(1); }; expect(1).toBe(1); });", None),
("describe(\"a test\", () => {  it(() => { expect(1).toBe(1); }); expect(1).toBe(1); });", None),
("expect(1).toBe(1);", None),
("{expect(1).toBe(1)}", None),
("it.each([1, true])(\"trues\", value => { expect(value).toBe(true); }); expect(1).toBe(1);", None),
("describe.each([1, true])(\"trues\", value => { expect(value).toBe(true); });", None),
("
			        import { expect as pleaseExpect } from '@jest/globals';
			
			        describe(\"a test\", () => { pleaseExpect(1).toBe(1); });
			      ", None),
("
			        import { expect as pleaseExpect } from '@jest/globals';
			
			        beforeEach(() => pleaseExpect.hasAssertions());
			      ", None)
    ];

    Tester::new(NoStandaloneExpect::NAME, pass, fail).test_and_snapshot();
}
