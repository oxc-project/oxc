use super::cpu::{Cpu, cpu_select};

#[cfg(target_arch = "x86_64")]
mod avx2;
#[cfg(target_arch = "x86_64")]
use avx2::classify_avx2;

#[cfg(target_arch = "aarch64")]
mod neon;
#[cfg(target_arch = "aarch64")]
use neon::classify_neon;

mod generic;
use generic::classify_generic;

#[cfg(any(test, target_arch = "x86_64", target_arch = "aarch64"))]
mod luts;
#[cfg(any(test, target_arch = "x86_64", target_arch = "aarch64"))]
mod punct1_luts;

mod punct1;

#[inline(always)]
pub(super) unsafe fn classify(
    ts: bool,
    src: *const u8,
    n: usize,
    nb: usize,
    word: *mut u64,
    st: *mut u64,
    kwinit: *mut u64,
    opch: *mut u64,
    digit: *mut u64,
    dot: *mut u64,
    misc: *mut u64,
    kind: *mut u8,
    cpu: impl Cpu,
) {
    cpu_select!(cpu, {
        Avx2(cpu) => classify_avx2(ts, src, n, word, st, kwinit, opch, digit, dot, misc, kind, cpu),
        Neon(cpu) => classify_neon(ts, src, n, word, st, kwinit, opch, digit, dot, misc, kind, cpu),
        Generic(_) => classify_generic(ts, src, n, word, st, kwinit, opch, digit, dot, misc, kind),
    });

    *word.add(nb) = 0;
    *st.add(nb) = 0;
    *kwinit.add(nb) = 0;
    *opch.add(nb) = 0;
    *digit.add(nb) = 0;
    *dot.add(nb) = 0;
    *misc.add(nb) = 0;
}
