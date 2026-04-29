use std::sync::OnceLock;

/// Initialize the data which relies on `is_atty` system calls so they don't block subsequent threads.
///
/// Idempotent: safe to call multiple times from the same process. The
/// `OnceLock` guarantees `miette::set_hook` only runs once, which is needed
/// when oxlint's napi `lint` entry point is invoked more than once in the
/// same Node process (e.g. by a long-lived linter daemon or LSP host).
///
/// # Panics
/// If `miette::set_hook` fails on the first call (e.g. another consumer in
/// the same process already installed a hook).
pub fn init_miette() {
    static MIETTE_INIT: OnceLock<()> = OnceLock::new();
    MIETTE_INIT.get_or_init(|| {
        miette::set_hook(Box::new(|_| Box::new(miette::MietteHandlerOpts::new().build()))).unwrap();
    });
}

/// To debug `oxc_resolver`:
/// `OXC_LOG=oxc_resolver oxlint --import-plugin`
///
/// Idempotent: see `init_miette` for rationale. Mirrors the oxfmt pattern at
/// `apps/oxfmt/src/core/utils.rs`.
///
/// # Panics
/// If the global tracing subscriber cannot be installed on the first call
/// (e.g. another consumer in the same process already installed one).
pub fn init_tracing() {
    use tracing_subscriber::{filter::Targets, prelude::*};

    static TRACING_INIT: OnceLock<()> = OnceLock::new();
    TRACING_INIT.get_or_init(|| {
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
            .try_init()
            .unwrap();
    });
}
