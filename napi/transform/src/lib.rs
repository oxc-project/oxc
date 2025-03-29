#[cfg(all(feature = "allocator", not(target_arch = "arm"), not(target_family = "wasm")))]
#[global_allocator]
static ALLOC: mimalloc_safe::MiMalloc = mimalloc_safe::MiMalloc;

mod isolated_declaration;
pub use isolated_declaration::*;

mod transformer;
pub use transformer::*;
