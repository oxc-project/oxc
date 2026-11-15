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
//! and picks an implementation with a `match` on [`DynCpu::from`]:
//!
//! ```ignore
//! match DynCpu::from(cpu) {
//!     DynCpu::Avx2(cpu) => classify_avx2(self, cpu),
//!     DynCpu::Neon(cpu) => classify_neon(self, cpu),
//!     DynCpu::Generic(_) => classify_generic(self),
//! }
//! ```
//!
//! [`DynCpu`] has all 3 variants on every platform. On a platform which doesn't have
//! a feature set, its type is uninhabited, so its arm can never run, and the compiler removes it.
//! The arm is still needed, because the type's field is private, so outside this module
//! rustc doesn't know that it's uninhabited.
//!
//! Use [`DynCpu::from`], not the [`Cpu::to_dyn`] method. The method is an extra `#[inline(always)]`
//! function around it, and that changes the code which some callers compile to, because rustc's
//! MIR inliner treats them differently.
//!
//! [`#[cpu_version]`] marks each implementation with the CPU feature set it's for.
//! [`#[cpu_multiversion]`] turns a function into one with a separate entry point for each CPU feature set.
//!
//! The attribute macros are in the `oxc_simd_macros` crate.
//!
//! - Given a specific type (e.g. `Avx2`): the choice is made at compile time.
//! - Given a [`DynCpu`]: the choice is made at runtime.
//!
//! Functions which pick an implementation typically take a generic `impl Cpu`.
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
//! - Each has a private field, so code outside this module can't create them.
//! - Only 3 places create them:
//!   - `DynCpu::detect` only creates an [`Avx2`] if that feature set is available.
//!     Otherwise it creates a [`Generic`], which guarantees nothing.
//!   - [`BASELINE`] is only an [`Avx2`] or a [`Neon`] when that feature set is enabled
//!     at compile time, which guarantees it's available.
//!     Otherwise it's a [`Generic`], which guarantees nothing.
//!   - `CpuPrivate::DISPATCH` holds an instance of the type, on platforms where it can exist.
//!     The trait is private, so only code in this module can read it.
//!     Only [`DynCpu::from`] reads it, and it takes an instance of the same type,
//!     so it only gives one to code which already has one.
//! - On a platform where a type is uninhabited, it can't be created at all.
//! - A [`DynCpu`] can only be built from one of these types, so it's unforgeable too.
//! - [`Cpu`] trait is sealed, so no other type can implement it.
//!
//! Therefore, holding one of these types guarantees that the CPU supports that feature set.
//!
//! The rest of the lexer relies on this for safety. A function which takes `Avx2` or `Neon`
//! can use instructions that rely on support for those features without any checks.
//!
//! DO NOT add any other way to create these types (e.g. a `Default` impl or a public constructor).
//!
//! [`#[cpu_version]`]: macro@crate::cpu_version
//! [`#[cpu_multiversion]`]: macro@crate::cpu_multiversion

use std::{
    convert::Infallible as Never,
    fmt::{self, Debug},
    mem,
};

mod detect;

#[cfg(any(test, feature = "testing"))]
mod testing;

#[cfg(test)]
mod tests;

/// Trait for types representing a CPU feature set.
///
/// Implemented by all types representing specific CPU feature sets, and by [`DynCpu`].
#[expect(private_bounds)]
pub trait Cpu: CpuPrivate + Copy {
    /// Get the [`DynCpu`] for this type.
    ///
    /// Same as [`DynCpu::from`], which works in `const` code too.
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn to_dyn(self) -> DynCpu {
        DynCpu::from(self)
    }
}

/// Private trait to guarantee that only types defined in this module can implement [`Cpu`].
///
/// It also holds [`DISPATCH`], for [`DynCpu::from`].
///
/// [`DISPATCH`] is on this trait, not on [`Cpu`], so code outside this module can't read it.
/// It holds an instance of the type, so if code outside this module could read it,
/// that would allow bypassing CPU feature detection.
///
/// # SAFETY
///
/// The unsafe code in [`DynCpu::from`] relies on [`DISPATCH`] being correct.
/// Each implementation must set it to:
///
/// - [`Dispatch::Dynamic`] only for [`DynCpu`].
/// - [`Dispatch::Static`] only for a type which can exist on this platform,
///   holding the [`DynCpu`] variant for that same type.
/// - [`Dispatch::Never`] only for a type which is uninhabited on this platform.
///
/// For [`Avx2`], [`Neon`] and [`Generic`], that means the `cfg_select!` in the implementation
/// must match the one on the type's field.
///
/// [`DISPATCH`]: Self::DISPATCH
unsafe trait CpuPrivate {
    /// How this type is dispatched on this platform. See [`Dispatch`].
    const DISPATCH: Dispatch;
}

/// How a [`Cpu`] type is dispatched on this platform.
///
/// [`DynCpu::from`] uses it to get the [`DynCpu`] for a value of the type.
enum Dispatch {
    /// [`DynCpu`], whose feature set is chosen at runtime.
    ///
    /// [`DynCpu::from`] returns the [`DynCpu`] it's given.
    Dynamic,
    /// A type which always means the same feature set, and which can exist on this platform.
    ///
    /// Holds the [`DynCpu`] for the type, which [`DynCpu::from`] returns.
    /// [`Avx2`], [`Neon`] and [`Generic`] each have only one value, so this can hold it.
    Static(DynCpu),
    /// A type which is uninhabited on this platform, so is never dispatched.
    ///
    /// No value of the type can exist, so [`DynCpu::from`] can never be called with one.
    Never,
}

// All `DynCpu`'s variants exist on all architectures, but variants impossible on each architecture
// are uninhabited. This allows code for all architectures to be compiled and linted on any machine,
// and Rust Analyzer sees all the code for all archs "as one".
// But the uninhabited variants allow compiler to apply dead code removal to code paths
// which aren't for the architecture code is being compiled for.

/// Enum representing a CPU feature set.
///
/// Used for runtime feature detection.
///
/// Implements [`Cpu`] and also provides a [`detect`] method which initializes [`DynCpu`]
/// via target feature detection.
///
/// [`detect`]: Self::detect
//
// x86_64 without all of AVX2, BMI2 and POPCNT features enabled at compile time is
// the only circumstance in which runtime feature detection is necessary,
// and the only case where `DynCpu` has more than 1 inhabited variant, so isn't a ZST.
// `Neon` is uninhabited, so `Generic` (0) and `Avx2` (1) are the only usable variants,
// making this type equivalent to a `bool`.
#[cfg(all(
    target_arch = "x86_64",
    not(all(target_feature = "avx2", target_feature = "bmi2", target_feature = "popcnt"))
))]
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)] // So `detect` can transmute it to/from `AtomicI8`
pub enum DynCpu {
    Generic(Generic) = 0,
    Avx2(Avx2) = 1,
    Neon(Neon) = 2,
}

#[cfg(all(
    target_arch = "x86_64",
    not(all(target_feature = "avx2", target_feature = "bmi2", target_feature = "popcnt"))
))]
const _: () = assert!(size_of::<DynCpu>() == 1);

/// Enum representing a CPU feature set.
///
/// Used for runtime feature detection.
///
/// Implements [`Cpu`] and also provides a [`detect`] method which initializes [`DynCpu`]
/// via target feature detection.
///
/// [`detect`]: Self::detect
//
// Runtime feature detection is not required for either:
// - x86_64 with AVX2, BMI2 and POPCNT all enabled at compile time
// - any architecture other than x86_64
//
// Only one variant of `DynCpu` is inhabited, so we use standard Rust repr (no `#[repr(u8)]`)
// so that `DynCpu` is a ZST.
#[cfg(not(all(
    target_arch = "x86_64",
    not(all(target_feature = "avx2", target_feature = "bmi2", target_feature = "popcnt"))
)))]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DynCpu {
    Generic(Generic),
    Avx2(Avx2),
    Neon(Neon),
}

#[cfg(not(all(
    target_arch = "x86_64",
    not(all(target_feature = "avx2", target_feature = "bmi2", target_feature = "popcnt"))
)))]
const _: () = assert!(size_of::<DynCpu>() == 0);

/// [`DynCpu`] always has alignment 1.
/// This also guarantees [`Generic`], [`Avx2`] and [`Neon`] have alignment 1.
const _: () = assert!(align_of::<DynCpu>() == 1);

impl DynCpu {
    /// Name of the CPU feature set, e.g. `"Avx2"`.
    pub fn name(self) -> &'static str {
        match self {
            DynCpu::Generic(_) => "Generic",
            DynCpu::Avx2(_) => "Avx2",
            DynCpu::Neon(_) => "Neon",
        }
    }

    /// Get the [`DynCpu`] for a type implementing [`Cpu`].
    ///
    /// Same as [`Cpu::to_dyn`], but this works in `const` code too.
    /// A `const fn` can't call a trait method on a generic type, so it can't call [`Cpu::to_dyn`].
    /// [`#[cpu_multiversion]`] uses this function, so it works in `const` code.
    /// Code which picks an implementation uses it too: `match DynCpu::from(cpu) { ... }`.
    /// See the module docs for why it's preferred to [`Cpu::to_dyn`] there.
    ///
    /// Only [`DynCpu`] needs `cpu`. It's taken for all types so that this can only be called
    /// by code which already has one of these types.
    ///
    /// [`#[cpu_multiversion]`]: macro@crate::cpu_multiversion
    #[expect(clippy::inline_always, reason = "folds to a constant, except for `DynCpu`")]
    #[inline(always)]
    pub const fn from<C: Cpu>(cpu: C) -> DynCpu {
        match C::DISPATCH {
            Dispatch::Dynamic => {
                // SAFETY: `C::DISPATCH` is only `Dynamic` when `C` is `DynCpu`
                unsafe { mem::transmute_copy::<C, DynCpu>(&cpu) }
            }
            Dispatch::Static(dyn_cpu) => dyn_cpu,
            Dispatch::Never => {
                if cfg!(debug_assertions) {
                    unreachable!();
                }
                // SAFETY: `C::DISPATCH` is only `Never` when `C` is uninhabited,
                // so `cpu` can't exist, and this can't be reached
                unsafe { std::hint::unreachable_unchecked() }
            }
        }
    }
}

impl Cpu for DynCpu {}

// SAFETY: This is `DynCpu`, the only type which can be `Dispatch::Dynamic`
unsafe impl CpuPrivate for DynCpu {
    const DISPATCH: Dispatch = Dispatch::Dynamic;
}

// Not derived, which would print the variant's field too, e.g. `Avx2(Avx2(()))`
impl Debug for DynCpu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// x86_64 AVX2 CPU feature set.
///
/// On platforms other than x86_64, this type is uninhabited - `Avx2` can never be created.
/// So [`DynCpu::Avx2`] can never exist there, and the compiler removes code which handles it.
///
/// The type still exists there, so [`DynCpu`] has the same variants on every platform.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Avx2(
    cfg_select! {
        target_arch = "x86_64" => (),
        _ => Never,
    },
);

impl Cpu for Avx2 {}

// SAFETY: This `cfg_select!` matches the one on `Avx2`'s field, so `DISPATCH` is `Static`
// (holding an `Avx2`) where `Avx2` can exist, and `Never` where it can't
unsafe impl CpuPrivate for Avx2 {
    const DISPATCH: Dispatch = cfg_select! {
        target_arch = "x86_64" => Dispatch::Static(DynCpu::Avx2(Avx2(()))),
        _ => Dispatch::Never,
    };
}

// Not derived, which would print the private field, e.g. `Avx2(())`
impl Debug for Avx2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Avx2")
    }
}

/// aarch64 Neon CPU feature set.
///
/// This type is uninhabited - `Neon` can never be created - on:
///
/// - Platforms other than aarch64.
/// - aarch64 with Neon disabled at compile time.
///
/// So [`DynCpu::Neon`] can never exist there, and the compiler removes code which handles it.
///
/// The type still exists there, so [`DynCpu`] has the same variants on every platform.
///
/// Neon is always enabled on aarch64 except on `aarch64*-unknown-none-softfloat` platforms
/// or when it's specifically disabled by compiling with `-C target-feature=-neon`.
/// Either way, it can be detected at compile time. No runtime feature check.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Neon(
    cfg_select! {
        all(target_arch = "aarch64", target_feature = "neon") => (),
        _ => Never,
    },
);

impl Cpu for Neon {}

// SAFETY: This `cfg_select!` matches the one on `Neon`'s field, so `DISPATCH` is `Static`
// (holding a `Neon`) where `Neon` can exist, and `Never` where it can't
unsafe impl CpuPrivate for Neon {
    const DISPATCH: Dispatch = cfg_select! {
        all(target_arch = "aarch64", target_feature = "neon") => {
            Dispatch::Static(DynCpu::Neon(Neon(())))
        }
        _ => Dispatch::Never,
    };
}

// Not derived, which would print the private field, e.g. `Neon(())`
impl Debug for Neon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Neon")
    }
}

/// Generic CPU feature set.
///
/// Where a SIMD feature set is enabled at compile time, this type is uninhabited -
/// `Generic` can never be created for:
///
/// - aarch64 with Neon enabled.
///   Every aarch64 target with `std` enables Neon by default,
///   so in practice this applies to all aarch64 builds.
/// - x86_64 with AVX2, BMI2 and POPCNT all enabled, e.g. with `-C target-cpu=x86-64-v3`.
///
/// The SIMD feature set is always available there, so the generic code would never run.
/// [`DynCpu::Generic`] can never exist there, and the compiler removes code which handles it,
/// so the generic code isn't in the binary.
///
/// The type still exists there, so code which uses it is still compiled and type-checked.
/// That means generic code is still linted, and is visible to rust-analyzer, when working on an ARM Mac.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Generic(
    cfg_select! {
        not(any(
            all(
                target_arch = "x86_64",
                all(target_feature = "avx2", target_feature = "bmi2", target_feature = "popcnt")
            ),
            all(target_arch = "aarch64", target_feature = "neon")
        )) => (),
        _ => Never,
    },
);

impl Cpu for Generic {}

// SAFETY: This `cfg_select!` matches the one on `Generic`'s field, so `DISPATCH` is `Static`
// (holding a `Generic`) where `Generic` can exist, and `Never` where it can't
unsafe impl CpuPrivate for Generic {
    const DISPATCH: Dispatch = cfg_select! {
        not(any(
            all(
                target_arch = "x86_64",
                all(target_feature = "avx2", target_feature = "bmi2", target_feature = "popcnt")
            ),
            all(target_arch = "aarch64", target_feature = "neon")
        )) => {
            Dispatch::Static(DynCpu::Generic(Generic(())))
        }
        _ => Dispatch::Never,
    };
}

// Not derived, which would print the private field, e.g. `Generic(())`
impl Debug for Generic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Generic")
    }
}

/// Feature set to use without detecting the CPU's features.
///
/// AVX2, BMI2 and POPCNT are enabled at compile time, which guarantees that they're available.
#[cfg(all(
    target_arch = "x86_64",
    all(target_feature = "avx2", target_feature = "bmi2", target_feature = "popcnt")
))]
pub const BASELINE: Avx2 = Avx2(());

/// Feature set to use without detecting the CPU's features.
///
/// Neon is enabled at compile time, which guarantees that it's available.
#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
pub const BASELINE: Neon = Neon(());

/// Feature set to use without detecting the CPU's features.
///
/// Only generic feature set is supported, or can be guaranteed to be supported.
#[cfg(not(any(
    all(
        target_arch = "x86_64",
        all(target_feature = "avx2", target_feature = "bmi2", target_feature = "popcnt")
    ),
    all(target_arch = "aarch64", target_feature = "neon")
)))]
pub const BASELINE: Generic = Generic(());
