use oxc_macros::declare_oxc_lint;

use crate::{
    context::LintContext,
    rule::Rule,
    rules::shared::prefer_hooks_in_order::{DOCUMENTATION, run_once},
};

#[derive(Debug, Default, Clone)]
pub struct PreferHooksInOrder;

declare_oxc_lint!(PreferHooksInOrder, vitest, style, docs = DOCUMENTATION, version = "0.6.0",);

impl Rule for PreferHooksInOrder {
    fn run_once(&self, ctx: &LintContext) {
        run_once(ctx);
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let mut pass = vec![
        ("beforeAll(() => {})", None),
        ("beforeEach(() => {})", None),
        ("afterEach(() => {})", None),
        ("afterAll(() => {})", None),
        ("describe(() => {})", None),
        (
            "
                beforeAll(() => {});
                beforeEach(() => {});
                afterEach(() => {});
                afterAll(() => {});
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
                beforeAll(() => {});
                afterAll(() => {});
            ",
            None,
        ),
        (
            "
                beforeEach(() => {});
                afterEach(() => {});
            ",
            None,
        ),
        (
            "
                beforeAll(() => {});
                afterEach(() => {});
            ",
            None,
        ),
        (
            "
                beforeAll(() => {});
                beforeEach(() => {});
            ",
            None,
        ),
        (
            "
                afterEach(() => {});
                afterAll(() => {});
            ",
            None,
        ),
        (
            "
                beforeAll(() => {});
                beforeAll(() => {});
            ",
            None,
        ),
        (
            "
                describe('my test', () => {
                    afterEach(() => {});
                    afterAll(() => {});
                });
            ",
            None,
        ),
        (
            "
                describe('my test', () => {
                    afterAll(() => {});

                    describe('when something is true', () => {
                        beforeAll(() => {});
                        beforeEach(() => {});
                    });
                });
            ",
            None,
        ),
        (
            "
                describe('foo', () => {
                    beforeAll(() => {
                        createMyDatabase();
                    });

                    beforeEach(() => {
                        seedMyDatabase();
                    });

                    it('accepts this input', () => {
                        // ...
                    });

                    it('returns that value', () => {
                        // ...
                    });

                    describe('when the database has specific values', () => {
                        const specificValue = '...';

                        beforeEach(() => {
                            seedMyDatabase(specificValue);
                        });

                        it('accepts that input', () => {
                            // ...
                        });

                        it('throws an error', () => {
                            // ...
                        });

                        beforeEach(() => {
                            mockLogger();
                        });

                        afterEach(() => {
                            clearLogger();
                        });

                        it('logs a message', () => {
                            // ...
                        });
                    });

                    afterAll(() => {
                        removeMyDatabase();
                    });
                });
            ",
            None,
        ),
        (
            "
                describe('my test', () => {
                    afterEach(() => {});
                    afterAll(() => {});

                    doSomething();

                    beforeAll(() => {});
                    beforeEach(() => {});
                });
            ",
            None,
        ),
        (
            "
                describe('my test', () => {
                    afterEach(() => {});
                    afterAll(() => {});

                    it('is a test', () => {});

                    beforeAll(() => {});
                    beforeEach(() => {});
                });
            ",
            None,
        ),
        (
            "
                describe('my test', () => {
                    afterAll(() => {});

                    describe('when something is true', () => {
                        beforeAll(() => {});
                        beforeEach(() => {});

                        it('does something', () => {});

                        beforeAll(() => {});
                        beforeEach(() => {});
                    });

                    beforeAll(() => {});
                    beforeEach(() => {});
                });

                describe('my test', () => {
                    beforeAll(() => {});
                    beforeEach(() => {});
                    afterAll(() => {});

                    describe('when something is true', () => {
                        it('does something', () => {});

                        beforeAll(() => {});
                        beforeEach(() => {});
                    });

                    beforeAll(() => {});
                    beforeEach(() => {});
                });
            ",
            None,
        ),
        (
            "
                const withDatabase = () => {
                    beforeAll(() => {
                        createMyDatabase();
                    });
                    afterAll(() => {
                        removeMyDatabase();
                    });
                };

                describe('my test', () => {
                    withDatabase();

                    afterAll(() => {});

                    describe('when something is true', () => {
                        beforeAll(() => {});
                        beforeEach(() => {});

                        it('does something', () => {});

                        beforeAll(() => {});
                        beforeEach(() => {});
                    });

                    beforeAll(() => {});
                    beforeEach(() => {});
                });

                describe('my test', () => {
                    beforeAll(() => {});
                    beforeEach(() => {});
                    afterAll(() => {});

                    withDatabase();

                    describe('when something is true', () => {
                        it('does something', () => {});

                        beforeAll(() => {});
                        beforeEach(() => {});
                    });

                    beforeAll(() => {});
                    beforeEach(() => {});
                });
            ",
            None,
        ),
        (
            "
                describe('A file with a lot of test', () => {
                    beforeAll(() => {
                        setupTheDatabase();
                        createMocks();
                    });

                    beforeAll(() => {
                        doEvenMore();
                    });

                    beforeEach(() => {
                        cleanTheDatabase();
                        resetSomeThings();
                    });

                    afterEach(() => {
                        cleanTheDatabase();
                        resetSomeThings();
                    });

                    afterAll(() => {
                        closeTheDatabase();
                        stop();
                    });

                    it('does something', () => {
                        const thing = getThing();
                        expect(thing).toBe('something');
                    });

                    it('throws', () => {
                        // Do something that throws
                    });

                    describe('Also have tests in here', () => {
                        afterAll(() => {});
                        it('tests something', () => {});
                        it('tests something else', () => {});
                        beforeAll(()=>{});
                    });
                });
            ",
            None,
        ),
    ];

    let mut fail = vec![
        (
            "
                const withDatabase = () => {
                    afterAll(() => {
                        removeMyDatabase();
                    });
                    beforeAll(() => {
                        createMyDatabase();
                    });
                };
            ",
            None,
        ),
        (
            "
                afterAll(() => {
                    removeMyDatabase();
                });
                beforeAll(() => {
                    createMyDatabase();
                });
            ",
            None,
        ),
        (
            "
                afterAll(() => {});
                beforeAll(() => {});
            ",
            None,
        ),
        (
            "
                afterEach(() => {});
                beforeEach(() => {});
            ",
            None,
        ),
        (
            "
                afterEach(() => {});
                beforeAll(() => {});
            ",
            None,
        ),
        (
            "
                beforeEach(() => {});
                beforeAll(() => {});
            ",
            None,
        ),
        (
            "
                afterAll(() => {});
                afterEach(() => {});
            ",
            None,
        ),
        (
            "
                afterAll(() => {});
                // The afterEach should do this
                // This comment does not matter for the order
                afterEach(() => {});
            ",
            None,
        ),
        (
            "
                afterAll(() => {});
                afterAll(() => {});
                afterEach(() => {});
            ",
            None,
        ),
        (
            "
                describe('my test', () => {
                    afterAll(() => {});
                    afterEach(() => {});
                });
            ",
            None,
        ),
        (
            "
                describe('my test', () => {
                    afterAll(() => {});
                    afterEach(() => {});

                    doSomething();

                    beforeEach(() => {});
                    beforeAll(() => {});
                });
            ",
            None,
        ),
        (
            "
                describe('my test', () => {
                    afterAll(() => {});
                    afterEach(() => {});

                    it('is a test', () => {});

                    beforeEach(() => {});
                    beforeAll(() => {});
                });
            ",
            None,
        ),
        (
            "
                describe('my test', () => {
                    afterAll(() => {});

                    describe('when something is true', () => {
                        beforeEach(() => {});
                        beforeAll(() => {});
                    });
                });
            ",
            None,
        ),
        (
            "
                describe('my test', () => {
                    beforeAll(() => {});
                    afterAll(() => {});
                    beforeAll(() => {});

                    describe('when something is true', () => {
                        beforeAll(() => {});
                        afterEach(() => {});
                        beforeEach(() => {});
                        afterEach(() => {});
                    });
                });
            ",
            None,
        ),
        (
            "
                describe('my test', () => {
                    beforeAll(() => {});
                    beforeAll(() => {});
                    afterAll(() => {});

                    it('foo nested', () => {
                        // this is a test
                    });

                    describe('when something is true', () => {
                        beforeAll(() => {});
                        afterEach(() => {});

                        it('foo nested', () => {
                            // this is a test
                        });

                        describe('deeply nested', () => {
                            afterAll(() => {});
                            afterAll(() => {});
                            // This comment does nothing
                            afterEach(() => {});

                            it('foo nested', () => {
                                // this is a test
                            });
                        })
                        beforeEach(() => {});
                        afterEach(() => {});
                    });
                });
            ",
            None,
        ),
        (
            "
                describe('my test', () => {
                    const setupDatabase = () => {
                        beforeEach(() => {
                            initDatabase();
                            fillWithData();
                        });
                        beforeAll(() => {
                            setupMocks();
                        });
                    };

                    it('foo', () => {
                        // this is a test
                    });

                    describe('my nested test', () => {
                        afterAll(() => {});
                        afterEach(() => {});

                        it('foo nested', () => {
                            // this is a test
                        });
                    });
                });
            ",
            None,
        ),
        (
            "
                describe('foo', () => {
                    beforeEach(() => {
                        seedMyDatabase();
                    });

                    beforeAll(() => {
                        createMyDatabase();
                    });

                    it('accepts this input', () => {
                        // ...
                    });

                    it('returns that value', () => {
                        // ...
                    });

                    describe('when the database has specific values', () => {
                        const specificValue = '...';

                        beforeEach(() => {
                            seedMyDatabase(specificValue);
                        });

                        it('accepts that input', () => {
                            // ...
                        });

                        it('throws an error', () => {
                            // ...
                        });

                        afterEach(() => {
                            clearLogger();
                        });

                        beforeEach(() => {
                            mockLogger();
                        });

                        it('logs a message', () => {
                            // ...
                        });
                    });

                    afterAll(() => {
                        removeMyDatabase();
                    });
                });
            ",
            None,
        ),
    ];

    let pass_vitest = vec![
        r"beforeAll(() => {})",
        r"beforeEach(() => {})",
        r"afterEach(() => {})",
        r"afterAll(() => {})",
        r"describe(() => {})",
        r"
            beforeAll(() => {});
            beforeEach(() => {});
            afterEach(() => {});
            afterAll(() => {});
        ",
        r"
            describe('foo', () => {
                someSetupFn();
                beforeEach(() => {});
                afterEach(() => {});

                test('bar', () => {
                    someFn();
                });
            });
        ",
        r"
            beforeAll(() => {});
            afterAll(() => {});
        ",
        r"
            beforeEach(() => {});
            afterEach(() => {});
        ",
        r"
            beforeAll(() => {});
            afterEach(() => {});
        ",
        r"
            beforeAll(() => {});
            beforeEach(() => {});
        ",
        r"
            afterEach(() => {});
            afterAll(() => {});
        ",
        r"
            beforeAll(() => {});
            beforeAll(() => {});
        ",
        r"
            describe('my test', () => {
                afterEach(() => {});
                afterAll(() => {});
            });
        ",
        r"
            describe('my test', () => {
                afterEach(() => {});
                afterAll(() => {});

                doSomething();

                beforeAll(() => {});
                beforeEach(() => {});
            });
        ",
        r"
            describe('my test', () => {
                afterEach(() => {});
                afterAll(() => {});

                it('is a test', () => {});

                beforeAll(() => {});
                beforeEach(() => {});
            });
        ",
        r"
            describe('my test', () => {
                afterAll(() => {});

                describe('when something is true', () => {
                    beforeAll(() => {});
                    beforeEach(() => {});
                });
            });
        ",
        r"
            describe('my test', () => {
                afterAll(() => {});

                describe('when something is true', () => {
                    beforeAll(() => {});
                    beforeEach(() => {});

                    it('does something', () => {});

                    beforeAll(() => {});
                    beforeEach(() => {});
                });

                beforeAll(() => {});
                beforeEach(() => {});
            });

            describe('my test', () => {
                beforeAll(() => {});
                beforeEach(() => {});
                afterAll(() => {});

                describe('when something is true', () => {
                    it('does something', () => {});

                    beforeAll(() => {});
                    beforeEach(() => {});
                });

                beforeAll(() => {});
                beforeEach(() => {});
            });
        ",
        r"
            const withDatabase = () => {
                beforeAll(() => {
                    createMyDatabase();
                });
                afterAll(() => {
                    removeMyDatabase();
                });
            };

            describe('my test', () => {
                withDatabase();

                afterAll(() => {});

                describe('when something is true', () => {
                    beforeAll(() => {});
                    beforeEach(() => {});

                    it('does something', () => {});

                    beforeAll(() => {});
                    beforeEach(() => {});
                });

                beforeAll(() => {});
                beforeEach(() => {});
            });

            describe('my test', () => {
                beforeAll(() => {});
                beforeEach(() => {});
                afterAll(() => {});

                withDatabase();

                describe('when something is true', () => {
                    it('does something', () => {});

                    beforeAll(() => {});
                    beforeEach(() => {});
                });

                beforeAll(() => {});
                beforeEach(() => {});
            });
        ",
        r"
            describe('foo', () => {
                beforeAll(() => {
                    createMyDatabase();
                });

                beforeEach(() => {
                    seedMyDatabase();
                });

                it('accepts this input', () => {
                    // ...
                });

                it('returns that value', () => {
                    // ...
                });

                describe('when the database has specific values', () => {
                    const specificValue = '...';

                    beforeEach(() => {
                        seedMyDatabase(specificValue);
                    });

                    it('accepts that input', () => {
                        // ...
                    });

                    it('throws an error', () => {
                        // ...
                    });

                    beforeEach(() => {
                        mockLogger();
                    });

                    afterEach(() => {
                        clearLogger();
                    });

                    it('logs a message', () => {
                        // ...
                    });
                });

                afterAll(() => {
                    removeMyDatabase();
                });
            });
        ",
        r"
            describe('A file with a lot of test', () => {
                beforeAll(() => {
                    setupTheDatabase();
                    createMocks();
                });

                beforeAll(() => {
                    doEvenMore();
                });

                beforeEach(() => {
                    cleanTheDatabase();
                    resetSomeThings();
                });

                afterEach(() => {
                    cleanTheDatabase();
                    resetSomeThings();
                });

                afterAll(() => {
                    closeTheDatabase();
                    stop();
                });

                it('does something', () => {
                    const thing = getThing();
                    expect(thing).toBe('something');
                });

                it('throws', () => {
                    // Do something that throws
                });

                describe('Also have tests in here', () => {
                    afterAll(() => {});
                    it('tests something', () => {});
                    it('tests something else', () => {});
                    beforeAll(()=>{});
                });
            });
        ",
    ];

    let fail_vitest = vec![
        r"
            const withDatabase = () => {
                afterAll(() => {
                    removeMyDatabase();
                });
                beforeAll(() => {
                    createMyDatabase();
                });
            };
        ",
        r"
            afterAll(() => {
                removeMyDatabase();
            });
            beforeAll(() => {
                createMyDatabase();
            });
        ",
        r"
            afterAll(() => {});
            beforeAll(() => {});
        ",
        r"
            afterEach(() => {});
            beforeEach(() => {});
        ",
        r"
            afterEach(() => {});
            beforeAll(() => {});
        ",
        r"
            beforeEach(() => {});
            beforeAll(() => {});
        ",
        r"
            afterAll(() => {});
            afterEach(() => {});
        ",
        r"
            afterAll(() => {});
            // The afterEach should do this
            // This comment does not matter for the order
            afterEach(() => {});
        ",
        r"
            afterAll(() => {});
            afterAll(() => {});
            afterEach(() => {});
        ",
        r"
            describe('my test', () => {
                afterAll(() => {});
                afterEach(() => {});
            });
        ",
        r"
            describe('my test', () => {
                afterAll(() => {});
                afterEach(() => {});

                doSomething();

                beforeEach(() => {});
                beforeAll(() => {});
            });
        ",
        r"
            describe('my test', () => {
                afterAll(() => {});
                afterEach(() => {});

                it('is a test', () => {});

                beforeEach(() => {});
                beforeAll(() => {});
            });
        ",
        r"
            describe('my test', () => {
                afterAll(() => {});

                describe('when something is true', () => {
                    beforeEach(() => {});
                    beforeAll(() => {});
                });
            });
        ",
        r"
            describe('my test', () => {
                beforeAll(() => {});
                afterAll(() => {});
                beforeAll(() => {});

                describe('when something is true', () => {
                    beforeAll(() => {});
                    afterEach(() => {});
                    beforeEach(() => {});
                    afterEach(() => {});
                });
            });
        ",
        r"
            describe('my test', () => {
                beforeAll(() => {});
                beforeAll(() => {});
                afterAll(() => {});

                it('foo nested', () => {
                    // this is a test
                });

                describe('when something is true', () => {
                    beforeAll(() => {});
                    afterEach(() => {});

                    it('foo nested', () => {
                        // this is a test
                    });

                    describe('deeply nested', () => {
                        afterAll(() => {});
                        afterAll(() => {});
                        // This comment does nothing
                        afterEach(() => {});

                        it('foo nested', () => {
                            // this is a test
                        });
                    })
                    beforeEach(() => {});
                    afterEach(() => {});
                });
            });
        ",
        r"
            describe('my test', () => {
                const setupDatabase = () => {
                    beforeEach(() => {
                        initDatabase();
                        fillWithData();
                    });
                    beforeAll(() => {
                        setupMocks();
                    });
                };

                it('foo', () => {
                    // this is a test
                });

                describe('my nested test', () => {
                    afterAll(() => {});
                    afterEach(() => {});

                    it('foo nested', () => {
                        // this is a test
                    });
                });
            });
        ",
        r"
            describe('foo', () => {
                beforeEach(() => {
                    seedMyDatabase();
                });

                beforeAll(() => {
                    createMyDatabase();
                });

                it('accepts this input', () => {
                    // ...
                });

                it('returns that value', () => {
                    // ...
                });

                describe('when the database has specific values', () => {
                    const specificValue = '...';

                    beforeEach(() => {
                        seedMyDatabase(specificValue);
                    });

                    it('accepts that input', () => {
                        // ...
                    });

                    it('throws an error', () => {
                        // ...
                    });

                    afterEach(() => {
                        clearLogger();
                    });

                    beforeEach(() => {
                        mockLogger();
                    });

                    it('logs a message', () => {
                        // ...
                    });
                });

                afterAll(() => {
                    removeMyDatabase();
                });
            });
        ",
    ];

    pass.extend(pass_vitest.into_iter().map(|x| (x, None)));
    fail.extend(fail_vitest.into_iter().map(|x| (x, None)));

    Tester::new(PreferHooksInOrder::NAME, PreferHooksInOrder::PLUGIN, pass, fail)
        .with_vitest_plugin(true)
        .test_and_snapshot();
}
