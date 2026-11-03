//! Types representing CPU feature sets, used to choose between SIMD and generic implementations.
//!
//! - [`Avx2`]: x86_64 with AVX2, BMI2 and POPCNT.
//! - [`Neon`]: aarch64 with Neon.
//! - [`Generic`]: No SIMD.
//!
//! All 3 types exist on all platforms. On a platform where a feature set is never used,
//! its type is uninhabited - it can never be created.
//!
//! [`DynCpu`] is an enum of these types. [`DynCpu::detect`] returns the best one available.
//!
//! [`BASELINE`] is the feature set to use without detecting the CPU's features.
//! It's the best one which is known at compile time to be available.
//!
//! Code which has SIMD implementations takes a type implementing [`Cpu`],
//! and uses [`cpu_select!`] to pick an implementation.
//! [`#[cpu_version]`] marks each implementation with the CPU feature set it's for.
//! [`#[cpu_multiversion]`] turns a function into one with a separate entry point for each CPU feature set.
//! [`#[cpu_impl]`] marks methods which only work on one CPU feature set's platform.
//!
//! The attribute macros are in the `oxc_lexer_macros` crate.
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
//! [`Avx2`], [`Neon`] and [`Generic`] types are all zero-sized, so can be passed between functions
//! without any runtime cost. They operate at the level of the type system only.
//!
//! # SAFETY
//!
//! These types are unforgeable.
//!
//! * Each has a private field, so code outside this module can't create them.
//! * Only 3 places create them:
//!   - `DynCpu::detect` only creates an [`Avx2`] if that feature set is available.
//!   - [`BASELINE`] is only a [`Neon`] when Neon is enabled at compile time,
//!     which guarantees it's available.
//!     Otherwise it's a [`Generic`], which guarantees nothing.
//!   - `CpuPrivate::DYN_CPU` holds an instance of the type.
//!     The trait is private, so only code in this file can read it.
//!     Only [`to_dyn`] reads it, and it takes an instance of the same type,
//!     so it only gives one to code which already has one.
//! * On a platform where a type is uninhabited, it can't be created at all.
//! * A [`DynCpu`] can only be built from one of these types, so it's unforgeable too.
//! * [`Cpu`] trait is sealed, so no other type can implement it.
//!
//! Therefore, holding one of these types guarantees that the CPU supports that feature set.
//!
//! The rest of the lexer relies on this for safety. A function which takes `Avx2` or `Neon`
//! can use instructions that rely on support for those features without any checks.
//!
//! DO NOT add any other way to create these types (e.g. a `Default` impl or a public constructor).
//!
//! [`#[cpu_version]`]: macro@cpu_version
//! [`#[cpu_multiversion]`]: macro@cpu_multiversion
//! [`#[cpu_impl]`]: macro@cpu_impl

use std::{convert::Infallible as Never, mem};

pub(crate) use oxc_lexer_macros::{cpu_impl, cpu_multiversion, cpu_version};

mod select_macro;

#[cfg(test)]
mod tests;

pub(crate) use select_macro::cpu_select;

/// Trait for types representing a CPU feature set.
///
/// Implemented by all types representing specific CPU feature sets, and by [`DynCpu`].
#[expect(private_bounds)]
pub trait Cpu: CpuPrivate + Copy {
    /// Get the [`DynCpu`] for this type.
    ///
    /// Same as the [`to_dyn`] function, which works in `const` code too.
    #[expect(clippy::inline_always, reason = "only calls `to_dyn`, which folds to a constant")]
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
    /// [`Avx2`], [`Neon`] and [`Generic`] each have only one value, so this can hold it.
    /// `None` for [`DynCpu`] itself, and for each of them on platforms where it's uninhabited.
    const DYN_CPU: Option<DynCpu>;
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
pub enum DynCpu {
    /// AVX2 feature set.
    /// Exists on all platforms, but can only be created on x86_64. See [`Avx2`].
    #[cfg_attr(not(target_arch = "x86_64"), expect(dead_code))]
    Avx2(Avx2),
    /// Generic feature set.
    /// Exists on all platforms, but can never be created on aarch64
    /// when Neon is enabled at compile time. See [`Generic`].
    #[cfg_attr(all(target_arch = "aarch64", target_feature = "neon"), expect(dead_code))]
    Generic(Generic),
    // `Neon` is last. rustc numbers variants in the order they're declared, uninhabited ones too.
    // So on x86_64, where `Neon` is uninhabited, the 2 variants which can exist are 0 and 1,
    // like a `bool`.
    /// Neon feature set.
    /// Exists on all platforms, but can only be created on aarch64. See [`Neon`].
    #[cfg_attr(not(target_arch = "aarch64"), expect(dead_code))]
    Neon(Neon),
}

impl Cpu for DynCpu {}

impl CpuPrivate for DynCpu {
    const DYN_CPU: Option<DynCpu> = None;
}

/// Get the [`DynCpu`] for a type implementing [`Cpu`].
///
/// Same as [`Cpu::to_dyn`], but this works in `const` code too.
/// A `const fn` can't call a trait method on a generic type, so it can't call [`Cpu::to_dyn`].
/// [`cpu_select!`] and [`#[cpu_multiversion]`] use this function, so they work in `const` code.
///
/// Only [`DynCpu`] needs `cpu`. It's taken for all types so that this can only be called
/// by code which already has one of these types.
///
/// [`#[cpu_multiversion]`]: macro@cpu_multiversion
#[expect(clippy::inline_always, reason = "folds to a constant, except for `DynCpu`")]
#[inline(always)]
pub const fn to_dyn<C: Cpu>(cpu: C) -> DynCpu {
    match C::DYN_CPU {
        Some(dyn_cpu) => dyn_cpu,
        // SAFETY: `DYN_CPU` is `None` only for `DynCpu`, and for CPU types which are uninhabited.
        // No value of an uninhabited type exists, so `cpu` must be a `DynCpu`.
        None => unsafe { mem::transmute_copy::<C, DynCpu>(&cpu) },
    }
}

/// x86_64 AVX2 CPU feature set.
///
/// On platforms other than x86_64, this type is uninhabited - `Avx2` can never be created.
/// So [`DynCpu::Avx2`] can never exist there, and the compiler removes code which handles it.
///
/// The type still exists there, so [`DynCpu`] has the same variants on every platform.
#[derive(Clone, Copy, Debug)]
pub struct Avx2(
    cfg_select! {
        target_arch = "x86_64" => (),
        _ => Never,
    },
);

impl Cpu for Avx2 {}

impl CpuPrivate for Avx2 {
    const DYN_CPU: Option<DynCpu> = cfg_select! {
        target_arch = "x86_64" => Some(DynCpu::Avx2(Avx2(()))),
        _ => None,
    };
}

/// aarch64 Neon CPU feature set.
///
/// On platforms other than aarch64, this type is uninhabited - `Neon` can never be created.
/// So [`DynCpu::Neon`] can never exist there, and the compiler removes code which handles it.
///
/// The type still exists there, so [`DynCpu`] has the same variants on every platform.
#[derive(Clone, Copy, Debug)]
pub struct Neon(
    cfg_select! {
        target_arch = "aarch64" => (),
        _ => Never,
    },
);

impl Cpu for Neon {}

impl CpuPrivate for Neon {
    const DYN_CPU: Option<DynCpu> = cfg_select! {
        target_arch = "aarch64" => Some(DynCpu::Neon(Neon(()))),
        _ => None,
    };
}

/// Generic CPU feature set.
///
/// On aarch64 with Neon enabled at compile time, this type is uninhabited - `Generic` can never be created.
/// So [`DynCpu::Generic`] can never exist there, and the compiler removes code which handles it.
/// Every aarch64 target with `std` enables Neon by default, so in practice this applies to all aarch64 builds.
///
/// The type still exists there, so code which uses it is still compiled and type-checked.
/// That means generic code is still linted, and is visible to rust-analyzer, when working on an ARM Mac.
#[derive(Clone, Copy, Debug)]
pub struct Generic(
    cfg_select! {
        not(all(target_arch = "aarch64", target_feature = "neon")) => (),
        _ => Never,
    },
);

impl Cpu for Generic {}

impl CpuPrivate for Generic {
    const DYN_CPU: Option<DynCpu> = cfg_select! {
        not(all(target_arch = "aarch64", target_feature = "neon")) => {
            Some(DynCpu::Generic(Generic(())))
        }
        _ => None,
    };
}

/// Feature set to use without detecting the CPU's features.
///
/// Always [`Generic`] on x86_64.
#[cfg(target_arch = "x86_64")]
pub const BASELINE: Generic = Generic(());

/// Feature set to use without detecting the CPU's features.
///
/// Neon is enabled at compile time, which guarantees that it's available.
#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
pub const BASELINE: Neon = Neon(());

/// Feature set to use without detecting the CPU's features.
///
/// Neon is disabled at compile time, so only the generic feature set is known to be available.
#[cfg(all(target_arch = "aarch64", not(target_feature = "neon")))]
pub const BASELINE: Generic = Generic(());

/// Feature set to use without detecting the CPU's features.
///
/// Only the generic feature set is available on platforms other than x86_64 and aarch64.
#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
pub const BASELINE: Generic = Generic(());

impl DynCpu {
    /// Detect CPU feature set.
    #[cfg(target_arch = "x86_64")]
    pub fn detect() -> Self {
        // Currently a compile-time check. Later on will be replaced by runtime detection.
        if cfg!(all(target_feature = "avx2", target_feature = "bmi2", target_feature = "popcnt")) {
            Avx2(()).to_dyn()
        } else {
            BASELINE.to_dyn()
        }
    }

    /// Detect CPU feature set.
    #[cfg(target_arch = "aarch64")]
    pub fn detect() -> Self {
        // Neon is always enabled except on `aarch64*-unknown-none-softfloat` platforms
        // or when it's specifically disabled by compiling with `-C target-feature=-neon`.
        // Either way, it can be detected at compile time. No runtime feature check.
        BASELINE.to_dyn()
    }

    /// Detect CPU feature set.
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    pub fn detect() -> Self {
        // Only generic feature set is available
        BASELINE.to_dyn()
    }
}
