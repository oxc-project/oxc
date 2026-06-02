use oxc_macros::declare_oxc_lint;

use crate::{
    context::LintContext,
    rule::Rule,
    rules::shared::no_duplicate_hooks::{DOCUMENTATION, run_once},
};

#[derive(Debug, Default, Clone)]
pub struct NoDuplicateHooks;

declare_oxc_lint!(NoDuplicateHooks, vitest, style, docs = DOCUMENTATION, version = "0.4.0",);

impl Rule for NoDuplicateHooks {
    fn run_once(&self, ctx: &LintContext) {
        run_once(ctx);
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let mut pass = vec![
        (
            "
                describe(\"foo\", () => {
                    beforeEach(() => {})
                    test(\"bar\", () => {
                        someFn();
                    })
                })
            ",
            None,
        ),
        (
            "
                beforeEach(() => {})
                test(\"bar\", () => {
                    someFn();
                })
            ",
            None,
        ),
        (
            "
                describe(\"foo\", () => {
                    beforeAll(() => {}),
                    beforeEach(() => {})
                    afterEach(() => {})
                    afterAll(() => {})

                    test(\"bar\", () => {
                        someFn();
                    })
                })
            ",
            None,
        ),
        (
            "
                describe.skip(\"foo\", () => {
                    beforeEach(() => {}),
                    beforeAll(() => {}),
                    test(\"bar\", () => {
                        someFn();
                    })
                })
                describe(\"foo\", () => {
                    beforeEach(() => {}),
                    afterAll(() => {}),
                    test(\"bar\", () => {
                        someFn();
                    })
                })
            ",
            None,
        ),
        (
            "
                describe(\"foo\", () => {
                    beforeEach(() => {}),
                    test(\"bar\", () => {
                        someFn();
                    })
                    describe(\"inner_foo\", () => {
                        beforeEach(() => {})
                        test(\"inner bar\", () => {
                            someFn();
                        })
                    })
                })
            ",
            None,
        ),
        (
            "
                describe.each(['hello'])('%s', () => {
                    beforeEach(() => {});

                    it(\"is fine\", () => {});
                });
            ",
            None,
        ),
        (
            "
                describe(\"something\", () => {
                    describe.each(['hello'])('%s', () => {
                        beforeEach(() => {});

                        it('is fine', () => {});
                    });

                    describe.each(['world'])('%s', () => {
                        beforeEach(() => {});

                        it('is fine', () => {});
                    });
                });
            ",
            None,
        ),
        (
            "
                describe.each``('%s', () => {
                    beforeEach(() => {});

                    it(\"is fine\", () => {});
                });
            ",
            None,
        ),
        (
            "
                describe(\"something\", () => {
                    describe.each``(\"%s\", () => {
                        beforeEach(() => {});

                        it('is fine', () => {});
                    });

                    describe.each``('%s', () => {
                        beforeEach(() => {});

                        it('is fine', () => {});
                    });
                });
            ",
            None,
        ),
    ];

    let mut fail = vec![
        (
            "
                describe(\"foo\", () => {
                    beforeEach(() => {});
                    beforeEach(() => {});
                    test(\"bar\", () => {
                        someFn();
                    })
                })
            ",
            None,
        ),
        (
            "
                describe.skip(\"foo\", () => {
                    beforeEach(() => {}),
                    beforeAll(() => {}),
                    beforeAll(() => {}),
                    test(\"bar\", () => {
                        someFn();
                    })
                })
            ",
            None,
        ),
        (
            "
                describe.skip(\"foo\", () => {
                    afterEach(() => {}),
                    afterEach(() => {}),
                    test(\"bar\", () => {
                        someFn();
                    })
                })
            ",
            None,
        ),
        (
            "
                import { afterEach } from '@jest/globals';

                describe.skip(\"foo\", () => {
                    afterEach(() => {}),
                    afterEach(() => {}),
                    test(\"bar\", () => {
                        someFn();
                    })
                })
            ",
            None,
        ),
        (
            "
                import { afterEach, afterEach as somethingElse } from '@jest/globals';

                describe.skip(\"foo\", () => {
                    afterEach(() => {}),
                    somethingElse(() => {}),
                    test(\"bar\", () => {
                        someFn();
                    })
                })
            ",
            None,
        ),
        (
            "
                describe.skip(\"foo\", () => {
                    afterAll(() => {}),
                    afterAll(() => {}),
                    test(\"bar\", () => {
                        someFn();
                    })
                })
            ",
            None,
        ),
        (
            "
                afterAll(() => {}),
                afterAll(() => {}),
                test(\"bar\", () => {
                    someFn();
                })
            ",
            None,
        ),
        (
            "
                describe(\"foo\", () => {
                    beforeEach(() => {}),
                    beforeEach(() => {}),
                    beforeEach(() => {}),
                    test(\"bar\", () => {
                        someFn();
                    })
                })
            ",
            None,
        ),
        (
            "
                describe.skip(\"foo\", () => {
                    afterAll(() => {}),
                    afterAll(() => {}),
                    beforeAll(() => {}),
                    beforeAll(() => {}),
                    test(\"bar\", () => {
                        someFn();
                    })
                })
            ",
            None,
        ),
        (
            "
                describe.skip(\"foo\", () => {
                    beforeEach(() => {}),
                    beforeAll(() => {}),
                    test(\"bar\", () => {
                        someFn();
                    })
                })
                describe(\"foo\", () => {
                    beforeEach(() => {}),
                    beforeEach(() => {}),
                    beforeAll(() => {}),
                    test(\"bar\", () => {
                        someFn();
                    })
                })
            ",
            None,
        ),
        (
            "
                describe(\"foo\", () => {
                    beforeAll(() => {}),
                    test(\"bar\", () => {
                        someFn();
                    })
                    describe(\"inner_foo\", () => {
                        beforeEach(() => {})
                        beforeEach(() => {})
                        test(\"inner bar\", () => {
                            someFn();
                        })
                    })
                })
            ",
            None,
        ),
        (
            "
                describe.each(['hello'])('%s', () => {
                    beforeEach(() => {});
                    beforeEach(() => {});

                    it(\"is not fine\", () => {});
                });
            ",
            None,
        ),
        (
            "
                describe('something', () => {
                    describe.each(['hello'])('%s', () => {
                        beforeEach(() => {});

                        it('is fine', () => {});
                    });

                    describe.each(['world'])('%s', () => {
                        beforeEach(() => {});
                        beforeEach(() => {});

                        it('is not fine', () => {});
                    });
                });
            ",
            None,
        ),
        (
            "
                describe('something', () => {
                    describe.each(['hello'])('%s', () => {
                        beforeEach(() => {});

                        it('is fine', () => {});
                    });

                    describe.each(['world'])('%s', () => {
                        describe('some more', () => {
                            beforeEach(() => {});
                            beforeEach(() => {});

                            it('is not fine', () => {});
                        });
                    });
                });
            ",
            None,
        ),
        (
            "
                describe.each``('%s', () => {
                    beforeEach(() => {});
                    beforeEach(() => {});

                    it('is fine', () => {});
                });
            ",
            None,
        ),
        (
            "
                describe('something', () => {
                    describe.each``('%s', () => {
                        beforeEach(() => {});

                        it('is fine', () => {});
                    });

                    describe.each``('%s', () => {
                        beforeEach(() => {});
                        beforeEach(() => {});

                        it('is not fine', () => {});
                    });
                });
            ",
            None,
        ),
    ];

    let pass_vitest = vec![
        (
            r#"
                describe("foo", () => {
                    beforeEach(() => {})
                    test("bar", () => {
                        someFn();
                    })
			    })
            "#,
            None,
        ),
        (
            r#"
                beforeEach(() => {})
			    test("bar", () => {
                    someFn();
			    })
            "#,
            None,
        ),
        (
            r#"
                describe("foo", () => {
                    beforeAll(() => {}),
                    beforeEach(() => {})
                    afterEach(() => {})
                    afterAll(() => {})

                    test("bar", () => {
                        someFn();
                    })
                })
            "#,
            None,
        ),
        (
            r#"
                describe.skip("foo", () => {
                    beforeEach(() => {}),
                    beforeAll(() => {}),
                    test("bar", () => {
                        someFn();
                    })
                })
                describe("foo", () => {
                    beforeEach(() => {}),
                    beforeAll(() => {}),
                    test("bar", () => {
                        someFn();
                    })
                })
            "#,
            None,
        ),
        (
            r#"
                describe("foo", () => {
                    beforeEach(() => {}),
                    test("bar", () => {
                        someFn();
                    })
                    describe("inner_foo", () => {
                        beforeEach(() => {})
                        test("inner bar", () => {
                            someFn();
                        })
                    })
                })
            "#,
            None,
        ),
        (
            "
                describe.each(['hello'])('%s', () => {
                    beforeEach(() => {});
                    it('is fine', () => {});
                });
            ",
            None,
        ),
    ];

    let fail_vitest = vec![
        (
            r#"
                describe("foo", () => {
                    beforeEach(() => {}),
                    beforeEach(() => {}),
                    test("bar", () => {
                        someFn();
                    })
                })
            "#,
            None,
        ),
        (
            r#"
                describe.skip("foo", () => {
                    afterEach(() => {}),
                    afterEach(() => {}),
                    test("bar", () => {
                        someFn();
                    })
                })
            "#,
            None,
        ),
        (
            r#"
                describe.skip("foo", () => {
                    beforeEach(() => {}),
                    beforeAll(() => {}),
                    test("bar", () => {
                        someFn();
                    })
                })
                describe("foo", () => {
                    beforeEach(() => {}),
                    beforeEach(() => {}),
                    beforeAll(() => {}),
                    test("bar", () => {
                        someFn();
                    })
                })
            "#,
            None,
        ),
        (
            r#"
                describe.skip("foo", () => {
                    beforeEach(() => {}),
                    beforeAll(() => {}),
                    test("bar", () => {
                        someFn();
                    })
                })
                describe("foo", () => {
                    beforeEach(() => {}),
                    beforeEach(() => {}),
                    beforeAll(() => {}),
                    test("bar", () => {
                        someFn();
                    })
                })
            "#,
            None,
        ),
        (
            "
                describe.each(['hello'])('%s', () => {
                    beforeEach(() => {});
                    beforeEach(() => {});

                    it('is not fine', () => {});
                });
            ",
            None,
        ),
        (
            "
                describe('something', () => {
                    describe.each(['hello'])('%s', () => {
                        beforeEach(() => {});

                        it('is fine', () => {});
                    });

                    describe.each(['world'])('%s', () => {
                        beforeEach(() => {});
                        beforeEach(() => {});

                        it('is not fine', () => {});
                    });
                });
            ",
            None,
        ),
    ];

    pass.extend(pass_vitest);
    fail.extend(fail_vitest);

    Tester::new(NoDuplicateHooks::NAME, NoDuplicateHooks::PLUGIN, pass, fail)
        .with_vitest_plugin(true)
        .test_and_snapshot();
}
