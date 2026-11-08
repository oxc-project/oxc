//! Testing with each CPU feature set this CPU supports.
//!
//! Only compiled for tests, or with the `testing` Cargo feature.

use std::mem;

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
    /// [`detect`]: Self::detect
    pub fn for_each_supported(mut f: impl FnMut(DynCpu)) {
        let mut run = |cpu: DynCpu| {
            // Dropped while unwinding if `f` panics, which prints `cpu`'s name
            let report = ReportCpuOnPanic(cpu);
            f(cpu);
            // `f` didn't panic, so don't report
            mem::forget(report);
        };

        let detected = DynCpu::detect();
        run(detected);

        // `BASELINE` is the generic feature set,
        // except where a SIMD feature set is enabled at compile time, and then it's `detected`
        let baseline = to_dyn(BASELINE);
        if baseline != detected {
            run(baseline);
        }
    }

    /// Name of the CPU feature set, for test failure messages.
    pub fn name(self) -> &'static str {
        match self {
            DynCpu::Generic(_) => "Generic",
            DynCpu::Avx2(_) => "Avx2",
            DynCpu::Neon(_) => "Neon",
        }
    }
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
