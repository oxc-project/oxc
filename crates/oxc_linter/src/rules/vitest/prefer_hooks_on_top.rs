use oxc_macros::declare_oxc_lint;

use crate::{
    context::LintContext,
    rule::Rule,
    rules::shared::prefer_hooks_on_top::{DOCUMENTATION, run_once},
};

#[derive(Debug, Default, Clone)]
pub struct PreferHooksOnTop;

declare_oxc_lint!(PreferHooksOnTop, vitest, style, docs = DOCUMENTATION, version = "0.4.2",);

impl Rule for PreferHooksOnTop {
    fn run_once(&self, ctx: &LintContext) {
        run_once(ctx);
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "
                describe('foo', () => {
                    beforeEach(() => {});
                    someSetupFn();
                    afterEach(() => {});

                    test('bar', () => {
                        someFn();
                    });
                });
            ",
            None,
        ),
        (
            "
                describe('foo', () => {
                    someSetupFn();
                    beforeEach(() => {});
                    afterEach(() => {});

                    test('bar', () => {
                        someFn();
                    });
                });
            ",
            None,
        ),
        (
            "
                describe.skip('foo', () => {
                    beforeEach(() => {});
                    beforeAll(() => {});

                    test('bar', () => {
                        someFn();
                    });
                });

                describe('foo', () => {
                    beforeEach(() => {});

                    test('bar', () => {
                        someFn();
                    });
                });
            ",
            None,
        ),
        (
            "
                describe('foo', () => {
                    beforeEach(() => {});
                    test('bar', () => {
                        someFn();
                    });

                    describe('inner_foo', () => {
                        beforeEach(() => {});
                        test('inner bar', () => {
                            someFn();
                        });
                    });
                });
            ",
            None,
        ),
    ];

    let fail = vec![
        (
            "
                describe('foo', () => {
                    beforeEach(() => {});
                    test('bar', () => {
                        someFn();
                    });

                    beforeAll(() => {});
                    test('bar', () => {
                        someFn();
                    });
                });
            ",
            None,
        ),
        (
            "
                describe('foo', () => {
                    beforeEach(() => {});
                    test.each``('bar', () => {
                        someFn();
                    });

                    beforeAll(() => {});
                    test.only('bar', () => {
                        someFn();
                    });
                });
            ",
            None,
        ),
        (
            "
                describe('foo', () => {
                    beforeEach(() => {});
                    test.only.each``('bar', () => {
                        someFn();
                    });

                    beforeAll(() => {});
                    test.only('bar', () => {
                        someFn();
                    });
                });
            ",
            None,
        ),
        (
            "
                describe.skip('foo', () => {
                    beforeEach(() => {});
                    test('bar', () => {
                        someFn();
                    });

                    beforeAll(() => {});
                    test('bar', () => {
                        someFn();
                    });
                });
                describe('foo', () => {
                    beforeEach(() => {});
                    beforeEach(() => {});
                    beforeAll(() => {});

                    test('bar', () => {
                        someFn();
                    });
                });

                describe('foo', () => {
                    test('bar', () => {
                        someFn();
                    });

                    beforeEach(() => {});
                    beforeEach(() => {});
                    beforeAll(() => {});
                });
            ",
            None,
        ),
        (
            "
                describe('foo', () => {
                    beforeAll(() => {});
                    test('bar', () => {
                        someFn();
                    });

                    describe('inner_foo', () => {
                        beforeEach(() => {});
                        test('inner bar', () => {
                            someFn();
                        });

                        test('inner bar', () => {
                            someFn();
                        });

                        beforeAll(() => {});
                        afterAll(() => {});
                        test('inner bar', () => {
                            someFn();
                        });
                    });
                });
            ",
            None,
        ),
    ];

    Tester::new(PreferHooksOnTop::NAME, PreferHooksOnTop::PLUGIN, pass, fail)
        .with_vitest_plugin(true)
        .test_and_snapshot();
}
