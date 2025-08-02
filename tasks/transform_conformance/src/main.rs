use oxc_tasks_common::CommonCliOptions;
use oxc_transform_conformance::{TestRunner, TestRunnerOptions};

fn main() {
    let mut args = pico_args::Arguments::from_env();
    let cli_options = CommonCliOptions::from_args(&mut args);

    let options = TestRunnerOptions {
        debug: cli_options.debug,
        filter: cli_options.filter,
        r#override: cli_options.r#override,
        exec: cli_options.exec,
    };

    if options.r#override {
        debug_assert!(
            options.filter.is_some(),
            "Cannot use `--override` without a specific `--filter`, because there's no
            doubt about it you do not want to override all Babel's tests"
        );
    }

    TestRunner::new(options).run();
}
