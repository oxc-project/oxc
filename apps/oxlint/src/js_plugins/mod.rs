mod external_linter;

#[cfg(all(target_pointer_width = "64", target_endian = "little"))]
pub mod fix;

#[cfg(all(target_pointer_width = "64", target_endian = "little"))]
pub mod parse;

pub use external_linter::{
    create_external_linter, print_startup_fallback, try_start_js_plugin_workers,
};
