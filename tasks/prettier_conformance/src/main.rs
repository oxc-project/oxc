use pico_args::Arguments;

use oxc_prettier_conformance::{
    TestRunner,
    jsdoc::JsdocTestRunner,
    options::{TestLanguage, TestRunnerOptions},
};

/// - `cargo run`: Run all Prettier conformance tests (JS + TS + JSDoc)
/// - `cargo run -- --filter <filter>`: Debug a specific test
fn main() {
    let mut args = Arguments::from_env();
    let debug = args.contains("--debug");
    let filter: Option<String> = args.opt_value_from_str("--filter").unwrap();

    let options = TestRunnerOptions { language: TestLanguage::Js, debug, filter: filter.clone() };
    TestRunner::new(options.clone()).run();
    TestRunner::new(TestRunnerOptions { language: TestLanguage::Ts, ..options }).run();
    JsdocTestRunner::new(filter, debug).run();
}
