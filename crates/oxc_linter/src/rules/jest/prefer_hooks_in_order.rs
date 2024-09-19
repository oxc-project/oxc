use oxc_ast::{ast::CallExpression, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{AstNode, ScopeId};
use oxc_span::Span;
use rustc_hash::FxHashMap;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{
        parse_jest_fn_call, JestFnKind, JestGeneralFnKind, ParsedJestFnCallNew, PossibleJestNode,
    },
};

fn reorder_hooks(x1: &str, x2: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer having hooks in a consistent order.")
        .with_help(format!("{x1:?} hooks should be before any {x2:?} hooks"))
        .with_label(span)
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
    /// This rule is compatible with [eslint-plugin-vitest](https://github.com/veritem/eslint-plugin-vitest/blob/main/docs/rules/prefer-hooks-in-order.md),
    /// to use it, add the following configuration to your `.eslintrc.json`:
    ///
    /// ```json
    /// {
    ///   "rules": {
    ///      "vitest/prefer-hooks-in-order": "error"
    ///   }
    /// }
    PreferHooksInOrder,
    style,
);

impl Rule for PreferHooksInOrder {
    fn run_once(&self, ctx: &LintContext) {
        let mut hook_groups: FxHashMap<ScopeId, Vec<AstNode>> = FxHashMap::default();

        for node in ctx.nodes() {
            hook_groups.entry(node.scope_id()).or_default().push(*node);
        }

        for (_, nodes) in hook_groups {
            let mut previous_hook_index = -1;

            for node in nodes {
                if let AstKind::CallExpression(call_expr) = node.kind() {
                    let possible_jest_node = &PossibleJestNode { node: &node, original: None };
                    Self::check(&mut previous_hook_index, possible_jest_node, call_expr, ctx);
                };
            }
        }
    }
}

impl PreferHooksInOrder {
    fn check<'a>(
        previous_hook_index: &mut i32,
        possible_jest_node: &PossibleJestNode<'a, '_>,
        call_expr: &'a CallExpression<'_>,
        ctx: &LintContext<'a>,
    ) {
        let Some(ParsedJestFnCallNew::GeneralJest(jest_fn_call)) =
            parse_jest_fn_call(call_expr, possible_jest_node, ctx)
        else {
            *previous_hook_index = -1;
            return;
        };

        if !matches!(jest_fn_call.kind, JestFnKind::General(JestGeneralFnKind::Hook)) {
            *previous_hook_index = -1;
            return;
        }

        let hook_orders = ["beforeAll", "beforeEach", "afterEach", "afterAll"];
        let hook_name = jest_fn_call.name;
        let hook_pos =
            hook_orders.iter().position(|h| h.eq_ignore_ascii_case(&hook_name)).unwrap_or_default();
        let previous_hook_pos = usize::try_from(*previous_hook_index).unwrap_or(0);

        if hook_pos < previous_hook_pos {
            let Some(previous_hook_name) = hook_orders.get(previous_hook_pos) else {
                return;
            };

            ctx.diagnostic(reorder_hooks(&hook_name, previous_hook_name, call_expr.span));
            return;
        }

        *previous_hook_index = i32::try_from(hook_pos).unwrap_or(-1);
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

    Tester::new(PreferHooksInOrder::NAME, pass, fail)
        .with_jest_plugin(true)
        .with_vitest_plugin(true)
        .test_and_snapshot();
}
