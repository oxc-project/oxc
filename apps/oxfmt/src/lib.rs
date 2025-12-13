pub mod cli;
mod core;
pub mod lsp;

// Only include code to run formatter when the `napi` feature is enabled.
#[cfg(feature = "napi")]
mod main_napi;
#[cfg(feature = "napi")]
pub use main_napi::*;

#[cfg(all(feature = "allocator", not(miri), not(target_family = "wasm")))]
#[global_allocator]
static GLOBAL: mimalloc_safe::MiMalloc = mimalloc_safe::MiMalloc;
