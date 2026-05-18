use oxc_macros::declare_oxc_lint;

use crate::{
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
    rules::shared::max_nested_describe::{DOCUMENTATION, MaxNestedDescribeConfig},
};

#[derive(Debug, Default, Clone)]
pub struct MaxNestedDescribe(Box<MaxNestedDescribeConfig>);

declare_oxc_lint!(
    MaxNestedDescribe,
    vitest,
    style,
    config = MaxNestedDescribeConfig,
    docs = DOCUMENTATION,
    version = "0.4.4",
);

impl Rule for MaxNestedDescribe {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        let config = serde_json::from_value::<DefaultRuleConfig<MaxNestedDescribeConfig>>(value)?;
        Ok(Self(Box::new(config.into_inner())))
    }

    fn run_once(&self, ctx: &LintContext) {
        self.0.run_once(ctx);
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let mut pass = vec![
        (
            "
                describe('foo', function() {
                    describe('bar', function () {
                        describe('baz', function () {
                            describe('qux', function () {
                                describe('qux', function () {
                                    it('should get something', () => {
                                        expect(getSomething()).toBe('Something');
                                    });
                                })
                            })
                        })
                    })
                });
            ",
            None,
        ),
        (
            "
                describe('foo', function() {
                    describe('bar', function () {
                        describe('baz', function () {
                            describe('qux', function () {
                                describe('qux', function () {
                                    it('should get something', () => {
                                        expect(getSomething()).toBe('Something');
                                    });
                                });

                                describe('qux', () => {
                                    it('something', async () => {
                                        expect('something').toBe('something');
                                    });
                                });
                            })
                        })
                    })
                });
            ",
            None,
        ),
        (
            "
                describe('foo', () => {
                    describe('bar', () => {
                        it('hello', async () => {
                            expect('hello').toBe('hello');
                        });
                    });
                });

                describe('foo', function() {
                    describe('bar', function() {
                        it('something', async () => {
                            expect('something').toBe('something');
                        });
                    });
                });
            ",
            None,
        ),
        (
            "
                describe('foo', () => {
                    describe('bar', () => {
                        it('should get something', () => {
                            expect(getSomething()).toBe('Something');
                        });
                    });
                    describe('qux', () => {
                        it('should get something', () => {
                            expect(getSomething()).toBe('Something');
                        });
                    });
                });

                describe('foo2', function () {
                    it('should get something', () => {
                        expect(getSomething()).toBe('Something');
                    });
                });

                describe('foo', function () {
                    describe('bar', function () {
                        describe('baz', function () {
                            describe('qux', function () {
                                describe('this is the limit', function () {
                                    it('should get something', () => {
                                        expect(getSomething()).toBe('Something');
                                    });
                                });
                            });
                        });
                    });
                });
            ",
            None,
        ),
        (
            "
                describe('foo', () => {
                    describe.only('bar', () => {
                        describe.skip('baz', () => {
                            it('something', async () => {
                                expect('something').toBe('something');
                            });
                        });
                    });
                });
            ",
            Some(serde_json::json!([{ "max": 3 }])),
        ),
        (
            "
                it('something', async () => {
                    expect('something').toBe('something');
                });
            ",
            Some(serde_json::json!([{ "max": 0 }])),
        ),
        (
            "
                describe('foo', () => {
                    describe.each(['hello', 'world'])(\"%s\", (a) => {});
                });
            ",
            None,
        ),
        (
            "
                describe('foo', () => {
                    describe.each`
                    foo  | bar
                    ${'1'} | ${'2'}
                    `('$foo $bar', ({ foo, bar }) => {});
                });
            ",
            None,
        ),
    ];

    let mut fail = vec![
        (
            "
                describe('foo', function() {
                    describe('bar', function () {
                        describe('baz', function () {
                            describe('qux', function () {
                                describe('quxx', function () {
                                    describe('over limit', function () {
                                        it('should get something', () => {
                                            expect(getSomething()).toBe('Something');
                                        });
                                    });
                                });
                            });
                        });
                    });
                });
            ",
            None,
        ),
        (
            "
                describe('foo', () => {
                    describe('bar', () => {
                        describe('baz', () => {
                            describe('baz1', () => {
                                describe('baz2', () => {
                                    describe('baz3', () => {
                                        it('should get something', () => {
                                            expect(getSomething()).toBe('Something');
                                        });
                                    });

                                    describe('baz4', () => {
                                        it('should get something', () => {
                                            expect(getSomething()).toBe('Something');
                                        });
                                    });
                                });
                            });
                        });

                        describe('qux', function () {
                            it('should get something', () => {
                                expect(getSomething()).toBe('Something');
                            });
                        });
                    })
                });
            ",
            None,
        ),
        (
            "
                describe('foo', () => {
                    describe.only('bar', () => {
                        describe.skip('baz', () => {
                            it('should get something', () => {
                                expect(getSomething()).toBe('Something');
                            });
                        });

                        describe('baz', () => {
                            it('should get something', () => {
                                expect(getSomething()).toBe('Something');
                            });
                        });
                    });
                });

                describe('qux', () => {
                    it('should get something', () => {
                        expect(getSomething()).toBe('Something');
                    });
                });
            ",
            Some(serde_json::json!([{ "max": 2 }])),
        ),
        (
            "
                describe('qux', () => {
                    it('should get something', () => {
                        expect(getSomething()).toBe('Something');
                    });
                });
            ",
            Some(serde_json::json!([{ "max": 0 }])),
        ),
        (
            "
                describe('foo', () => {
                    describe.each(['hello', 'world'])(\"%s\", (a) => {});
                });
            ",
            Some(serde_json::json!([{ "max": 1 }])),
        ),
        (
            "
                describe('foo', () => {
                    describe.each`
                    foo  | bar
                    ${'1'} | ${'2'}
                    `('$foo $bar', ({ foo, bar }) => {});
                });
            ",
            Some(serde_json::json!([{ "max": 1 }])),
        ),
    ];

    let pass_vitest = vec![
        (
            "
                describe('another suite', () => {
                    describe('another suite', () => {
                        it('skipped test', () => {
                            // Test skipped, as tests are running in Only mode
                            assert.equal(Math.sqrt(4), 3)
                        })

                        it.only('test', () => {
                            // Only this test (and others marked with only) are run
                            assert.equal(Math.sqrt(4), 2)
                        })
                    })
                })
            ",
            None,
        ),
        (
            "
                describe('another suite', () => {
                    describe('another suite', () => {
                        describe('another suite', () => {
                            describe('another suite', () => {

                            })
                        })
                    })
                })
            ",
            None,
        ),
    ];

    let fail_vitest = vec![
        (
            "
                describe('another suite', () => {
                    describe('another suite', () => {
                        describe('another suite', () => {
                            describe('another suite', () => {
                                describe('another suite', () => {
                                    describe('another suite', () => {

                                    })
                                })
                            })
                        })
                    })
                })
            ",
            None,
        ),
        (
            "
                describe('another suite', () => {
                    describe('another suite', () => {
                        describe('another suite', () => {
                            describe('another suite', () => {
                                describe('another suite', () => {
                                    describe('another suite', () => {
                                        it('skipped test', () => {
                                            // Test skipped, as tests are running in Only mode
                                            assert.equal(Math.sqrt(4), 3)
                                        })

                                        it.only('test', () => {
                                            // Only this test (and others marked with only) are run
                                            assert.equal(Math.sqrt(4), 2)
                                        })
                                    })
                                })
                            })
                        })
                    })
                })
            ",
            None,
        ),
    ];

    pass.extend(pass_vitest);
    fail.extend(fail_vitest);

    Tester::new(MaxNestedDescribe::NAME, MaxNestedDescribe::PLUGIN, pass, fail)
        .with_vitest_plugin(true)
        .test_and_snapshot();
}
