//! Types representing CPU feature sets, used to choose between SIMD and generic implementations.
//!
//! - `Avx2`: x86_64 with AVX2, BMI2 and POPCNT.
//! - `Neon`: aarch64 with Neon.
//! - [`Generic`]: No SIMD. Exists on all platforms.
//!
//! [`DynCpu`] is an enum of these types. [`DynCpu::detect`] returns the best one available.
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
//! * Each has a private field, and `DynCpu::detect` is the only code which creates them.
//!   It only creates an `Avx2` or a `Neon` if that feature set is available.
//! * A [`DynCpu`] can only be built from one of these types, so it's unforgeable too.
//! * [`Cpu`] trait is sealed, so no other type can implement it.
//!
//! Therefore, holding one of these types guarantees that that the CPU supports that feature set.
//!
//! The rest of the lexer relies on this for safety. A function which takes `Avx2` or `Neon`
//! can use instructions that rely on support for those features without any checks.
//!
//! DO NOT add any other way to create these types (e.g. a `Default` impl or a public constructor).

mod select_macro;
mod versions_macro;

pub(super) use select_macro::cpu_select;
pub(super) use versions_macro::{__cpu_versions, apply, cpu_versions};

/// Trait for types representing a CPU feature set.
///
/// Implemented by all types representing specific CPU feature sets, and by [`DynCpu`].
#[expect(private_bounds)]
pub(super) trait Cpu: Sealed + Copy {
    fn to_dyn(self) -> DynCpu;
}

/// Private trait to guarantee that only types defined in this file can implement [`Cpu`].
trait Sealed {}

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
    Generic(Generic),
}

impl Cpu for DynCpu {
    /// Get [`DynCpu`].
    ///
    /// Just returns itself.
    #[inline(always)]
    fn to_dyn(self) -> DynCpu {
        self
    }
}

impl Sealed for DynCpu {}

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

    impl Cpu for Generic {
        /// Get [`DynCpu`] representing generic CPU feature set.
        #[inline(always)]
        fn to_dyn(self) -> DynCpu {
            DynCpu::Generic(self)
        }
    }

    impl Sealed for Generic {}

    impl DynCpu {
        /// Create [`DynCpu::Generic`].
        ///
        /// Private within this file. Used only in [`detect`] implementations.
        /// Not available on aarch64 with Neon enabled at compile time, where a `Generic` can't be created.
        ///
        /// [`detect`]: Self::detect
        #[cfg(not(all(target_arch = "aarch64", target_feature = "neon")))]
        pub(super) fn generic() -> Self {
            Generic(()).to_dyn()
        }
    }
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

    impl Cpu for Avx2 {
        /// Get [`DynCpu`] representing AVX2 CPU feature set.
        #[inline(always)]
        fn to_dyn(self) -> DynCpu {
            DynCpu::Avx2(self)
        }
    }

    impl Sealed for Avx2 {}

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
                DynCpu::generic()
            }
        }
    }
}

#[cfg(target_arch = "x86_64")]
pub(super) use x86_64::Avx2;

/// aarch64 features.
/// Neon or generic.
#[cfg(target_arch = "aarch64")]
mod aarch64 {
    use super::*;

    /// aarch64 Neon CPU feature set.
    #[derive(Clone, Copy, Debug)]
    pub struct Neon(());

    impl Cpu for Neon {
        /// Get [`DynCpu`] representing Neon CPU feature set.
        #[inline(always)]
        fn to_dyn(self) -> DynCpu {
            DynCpu::Neon(self)
        }
    }

    impl Sealed for Neon {}

    impl DynCpu {
        /// Detect CPU feature set for aarch64 platform.
        pub fn detect() -> Self {
            // Neon is always enabled except on `aarch64*-unknown-none-softfloat` platforms
            // or when it's specifically disabled by compiling with `-C target-feature=-neon`.
            // Either way, it can be detected at compile time. No runtime feature check.
            cfg_select! {
                target_feature = "neon" => Neon(()).to_dyn(),
                _ => DynCpu::generic(),
            }
        }
    }
}

#[cfg(target_arch = "aarch64")]
pub(super) use aarch64::Neon;

/// Fallback implementation for other platforms.
///
/// Only generic feature set is available.
#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
impl DynCpu {
    /// Detect CPU feature set for other platform.
    ///
    /// Only generic feature set is available.
    pub fn detect() -> Self {
        DynCpu::generic()
    }
}
