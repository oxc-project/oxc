use pico_args::Arguments;

use oxc_prettier_conformance::{
    options::{TestLanguage, TestRunnerOptions},
    TestRunner,
};

/// This CLI runs in 2 modes:
/// - `cargo run`: Run all tests and generate coverage reports
/// - `cargo run -- --filter <filter>`: Debug a specific test, not generating coverage reports
fn main() {
    let mut args = Arguments::from_env();
    let filter = args.opt_value_from_str("--filter").unwrap();

    TestRunner::new(TestRunnerOptions { filter: filter.clone(), language: TestLanguage::Js }).run();
    TestRunner::new(TestRunnerOptions { filter: filter.clone(), language: TestLanguage::Ts }).run();
}
