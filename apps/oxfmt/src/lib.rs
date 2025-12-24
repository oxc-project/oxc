pub mod cli;
mod core;
#[cfg(feature = "napi")]
pub mod lsp;
#[cfg(feature = "napi")]
mod main_napi;
#[cfg(feature = "napi")]
pub mod stdin;
#[cfg(feature = "napi")]
pub use main_napi::*;

#[cfg(all(feature = "allocator", not(miri), not(target_family = "wasm")))]
#[global_allocator]
static GLOBAL: mimalloc_safe::MiMalloc = mimalloc_safe::MiMalloc;
