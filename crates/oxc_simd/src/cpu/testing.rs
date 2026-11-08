//! Testing with each CPU feature set this CPU supports.
//!
//! Only compiled for tests, or with the `testing` Cargo feature.

use std::{
    env, mem,
    sync::atomic::{AtomicBool, Ordering},
};

use super::{BASELINE, DynCpu, to_dyn};

impl DynCpu {
    /// Call `f` with each CPU feature set that this CPU supports, best first.
    ///
    /// That's the one [`detect`] returns, then the generic one, if it's different.
    /// Where a SIMD feature set is enabled at compile time, the generic one doesn't exist,
    /// so `f` is only called once.
    ///
    /// If `f` panics, print which CPU feature set it panicked with,
    /// so a test failure says which one failed.
    ///
    /// # Panics
    ///
    /// Panics if the `OXC_SIMD_EXPECT` environment variable is set, and isn't the name of the
    /// CPU feature set which [`detect`] returns (e.g. `Avx2`). CI sets it, so that a CI machine
    /// which doesn't support the expected SIMD feature set fails, instead of quietly testing
    /// fewer feature sets.
    ///
    /// [`detect`]: Self::detect
    pub fn for_each_supported(mut f: impl FnMut(DynCpu)) {
        let detected = DynCpu::detect();
        check_expected(detected);

        let mut run = |cpu: DynCpu| {
            // Dropped while unwinding if `f` panics, which prints `cpu`'s name
            let report = ReportCpuOnPanic(cpu);
            f(cpu);
            // `f` didn't panic, so don't report
            mem::forget(report);
        };

        run(detected);

        // `BASELINE` is the generic feature set,
        // except where a SIMD feature set is enabled at compile time, and then it's `detected`
        let baseline = to_dyn(BASELINE);
        if baseline != detected {
            run(baseline);
        }
    }
}

/// Panic if the `OXC_SIMD_EXPECT` environment variable is set, and isn't the name of `detected`.
/// See [`DynCpu::for_each_supported`].
///
/// Reading an environment variable takes a lock and allocates, and `for_each_supported` is called
/// for every conformance fixture, so this only checks it until the check passes once.
/// If it fails, every later call checks again, and fails again.
fn check_expected(detected: DynCpu) {
    /// Whether the check has passed.
    static CHECKED: AtomicBool = AtomicBool::new(false);

    if CHECKED.load(Ordering::Relaxed) {
        return;
    }

    if let Some(expected) = env::var_os("OXC_SIMD_EXPECT") {
        let actual = detected.name();
        assert!(
            expected == actual,
            "`OXC_SIMD_EXPECT` is `{}`, but the best CPU feature set this CPU supports is `{actual}`",
            expected.display(),
        );
    }

    CHECKED.store(true, Ordering::Relaxed);
}

/// Guard which prints the name of a CPU feature set when it's dropped.
///
/// The panic message is printed before unwinding starts, so this goes after it.
///
/// [`DynCpu::for_each_supported`] forgets it when `f` returns, so it's only dropped
/// while unwinding from a panic in `f`.
struct ReportCpuOnPanic(DynCpu);

impl Drop for ReportCpuOnPanic {
    #[expect(clippy::print_stderr, reason = "it's for test failure output")]
    fn drop(&mut self) {
        eprintln!("Failed with CPU feature set: {}", self.0.name());
    }
}
