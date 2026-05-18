use oxc_macros::declare_oxc_lint;

use crate::{
    context::LintContext, rule::Rule, rules::shared::no_mocks_import as SharedNoMocksImport,
};

#[derive(Debug, Default, Clone)]
pub struct NoMocksImport;

declare_oxc_lint!(
    NoMocksImport,
    vitest,
    style,
    docs = SharedNoMocksImport::DOCUMENTATION,
    version = "0.0.13",
);

impl Rule for NoMocksImport {
    fn run_once(&self, ctx: &LintContext) {
        SharedNoMocksImport::run_once(ctx);
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let mut pass = vec![
        ("import something from 'something'", None),
        ("require('somethingElse')", None),
        ("require('./__mocks__.js')", None),
        ("require('./__mocks__x')", None),
        ("require('./__mocks__x/x')", None),
        ("require('./x__mocks__')", None),
        ("require('./x__mocks__/x')", None),
        ("require()", None),
        ("var path = './__mocks__.js'; require(path)", None),
        ("entirelyDifferent(fn)", None),
    ];

    let mut fail = vec![
        ("require('./__mocks__')", None),
        ("require('./__mocks__/')", None),
        ("require('./__mocks__/index')", None),
        ("require('__mocks__')", None),
        ("require('__mocks__/')", None),
        ("require('__mocks__/index')", None),
        ("import thing from './__mocks__/index'", None),
    ];

    let pass_vitest = vec![
        ("import something from 'something'", None),
        ("require('somethingElse')", None),
        ("require('./__mocks__.js')", None),
        ("require('./__mocks__x')", None),
        ("require('./__mocks__x/x')", None),
        ("require('./x__mocks__')", None),
        ("require('./x__mocks__/x')", None),
        ("require()", None),
        ("var path = './__mocks__.js'; require(path)", None),
        ("entirelyDifferent(fn)", None),
    ];

    let fail_vitest = vec![
        ("require('./__mocks__')", None),
        ("require('./__mocks__/')", None),
        ("require('./__mocks__/index')", None),
        ("require('__mocks__')", None),
        ("require('__mocks__/')", None),
        ("require('__mocks__/index')", None),
        ("import thing from './__mocks__/index'", None),
    ];

    pass.extend(pass_vitest);
    fail.extend(fail_vitest);

    Tester::new(NoMocksImport::NAME, NoMocksImport::PLUGIN, pass, fail)
        .with_vitest_plugin(true)
        .test_and_snapshot();
}
