use oxc_tasks_common::CommonCliOptions;

use oxc_prettier_conformance::{
    TestRunner,
    options::{TestLanguage, TestRunnerOptions},
};

/// This CLI runs in 2 modes:
/// - `cargo run`: Run all tests and generate coverage reports
/// - `cargo run -- --filter <filter>`: Debug a specific test, not generating coverage reports
fn main() {
    let mut args = pico_args::Arguments::from_env();
    let cli_options = CommonCliOptions::from_args(&mut args);
    
    let options = TestRunnerOptions {
        language: TestLanguage::Js,
        debug: cli_options.debug,
        filter: cli_options.filter,
    };

    TestRunner::new(options.clone()).run();
    TestRunner::new(TestRunnerOptions { language: TestLanguage::Ts, ..options }).run();
}
