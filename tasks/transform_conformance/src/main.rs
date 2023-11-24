mod ts_fixtures;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[cfg(target_os = "windows")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use oxc_transform_conformance::{TestRunner, TestRunnerOptions};
use pico_args::Arguments;
use ts_fixtures::TypeScriptFixtures;

fn main() {
    let mut args = Arguments::from_env();

    let options = TestRunnerOptions {
        filter: args.opt_value_from_str("--filter").unwrap(),
        exec: args.contains("--exec"),
    };

    TestRunner::new(options.clone()).run();
    TypeScriptFixtures::new(options).run();
}
