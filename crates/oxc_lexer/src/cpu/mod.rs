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
//! and picks an implementation with a `match` on [`to_dyn`]:
//!
//! ```ignore
//! match to_dyn(cpu) {
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
//! Use the [`to_dyn`] function, not the [`Cpu::to_dyn`] method. The method is an extra
//! `#[inline(always)]` function around it, and that changes the code which some callers
//! compile to, because rustc's MIR inliner treats them differently.
//!
//! [`#[cpu_version]`] marks each implementation with the CPU feature set it's for.
//! [`#[cpu_multiversion]`] turns a function into one with a separate entry point for each CPU feature set.
//!
//! The attribute macros are in the `oxc_lexer_macros` crate.
//!
//! * Given a specific type (e.g. `Avx2`): the choice is made at compile time.
//! * Given a [`DynCpu`]: the choice is made at runtime.
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
//! * Each has a private field, so code outside this module can't create them.
//! * Only 3 places create them:
//!   - `DynCpu::detect` only creates an [`Avx2`] if that feature set is available.
//!   - [`BASELINE`] is only an [`Avx2`] or a [`Neon`] when that feature set is enabled
//!     at compile time, which guarantees it's available.
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

use std::{convert::Infallible as Never, mem};

pub(crate) use oxc_lexer_macros::{cpu_multiversion, cpu_version};

#[cfg(test)]
mod tests;

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

// `DynCpu` is defined differently for each architecture.
//
// All variants exist on all architectures, but variants impossible on each architecture
// are uninhabited. This allows code for all architectures to be compiled and linted
// on any machine, and Rust Analyzer sees all the code for all archs "as one".
// But the uninhabited variants allow compiler to apply dead code removal to code paths
// which aren't for the architecture crate is being compiled for.

/// Enum representing a CPU feature set.
///
/// Used for runtime feature detection.
///
/// Implements [`Cpu`] and also provides a [`detect`] method which initializes [`DynCpu`]
/// via target feature detection.
///
/// [`detect`]: Self::detect
#[cfg(all(
    target_arch = "x86_64",
    all(target_feature = "avx2", target_feature = "bmi2", target_feature = "popcnt")
))]
#[derive(Clone, Copy, Debug)]
pub enum DynCpu {
    // Can never be created on x86_64 when AVX2, BMI2 and POPCNT are enabled at compile time.
    // See `Generic`.
    #[expect(dead_code)]
    Generic(Generic),
    Avx2(Avx2),
    #[expect(dead_code)]
    Neon(Neon),
}

/// Enum representing a CPU feature set.
///
/// Used for runtime feature detection.
///
/// Implements [`Cpu`] and also provides a [`detect`] method which initializes [`DynCpu`]
/// via target feature detection.
///
/// [`detect`]: Self::detect
//
// This is the only circumstance in which runtime feature detection is necessary.
// `Neon` is uninhabited, so `Generic` (0) and `Avx2` (1) are the only usable variants,
// making this type equivalent to a `bool`.
#[cfg(all(
    target_arch = "x86_64",
    not(all(target_feature = "avx2", target_feature = "bmi2", target_feature = "popcnt"))
))]
#[derive(Clone, Copy, Debug)]
#[repr(u8)] // So `detect` can transmute it to/from `AtomicI8`
pub enum DynCpu {
    Generic(Generic) = 0,
    Avx2(Avx2) = 1,
    #[expect(dead_code)]
    Neon(Neon) = 2,
}

/// Enum representing a CPU feature set.
///
/// Used for runtime feature detection.
///
/// Implements [`Cpu`] and also provides a [`detect`] method which initializes [`DynCpu`]
/// via target feature detection.
///
/// [`detect`]: Self::detect
#[cfg(target_arch = "aarch64")]
#[derive(Clone, Copy, Debug)]
pub enum DynCpu {
    // Can never be created on aarch64 when Neon is enabled at compile time. See `Generic`.
    #[cfg_attr(target_feature = "neon", expect(dead_code))]
    Generic(Generic),
    #[expect(dead_code)]
    Avx2(Avx2),
    // Can never be created on aarch64 when Neon is disabled at compile time. See `Neon`.
    #[cfg_attr(not(target_feature = "neon"), expect(dead_code))]
    Neon(Neon),
}

/// Enum representing a CPU feature set.
///
/// Used for runtime feature detection.
///
/// Implements [`Cpu`] and also provides a [`detect`] method which initializes [`DynCpu`]
/// via target feature detection.
///
/// [`detect`]: Self::detect
#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
#[derive(Clone, Copy, Debug)]
pub enum DynCpu {
    Generic(Generic),
    #[expect(dead_code)]
    Avx2(Avx2),
    #[expect(dead_code)]
    Neon(Neon),
}

/// [`DynCpu`] has 2 inhabited variants on x86_64 when not all of AVX2, BMI2 and POPCNT
/// are enabled at compile time (so runtime detection is required).
/// On all other platforms it only has 1 inhabited variant, so is a ZST.
const _: () = {
    let expected_size = cfg_select! {
        all(
            target_arch = "x86_64",
            not(all(target_feature = "avx2", target_feature = "bmi2", target_feature = "popcnt"))
        ) => 1,
        _ => 0,
    };
    assert!(size_of::<DynCpu>() == expected_size);
};

/// [`DynCpu`] always has alignment 1.
/// This also guarantees [`Generic`], [`Avx2`] and [`Neon`] have alignment 1.
const _: () = assert!(align_of::<DynCpu>() == 1);

impl Cpu for DynCpu {}

impl CpuPrivate for DynCpu {
    const DYN_CPU: Option<DynCpu> = None;
}

/// Get the [`DynCpu`] for a type implementing [`Cpu`].
///
/// Same as [`Cpu::to_dyn`], but this works in `const` code too.
/// A `const fn` can't call a trait method on a generic type, so it can't call [`Cpu::to_dyn`].
/// [`#[cpu_multiversion]`] uses this function, so it works in `const` code.
/// Code which picks an implementation uses it too: `match to_dyn(cpu) { ... }`.
/// See the module docs for why it's preferred to [`Cpu::to_dyn`] there.
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
#[derive(Clone, Copy, Debug)]
pub struct Neon(
    cfg_select! {
        all(target_arch = "aarch64", target_feature = "neon") => (),
        _ => Never,
    },
);

impl Cpu for Neon {}

impl CpuPrivate for Neon {
    const DYN_CPU: Option<DynCpu> = cfg_select! {
        all(target_arch = "aarch64", target_feature = "neon") => Some(DynCpu::Neon(Neon(()))),
        _ => None,
    };
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
#[derive(Clone, Copy, Debug)]
pub struct Generic(
    cfg_select! {
        not(any(
            all(target_arch = "aarch64", target_feature = "neon"),
            all(
                target_arch = "x86_64",
                all(target_feature = "avx2", target_feature = "bmi2", target_feature = "popcnt")
            )
        )) => (),
        _ => Never,
    },
);

impl Cpu for Generic {}

impl CpuPrivate for Generic {
    const DYN_CPU: Option<DynCpu> = cfg_select! {
        not(any(
            all(target_arch = "aarch64", target_feature = "neon"),
            all(
                target_arch = "x86_64",
                all(target_feature = "avx2", target_feature = "bmi2", target_feature = "popcnt")
            )
        )) => {
            Some(DynCpu::Generic(Generic(())))
        }
        _ => None,
    };
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

impl DynCpu {
    /// Detect CPU feature set.
    ///
    /// The result is cached, so only the first call checks the CPU's features.
    #[cfg(all(
        target_arch = "x86_64",
        not(all(target_feature = "avx2", target_feature = "bmi2", target_feature = "popcnt"))
    ))]
    #[inline]
    pub fn detect() -> Self {
        use std::sync::atomic::{AtomicI8, Ordering};

        /// Discriminant of the detected [`DynCpu`], or -1 if not detected yet.
        static DETECTED: AtomicI8 = AtomicI8::new(-1);

        /// Detect CPU feature set, and cache the result in [`DETECTED`].
        ///
        /// If threads race to call this, they each check the CPU and store the same value.
        /// No thread blocks or sleeps.
        #[cold]
        #[inline(never)]
        fn detect_uncached() -> DynCpu {
            let cpu = if is_x86_feature_detected!("avx2")
                && is_x86_feature_detected!("bmi2")
                && is_x86_feature_detected!("popcnt")
            {
                Avx2(()).to_dyn()
            } else {
                BASELINE.to_dyn()
            };

            // SAFETY: `DynCpu` is `#[repr(u8)]` and its variants' fields are all zero-sized,
            // so it's just its discriminant
            let discriminant = unsafe { mem::transmute::<DynCpu, i8>(cpu) };
            DETECTED.store(discriminant, Ordering::Relaxed);
            cpu
        }

        let detected = DETECTED.load(Ordering::Relaxed);
        if detected >= 0 {
            // SAFETY: `DETECTED` only holds -1, or a `DynCpu` which `detect_uncached` stored
            unsafe { mem::transmute::<i8, DynCpu>(detected) }
        } else {
            detect_uncached()
        }
    }

    /// Detect CPU feature set.
    #[cfg(not(all(
        target_arch = "x86_64",
        not(all(target_feature = "avx2", target_feature = "bmi2", target_feature = "popcnt"))
    )))]
    #[expect(clippy::inline_always, reason = "folds to a constant")]
    #[inline(always)]
    pub fn detect() -> Self {
        // No runtime feature check is required on:
        // - x86_64 with AVX2, BMI2, and POPCNT enabled at compile time
        // - aarch64
        // - any other platform
        BASELINE.to_dyn()
    }
}
