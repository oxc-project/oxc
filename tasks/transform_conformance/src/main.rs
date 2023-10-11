#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[cfg(target_os = "windows")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use oxc_transform_conformance::{babel, BabelOptions};
use pico_args::Arguments;

fn main() {
    let mut args = Arguments::from_env();

    let options = BabelOptions { filter: args.opt_value_from_str("--filter").unwrap() };

    babel(&options);
}
