#[cfg(all(
    feature = "allocator",
    not(any(
        target_arch = "arm",
        target_os = "freebsd",
        target_family = "wasm",
        target_pointer_width = "32"
    ))
))]
#[global_allocator]
static ALLOC: mimalloc_safe::MiMalloc = mimalloc_safe::MiMalloc;

mod isolated_declaration;
pub use isolated_declaration::*;

mod transformer;
pub use transformer::*;
