//! [`NonZeroU32`] equivalent, except with the illegal value at the top of the range (`u32::MAX`).
//!
//! [`NonMaxU32`] can represent any number from 0 to `u32::MAX - 1` inclusive.
//!
//! `NonMaxU32` has a niche, so `Option<NonMaxU32>` is 4 bytes.
//!
//! On *nix, Mac, and WASI, this type is completely zero cost.
//!
//! On Windows, we wrap a `NonZeroU32` and XOR it with `u32::MAX` during conversion to/from `u32`,
//! same as `nonmax` crate does. This does have a (small) cost.
//!
//! # Hashing
//! Note that the Unix and Windows versions will produce different hashes from each other.
//!
//! [`NonZeroU32`]: std::num::NonZeroU32

// Version for *nix, Mac and WASI.
// `os::fd` is only available on these platforms.
// https://github.com/rust-lang/rust/blob/75530e9f72a1990ed2305e16fd51d02f47048f12/library/std/src/os/mod.rs#L185-L186
#[cfg(any(unix, target_os = "hermit", target_os = "wasi"))]
mod unix;
#[cfg(any(unix, target_os = "hermit", target_os = "wasi"))]
pub use unix::NonMaxU32;

// Version for other platforms (primarily Windows)
#[cfg(not(any(unix, target_os = "hermit", target_os = "wasi")))]
mod windows;
#[cfg(not(any(unix, target_os = "hermit", target_os = "wasi")))]
pub use windows::NonMaxU32;

// Implementations which are shared between both versions
mod shared;
pub use shared::TryFromU32Error;

// Tests
#[cfg(test)]
mod test;
