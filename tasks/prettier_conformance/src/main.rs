use oxc_prettier_conformance::{TestLanguage, TestRunner, TestRunnerOptions};
use pico_args::Arguments;

fn main() {
    let mut args = Arguments::from_env();

    let options = TestRunnerOptions { filter: args.opt_value_from_str("--filter").unwrap() };

    TestRunner::new(TestLanguage::Js, options.clone()).run();
    TestRunner::new(TestLanguage::Ts, options).run();
}
