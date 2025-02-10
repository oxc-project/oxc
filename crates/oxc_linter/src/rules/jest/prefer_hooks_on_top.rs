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
        collect_possible_jest_call_node, is_type_of_jest_fn_call, JestFnKind, JestGeneralFnKind,
        PossibleJestNode,
    },
};

fn no_hook_on_top(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Suggest having hooks before any test cases.")
        .with_help("Hooks should come before test cases")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferHooksOnTop;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// While hooks can be setup anywhere in a test file, they are always called in a
    /// specific order, which means it can be confusing if they're intermixed with test
    /// cases.
    ///
    /// ### Example
    ///
    /// ```javascript
    /// // invalid
    /// describe('foo', () => {
    ///     beforeEach(() => {
    ///         seedMyDatabase();
    ///     });
    ///
    ///     it('accepts this input', () => {
    ///         // ...
    ///     });
    ///
    ///     beforeAll(() => {
    ///         createMyDatabase();
    ///     });
    ///
    ///     it('returns that value', () => {
    ///         // ...
    ///     });
    ///
    ///     describe('when the database has specific values', () => {
    ///         const specificValue = '...';
    ///         beforeEach(() => {
    ///             seedMyDatabase(specificValue);
    ///         });
    ///
    ///         it('accepts that input', () => {
    ///             // ...
    ///         });
    ///
    ///         it('throws an error', () => {
    ///             // ...
    ///         });
    ///
    ///         afterEach(() => {
    ///             clearLogger();
    ///         });
    ///
    ///         beforeEach(() => {
    ///             mockLogger();
    ///         });
    ///
    ///         it('logs a message', () => {
    ///             // ...
    ///         });
    ///     });
    ///
    ///     afterAll(() => {
    ///         removeMyDatabase();
    ///     });
    /// });
    ///
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
    ///     afterAll(() => {
    ///         clearMyDatabase();
    ///     });
    ///
    ///     it('accepts this input', () => {
    ///         // ...
    ///     });
    ///
    ///     it('returns that value', () => {
    ///         // ...
    ///     });
    ///
    ///     describe('when the database has specific values', () => {
    ///         const specificValue = '...';
    ///
    ///         beforeEach(() => {
    ///             seedMyDatabase(specificValue);
    ///         });
    ///
    ///         beforeEach(() => {
    ///             mockLogger();
    ///         });
    ///
    ///         afterEach(() => {
    ///             clearLogger();
    ///         });
    ///
    ///         it('accepts that input', () => {
    ///             // ...
    ///         });
    ///
    ///         it('throws an error', () => {
    ///             // ...
    ///         });
    ///
    ///         it('logs a message', () => {
    ///             // ...
    ///         });
    ///     });
    /// });
    /// ```
    PreferHooksOnTop,
    jest,
    style,
);

impl Rule for PreferHooksOnTop {
    fn run_once(&self, ctx: &LintContext) {
        let mut hooks_contexts: FxHashMap<ScopeId, bool> = FxHashMap::default();
        let mut possibles_jest_nodes = collect_possible_jest_call_node(ctx);
        possibles_jest_nodes.sort_by_key(|n| n.node.id());

        for possible_jest_node in &possibles_jest_nodes {
            Self::run(possible_jest_node, &mut hooks_contexts, ctx);
        }
    }
}

impl PreferHooksOnTop {
    fn run<'a>(
        possible_jest_node: &PossibleJestNode<'a, '_>,
        hooks_context: &mut FxHashMap<ScopeId, bool>,
        ctx: &LintContext<'a>,
    ) {
        let node = possible_jest_node.node;
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        if is_type_of_jest_fn_call(
            call_expr,
            possible_jest_node,
            ctx,
            &[JestFnKind::General(JestGeneralFnKind::Test)],
        ) {
            hooks_context.insert(node.scope_id(), true);
        }

        let Some((_, has_hook)) = hooks_context.get_key_value(&node.scope_id()) else {
            return;
        };

        if *has_hook
            && is_type_of_jest_fn_call(
                call_expr,
                possible_jest_node,
                ctx,
                &[JestFnKind::General(JestGeneralFnKind::Hook)],
            )
        {
            ctx.diagnostic(no_hook_on_top(call_expr.span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    // Note: Both Jest and Vitest share the same unit tests

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
        .with_jest_plugin(true)
        .test_and_snapshot();
}
