use std::sync::OnceLock;

/// To debug `oxc_resolver`:
/// `OXC_LOG=oxc_resolver oxlint --import-plugin`
///
/// Idempotent so repeated napi invocations can share one process. Mirrors the
/// oxfmt pattern at `apps/oxfmt/src/core/utils.rs`.
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
