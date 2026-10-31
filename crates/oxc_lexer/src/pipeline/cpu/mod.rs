//! Types representing CPU feature sets, used to choose between SIMD and generic implementations.
//!
//! - `Avx2`: x86_64 with AVX2, BMI2 and POPCNT.
//! - `Neon`: aarch64 with Neon.
//! - [`Generic`]: No SIMD. Exists on all platforms.
//!
//! [`DynCpu`] is an enum of these types. [`DynCpu::detect`] returns the best one available.
//!
//! [`BASELINE`] is the feature set to use without detecting the CPU's features.
//! It's the best one which is known at compile time to be available.
//!
//! Code which has SIMD implementations takes a type implementing [`Cpu`],
//! and uses [`cpu_select!`] to pick an implementation.
//! [`cpu_versions!`] turns a function into one with a separate entry point for each CPU feature set.
//!
//! * Given a specific type (e.g. `Avx2`): the choice is made at compile time.
//! * Given a [`DynCpu`]: the choice is made at runtime.
//!
//! Functions which use [`cpu_select!`] typically take a generic `impl Cpu`.
//! This compiles a separate version of the function for each [`Cpu`] type that callers use with it.
//!
//! [`DynCpu`] also implements [`Cpu`], so functions that receive an `impl Cpu`
//! can do either a compile-time or runtime features check, depending on whether it's called with
//! a concrete type (e.g. `Avx2`) or [`DynCpu`].
//!
//! `Avx2`, `Neon`, and [`Generic`] types are all zero-sized, so can be passed between functions
//! without any runtime cost. They operate at the level of the type system only.
//!
//! # SAFETY
//!
//! These types are unforgeable.
//!
//! * Code outside this file can't create them.
//! * Each has a private field, and only 3 places create them:
//!   - `DynCpu::detect` only creates an `Avx2` if that feature set is available.
//!   - [`BASELINE`] is only a `Neon` when Neon is enabled at compile time, which guarantees it's available.
//!     Otherwise it's a [`Generic`], which guarantees nothing.
//!   - `CpuPrivate::DYN_CPU` holds an instance of the type.
//!     The trait is private, so only code in this file can read it.
//!     Only [`to_dyn`] reads it, and it takes an instance of the same type,
//!     so it only gives one to code which already has one.
//! * A [`DynCpu`] can only be built from one of these types, so it's unforgeable too.
//! * [`Cpu`] trait is sealed, so no other type can implement it.
//!
//! Therefore, holding one of these types guarantees that the CPU supports that feature set.
//!
//! The rest of the lexer relies on this for safety. A function which takes `Avx2` or `Neon`
//! can use instructions that rely on support for those features without any checks.
//!
//! DO NOT add any other way to create these types (e.g. a `Default` impl or a public constructor).

use std::mem;

mod select_macro;
mod versions_macro;

#[cfg(test)]
mod tests;

pub(super) use select_macro::cpu_select;
pub(super) use versions_macro::{__cpu_versions, apply, cpu_versions};

/// Trait for types representing a CPU feature set.
///
/// Implemented by all types representing specific CPU feature sets, and by [`DynCpu`].
#[expect(private_bounds)]
pub(super) trait Cpu: CpuPrivate + Copy {
    /// Get the [`DynCpu`] for this type.
    ///
    /// Same as the [`to_dyn`] function, which works in `const` code too.
    #[inline(always)]
    fn to_dyn(self) -> DynCpu {
        to_dyn(self)
    }
}

/// Private trait to guarantee that only types defined in this file can implement [`Cpu`].
///
/// It also holds [`DYN_CPU`], for [`to_dyn`].
///
/// [`DYN_CPU`] is on this trait, not on [`Cpu`], so code outside this file can't read it.
/// It holds an instance of the type, so if code outside this file could read it,
/// that would allow bypassing CPU feature detection.
///
/// [`DYN_CPU`]: Self::DYN_CPU
trait CpuPrivate {
    /// The [`DynCpu`] which [`to_dyn`] returns for this type.
    ///
    /// `Avx2`, `Neon` and [`Generic`] each have only one value, so this can hold it.
    /// `None` for [`DynCpu`] itself, and for [`Generic`] when it's uninhabited.
    const DYN_CPU: Option<DynCpu>;
}

/// Get the [`DynCpu`] for a type implementing [`Cpu`].
///
/// Same as [`Cpu::to_dyn`], but this works in `const` code too.
/// A `const fn` can't call a trait method on a generic type, so it can't call [`Cpu::to_dyn`].
/// [`cpu_select!`] and [`cpu_versions!`] use this function, so they work in `const` code.
///
/// Only [`DynCpu`] needs `cpu`. It's taken for all types so that this can only be called
/// by code which already has one of these types.
#[inline(always)]
pub(super) const fn to_dyn<C: Cpu>(cpu: C) -> DynCpu {
    match C::DYN_CPU {
        Some(dyn_cpu) => dyn_cpu,
        // SAFETY: `DYN_CPU` is `None` only for `DynCpu`, and for `Generic` when it's uninhabited.
        // No value of an uninhabited type exists, so `cpu` must be a `DynCpu`.
        None => unsafe { mem::transmute_copy::<C, DynCpu>(&cpu) },
    }
}

/// Enum representing a CPU feature set.
///
/// Used for runtime feature detection.
///
/// Implements [`Cpu`] and also provides a [`detect`] method which initializes [`DynCpu`]
/// via target feature detection.
///
/// [`detect`]: Self::detect
#[derive(Clone, Copy, Debug)]
pub(super) enum DynCpu {
    /// AVX2 feature set. Only available on x86_64.
    #[cfg(target_arch = "x86_64")]
    Avx2(Avx2),
    /// Neon feature set. Only available on aarch64.
    #[cfg(target_arch = "aarch64")]
    Neon(Neon),
    /// Exists on all platforms, but can never be created on aarch64
    /// when Neon is enabled at compile time. See [`Generic`].
    #[cfg_attr(all(target_arch = "aarch64", target_feature = "neon"), expect(dead_code))]
    Generic(Generic),
}

impl Cpu for DynCpu {}

impl CpuPrivate for DynCpu {
    const DYN_CPU: Option<DynCpu> = None;
}

/// Generic implementation.
mod generic {
    use super::*;

    /// Generic CPU feature set.
    ///
    /// On aarch64 with Neon enabled at compile time, this type is uninhabited - `Generic` can never be created.
    /// So [`DynCpu::Generic`] can never exist there, and the compiler removes code which handles it.
    /// Every aarch64 target with `std` enables Neon by default, so in practice this applies to all aarch64 builds.
    ///
    /// The type still exists there, so code which uses it is still compiled and type-checked.
    /// That means generic code is still linted, and is visible to rust-analyzer, when working on an ARM Mac.
    #[derive(Clone, Copy, Debug)]
    pub struct Generic(GenericInner);

    #[cfg(not(all(target_arch = "aarch64", target_feature = "neon")))]
    type GenericInner = ();

    // When Neon is enabled at compile time, Neon is always available, so the generic implementation
    // is never needed. `Infallible` has no values, which makes `Generic` impossible to create.
    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    type GenericInner = std::convert::Infallible;

    impl Cpu for Generic {}

    impl CpuPrivate for Generic {
        const DYN_CPU: Option<DynCpu> = cfg_select! {
            all(target_arch = "aarch64", target_feature = "neon") => None,
            _ => Some(DynCpu::Generic(GENERIC)),
        };
    }

    /// [`Generic`] feature set.
    ///
    /// Private within this file. Used only to define [`BASELINE`] and `CpuPrivate::DYN_CPU`.
    /// Not available on aarch64 with Neon enabled at compile time, where a [`Generic`] can't be created.
    #[cfg(not(all(target_arch = "aarch64", target_feature = "neon")))]
    pub(super) const GENERIC: Generic = Generic(());
}

pub(super) use generic::Generic;

/// x86_64 features.
/// AVX2 or generic.
#[cfg(target_arch = "x86_64")]
mod x86_64 {
    use super::*;

    /// x64_64 AVX2 CPU feature set.
    #[derive(Clone, Copy, Debug)]
    pub struct Avx2(());

    impl Cpu for Avx2 {}

    impl CpuPrivate for Avx2 {
        const DYN_CPU: Option<DynCpu> = Some(DynCpu::Avx2(Avx2(())));
    }

    /// Feature set to use without detecting the CPU's features.
    ///
    /// Always [`Generic`] on x86_64.
    pub const BASELINE: Generic = generic::GENERIC;

    impl DynCpu {
        /// Detect CPU feature set for x86_64 platform.
        pub fn detect() -> Self {
            // Currently a compile-time check. Later on will be replaced by runtime detection.
            if cfg!(all(
                target_feature = "avx2",
                target_feature = "bmi2",
                target_feature = "popcnt"
            )) {
                Avx2(()).to_dyn()
            } else {
                BASELINE.to_dyn()
            }
        }
    }
}

#[cfg(target_arch = "x86_64")]
#[cfg_attr(not(test), expect(unused_imports))]
pub(super) use x86_64::{Avx2, BASELINE};

/// aarch64 features.
/// Neon or generic.
#[cfg(target_arch = "aarch64")]
mod aarch64 {
    use super::*;

    /// aarch64 Neon CPU feature set.
    #[derive(Clone, Copy, Debug)]
    pub struct Neon(());

    impl Cpu for Neon {}

    impl CpuPrivate for Neon {
        const DYN_CPU: Option<DynCpu> = Some(DynCpu::Neon(Neon(())));
    }

    /// Feature set to use without detecting the CPU's features.
    ///
    /// Neon is enabled at compile time, which guarantees that it's available.
    #[cfg(target_feature = "neon")]
    pub const BASELINE: Neon = Neon(());

    /// Feature set to use without detecting the CPU's features.
    ///
    /// Neon is disabled at compile time, so only the generic feature set is known to be available.
    #[cfg(not(target_feature = "neon"))]
    pub const BASELINE: Generic = generic::GENERIC;

    impl DynCpu {
        /// Detect CPU feature set for aarch64 platform.
        pub fn detect() -> Self {
            // Neon is always enabled except on `aarch64*-unknown-none-softfloat` platforms
            // or when it's specifically disabled by compiling with `-C target-feature=-neon`.
            // Either way, it can be detected at compile time. No runtime feature check.
            BASELINE.to_dyn()
        }
    }
}

#[cfg(target_arch = "aarch64")]
#[cfg_attr(not(test), expect(unused_imports))]
pub(super) use aarch64::{BASELINE, Neon};

/// Fallback implementation for other platforms.
///
/// Only generic feature set is available.
#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
mod other {
    use super::*;

    /// Feature set to use without detecting the CPU's features.
    ///
    /// Only the generic feature set is available on platforms other than x86_64 and aarch64.
    pub const BASELINE: Generic = generic::GENERIC;

    impl DynCpu {
        /// Detect CPU feature set for other platform.
        ///
        /// Only generic feature set is available.
        pub fn detect() -> Self {
            BASELINE.to_dyn()
        }
    }
}

#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
#[cfg_attr(not(test), expect(unused_imports))]
pub(super) use other::BASELINE;
