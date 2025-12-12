/// Initialize the data which relies on `is_atty` system calls so they don't block subsequent threads.
/// # Panics
pub fn init_miette() {
    miette::set_hook(Box::new(|_| Box::new(miette::MietteHandlerOpts::new().build()))).unwrap();
}

/// Initialize Rayon global thread pool with specified number of threads.
///
/// If `--threads` option is not used, or `--threads 0` is given,
/// default to the number of available CPU cores.
///
/// # Panics
/// Panics if the global thread pool has already been initialized.
#[expect(clippy::print_stderr)]
pub fn init_rayon(threads: Option<usize>) {
    // Always initialize thread pool, even if using default thread count,
    // to ensure thread pool's thread count is locked after this point.
    // `rayon::current_num_threads()` will always return the same number after this point.
    //
    // If you don't initialize the global thread pool explicitly, or don't specify `num_threads`,
    // Rayon will initialize the thread pool when it's first used, with a thread count of
    // `std::thread::available_parallelism()`, and that thread count won't change thereafter.
    // So we don't *need* to initialize the thread pool here if we just want the default thread count.
    //
    // However, Rayon's docs state that:
    // > In the future, the default behavior may change to dynamically add or remove threads as needed.
    // https://docs.rs/rayon/1.11.0/rayon/struct.ThreadPoolBuilder.html#method.num_threads
    //
    // To ensure we continue to have a "locked" thread count, even after future Rayon upgrades,
    // we always initialize the thread pool and explicitly specify thread count here.

    let thread_count = if let Some(thread_count) = threads
        && thread_count > 0
    {
        thread_count
    } else if let Ok(thread_count) = std::thread::available_parallelism() {
        thread_count.get()
    } else {
        eprintln!(
            "Unable to determine available thread count. Defaulting to 1.\nConsider specifying the number of threads explicitly with `--threads` option."
        );
        1
    };

    rayon::ThreadPoolBuilder::new().num_threads(thread_count).build_global().unwrap();
}

/// To debug `oxc_formatter`:
/// `OXC_LOG=oxc_formatter oxfmt`
/// # Panics
pub fn init_tracing() {
    use tracing_subscriber::{filter::Targets, prelude::*};

    // Usage without the `regex` feature.
    // <https://github.com/tokio-rs/tracing/issues/1436#issuecomment-918528013>
    tracing_subscriber::registry()
        .with(std::env::var("OXC_LOG").map_or_else(
            |_| Targets::new(),
            |env_var| {
                use std::str::FromStr;
                Targets::from_str(&env_var).unwrap()
            },
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
}
