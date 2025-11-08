mod command;
mod format;
mod init;
mod reporter;
mod result;
mod service;
mod walk;

// Public re-exports for use in main.rs and lib consumers
pub use command::format_command;
pub use format::FormatRunner;
pub use init::{init_miette, init_tracing};
pub use result::CliRunResult;

// Only include code to run formatter when the `napi` feature is enabled.
#[cfg(feature = "napi")]
mod prettier_plugins;
#[cfg(feature = "napi")]
mod run;
#[cfg(feature = "napi")]
pub use run::*;

#[cfg(all(feature = "allocator", not(miri), not(target_family = "wasm")))]
#[global_allocator]
static GLOBAL: mimalloc_safe::MiMalloc = mimalloc_safe::MiMalloc;
