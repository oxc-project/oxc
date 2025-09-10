// Fixed size allocators are only supported on 64-bit little-endian platforms at present.
// They are only enabled if `fixed_size` feature enabled, and `disable_fixed_size` feature is not enabled.
//
// Note: Importing the `fixed_size` module would cause a compilation error on 32-bit systems.
#[cfg(all(
    feature = "fixed_size",
    not(feature = "disable_fixed_size"),
    target_pointer_width = "64",
    target_endian = "little"
))]
mod fixed_size;
#[cfg(all(
    feature = "fixed_size",
    not(feature = "disable_fixed_size"),
    target_pointer_width = "64",
    target_endian = "little"
))]
pub use fixed_size::*;

#[cfg(not(all(
    feature = "fixed_size",
    not(feature = "disable_fixed_size"),
    target_pointer_width = "64",
    target_endian = "little"
)))]
mod standard;
#[cfg(not(all(
    feature = "fixed_size",
    not(feature = "disable_fixed_size"),
    target_pointer_width = "64",
    target_endian = "little"
)))]
pub use standard::*;
