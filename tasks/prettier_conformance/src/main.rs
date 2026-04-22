use pico_args::Arguments;

use oxc_prettier_conformance::{
    TestRunner,
    jsdoc::JsdocTestRunner,
    options::{TestLanguage, TestRunnerOptions},
};

/// This CLI runs in 3 modes:
/// - `cargo run`: Run all Prettier conformance tests
/// - `cargo run -- --jsdoc`: Run JSDoc plugin conformance tests
/// - `cargo run -- --filter <filter>`: Debug a specific test
fn main() {
    let mut args = Arguments::from_env();
    let jsdoc = args.contains("--jsdoc");
    let debug = args.contains("--debug");
    let filter: Option<String> = args.opt_value_from_str("--filter").unwrap();

    if jsdoc {
        JsdocTestRunner::new(filter, debug).run();
    } else {
        let options = TestRunnerOptions { language: TestLanguage::Js, debug, filter };
        TestRunner::new(options.clone()).run();
        TestRunner::new(TestRunnerOptions { language: TestLanguage::Ts, ..options }).run();
    }
}
