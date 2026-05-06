use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;
use serde::Deserialize;

use crate::{
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
    rules::shared::require_hook::{DOCUMENTATION, RequireHookConfig},
};

#[derive(Debug, Default, Clone, Deserialize)]
pub struct RequireHook(Box<RequireHookConfig>);

declare_oxc_lint!(
    RequireHook,
    jest,
    style,
    config = RequireHookConfig,
    docs = DOCUMENTATION,
    version = "0.3.2",
);

impl Rule for RequireHook {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        self.0.run(node, ctx);
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

    Tester::new(RequireHook::NAME, RequireHook::PLUGIN, pass, fail)
        .with_jest_plugin(true)
        .test_and_snapshot();
}
