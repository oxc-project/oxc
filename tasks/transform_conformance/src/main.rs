use oxc_transform_conformance::{TestRunner, TestRunnerOptions};
use pico_args::Arguments;

fn main() {
    let mut args = Arguments::from_env();

    let options = TestRunnerOptions {
        debug: args.contains("--debug"),
        filter: args.opt_value_from_str("--filter").unwrap(),
        r#override: args.contains("--override"),
        exec: args.contains("--exec"),
    };

    if options.r#override {
        debug_assert!(
            options.filter.is_some(),
            "Cannot use `--override` without a specific `--filter`, because there's no
            doubt about it you do not want to override all Babel's tests"
        );
    }

    TestRunner::new(options.clone()).run();
}
