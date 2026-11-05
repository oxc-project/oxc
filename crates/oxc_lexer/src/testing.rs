//! Functions for testing the lexer with each CPU feature set.
//!
//! Only available with the `testing` Cargo feature.

use crate::{
    Arena, LexOptions, LexResult, SCRATCH,
    cpu::{BASELINE, DynCpu, to_dyn},
    lex_utf8,
};

/// A CPU feature set which this CPU supports.
///
/// Only [`for_each_cpu`] creates them, so code outside this crate can only lex with
/// CPU feature sets which this CPU supports.
///
/// A wrapper around `DynCpu`, so `DynCpu` stays private to this crate.
#[derive(Clone, Copy)]
pub struct CpuFeatureSet(DynCpu);

impl CpuFeatureSet {
    /// Name of the CPU feature set, for test failure messages.
    pub fn name(self) -> &'static str {
        self.0.name()
    }
}

/// Call `f` with each CPU feature set that this CPU supports, best first.
///
/// That's the one `DynCpu::detect` returns, then the generic one, if it's different.
/// Where a SIMD feature set is enabled at compile time, the generic one doesn't exist,
/// so `f` is only called once.
pub fn for_each_cpu(mut f: impl FnMut(CpuFeatureSet)) {
    let detected = DynCpu::detect();
    f(CpuFeatureSet(detected));

    // `BASELINE` is the generic feature set,
    // except where a SIMD feature set is enabled at compile time, and then it's `detected`
    let baseline = to_dyn(BASELINE);
    if baseline != detected {
        f(CpuFeatureSet(baseline));
    }
}

/// Same as [`lex_utf8`], but lexes with CPU feature set `cpu`,
/// instead of the best one this CPU supports.
///
/// # Panics
/// Panics if `src` does not extend at least [`PAD`] zeroed bytes past `len`.
///
/// [`PAD`]: crate::PAD
pub fn lex_utf8_with_cpu(
    src: &[u8],
    len: u32,
    options: LexOptions,
    cpu: CpuFeatureSet,
) -> (LexResult, Arena) {
    let _switch = SwitchCpu::new(cpu.0);
    lex_utf8(src, len, options)
}

/// Guard which switches the thread's [`Lexer`] to a different CPU feature set,
/// and switches it back when dropped, even if lexing panics.
///
/// [`Lexer`]: crate::Lexer
struct SwitchCpu {
    /// The CPU feature set to switch back to.
    previous: DynCpu,
}

impl SwitchCpu {
    /// Switch the thread's [`Lexer`] to CPU feature set `cpu`.
    ///
    /// [`Lexer`]: crate::Lexer
    fn new(cpu: DynCpu) -> Self {
        let previous = SCRATCH.with(|cell| cell.borrow_mut().replace_cpu(cpu));
        Self { previous }
    }
}

impl Drop for SwitchCpu {
    /// Switch the thread's [`Lexer`] back to the CPU feature set it had before.
    ///
    /// [`Lexer`]: crate::Lexer
    fn drop(&mut self) {
        SCRATCH.with(|cell| cell.borrow_mut().replace_cpu(self.previous));
    }
}
