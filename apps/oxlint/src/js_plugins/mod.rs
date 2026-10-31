mod external_linter;
mod js_plugin_workers;

#[cfg(all(target_pointer_width = "64", target_endian = "little"))]
pub mod fix;

#[cfg(all(target_pointer_width = "64", target_endian = "little"))]
pub mod parse;

pub use external_linter::create_external_linter;
pub use js_plugin_workers::{
    JsThreadCallbacks, register_js_plugin_worker, take_registered_js_plugin_workers,
};
