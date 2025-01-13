use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::ScopeId;
use oxc_span::Span;
use rustc_hash::FxHashMap;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{
        parse_jest_fn_call, JestFnKind, JestGeneralFnKind, ParsedJestFnCallNew, PossibleJestNode,
    },
};

fn reorder_hooks(hook: (&str, Span), previous_hook: (&str, Span)) -> OxcDiagnostic {
    OxcDiagnostic::warn("Test hooks are not in a consistent order.")
        .with_help(format!("{:?} hooks should be before any {:?} hooks", hook.0, previous_hook.0))
        .with_label(
            hook.1.label(format!("this should be moved to before the {:?} hook", previous_hook.0)),
        )
        .and_label(previous_hook.1.label(format!("{:?} hook should be called before this", hook.0)))
}

#[derive(Debug, Default, Clone)]
pub struct PreferHooksInOrder;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// While hooks can be setup in any order, they're always called by `jest` in this
    /// specific order:
    /// 1. `beforeAll`
    /// 2. `beforeEach`
    /// 3. `afterEach`
    /// 4. `afterAll`
    ///
    /// This rule aims to make that more obvious by enforcing grouped hooks be setup in
    /// that order within tests.
    ///
    /// ### Example
    ///
    /// ```javascript
    /// // invalid
    /// describe('foo', () => {
    ///     beforeEach(() => {
    ///         seedMyDatabase();
    ///     });
    ///     beforeAll(() => {
    ///         createMyDatabase();
    ///     });
    ///     it('accepts this input', () => {
    ///         // ...
    ///     });
    ///     it('returns that value', () => {
    ///         // ...
    ///     });
    ///     describe('when the database has specific values', () => {
    ///         const specificValue = '...';
    ///         beforeEach(() => {
    ///             seedMyDatabase(specificValue);
    ///         });
    ///
    ///         it('accepts that input', () => {
    ///             // ...
    ///         });
    ///         it('throws an error', () => {
    ///             // ...
    ///         });
    ///         afterEach(() => {
    ///             clearLogger();
    ///         });
    ///         beforeEach(() => {
    ///             mockLogger();
    ///         });
    ///         it('logs a message', () => {
    ///             // ...
    ///         });
    ///     });
    ///     afterAll(() => {
    ///         removeMyDatabase();
    ///     });
    /// });
    /// ```
    ///
    /// ```javascript
    /// // valid
    /// describe('foo', () => {
    ///     beforeAll(() => {
    ///         createMyDatabase();
    ///     });
    ///
    ///     beforeEach(() => {
    ///         seedMyDatabase();
    ///     });
    ///
    ///     it('accepts this input', () => {
    ///         // ...
    ///     });
    ///     it('returns that value', () => {
    ///         // ...
    ///     });
    ///     describe('when the database has specific values', () => {
    ///         const specificValue = '...';
    ///         beforeEach(() => {
    ///             seedMyDatabase(specificValue);
    ///         });
    ///         it('accepts that input', () => {
    ///             // ...
    ///         });
    ///         it('throws an error', () => {
    ///             // ...
    ///         });
    ///         beforeEach(() => {
    ///             mockLogger();
    ///         });
    ///         afterEach(() => {
    ///             clearLogger();
    ///         });
    ///         it('logs a message', () => {
    ///             // ...
    ///         });
    ///     });
    ///     afterAll(() => {
    ///         removeMyDatabase();
    ///     });
    /// });
    /// ```
    ///
    ///
    /// This rule is compatible with [eslint-plugin-vitest](https://github.com/veritem/eslint-plugin-vitest/blob/v1.1.9/docs/rules/prefer-hooks-in-order.md),
    /// to use it, add the following configuration to your `.eslintrc.json`:
    ///
    /// ```json
    /// {
    ///   "rules": {
    ///      "vitest/prefer-hooks-in-order": "error"
    ///   }
    /// }
    PreferHooksInOrder,
    jest,
    style,
);

impl Rule for PreferHooksInOrder {
    fn run_once(&self, ctx: &LintContext) {
        let mut previous_hook_orders: FxHashMap<ScopeId, (u8, Span)> = FxHashMap::default();

        for node in ctx.nodes() {
            if let AstKind::CallExpression(call_expr) = node.kind() {
                let possible_jest_node = &PossibleJestNode { node, original: None };
                let Some(ParsedJestFnCallNew::GeneralJest(jest_fn_call)) =
                    parse_jest_fn_call(call_expr, possible_jest_node, ctx)
                else {
                    previous_hook_orders.remove(&node.scope_id());
                    continue;
                };

                if !matches!(jest_fn_call.kind, JestFnKind::General(JestGeneralFnKind::Hook)) {
                    previous_hook_orders.remove(&node.scope_id());
                    continue;
                }

                let previous_hook_order = previous_hook_orders.get(&node.scope_id());

                let hook_name = jest_fn_call.name.as_ref();
                let Some(hook_order) = get_hook_order(hook_name) else {
                    continue;
                };

                if let Some((previous_hook_order, previous_hook_span)) = previous_hook_order {
                    if hook_order < *previous_hook_order {
                        let Some(previous_hook_name) = get_hook_name(*previous_hook_order) else {
                            continue;
                        };

                        ctx.diagnostic(reorder_hooks(
                            (hook_name, call_expr.span),
                            (previous_hook_name, *previous_hook_span),
                        ));
                        continue;
                    }
                }
                previous_hook_orders.insert(node.scope_id(), (hook_order, call_expr.span));
            };
        }
    }
}

fn get_hook_order(hook_name: &str) -> Option<u8> {
    match hook_name {
        "beforeAll" => Some(0),
        "beforeEach" => Some(1),
        "afterEach" => Some(2),
        "afterAll" => Some(3),
        _ => None,
    }
}

fn get_hook_name(hook_order: u8) -> Option<&'static str> {
    match hook_order {
        0 => Some("beforeAll"),
        1 => Some("beforeEach"),
        2 => Some("afterEach"),
        3 => Some("afterAll"),
        _ => None,
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
        .with_jest_plugin(true)
        .with_vitest_plugin(true)
        .test_and_snapshot();
}
