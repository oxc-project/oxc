/// On x86_64 without all of AVX2, BMI2 and POPCNT features enabled at compile time,
/// detect CPU features at runtime.
#[cfg(all(
    target_arch = "x86_64",
    not(all(target_feature = "avx2", target_feature = "bmi2", target_feature = "popcnt"))
))]
mod runtime_x86_64 {
    use std::{
        mem,
        sync::atomic::{AtomicI8, Ordering},
    };

    use crate::cpu::{Avx2, DynCpu, Generic};

    /// Discriminant of the detected [`DynCpu`], or -1 if not detected yet.
    static DETECTED: AtomicI8 = AtomicI8::new(-1);

    impl DynCpu {
        /// Detect CPU feature set.
        ///
        /// The result is cached, so only the first call checks the CPU's features.
        #[inline]
        pub fn detect() -> Self {
            let detected = DETECTED.load(Ordering::Relaxed);
            if detected >= 0 {
                // SAFETY: `DETECTED` only holds -1, or a `DynCpu` which `detect_uncached` stored
                unsafe { mem::transmute::<i8, DynCpu>(detected) }
            } else {
                detect_uncached()
            }
        }
    }

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
            DynCpu::Avx2(Avx2(()))
        } else {
            DynCpu::Generic(Generic(()))
        };

        // SAFETY: `DynCpu` is `#[repr(u8)]` and its variants' fields are all zero-sized,
        // so it's just its discriminant
        let discriminant = unsafe { mem::transmute::<DynCpu, i8>(cpu) };
        DETECTED.store(discriminant, Ordering::Relaxed);
        cpu
    }
}

/// No runtime feature check is required on:
///
/// - x86_64 with AVX2, BMI2, and POPCNT enabled at compile time
/// - aarch64
/// - any other platform
///
/// [`BASELINE`] is already the best CPU feature set available.
///
/// [`BASELINE`]: super::BASELINE
#[cfg(not(all(
    target_arch = "x86_64",
    not(all(target_feature = "avx2", target_feature = "bmi2", target_feature = "popcnt"))
)))]
mod compile_time {
    use crate::cpu::{BASELINE, Cpu, DynCpu};

    impl DynCpu {
        /// Detect CPU feature set.
        #[expect(clippy::inline_always, reason = "folds to a constant")]
        #[inline(always)]
        pub fn detect() -> Self {
            BASELINE.to_dyn()
        }
    }
}
