#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[cfg(target_os = "windows")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use oxc_coverage::AppArgs;
use pico_args::Arguments;

fn main() {
    let mut args = Arguments::from_env();
    let command = args.subcommand().expect("subcommands");

    let args = AppArgs {
        filter: args.opt_value_from_str("--filter").unwrap(),
        detail: args.contains("--detail"),
        diff: args.contains("--diff"),
    };

    let task = command.as_deref().unwrap_or("default");

    match task {
        "parser" => args.run_parser(),
        "codegen" => args.run_codegen(),
        "codegen-runtime" => args.run_codegen_runtime(),
        "minifier" => args.run_minifier(),
        "v8_test262_status" => args.run_sync_v8_test262_status(),
        _ => args.run_all(),
    };
}
