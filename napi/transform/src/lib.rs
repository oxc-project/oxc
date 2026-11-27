mod isolated_declaration;
mod module_runner;
mod options;
mod transformer;

pub use isolated_declaration::*;
pub use module_runner::*;
pub use options::*;
pub use transformer::*;

#[cfg(all(
    feature = "allocator",
    not(any(
        target_arch = "arm",
        target_os = "android",
        target_os = "freebsd",
        target_os = "windows",
        target_family = "wasm"
    ))
))]
#[global_allocator]
static ALLOC: mimalloc_safe::MiMalloc = mimalloc_safe::MiMalloc;
