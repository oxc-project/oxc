use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn valid_expect_in_promise_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Promise containing expect was not returned or awaited")
        .with_help("Return or await the promise to ensure the expects in its chain are called")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct ValidExpectInPromise;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Ensures that promises containing `expect` assertions are properly returned or awaited
    /// in test functions.
    ///
    /// ### Why is this bad?
    ///
    /// When a promise containing `expect` calls in its `.then()`, `.catch()`, or `.finally()`
    /// callbacks is not returned or awaited, the test may complete before the assertions run.
    /// This can lead to tests that pass even when the assertions would fail, giving false
    /// confidence in the code being tested.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// it('promises a person', () => {
    ///   api.getPersonByName('bob').then(person => {
    ///     expect(person).toHaveProperty('name', 'Bob');
    ///   });
    /// });
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// it('promises a person', async () => {
    ///   await api.getPersonByName('bob').then(person => {
    ///     expect(person).toHaveProperty('name', 'Bob');
    ///   });
    /// });
    ///
    /// it('promises a person', () => {
    ///   return api.getPersonByName('bob').then(person => {
    ///     expect(person).toHaveProperty('name', 'Bob');
    ///   });
    /// });
    /// ```
    ValidExpectInPromise,
    jest,
    correctness,
    pending
);

impl Rule for ValidExpectInPromise {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {}
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![];

    let fail = vec![];

    Tester::new(ValidExpectInPromise::NAME, ValidExpectInPromise::PLUGIN, pass, fail)
        .test_and_snapshot();
}
