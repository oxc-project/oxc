use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::ScopeId;
use oxc_span::Span;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{
        collect_possible_jest_call_node, is_type_of_jest_fn_call, JestFnKind, JestGeneralFnKind,
        PossibleJestNode,
    },
};

fn exceeded_max_depth(current: usize, max: usize, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Enforces a maximum depth to nested describe calls.")
        .with_help(format!("Too many nested describe calls ({current}) - maximum allowed is {max}"))
        .with_label(span)
}

#[derive(Debug, Clone)]
pub struct MaxNestedDescribe {
    pub max: usize,
}

impl Default for MaxNestedDescribe {
    fn default() -> Self {
        Self { max: 5 }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule enforces a maximum depth to nested `describe()` calls.
    ///
    /// ### Why is this bad?
    ///
    /// Nesting `describe()` blocks too deeply can make the test suite hard to read and understand.
    ///
    ///
    /// ### Example
    ///
    /// The following patterns are considered warnings (with the default option of
    /// `{ "max": 5 } `):
    ///
    /// /// /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// describe('foo', () => {
    ///     describe('bar', () => {
    ///         describe('baz', () => {
    ///             describe('qux', () => {
    ///                 describe('quxx', () => {
    ///                     describe('too many', () => {
    ///                         it('should get something', () => {
    ///                             expect(getSomething()).toBe('Something');
    ///                         });
    ///                     });
    ///                 });
    ///             });
    ///         });
    ///     });
    /// });
    ///
    /// describe('foo', function () {
    ///     describe('bar', function () {
    ///         describe('baz', function () {
    ///             describe('qux', function () {
    ///                 describe('quxx', function () {
    ///                     describe('too many', function () {
    ///                         it('should get something', () => {
    ///                             expect(getSomething()).toBe('Something');
    ///                         });
    ///                     });
    ///                 });
    ///             });
    ///         });
    ///     });
    /// });
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// describe('foo', () => {
    ///     describe('bar', () => {
    ///         it('should get something', () => {
    ///             expect(getSomething()).toBe('Something');
    ///         });
    ///     });
    ///     describe('qux', () => {
    ///         it('should get something', () => {
    ///             expect(getSomething()).toBe('Something');
    ///         });
    ///     });
    /// });
    ///
    /// describe('foo2', function () {
    ///     it('should get something', () => {
    ///         expect(getSomething()).toBe('Something');
    ///     });
    /// });
    ///
    /// describe('foo', function () {
    ///     describe('bar', function () {
    ///         describe('baz', function () {
    ///             describe('qux', function () {
    ///                 describe('this is the limit', function () {
    ///                     it('should get something', () => {
    ///                         expect(getSomething()).toBe('Something');
    ///                     });
    ///                 });
    ///             });
    ///         });
    ///     });
    /// });
    /// ```
    ///
    MaxNestedDescribe,
    jest,
    style,
);

impl Rule for MaxNestedDescribe {
    fn from_configuration(value: serde_json::Value) -> Self {
        let max = value
            .get(0)
            .and_then(|config| config.get("max"))
            .and_then(serde_json::Value::as_number)
            .and_then(serde_json::Number::as_u64)
            .map_or(5, |v| usize::try_from(v).unwrap_or(5));

        Self { max }
    }

    fn run_once(&self, ctx: &LintContext) {
        let mut describes_hooks_depth: Vec<ScopeId> = vec![];
        let mut possibles_jest_nodes = collect_possible_jest_call_node(ctx);
        possibles_jest_nodes.sort_by_key(|n| n.node.id());

        for possible_jest_node in &possibles_jest_nodes {
            self.run(possible_jest_node, &mut describes_hooks_depth, ctx);
        }
    }
}

impl MaxNestedDescribe {
    fn run<'a>(
        &self,
        possible_jest_node: &PossibleJestNode<'a, '_>,
        describes_hooks_depth: &mut Vec<ScopeId>,
        ctx: &LintContext<'a>,
    ) {
        let node = possible_jest_node.node;
        let scope_id = node.scope_id();
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };
        let is_describe_call = is_type_of_jest_fn_call(
            call_expr,
            possible_jest_node,
            ctx,
            &[JestFnKind::General(JestGeneralFnKind::Describe)],
        );

        if is_describe_call && !describes_hooks_depth.contains(&scope_id) {
            describes_hooks_depth.push(scope_id);
        }

        if is_describe_call && describes_hooks_depth.len() > self.max {
            ctx.diagnostic(exceeded_max_depth(
                describes_hooks_depth.len(),
                self.max,
                call_expr.span,
            ));
        }
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

                                fdescribe('qux', () => {
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

                xdescribe('foo', function() {
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
                fdescribe('foo', () => {
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

                xdescribe('qux', () => {
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
        .with_jest_plugin(true)
        .test_and_snapshot();
}
