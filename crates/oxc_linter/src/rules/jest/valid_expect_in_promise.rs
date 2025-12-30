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

    let pass = vec![
        r#"
            it('passes', async () => {
                await somePromise().then(data => {
                    expect(data).toBe('foo');
                });
            });
        "#,
        r#"
            it('passes', () => {
                return somePromise().then(data => {
                    expect(data).toBe('foo');
                });
            });
        "#,
        r#"
            it('passes', () => somePromise().then(data => expect(data).toBe('foo')));
        "#,
        r#"
            it('passes', async () => {
                await somePromise().catch(err => {
                    expect(err).toBeInstanceOf(Error);
                });
            });
        "#,
        r#"
            it('passes', () => {
                return somePromise().catch(err => {
                    expect(err).toBeInstanceOf(Error);
                });
            });
        "#,
        r#"
            it('passes', async () => {
                await somePromise().finally(() => {
                    expect(cleanup).toHaveBeenCalled();
                });
            });
        "#,
        r#"
            it('passes', async () => {
                await Promise.all([
                    somePromise().then(data => expect(data).toBe('foo')),
                    otherPromise().then(data => expect(data).toBe('bar'))
                ]);
            });
        "#,
        r#"
            it('passes', () => {
                return Promise.all([
                    somePromise().then(data => expect(data).toBe('foo')),
                    otherPromise().then(data => expect(data).toBe('bar'))
                ]);
            });
        "#,
        r#"
            it('passes', async () => {
                await Promise.race([
                    somePromise().then(data => expect(data).toBe('foo')),
                    otherPromise().then(data => expect(data).toBe('bar'))
                ]);
            });
        "#,
        r#"
            it('passes', async () => {
                await Promise.allSettled([
                    somePromise().then(data => expect(data).toBe('foo'))
                ]);
            });
        "#,
        r#"
            it('passes', async () => {
                await Promise.any([
                    somePromise().then(data => expect(data).toBe('foo'))
                ]);
            });
        "#,
        r#"
            it('passes', async () => {
                const promise = somePromise().then(data => {
                    expect(data).toBe('foo');
                });
                await promise;
            });
        "#,
        r#"
            it('passes', () => {
                const promise = somePromise().then(data => {
                    expect(data).toBe('foo');
                });
                return promise;
            });
        "#,
        r#"
            it('passes', () => {
                somePromise().then(data => {
                    console.log(data);
                });
                expect(true).toBe(true);
            });
        "#,
        r#"
            it('passes', async () => {
                await somePromise()
                    .then(data => data.json())
                    .then(json => {
                        expect(json).toHaveProperty('foo');
                    });
            });
        "#,
        r#"
            it('passes', () => {
                return somePromise()
                    .then(data => {
                        expect(data).toBe('foo');
                    })
                    .catch(err => {
                        expect(err).toBeInstanceOf(Error);
                    });
            });
        "#,
        r#"
            it('passes', async () => {
                await Promise.resolve().then(() => {
                    expect(true).toBe(true);
                });
            });
        "#,
        r#"
            it('passes', () => {
                expect(true).toBe(true);
            });
        "#,
        r#"
            it('passes', async () => {
                await expect(somePromise()).resolves.toBe('foo');
            });
        "#,
        r#"
            it('passes', async () => {
                await expect(somePromise()).rejects.toThrow();
            });
        "#,
    ];

    let fail = vec![
        r#"
            it('fails', () => {
                somePromise().then(data => {
                    expect(data).toBe('foo');
                });
            });
        "#,
        r#"
            it('fails', () => {
                somePromise().catch(err => {
                    expect(err).toBeInstanceOf(Error);
                });
            });
        "#,
        r#"
            it('fails', () => {
                somePromise().finally(() => {
                    expect(cleanup).toHaveBeenCalled();
                });
            });
        "#,
        r#"
            it('fails', () => {
                Promise.all([
                    somePromise().then(data => expect(data).toBe('foo'))
                ]);
            });
        "#,
        r#"
            it('fails', () => {
                Promise.race([
                    somePromise().then(data => expect(data).toBe('foo'))
                ]);
            });
        "#,
        r#"
            it('fails', () => {
                const promise = somePromise().then(data => {
                    expect(data).toBe('foo');
                });
            });
        "#,
        r#"
            it('fails', () => {
                somePromise().then(data => {
                    expect(data).toBe('foo');
                });
                otherPromise().then(data => {
                    expect(data).toBe('bar');
                });
            });
        "#,
        r#"
            it('fails', () => {
                somePromise()
                    .then(data => data.json())
                    .then(json => {
                        expect(json).toHaveProperty('foo');
                    });
            });
        "#,
        r#"
            it('fails', async () => {
                somePromise().then(data => {
                    expect(data).toBe('foo');
                });
            });
        "#,
        r#"
            it('fails', () => {
                Promise.resolve().then(() => {
                    expect(true).toBe(true);
                });
            });
        "#,
        r#"
            it('fails', () => {
                const promise = somePromise().then(data => {
                    expect(data).toBe('foo');
                });
                promise.then(() => {
                    expect(true).toBe(true);
                });
            });
        "#,
    ];

    Tester::new(ValidExpectInPromise::NAME, ValidExpectInPromise::PLUGIN, pass, fail)
        .with_jest_plugin(true)
        .test_and_snapshot();
}
