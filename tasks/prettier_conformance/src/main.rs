use pico_args::Arguments;

use oxc_prettier_conformance::{
    TestRunner,
    options::{TestLanguage, TestRunnerOptions},
};

/// This CLI runs in 2 modes:
/// - `cargo run`: Run all tests and generate coverage reports
/// - `cargo run -- --filter <filter>`: Debug a specific test, not generating coverage reports
fn main() {
    let mut args = Arguments::from_env();
    let options = TestRunnerOptions {
        language: TestLanguage::Js,
        debug: args.contains("--debug"),
        filter: args.opt_value_from_str("--filter").unwrap(),
    };

    TestRunner::new(options.clone()).run();
    TestRunner::new(TestRunnerOptions { language: TestLanguage::Ts, ..options }).run();
}
