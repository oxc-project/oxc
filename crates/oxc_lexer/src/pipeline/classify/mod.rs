use crate::cpu::{Cpu, DynCpu, cpu_multiversion, to_dyn};

mod avx2;
use avx2::classify_avx2;

mod neon;
use neon::classify_neon;

mod generic;
use generic::classify_generic;

mod luts;
mod punct1_luts;

mod punct1;

#[cpu_multiversion]
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
    match to_dyn(cpu) {
        DynCpu::Avx2(cpu) => {
            classify_avx2(ts, src, n, word, st, kwinit, opch, digit, dot, misc, kind, cpu)
        }
        DynCpu::Neon(cpu) => {
            classify_neon(ts, src, n, word, st, kwinit, opch, digit, dot, misc, kind, cpu)
        }
        DynCpu::Generic(_) => {
            classify_generic(ts, src, n, word, st, kwinit, opch, digit, dot, misc, kind)
        }
    };

    *word.add(nb) = 0;
    *st.add(nb) = 0;
    *kwinit.add(nb) = 0;
    *opch.add(nb) = 0;
    *digit.add(nb) = 0;
    *dot.add(nb) = 0;
    *misc.add(nb) = 0;
}
