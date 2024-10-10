use oxc_allocator::Vec as OxcVec;
use oxc_ast::{
    ast::{Argument, Expression, Statement, VariableDeclarationKind},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;
use oxc_span::{CompactStr, Span};

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{
        get_node_name, is_type_of_jest_fn_call, parse_jest_fn_call, JestFnKind, JestGeneralFnKind,
        PossibleJestNode,
    },
};

fn use_hook(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Require setup and teardown code to be within a hook.")
        .with_help("This should be done within a hook")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct RequireHookConfig {
    allowed_function_calls: Vec<CompactStr>,
}

#[derive(Debug, Default, Clone)]
pub struct RequireHook(Box<RequireHookConfig>);

impl std::ops::Deref for RequireHook {
    type Target = RequireHookConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    /// This rule flags any expression that is either at the toplevel of a test file or
    /// directly within the body of a `describe`, _except_ for the following:
    ///
    /// - `import` statements
    /// - `const` variables
    /// - `let` _declarations_, and initializations to `null` or `undefined`
    /// - Classes
    /// - Types
    /// - Calls to the standard Jest globals
    ///
    /// ### Example
    /// ```javascript
    /// // invalid
    /// import { database, isCity } from '../database';
    /// import { Logger } from '../../../src/Logger';
    /// import { loadCities } from '../api';
    ///
    /// jest.mock('../api');
    ///
    /// const initializeCityDatabase = () => {
    ///     database.addCity('Vienna');
    ///     database.addCity('San Juan');
    ///     database.addCity('Wellington');
    /// };
    ///
    /// const clearCityDatabase = () => {
    ///     database.clear();
    /// };
    ///
    /// initializeCityDatabase();
    ///
    /// test('that persists cities', () => {
    ///     expect(database.cities.length).toHaveLength(3);
    /// });
    /// test('city database has Vienna', () => {
    ///     expect(isCity('Vienna')).toBeTruthy();
    /// });
    ///
    /// test('city database has San Juan', () => {
    ///     expect(isCity('San Juan')).toBeTruthy();
    /// });
    ///
    /// describe('when loading cities from the api', () => {
    ///     let consoleWarnSpy = jest.spyOn(console, 'warn');
    ///     loadCities.mockResolvedValue(['Wellington', 'London']);
    ///
    ///     it('does not duplicate cities', async () => {
    ///         await database.loadCities();
    ///         expect(database.cities).toHaveLength(4);
    ///     });
    /// });
    /// clearCityDatabase();
    ///
    /// // valid
    /// import { database, isCity } from '../database';
    /// import { Logger } from '../../../src/Logger';
    /// import { loadCities } from '../api';
    ///
    /// jest.mock('../api');
    /// const initializeCityDatabase = () => {
    ///     database.addCity('Vienna');
    ///     database.addCity('San Juan');
    ///     database.addCity('Wellington');
    /// };
    ///
    /// const clearCityDatabase = () => {
    ///     database.clear();
    /// };
    ///
    /// beforeEach(() => {
    ///     initializeCityDatabase();
    /// });
    ///
    /// test('that persists cities', () => {
    ///     expect(database.cities.length).toHaveLength(3);
    /// });
    ///
    /// test('city database has Vienna', () => {
    ///     expect(isCity('Vienna')).toBeTruthy();
    /// });
    ///
    /// test('city database has San Juan', () => {
    ///     expect(isCity('San Juan')).toBeTruthy();
    /// });
    ///
    /// describe('when loading cities from the api', () => {
    ///     let consoleWarnSpy;
    ///     beforeEach(() => {
    ///         consoleWarnSpy = jest.spyOn(console, 'warn');
    ///         loadCities.mockResolvedValue(['Wellington', 'London']);
    ///     });
    ///
    ///     it('does not duplicate cities', async () => {
    ///         await database.loadCities();
    ///         expect(database.cities).toHaveLength(4);
    ///     });
    /// });
    /// afterEach(() => {
    ///     clearCityDatabase();
    /// });
    /// ```
    ///
    RequireHook,
    style
);

impl Rule for RequireHook {
    fn from_configuration(value: serde_json::Value) -> Self {
        let allowed_function_calls = value
            .get(0)
            .and_then(|config| config.get("allowedFunctionCalls"))
            .and_then(serde_json::Value::as_array)
            .map(|v| v.iter().filter_map(serde_json::Value::as_str).map(CompactStr::from).collect())
            .unwrap_or_default();

        Self(Box::new(RequireHookConfig { allowed_function_calls }))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let kind = node.kind();

        if let AstKind::Program(program) = kind {
            self.check_block_body(node, &program.body, ctx);
        } else if let AstKind::CallExpression(call_expr) = kind {
            if !is_type_of_jest_fn_call(
                call_expr,
                &PossibleJestNode { node, original: None },
                ctx,
                &[JestFnKind::General(JestGeneralFnKind::Describe)],
            ) || call_expr.arguments.len() < 2
            {
                return;
            }

            match &call_expr.arguments[1] {
                Argument::FunctionExpression(func_expr) => {
                    if let Some(func_body) = &func_expr.body {
                        self.check_block_body(node, &func_body.statements, ctx);
                    };
                }
                Argument::ArrowFunctionExpression(arrow_func_expr) => {
                    if !arrow_func_expr.expression {
                        self.check_block_body(node, &arrow_func_expr.body.statements, ctx);
                    }
                }
                _ => (),
            }
        }
    }
}

impl RequireHook {
    fn check_block_body<'a>(
        &self,
        node: &AstNode<'a>,
        statements: &'a OxcVec<'a, Statement<'_>>,
        ctx: &LintContext<'a>,
    ) {
        for stmt in statements {
            self.check(node, stmt, ctx);
        }
    }

    fn check<'a>(&self, node: &AstNode<'a>, stmt: &'a Statement<'_>, ctx: &LintContext<'a>) {
        if let Statement::ExpressionStatement(expr_stmt) = stmt {
            self.check_should_report_in_hook(node, &expr_stmt.expression, ctx);
        } else if let Statement::VariableDeclaration(var_decl) = stmt {
            if var_decl.kind != VariableDeclarationKind::Const
                && var_decl.declarations.iter().any(|decl| {
                    let Some(init_call) = &decl.init else {
                        return false;
                    };
                    !init_call.is_null_or_undefined()
                })
            {
                ctx.diagnostic(use_hook(var_decl.span));
            }
        }
    }

    fn check_should_report_in_hook<'a>(
        &self,
        node: &AstNode<'a>,
        expr: &'a Expression<'a>,
        ctx: &LintContext<'a>,
    ) {
        if let Expression::CallExpression(call_expr) = expr {
            let name = get_node_name(&call_expr.callee);

            if !(parse_jest_fn_call(call_expr, &PossibleJestNode { node, original: None }, ctx)
                .is_some()
                || name.starts_with("jest.")
                || self.allowed_function_calls.contains(&name))
            {
                ctx.diagnostic(use_hook(call_expr.span));
            }
        }
    }
}

#[test]
fn tests() {
    use crate::tester::Tester;

    let pass = vec![
        ("describe()", None),
        ("describe(\"just a title\")", None),
        (
            "
                describe('a test', () =>
                    test('something', () => {
                        expect(true).toBe(true);
                    })
                );
            ",
            None,
        ),
        (
            "
                test('it', () => {
                    //
                });
            ",
            None,
        ),
        (
            "
                const { myFn } = require('../functions');

                test('myFn', () => {
                    expect(myFn()).toBe(1);
                });
            ",
            None,
        ),
        (
            "
                import { myFn } from '../functions';
                test('myFn', () => {
                    expect(myFn()).toBe(1);
                });
            ",
            None,
        ),
        (
            "
                class MockLogger {
                    log() {}
                }

                test('myFn', () => {
                    expect(myFn()).toBe(1);
                });
            ",
            None,
        ),
        (
            "
                const { myFn } = require('../functions');

                describe('myFn', () => {
                    it('returns one', () => {
                        expect(myFn()).toBe(1);
                    });
                });
            ",
            None,
        ),
        (
            "
                const { myFn } = require('../functions');

                describe('myFn', function () {
                    it('returns one', () => {
                        expect(myFn()).toBe(1);
                    });
                });
            ",
            None,
        ),
        (
            "
                describe('some tests', () => {
                    it('is true', () => {
                        expect(true).toBe(true);
                    });
                });
            ",
            None,
        ),
        (
            "
                describe('some tests', () => {
                    it('is true', () => {
                        expect(true).toBe(true);
                    });

                    describe('more tests', () => {
                        it('is false', () => {
                            expect(true).toBe(false);
                        });
                    });
                });
            ",
            None,
        ),
        (
            "
                describe('some tests', () => {
                    let consoleLogSpy;

                    beforeEach(() => {
                        consoleLogSpy = jest.spyOn(console, 'log');
                    });

                    it('prints a message', () => {
                        printMessage('hello world');
                        expect(consoleLogSpy).toHaveBeenCalledWith('hello world');
                    });
                });
            ",
            None,
        ),
        (
            "
                let consoleErrorSpy = null;

                beforeEach(() => {
                    consoleErrorSpy = jest.spyOn(console, 'error');
                });
            ",
            None,
        ),
        (
            "
                let consoleErrorSpy = undefined;

                beforeEach(() => {
                    consoleErrorSpy = jest.spyOn(console, 'error');
                });
            ",
            None,
        ),
        (
            "
                describe('some tests', () => {
                    beforeEach(() => {
                        setup();
                    });
                });
            ",
            None,
        ),
        (
            "
                beforeEach(() => {
                    initializeCityDatabase();
                });

                afterEach(() => {
                    clearCityDatabase();
                });

                test('city database has Vienna', () => {
                    expect(isCity('Vienna')).toBeTruthy();
                });

                test('city database has San Juan', () => {
                    expect(isCity('San Juan')).toBeTruthy();
                });
            ",
            None,
        ),
        (
            "
                describe('cities', () => {
                    beforeEach(() => {
                        initializeCityDatabase();
                    });

                    test('city database has Vienna', () => {
                        expect(isCity('Vienna')).toBeTruthy();
                    });

                    test('city database has San Juan', () => {
                        expect(isCity('San Juan')).toBeTruthy();
                    });

                    afterEach(() => {
                        clearCityDatabase();
                    });
                });
            ",
            None,
        ),
        (
            "
                enableAutoDestroy(afterEach);

                describe('some tests', () => {
                    it('is false', () => {
                        expect(true).toBe(true);
                    });
                });
            ",
            Some(serde_json::json!([{ "allowedFunctionCalls": ["enableAutoDestroy"] }])),
        ),
        (
            "
                import { myFn } from '../functions';

                // todo: https://github.com/DefinitelyTyped/DefinitelyTyped/pull/56545
                declare module 'eslint' {
                    namespace ESLint {
                        interface LintResult {
                            fatalErrorCount: number;
                        }
                    }
                }

                test('myFn', () => {
                    expect(myFn()).toBe(1);
                });
            ",
            None,
        ),
    ];

    let fail = vec![
        ("setup();", None),
        (
            "
                describe('some tests', () => {
                    setup();
                });
            ",
            None,
        ),
        (
            "
                let { setup } = require('./test-utils');

                describe('some tests', () => {
                    setup();
                });
            ",
            None,
        ),
        (
            "
                describe('some tests', () => {
                    setup();

                    it('is true', () => {
                        expect(true).toBe(true);
                    });

                    describe('more tests', () => {
                        setup();

                        it('is false', () => {
                        expect(true).toBe(false);
                        });
                    });
                });
            ",
            None,
        ),
        (
            "
                let consoleErrorSpy = jest.spyOn(console, 'error');

                describe('when loading cities from the api', () => {
                    let consoleWarnSpy = jest.spyOn(console, 'warn');
                });
            ",
            None,
        ),
        (
            "
                let consoleErrorSpy = null;

                describe('when loading cities from the api', () => {
                  let consoleWarnSpy = jest.spyOn(console, 'warn');
                });
            ",
            None,
        ),
        ("let value = 1", None),
        ("let consoleErrorSpy, consoleWarnSpy = jest.spyOn(console, 'error');", None),
        ("let consoleErrorSpy = jest.spyOn(console, 'error'), consoleWarnSpy;", None),
        (
            "
                import { database, isCity } from '../database';
                import { loadCities } from '../api';

                jest.mock('../api');

                const initializeCityDatabase = () => {
                    database.addCity('Vienna');
                    database.addCity('San Juan');
                    database.addCity('Wellington');
                };

                const clearCityDatabase = () => {
                    database.clear();
                };

                initializeCityDatabase();

                test('that persists cities', () => {
                    expect(database.cities.length).toHaveLength(3);
                });

                test('city database has Vienna', () => {
                    expect(isCity('Vienna')).toBeTruthy();
                });

                test('city database has San Juan', () => {
                    expect(isCity('San Juan')).toBeTruthy();
                });

                describe('when loading cities from the api', () => {
                    let consoleWarnSpy = jest.spyOn(console, 'warn');

                    loadCities.mockResolvedValue(['Wellington', 'London']);

                    it('does not duplicate cities', async () => {
                        await database.loadCities();

                        expect(database.cities).toHaveLength(4);
                    });

                    it('logs any duplicates', async () => {
                        await database.loadCities();

                        expect(consoleWarnSpy).toHaveBeenCalledWith(
                            'Ignored duplicate cities: Wellington',
                        );
                    });
                });

                clearCityDatabase();
            ",
            None,
        ),
        (
            "
                enableAutoDestroy(afterEach);

                describe('some tests', () => {
                    it('is false', () => {
                        expect(true).toBe(true);
                    });
                });
            ",
            Some(serde_json::json!([{ "allowedFunctionCalls": ["someOtherName"] }])),
        ),
        (
            "
                import { setup } from '../test-utils';

                // todo: https://github.com/DefinitelyTyped/DefinitelyTyped/pull/56545
                declare module 'eslint' {
                    namespace ESLint {
                        interface LintResult {
                            fatalErrorCount: number;
                        }
                    }
                }

                describe('some tests', () => {
                    setup();
                });
            ",
            None,
        ),
    ];

    Tester::new(RequireHook::NAME, pass, fail).with_jest_plugin(true).test_and_snapshot();
}
