use pico_args::Arguments;

use oxc_prettier_conformance::{
    TestRunner,
    jsdoc::JsdocTestRunner,
    options::{TestLanguage, TestRunnerOptions},
};

/// This CLI runs in 2 modes:
/// - `cargo run`: Run all Prettier conformance tests
/// - `cargo run -- --filter <filter>`: Debug a specific test
fn main() {
    let mut args = Arguments::from_env();
    let debug = args.contains("--debug");
    let filter: Option<String> = args.opt_value_from_str("--filter").unwrap();

    for language in [
        TestLanguage::Js,
        TestLanguage::Ts,
        TestLanguage::Json,
        TestLanguage::Jsonc,
        TestLanguage::Json5,
    ] {
        TestRunner::new(TestRunnerOptions { language, debug, filter: filter.clone() }).run();
    }
    JsdocTestRunner::new(filter, debug).run();
}
