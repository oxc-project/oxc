use std::{
    backtrace::Backtrace,
    fs::OpenOptions,
    io::Write,
    path::PathBuf,
    sync::Once,
    time::{SystemTime, UNIX_EPOCH},
};

static PANIC_HOOK_ONCE: Once = Once::new();

/// Initialize the data which relies on `is_atty` system calls so they don't block subsequent threads.
/// # Panics
pub fn init_miette() {
    miette::set_hook(Box::new(|_| Box::new(miette::MietteHandlerOpts::new().build()))).unwrap();
}

fn panic_log_path() -> PathBuf {
    std::env::var_os("OXLINT_PANIC_LOG")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join("oxlint-panic.log"))
}

fn install_panic_hook() {
    PANIC_HOOK_ONCE.call_once(|| {
        let default_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            let thread = std::thread::current();
            let thread_name = thread.name().unwrap_or("<unnamed>");
            let payload = panic_info
                .payload()
                .downcast_ref::<&str>()
                .copied()
                .or_else(|| panic_info.payload().downcast_ref::<String>().map(String::as_str))
                .unwrap_or("<non-string panic payload>");
            let location = panic_info
                .location()
                .map(|loc| format!("{}:{}:{}", loc.file(), loc.line(), loc.column()))
                .unwrap_or_else(|| "<unknown>".to_string());
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|duration| duration.as_secs_f64())
                .unwrap_or_default();
            let backtrace = Backtrace::force_capture();
            let report = format!(
                "\n=== oxlint panic ===\n\
                 pid: {}\n\
                 thread: {}\n\
                 timestamp_unix_s: {:.3}\n\
                 location: {}\n\
                 payload: {}\n\
                 backtrace:\n{}\n\
                 === end oxlint panic ===\n",
                std::process::id(),
                thread_name,
                timestamp,
                location,
                payload,
                backtrace
            );

            let log_path = panic_log_path();
            if let Ok(mut log_file) = OpenOptions::new().create(true).append(true).open(&log_path) {
                let _ = log_file.write_all(report.as_bytes());
                let _ = log_file.flush();
            }
            let mut stderr = std::io::stderr();
            let _ = stderr.write_all(report.as_bytes());
            let _ = stderr.flush();

            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                default_hook(panic_info);
            }));
        }));
    });
}

/// To debug `oxc_resolver`:
/// `OXC_LOG=oxc_resolver oxlint --import-plugin`
/// # Panics
pub fn init_tracing() {
    use tracing_subscriber::{filter::Targets, prelude::*};
    install_panic_hook();

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
        .with(
            tracing_subscriber::fmt::layer()
                // https://github.com/tokio-rs/tracing/issues/2492
                .with_writer(std::io::stderr),
        )
        .init();
}
