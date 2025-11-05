mod command;
mod format;
mod reporter;
mod result;
mod service;
mod walk;

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
